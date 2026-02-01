use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use std::io::stdout;
use std::path::PathBuf;

use crate::fs::mald_home;
use crate::index::metadata::{FtsResult, MetadataStore};

// ─── Views ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum View {
    Home,
    Search,
    Browse,
    Tasks,
    Edit,
}

impl View {
    fn titles() -> Vec<&'static str> {
        vec!["Home", "Search", "Browse", "Tasks", "Edit"]
    }

    fn index(self) -> usize {
        match self {
            View::Home => 0,
            View::Search => 1,
            View::Browse => 2,
            View::Tasks => 3,
            View::Edit => 4,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => View::Home,
            1 => View::Search,
            2 => View::Browse,
            3 => View::Tasks,
            4 => View::Edit,
            _ => View::Home,
        }
    }

    fn next(self) -> Self {
        Self::from_index((self.index() + 1) % 5)
    }

    fn prev(self) -> Self {
        Self::from_index((self.index() + 4) % 5)
    }
}

// ─── App State ─────────────────────────────────────────────────────

struct TuiApp {
    view: View,
    search: SearchState,
    home: HomeState,
    browse: BrowseState,
    tasks: TasksState,
    editor: EditorState,
    show_help: bool,
    ext_editor: String,
    open_file_external: Option<String>,
    all_note_names: Vec<String>,
    daemon_running: bool,
}

struct HomeState {
    kb_stats: Vec<(String, usize)>,
    recent: Vec<String>,
    tasks: Vec<(String, String)>,
    version: String,
}

struct SearchState {
    query: String,
    results: Vec<FtsResult>,
    selected: ListState,
    meta: Option<MetadataStore>,
    preview: String,
}

struct BrowseState {
    files: Vec<PathBuf>,
    selected: ListState,
    preview: String,
    filter: String,
    filtering: bool,
}

struct TasksState {
    tasks: Vec<(String, String, String)>, // (kb, note, task_text)
    selected: ListState,
}

// ─── Editor State ──────────────────────────────────────────────────

struct EditorState {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    scroll_y: usize,
    scroll_x: usize,
    file_path: Option<PathBuf>,
    modified: bool,
    autocomplete: Option<Autocomplete>,
    backlinks: Vec<(String, String)>,
    forward_links: Vec<String>,
    tags: Vec<String>,
    status_msg: String,
}

struct Autocomplete {
    all_items: Vec<String>,
    filtered: Vec<String>,
    selected: usize,
    query: String,
    trigger_col: usize,
}

