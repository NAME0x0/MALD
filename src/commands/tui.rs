use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::stdout;

use crate::fs::mald_home;
use crate::index::metadata::{FtsResult, MetadataStore};

struct SearchState {
    query: String,
    results: Vec<FtsResult>,
    selected: ListState,
    meta: MetadataStore,
    preview: String,
}

impl SearchState {
    fn new(meta: MetadataStore) -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: ListState::default(),
            meta,
            preview: String::new(),
        }
    }

    fn search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            self.preview.clear();
            return;
        }
        let fts_query: String = self
            .query
            .split_whitespace()
            .map(|w| format!("{}*", w))
            .collect::<Vec<_>>()
            .join(" ");
        self.results = self.meta.fts_search(&fts_query, 20).unwrap_or_default();
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
            Some(path) => {
                // Read file content for preview
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        // Strip frontmatter for cleaner preview
                        let trimmed = content.trim_start();
                        if trimmed.starts_with("---") {
                            if let Some(end) = trimmed[3..].find("\n---") {
                                trimmed[3 + end + 4..]
                                    .trim_start_matches('\n')
                                    .chars()
                                    .take(2000)
                                    .collect()
                            } else {
                                content.chars().take(2000).collect()
                            }
                        } else {
                            content.chars().take(2000).collect()
                        }
                    }
                    Err(_) => "Unable to read file".to_string(),
                }
            }
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
    let mut state = SearchState::new(meta);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_tui_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Ok(Some(path)) = result {
        let config_path = mald_home().join("config").join("config.json");
        let config = crate::config::ConfigManager::load(&config_path)?;
        let editor = config
            .get_string("editor")
            .unwrap_or_else(|| "nvim".into());
        std::process::Command::new(&editor).arg(&path).status()?;
    }

    Ok(())
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut SearchState,
) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| draw(f, state))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => return Ok(None),
                    KeyCode::Enter => {
                        return Ok(state.selected_path().map(String::from));
                    }
                    KeyCode::Up => state.move_selection(-1),
                    KeyCode::Down => state.move_selection(1),
                    KeyCode::Backspace => {
                        state.query.pop();
                        state.search();
                    }
                    KeyCode::Char(c) => {
                        state.query.push(c);
                        state.search();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn draw(f: &mut Frame, state: &mut SearchState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Min(1),   // results + preview
            Constraint::Length(1), // status
        ])
        .split(f.area());

    // Search bar
    let input = Paragraph::new(state.query.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    f.render_widget(input, main_chunks[0]);

    // Cursor position
    f.set_cursor_position((
        main_chunks[0].x + state.query.len() as u16 + 1,
        main_chunks[0].y + 1,
    ));

    // Split middle into results (left) and preview (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // results list
            Constraint::Percentage(60), // preview pane
        ])
        .split(main_chunks[1]);

    // Results list
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
        .highlight_symbol("▸ ");

    f.render_stateful_widget(list, content_chunks[0], &mut state.selected);

    // Preview pane
    let preview_text = if state.preview.is_empty() {
        "Select a result to preview".to_string()
    } else {
        state.preview.clone()
    };

    let preview = Paragraph::new(preview_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Preview "),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(preview, content_chunks[1]);

    // Status bar
    let status = Paragraph::new("  ↑↓ navigate  Enter open  Esc quit  Tab preview")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, main_chunks[2]);
}