impl EditorState {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_y: 0,
            scroll_x: 0,
            file_path: None,
            modified: false,
            autocomplete: None,
            backlinks: Vec::new(),
            forward_links: Vec::new(),
            tags: Vec::new(),
            status_msg: "No file open. Open a file from Browse or Search.".into(),
        }
    }

    fn load_file(&mut self, path: &str, all_note_names: &[String]) {
        let path = PathBuf::from(path);
        if let Ok(content) = std::fs::read_to_string(&path) {
            self.lines = content.lines().map(String::from).collect();
            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
            self.cursor_row = 0;
            self.cursor_col = 0;
            self.scroll_y = 0;
            self.scroll_x = 0;
            self.modified = false;

            // Parse links and tags from content
            self.forward_links = crate::parser::links::extract_wikilinks(&content);
            self.tags = crate::parser::tags::extract_tags(&content);

            // Find backlinks
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            self.backlinks = crate::commands::fix_links::find_backlinks(&stem);

            self.status_msg = format!(
                "{} | {} links | {} backlinks | {} tags",
                path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                self.forward_links.len(),
                self.backlinks.len(),
                self.tags.len(),
            );
            self.file_path = Some(path);
            let _ = all_note_names; // used for autocomplete context
        }
    }

    fn save(&mut self) -> bool {
        if let Some(ref path) = self.file_path {
            let content = self.lines.join("\n");
            if std::fs::write(path, &content).is_ok() {
                self.modified = false;
                self.status_msg = format!("Saved {}", path.display());

                // Re-parse links after save
                self.forward_links = crate::parser::links::extract_wikilinks(&content);
                self.tags = crate::parser::tags::extract_tags(&content);
                if let Some(stem) = path.file_stem() {
                    self.backlinks =
                        crate::commands::fix_links::find_backlinks(&stem.to_string_lossy());
                }
                return true;
            }
        }
        false
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_row < self.lines.len() {
            let line = &mut self.lines[self.cursor_row];
            let col = self.cursor_col.min(line.len());
            line.insert(col, c);
            self.cursor_col = col + 1;
            self.modified = true;
        }
    }

    fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let col = self.cursor_col.min(line.len());
            if col > 0 {
                line.remove(col - 1);
                self.cursor_col = col - 1;
                self.modified = true;
            }
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current);
            self.modified = true;
        }
    }

    fn delete(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_row].remove(self.cursor_col);
            self.modified = true;
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
            self.modified = true;
        }
    }

    fn enter(&mut self) {
        let line = &self.lines[self.cursor_row];
        let col = self.cursor_col.min(line.len());
        let rest = line[col..].to_string();
        self.lines[self.cursor_row].truncate(col);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
        self.modified = true;
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn home(&mut self) {
        self.cursor_col = 0;
    }

    fn end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].len();
    }

    fn clamp_col(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    /// Check if we just typed `[[` and should trigger autocomplete.
    fn check_autocomplete_trigger(&self) -> bool {
        if self.cursor_col >= 2 {
            let line = &self.lines[self.cursor_row];
            if line.len() >= self.cursor_col {
                let chars: Vec<char> = line.chars().collect();
                if self.cursor_col >= 2
                    && chars.get(self.cursor_col - 2) == Some(&'[')
                    && chars.get(self.cursor_col - 1) == Some(&'[')
                {
                    return true;
                }
            }
        }
        false
    }

    fn start_autocomplete(&mut self, note_names: &[String]) {
        self.autocomplete = Some(Autocomplete {
            all_items: note_names.to_vec(),
            filtered: note_names.to_vec(),
            selected: 0,
            query: String::new(),
            trigger_col: self.cursor_col,
        });
    }

    fn update_autocomplete_filter(&mut self) {
        if let Some(ref mut ac) = self.autocomplete {
            let lower = ac.query.to_lowercase();
            ac.filtered = if lower.is_empty() {
                ac.all_items.clone()
            } else {
                ac.all_items
                    .iter()
                    .filter(|n| n.to_lowercase().contains(&lower))
                    .cloned()
                    .collect()
            };
            if ac.selected >= ac.filtered.len() {
                ac.selected = 0;
            }
        }
    }

    fn confirm_autocomplete(&mut self) {
        if let Some(ac) = self.autocomplete.take() {
            if let Some(selected) = ac.filtered.get(ac.selected) {
                // Remove the typed query (it's after the `[[`)
                let line = &mut self.lines[self.cursor_row];
                let remove_start = ac.trigger_col;
                let remove_end = self.cursor_col.min(line.len());
                if remove_start <= remove_end && remove_end <= line.len() {
                    line.drain(remove_start..remove_end);
                    self.cursor_col = remove_start;
                }
                // Insert the selected name + closing ]]
                let insertion = format!("{selected}]]");
                let line = &mut self.lines[self.cursor_row];
                let col = self.cursor_col.min(line.len());
                line.insert_str(col, &insertion);
                self.cursor_col = col + insertion.len();
                self.modified = true;
            }
        }
    }
}

// ─── Constructor ───────────────────────────────────────────────────

impl TuiApp {
    fn new() -> Self {
        let home = mald_home();
        let config_path = home.join("config").join("config.json");
        let config = crate::config::ConfigManager::load(&config_path).ok();
        let ext_editor = config
            .as_ref()
            .and_then(|c| c.get_string("editor"))
            .unwrap_or_else(|| "nvim".into());

        let meta_path = home.join("index").join("metadata.db");
        let meta = if meta_path.exists() {
            MetadataStore::open(&meta_path).ok()
        } else {
            None
        };

        let kb_dir = home.join("kb");
        let mut kb_stats = Vec::new();
        let mut all_files: Vec<PathBuf> = Vec::new();
        if kb_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&kb_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let files = crate::fs::find_files(&entry.path(), "md").unwrap_or_default();
                        kb_stats.push((name, files.len()));
                        all_files.extend(files);
                    }
                }
            }
        }

        // Recent files
        let mut recent_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();
        for f in &all_files {
            if let Ok(meta) = f.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() < 7 * 24 * 3600 {
                            let name = f
                                .strip_prefix(&kb_dir)
                                .unwrap_or(f)
                                .to_string_lossy()
                                .replace('\\', "/");
                            recent_with_time.push((name, modified));
                        }
                    }
                }
            }
        }
        recent_with_time.sort_by(|a, b| b.1.cmp(&a.1));
        recent_with_time.truncate(10);
        let recent: Vec<String> = recent_with_time.into_iter().map(|(n, _)| n).collect();

        // Tasks
        let tasks_raw = crate::commands::dashboard::collect_open_tasks(&kb_dir);
        let tasks_home: Vec<(String, String)> = tasks_raw.iter().take(10).cloned().collect();

        let mut tasks_view: Vec<(String, String, String)> = Vec::new();
        if kb_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&kb_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        let kb_name = entry.file_name().to_string_lossy().to_string();
                        let kt = crate::commands::dashboard::collect_open_tasks(&entry.path());
                        for (note, task) in kt {
                            tasks_view.push((kb_name.clone(), note, task));
                        }
                    }
                }
            }
        }

        all_files.sort_by(|a, b| {
            let ma = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            let mb = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            mb.cmp(&ma)
        });

        let mut browse_selected = ListState::default();
        if !all_files.is_empty() {
            browse_selected.select(Some(0));
        }
        let mut tasks_selected = ListState::default();
        if !tasks_view.is_empty() {
            tasks_selected.select(Some(0));
        }

        let all_note_names = crate::commands::fix_links::collect_all_note_names();
        let daemon_running = crate::commands::daemon::is_running();

        Self {
            view: View::Home,
            search: SearchState {
                query: String::new(),
                results: Vec::new(),
                selected: ListState::default(),
                meta,
                preview: String::new(),
            },
            home: HomeState {
                kb_stats,
                recent,
                tasks: tasks_home,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            browse: BrowseState {
                files: all_files,
                selected: browse_selected,
                preview: String::new(),
                filter: String::new(),
                filtering: false,
            },
            tasks: TasksState {
                tasks: tasks_view,
                selected: tasks_selected,
            },
            editor: EditorState::new(),
            show_help: false,
            ext_editor,
            open_file_external: None,
            all_note_names,
            daemon_running,
        }
    }

    fn open_in_editor(&mut self, path: &str) {
        self.editor.load_file(path, &self.all_note_names);
        self.view = View::Edit;
    }
}

// ─── Public entry points ───────────────────────────────────────────

pub async fn run_full_tui() -> Result<()> {
    let kb_dir = mald_home().join("kb");
    if kb_dir.exists() {
        let _ = crate::daemon::indexer::fts_index_kb(&kb_dir);
    }

    let mut app = TuiApp::new();
    update_browse_preview(&mut app.browse);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_app_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Ok(()) = result {
        if let Some(path) = app.open_file_external.take() {
            std::process::Command::new(&app.ext_editor)
                .arg(&path)
                .status()?;
        }
    }

    Ok(())
}

pub fn run_search_tui() -> Result<()> {
    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");

    if !meta_path.exists() {
        let kb_dir = mald_home().join("kb");
        if kb_dir.exists() {
            crate::daemon::indexer::fts_index_kb(&kb_dir)?;
        }
    }
    if !meta_path.exists() {
        println!("No knowledge bases indexed. Run `mald init` first.");
        return Ok(());
    }

    let meta = MetadataStore::open(&meta_path)?;
    let mut state = SearchState {
        query: String::new(),
        results: Vec::new(),
        selected: ListState::default(),
        meta: Some(meta),
        preview: String::new(),
    };

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let result = run_search_loop(&mut terminal, &mut state);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Ok(Some(path)) = result {
        let config_path = mald_home().join("config").join("config.json");
        let config = crate::config::ConfigManager::load(&config_path)?;
        let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
        std::process::Command::new(&editor).arg(&path).status()?;
    }
    Ok(())
}

// ─── Main loop ─────────────────────────────────────────────────────

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut TuiApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_app(f, app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.show_help {
                    app.show_help = false;
                    continue;
                }

                // Editor autocomplete has priority
                if app.view == View::Edit && app.editor.autocomplete.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.editor.autocomplete = None;
                            continue;
                        }
                        KeyCode::Enter | KeyCode::Tab => {
                            app.editor.confirm_autocomplete();
                            continue;
                        }
                        KeyCode::Up => {
                            if let Some(ref mut ac) = app.editor.autocomplete {
                                if ac.selected > 0 {
                                    ac.selected -= 1;
                                }
                            }
                            continue;
                        }
                        KeyCode::Down => {
                            if let Some(ref mut ac) = app.editor.autocomplete {
                                if ac.selected + 1 < ac.filtered.len() {
                                    ac.selected += 1;
                                }
                            }
                            continue;
                        }
                        KeyCode::Backspace => {
                            if let Some(ref mut ac) = app.editor.autocomplete {
                                if ac.query.is_empty() {
                                    app.editor.autocomplete = None;
                                    app.editor.backspace();
                                } else {
                                    ac.query.pop();
                                    app.editor.backspace();
                                    app.editor.update_autocomplete_filter();
                                }
                            }
                            continue;
                        }
                        KeyCode::Char(c) => {
                            app.editor.insert_char(c);
                            if let Some(ref mut ac) = app.editor.autocomplete {
                                ac.query.push(c);
                            }
                            app.editor.update_autocomplete_filter();
                            continue;
                        }
                        _ => {
                            app.editor.autocomplete = None;
                        }
                    }
                    continue;
                }

                // Editor view keys
                if app.view == View::Edit {
                    // Ctrl+S = save
                    if key.code == KeyCode::Char('s')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        app.editor.save();
                        continue;
                    }
                    // Ctrl+Q = back to home
                    if key.code == KeyCode::Char('q')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if app.editor.modified {
                            app.editor.status_msg =
                                "Unsaved changes! Ctrl+S to save, Ctrl+Q again to discard.".into();
                            app.editor.modified = false; // allow second Ctrl+Q to quit
                        } else {
                            app.view = View::Home;
                        }
                        continue;
                    }
                    // Ctrl+E = open in external editor
                    if key.code == KeyCode::Char('e')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if let Some(ref path) = app.editor.file_path {
                            app.open_file_external = Some(path.to_string_lossy().to_string());
                            return Ok(());
                        }
                        continue;
                    }
                    // Tab switches view only with Ctrl
                    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.view = app.view.next();
                        continue;
                    }

                    // All other keys go to editor
                    match key.code {
                        KeyCode::Up => app.editor.move_up(),
                        KeyCode::Down => app.editor.move_down(),
                        KeyCode::Left => app.editor.move_left(),
                        KeyCode::Right => app.editor.move_right(),
                        KeyCode::Home => app.editor.home(),
                        KeyCode::End => app.editor.end(),
                        KeyCode::Backspace => app.editor.backspace(),
                        KeyCode::Delete => app.editor.delete(),
                        KeyCode::Enter => app.editor.enter(),
                        KeyCode::Tab => {
                            app.editor.insert_str("  ");
                        }
                        KeyCode::Char(c) => {
                            app.editor.insert_char(c);
                            // Check for [[ trigger
                            if c == '[' && app.editor.check_autocomplete_trigger() {
                                app.editor.start_autocomplete(&app.all_note_names);
                            }
                        }
                        KeyCode::Esc => {
                            app.view = View::Home;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Global keys (non-editor views)
                match key.code {
                    KeyCode::Char('q')
                        if app.view != View::Search || app.search.query.is_empty() =>
                    {
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        if app.view == View::Search && !app.search.query.is_empty() {
                            app.search.query.clear();
                            app.search.results.clear();
                            app.search.preview.clear();
                        } else if app.browse.filtering {
                            app.browse.filtering = false;
                        } else {
                            return Ok(());
                        }
                        continue;
                    }
                    KeyCode::Char('?') if app.view != View::Search => {
                        app.show_help = true;
                        continue;
                    }
                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.view = app.view.prev();
                        } else {
                            app.view = app.view.next();
                        }
                        continue;
                    }
                    KeyCode::Char('1') if app.view != View::Search => {
                        app.view = View::Home;
                        continue;
                    }
                    KeyCode::Char('2') if app.view != View::Search => {
                        app.view = View::Search;
                        continue;
                    }
                    KeyCode::Char('3') if app.view != View::Search => {
                        app.view = View::Browse;
                        continue;
                    }
                    KeyCode::Char('4') if app.view != View::Search => {
                        app.view = View::Tasks;
                        continue;
                    }
                    KeyCode::Char('5') if app.view != View::Search => {
                        app.view = View::Edit;
                        continue;
                    }
                    _ => {}
                }

                // View-specific keys
                match app.view {
                    View::Home => {}
                    View::Search => {
                        handle_search_key(key.code, &mut app.search);
                        if key.code == KeyCode::Enter {
                            if let Some(path) = app.search.selected_path().map(String::from) {
                                app.open_in_editor(&path);
                            }
                        }
                    }
                    View::Browse => {
                        if let Some(path) = handle_browse_key(key.code, &mut app.browse) {
                            app.open_in_editor(&path);
                        }
                    }
                    View::Tasks => {
                        if let Some(path) = handle_tasks_key(key.code, &mut app.tasks) {
                            app.open_in_editor(&path);
                        }
                    }
                    View::Edit => {} // handled above
                }
            }
        }
    }
}

fn run_search_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut SearchState,
) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| draw_search_standalone(f, state))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => return Ok(None),
                    KeyCode::Enter => return Ok(state.selected_path().map(String::from)),
                    _ => handle_search_key(key.code, state),
                }
            }
        }
    }
}

// ─── Key handlers ──────────────────────────────────────────────────

fn handle_search_key(key: KeyCode, state: &mut SearchState) {
    match key {
        KeyCode::Up => state.move_selection(-1),
        KeyCode::Down => state.move_selection(1),
        KeyCode::Backspace => {
            state.query.pop();
            state.do_search();
        }
        KeyCode::Char(c) => {
            state.query.push(c);
            state.do_search();
        }
        _ => {}
    }
}

fn handle_browse_key(key: KeyCode, state: &mut BrowseState) -> Option<String> {
    if state.filtering {
        match key {
            KeyCode::Backspace => {
                state.filter.pop();
                state.update_selection();
            }
            KeyCode::Char(c) => {
                state.filter.push(c);
                state.update_selection();
            }
            KeyCode::Enter | KeyCode::Esc => state.filtering = false,
            _ => {}
        }
        return None;
    }
    match key {
        KeyCode::Up => {
            if let Some(i) = state.selected.selected() {
                if i > 0 {
                    state.selected.select(Some(i - 1));
                    update_browse_preview(state);
                }
            }
        }
        KeyCode::Down => {
            let count = state.filtered_files().len();
            if let Some(i) = state.selected.selected() {
                if i + 1 < count {
                    state.selected.select(Some(i + 1));
                    update_browse_preview(state);
                }
            }
        }
        KeyCode::Char('/') => state.filtering = true,
        KeyCode::Char('d') => {
            let filtered = state.filtered_files();
            if let Some(i) = state.selected.selected() {
                if i < filtered.len() {
                    let path = filtered[i].clone();
                    let _ = crate::fs::trash(&path);
                    state.files.retain(|f| f != &path);
                    if state.selected.selected().unwrap_or(0) >= state.files.len()
                        && !state.files.is_empty()
                    {
                        state.selected.select(Some(state.files.len() - 1));
                    }
                    update_browse_preview(state);
                }
            }
        }
        KeyCode::Enter => {
            let filtered = state.filtered_files();
            if let Some(i) = state.selected.selected() {
                if i < filtered.len() {
                    return Some(filtered[i].to_string_lossy().to_string());
                }
            }
        }
        _ => {}
    }
    None
}

fn handle_tasks_key(key: KeyCode, state: &mut TasksState) -> Option<String> {
    match key {
        KeyCode::Up => {
            if let Some(i) = state.selected.selected() {
                if i > 0 {
                    state.selected.select(Some(i - 1));
                }
            }
        }
        KeyCode::Down => {
            if let Some(i) = state.selected.selected() {
                if i + 1 < state.tasks.len() {
                    state.selected.select(Some(i + 1));
                }
            }
        }
        KeyCode::Enter => {
            if let Some(i) = state.selected.selected() {
                if i < state.tasks.len() {
                    let (kb, note, _) = &state.tasks[i];
                    let path = mald_home().join("kb").join(kb).join(format!("{note}.md"));
                    if path.exists() {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        _ => {}
    }
    None
}

// ─── Search state ──────────────────────────────────────────────────

impl SearchState {
    fn do_search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            self.preview.clear();
            return;
        }
        let Some(meta) = &self.meta else { return };
        let fts_query: String = self
            .query
            .split_whitespace()
            .map(|w| format!("{w}*"))
            .collect::<Vec<_>>()
            .join(" ");
        self.results = meta.fts_search(&fts_query, 20).unwrap_or_default();
        if !self.results.is_empty() && self.selected.selected().is_none() {
            self.selected.select(Some(0));
        }
        if self.results.is_empty() {
            self.selected.select(None);
            self.preview.clear();
        }
        self.update_preview();
    }

    fn move_selection(&mut self, delta: i32) {
        if self.results.is_empty() {
            return;
        }
        let current = self.selected.selected().unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, self.results.len() as i32 - 1) as usize;
        self.selected.select(Some(next));
        self.update_preview();
    }

    fn update_preview(&mut self) {
        self.preview = match self.selected_path() {
            Some(path) => read_preview(path),
            None => String::new(),
        };
    }

    fn selected_path(&self) -> Option<&str> {
        self.selected
            .selected()
            .and_then(|i| self.results.get(i))
            .map(|r| r.path.as_str())
    }
}

// ─── Browse state ──────────────────────────────────────────────────

impl BrowseState {
    fn filtered_files(&self) -> Vec<PathBuf> {
        if self.filter.is_empty() {
            self.files.clone()
        } else {
            let lower = self.filter.to_lowercase();
            self.files
                .iter()
                .filter(|f| {
                    f.file_stem()
                        .map(|s| s.to_string_lossy().to_lowercase().contains(&lower))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    }

    fn update_selection(&mut self) {
        let filtered = self.filtered_files();
        if filtered.is_empty() {
            self.selected.select(None);
        } else if self.selected.selected().is_none()
            || self.selected.selected().unwrap_or(0) >= filtered.len()
        {
            self.selected.select(Some(0));
        }
        update_browse_preview(self);
    }
}

fn update_browse_preview(state: &mut BrowseState) {
    let filtered = state.filtered_files();
    state.preview = match state.selected.selected() {
        Some(i) if i < filtered.len() => read_preview(&filtered[i].to_string_lossy()),
        _ => String::new(),
    };
}

fn read_preview(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let trimmed = content.trim_start();
            if let Some(after_prefix) = trimmed.strip_prefix("---") {
                if let Some(end) = after_prefix.find("\n---") {
                    return after_prefix[end + 4..]
                        .trim_start_matches('\n')
                        .chars()
                        .take(2000)
                        .collect();
                }
            }
            content.chars().take(2000).collect()
        }
        Err(_) => "Unable to read file".to_string(),
    }
}

// ─── Drawing ───────────────────────────────────────────────────────

fn draw_app(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(1),    // content
            Constraint::Length(1), // status
        ])
        .split(f.area());

    // Tab bar with daemon indicator
    let daemon_indicator = if app.daemon_running {
        " [daemon:on]"
    } else {
        ""
    };
    let titles: Vec<Line> = View::titles()
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            if i == app.view.index() {
                Line::from(t).bold()
            } else {
                Line::from(t)
            }
        })
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" MALD v{}{} ", app.home.version, daemon_indicator)),
        )
        .select(app.view.index())
        .highlight_style(Style::default().fg(Color::Cyan).bold());
    f.render_widget(tabs, chunks[0]);

    match app.view {
        View::Home => draw_home(f, &app.home, chunks[1]),
        View::Search => draw_search_in_area(f, &mut app.search, chunks[1]),
        View::Browse => draw_browse(f, &mut app.browse, chunks[1]),
        View::Tasks => draw_tasks(f, &mut app.tasks, chunks[1]),
        View::Edit => draw_editor(f, &mut app.editor, chunks[1]),
    }

    let status_text = match app.view {
        View::Home => "  Tab cycle  1-5 jump  ? help  q quit",
        View::Search => "  Type to search  Up/Down select  Enter open in editor  Tab next",
        View::Browse => "  Up/Down  Enter edit  / filter  d trash  Tab next  q quit",
        View::Tasks => "  Up/Down  Enter open note  Tab next  q quit",
        View::Edit => "  Ctrl+S save  Ctrl+Q close  Ctrl+E ext editor  [[ autocomplete",
    };
    let status = Paragraph::new(status_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);

    if app.show_help {
        draw_help_overlay(f);
    }
}

fn draw_home(f: &mut Frame, home: &HomeState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let total_notes: usize = home.kb_stats.iter().map(|(_, c)| c).sum();
    let kbs_text: String = home
        .kb_stats
        .iter()
        .map(|(n, c)| format!("{n}: {c}"))
        .collect::<Vec<_>>()
        .join("  ");
    let top_text = format!(
        "  {} notes across {} KBs  |  {}",
        total_notes,
        home.kb_stats.len(),
        kbs_text
    );
    let top = Paragraph::new(top_text).block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(top, chunks[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let recent_items: Vec<ListItem> = home
        .recent
        .iter()
        .map(|r| ListItem::new(format!("  {r}")))
        .collect();
    let recent_list = List::new(if recent_items.is_empty() {
        vec![ListItem::new("  No recent files")]
    } else {
        recent_items
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent (7d) "),
    );
    f.render_widget(recent_list, cols[0]);

    let task_items: Vec<ListItem> = home
        .tasks
        .iter()
        .map(|(note, task)| {
            let stem = std::path::Path::new(note)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            ListItem::new(format!("  [ ] {task} ({stem})"))
        })
        .collect();
    let task_list = List::new(if task_items.is_empty() {
        vec![ListItem::new("  No open tasks")]
    } else {
        task_items
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tasks ({}) ", home.tasks.len())),
    );
    f.render_widget(task_list, cols[1]);
}

fn draw_search_standalone(f: &mut Frame, state: &mut SearchState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());
    let input = Paragraph::new(state.query.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    f.render_widget(input, chunks[0]);
    f.set_cursor_position((chunks[0].x + state.query.len() as u16 + 1, chunks[0].y + 1));
    draw_search_results(f, state, chunks[1]);
    let status = Paragraph::new("  Up/Down  Enter open  Esc quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);
}

fn draw_search_in_area(f: &mut Frame, state: &mut SearchState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);
    let input = Paragraph::new(state.query.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    f.render_widget(input, chunks[0]);
    f.set_cursor_position((chunks[0].x + state.query.len() as u16 + 1, chunks[0].y + 1));
    draw_search_results(f, state, chunks[1]);
}

fn draw_search_results(f: &mut Frame, state: &mut SearchState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = state
        .results
        .iter()
        .map(|r| {
            let title = if r.title.is_empty() {
                &r.path
            } else {
                &r.title
            };
            ListItem::new(title.as_str())
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} results ", state.results.len())),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, cols[0], &mut state.selected);

    let preview_text = if state.preview.is_empty() {
        "Select a result to preview".to_string()
    } else {
        state.preview.clone()
    };
    let preview = Paragraph::new(preview_text)
        .block(Block::default().borders(Borders::ALL).title(" Preview "))
        .wrap(Wrap { trim: false });
    f.render_widget(preview, cols[1]);
}

fn draw_browse(f: &mut Frame, state: &mut BrowseState, area: Rect) {
    let kb_dir = mald_home().join("kb");
    let filtered = state.filtered_files();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let title = if state.filtering {
        format!(" Files — /{} ", state.filter)
    } else if !state.filter.is_empty() {
        format!(" Files ({} matches) ", filtered.len())
    } else {
        format!(" Files ({}) ", filtered.len())
    };

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|f| {
            let name = f
                .strip_prefix(&kb_dir)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            ListItem::new(format!("  {name}"))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, cols[0], &mut state.selected);

    let preview_text = if state.preview.is_empty() {
        "Select a file to preview".to_string()
    } else {
        state.preview.clone()
    };
    let preview = Paragraph::new(preview_text)
        .block(Block::default().borders(Borders::ALL).title(" Preview "))
        .wrap(Wrap { trim: false });
    f.render_widget(preview, cols[1]);
}

fn draw_tasks(f: &mut Frame, state: &mut TasksState, area: Rect) {
    let items: Vec<ListItem> = state
        .tasks
        .iter()
        .map(|(kb, note, task)| ListItem::new(format!("  [ ] {task} ({kb}/{note})")))
        .collect();
    let list = List::new(if items.is_empty() {
        vec![ListItem::new("  No open tasks")]
    } else {
        items
    })
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Tasks ({}) — Enter opens note ",
        state.tasks.len()
    )))
    .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
    .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut state.selected);
}

// ─── Editor drawing ────────────────────────────────────────────────

fn draw_editor(f: &mut Frame, editor: &mut EditorState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let editor_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(cols[0]);

    // Editor area
    let editor_height = editor_chunks[0].height.saturating_sub(2) as usize;
    let _editor_width = editor_chunks[0].width.saturating_sub(6) as usize; // 4 for line nums + 2 border

    // Adjust scroll to keep cursor visible
    if editor.cursor_row < editor.scroll_y {
        editor.scroll_y = editor.cursor_row;
    }
    if editor.cursor_row >= editor.scroll_y + editor_height {
        editor.scroll_y = editor.cursor_row - editor_height + 1;
    }

    // Build styled lines with syntax highlighting
    let mut text_lines: Vec<Line> = Vec::new();
    let start = editor.scroll_y;
    let end = (start + editor_height).min(editor.lines.len());

    for i in start..end {
        let line = &editor.lines[i];
        let line_num = format!("{:>3} ", i + 1);

        let mut spans = vec![Span::styled(line_num, Style::default().fg(Color::DarkGray))];

        // Simple syntax highlighting
        let highlighted = highlight_markdown_line(line);
        spans.extend(highlighted);

        text_lines.push(Line::from(spans));
    }

    let modified_indicator = if editor.modified { " [+]" } else { "" };
    let title = match &editor.file_path {
        Some(p) => {
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            format!(" {name}{modified_indicator} ")
        }
        None => format!(" Editor{modified_indicator} "),
    };

    let editor_widget =
        Paragraph::new(text_lines).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(editor_widget, editor_chunks[0]);

    // Set cursor position
    let cursor_x = editor_chunks[0].x + 5 + (editor.cursor_col as u16);
    let cursor_y = editor_chunks[0].y + 1 + ((editor.cursor_row - editor.scroll_y) as u16);
    if cursor_y < editor_chunks[0].y + editor_chunks[0].height - 1
        && cursor_x < editor_chunks[0].x + editor_chunks[0].width - 1
    {
        f.set_cursor_position((cursor_x, cursor_y));
    }

    // Status line
    let status = format!(
        " L{} C{} | {}",
        editor.cursor_row + 1,
        editor.cursor_col + 1,
        editor.status_msg
    );
    let status_widget =
        Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(status_widget, editor_chunks[1]);

    // Sidebar: backlinks + forward links + tags
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // backlinks
            Constraint::Percentage(35), // forward links
            Constraint::Percentage(25), // tags
        ])
        .split(cols[1]);

    // Backlinks
    let bl_items: Vec<ListItem> = editor
        .backlinks
        .iter()
        .map(|(name, ctx)| {
            let display = if ctx.is_empty() {
                format!("  <- {name}")
            } else {
                let short: String = ctx.chars().take(40).collect();
                format!("  <- {name} ({short})")
            };
            ListItem::new(display)
        })
        .collect();
    let bl_list = List::new(if bl_items.is_empty() {
        vec![ListItem::new("  (no backlinks)")]
    } else {
        bl_items
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Backlinks ({}) ", editor.backlinks.len())),
    );
    f.render_widget(bl_list, sidebar_chunks[0]);

    // Forward links
    let fl_items: Vec<ListItem> = editor
        .forward_links
        .iter()
        .map(|l| ListItem::new(format!("  -> [[{l}]]")))
        .collect();
    let fl_list = List::new(if fl_items.is_empty() {
        vec![ListItem::new("  (no links)")]
    } else {
        fl_items
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Links ({}) ", editor.forward_links.len())),
    );
    f.render_widget(fl_list, sidebar_chunks[1]);

    // Tags
    let tag_text = if editor.tags.is_empty() {
        "  (no tags)".to_string()
    } else {
        editor
            .tags
            .iter()
            .map(|t| format!("  #{t}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let tags_widget = Paragraph::new(tag_text)
        .block(Block::default().borders(Borders::ALL).title(" Tags "))
        .wrap(Wrap { trim: false });
    f.render_widget(tags_widget, sidebar_chunks[2]);

    // Autocomplete popup
    if let Some(ref ac) = editor.autocomplete {
        if !ac.filtered.is_empty() {
            let popup_x = cursor_x;
            let popup_y = cursor_y + 1;
            let popup_width = 30u16.min(area.width.saturating_sub(popup_x));
            let popup_height = (ac.filtered.len() as u16 + 2).min(10);

            if popup_y + popup_height <= area.y + area.height {
                let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

                let items: Vec<ListItem> = ac
                    .filtered
                    .iter()
                    .enumerate()
                    .take(8)
                    .map(|(i, name)| {
                        if i == ac.selected {
                            ListItem::new(format!(" > {name}"))
                                .style(Style::default().bg(Color::Cyan).fg(Color::Black))
                        } else {
                            ListItem::new(format!("   {name}"))
                        }
                    })
                    .collect();

                let popup = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" [[link]] ")
                        .style(Style::default().bg(Color::Black)),
                );
                f.render_widget(Clear, popup_area);
                f.render_widget(popup, popup_area);
            }
        }
    }
}

/// Simple markdown syntax highlighting for a single line.
fn highlight_markdown_line(line: &str) -> Vec<Span<'static>> {
    let owned = line.to_string();

    // Headings
    if owned.starts_with('#') {
        return vec![Span::styled(owned, Style::default().fg(Color::Cyan).bold())];
    }

    // Task checkboxes
    if owned.trim().starts_with("- [ ] ") {
        return vec![Span::styled(owned, Style::default().fg(Color::Yellow))];
    }
    if owned.trim().starts_with("- [x] ") || owned.trim().starts_with("- [X] ") {
        return vec![Span::styled(owned, Style::default().fg(Color::DarkGray))];
    }

    // Code blocks
    if owned.trim().starts_with("```") {
        return vec![Span::styled(owned, Style::default().fg(Color::DarkGray))];
    }

    // Frontmatter
    if owned.trim() == "---" {
        return vec![Span::styled(owned, Style::default().fg(Color::DarkGray))];
    }

    // Lines with wikilinks or tags get colored spans
    if owned.contains("[[") || owned.contains('#') {
        let mut spans = Vec::new();
        let mut remaining = owned.as_str();

        while !remaining.is_empty() {
            if let Some(start) = remaining.find("[[") {
                if start > 0 {
                    spans.push(Span::raw(remaining[..start].to_string()));
                }
                if let Some(end) = remaining[start..].find("]]") {
                    let link = &remaining[start..start + end + 2];
                    spans.push(Span::styled(
                        link.to_string(),
                        Style::default().fg(Color::Green).bold(),
                    ));
                    remaining = &remaining[start + end + 2..];
                } else {
                    spans.push(Span::raw(remaining.to_string()));
                    remaining = "";
                }
            } else {
                spans.push(Span::raw(remaining.to_string()));
                remaining = "";
            }
        }
        return spans;
    }

    vec![Span::raw(owned)]
}

fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(65, 70, f.area());
    let help_text = "\
MALD Keyboard Shortcuts

  Navigation
    Tab / Shift+Tab     Cycle views
    1-5                 Jump to Home/Search/Browse/Tasks/Edit
    ?                   Toggle this help
    q / Esc             Quit (except in editor)

  Search
    Type to search      Start typing to filter notes
    Up / Down           Navigate results
    Enter               Open in editor (Edit tab)

  Browse
    Up / Down           Navigate files
    Enter               Open in editor (Edit tab)
    /                   Filter by filename
    d                   Move selected file to trash

  Tasks
    Up / Down           Navigate tasks
    Enter               Open the note containing the task

  Editor
    Type                Insert text at cursor
    [[                  Trigger wikilink autocomplete
    Arrow keys          Move cursor
    Home / End          Start / end of line
    Ctrl+S              Save file
    Ctrl+Q              Close editor (back to Home)
    Ctrl+E              Open in external editor ($EDITOR)
    Tab (in autocomplete)  Accept suggestion
    Esc (in autocomplete)  Cancel

  Muscle Memory (CLI)
    mald                Open today's daily note
    mald <text>         Smart search and open
    mald hub            Open this TUI
    mald q <text>       Quick capture
    mald f <query>      Find and open
";
    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help (press any key to close) ")
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}
