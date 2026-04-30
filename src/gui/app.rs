//! Main application state and view composition.
//!
//! Implements a VSCode+Obsidian hybrid layout with:
//! - Activity bar (48px, leftmost)
//! - Sidebar (250px, mode-dependent content)
//! - Main editor area (tabs, search, content)
//! - Feature panel (300px, backlinks/AI/outline)
//! - Terminal (200px, bottom, collapsible)
//! - Status bar (24px, bottom)

use iced::widget::{
    column, container, mouse_area, pane_grid, row, scrollable, text, text_editor, text_input, Row,
    Space,
};
use iced::{font, keyboard, mouse, Element, Length, Subscription, Task as IcedTask, Theme};
use iced_anim::widget::button;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::gui::animations::{AnimationState, ModalAnimation};
use crate::gui::canvas::graph::{self, PhysicsSimulation};
use crate::gui::components::{
    activity_bar, feature_panel, sidebar_content, tab_bar, terminal_panel, top_search,
};
use crate::gui::icons;
use crate::gui::layout::{Direction, LayoutState, PaneContent};
use crate::gui::message::*;
use crate::gui::syntax::SyntaxHighlighter;
use crate::gui::theme::{self, colors, MaldTheme};
use crate::gui::widgets::code_cell::{CellStatus, CodeCell};
use crate::gui::widgets::markdown_view;
use crate::gui::widgets::status_bar;
use crate::parser::MarkdownDocument;
use iced_aw::iced_fonts::BOOTSTRAP_FONT_BYTES;

// Layout constants — use theme tokens
use crate::gui::theme::layout;
const SIDEBAR_DEFAULT_WIDTH: f32 = layout::SIDEBAR_DEFAULT_WIDTH;
const FEATURE_PANEL_DEFAULT_WIDTH: f32 = layout::FEATURE_PANEL_DEFAULT_WIDTH;
const TERMINAL_DEFAULT_HEIGHT: f32 = layout::TERMINAL_DEFAULT_HEIGHT;
const SIDEBAR_RESIZE_HANDLE_WIDTH: f32 = 8.0;

struct TerminalSession {
    handle: crate::gui::util::pty::PtyHandle,
    output: Receiver<Vec<u8>>,
}

enum AiStreamEvent {
    Chunk(String),
    Finished(String),
    Error(String),
}

#[derive(Debug, Clone)]
enum EditorPreviewPane {
    Source,
    Preview,
}

/// Top-level application state (Elm Model).
pub struct MaldApp {
    // ── Theme ──
    pub mald_theme: MaldTheme,

    // ── Layout state ──
    pub layout: LayoutState,
    pub active_view: ActiveView,

    // ── Activity bar & Sidebar ──
    pub activity_mode: ActivityMode,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,

    // ── Feature panel ──
    pub feature_panel_visible: bool,
    pub feature_panel_width: f32,
    pub feature_panel_content: FeaturePanelContent,

    // ── Top search ──
    pub top_search_query: String,
    pub top_search_focused: bool,

    // ── Terminal ──
    pub terminal_visible: bool,
    pub terminal_height: f32,
    pub terminal_input: String,
    pub terminal_lines: Vec<String>,
    pub terminal_partial_line: String,
    terminal_session: Option<TerminalSession>,

    // ── File tree ──
    pub file_tree_entries: Vec<FileEntry>,
    pub expanded_dirs: HashSet<PathBuf>,

    // ── Editor ──
    pub open_tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub editor_content: text_editor::Content,
    editor_preview_panes: pane_grid::State<EditorPreviewPane>,
    markdown_preview: iced::widget::markdown::Content,
    pub cursor_line: usize,
    pub cursor_col: usize,

    // ── Autocomplete ──
    pub autocomplete_visible: bool,
    pub autocomplete_query: String,
    pub autocomplete_results: Vec<String>,
    pub autocomplete_selected: usize,

    // ── AI Chat ──
    pub ai_chat_messages: Vec<(String, String)>, // (role, content)
    pub ai_chat_input: String,
    pub ai_streaming: bool,
    ai_stream_receiver: Option<Receiver<AiStreamEvent>>,

    // ── Search ──
    pub search_visible: bool,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,

    // ── Command palette ──
    pub palette_visible: bool,
    pub palette_query: String,
    pub palette_commands: Vec<PaletteCommand>,
    pub palette_filtered: Vec<usize>,
    pub palette_selected: usize,

    // ── Graph ──
    pub graph_nodes: Vec<GraphNode>,
    pub graph_edges: Vec<GraphEdge>,
    pub graph_zoom: f32,
    pub graph_pan: (f32, f32),

    // ── Tasks ──
    pub tasks: Vec<TaskItem>,
    pub tasks_kanban: bool,

    // ── Backlinks ──
    pub backlinks: Vec<BacklinkEntry>,

    // ── Outline ──
    pub outline: Vec<OutlineEntry>,

    // ── Daemon ──
    pub daemon_status: DaemonStatus,

    // ── Indexer footer ──
    pub index_stats: Option<IndexStats>,

    // ── Keybindings help ──
    pub keybindings_visible: bool,

    // ── New note modal ──
    pub new_note_visible: bool,
    pub new_note_title: String,
    pub new_note_path: String,
    pub new_note_kb: String,

    // ── KB info ──
    pub current_kb: String,
    pub known_kbs: Vec<String>,
    pub detected_editors: Vec<crate::commands::launch::DetectedEditor>,
    pub mald_shell_available: bool,
    pub settings_form: GuiSettingsForm,

    // â”€â”€ Animations â”€â”€
    pub sidebar_animation: Option<AnimationState>,
    pub terminal_animation: Option<AnimationState>,
    pub feature_panel_animation: Option<AnimationState>,

    // â”€â”€ Graph physics â”€â”€
    pub graph_simulation: PhysicsSimulation,
    pub graph_settings_visible: bool,
    pub graph_hide_orphans: bool,

    // ── Unsaved close confirmation ──
    pub pending_close_tab: Option<usize>,

    // â"€â"€ Code cells & syntax highlighting â"€â"€
    pub code_cells: Vec<CodeCell>,
    pub syntax_highlighter: SyntaxHighlighter,

    // â"€â"€ Hover states â"€â"€
    pub hovered_button: Option<String>,

    // ── Toast notifications ──
    pub toasts: Vec<crate::gui::widgets::toast::Toast>,
    pub toast_counter: usize,

    // â"€â"€ Modal animation â"€â"€
    pub modal_animation: Option<ModalAnimation>,
    pub modal_closing_kind: Option<ModalKind>,

    // â"€â"€ Activity bar pulse â"€â"€
    pub activity_pulse_mode: Option<ActivityMode>,
    pub activity_pulse_start: Option<Instant>,

    // â"€â"€ Command palette selection â"€â"€
    pub palette_prev_selected: usize,
    pub palette_select_time: Option<Instant>,

    // ── Async task generation counters (stale result prevention) ──
    pub search_generation: u64,
    pub backlinks_generation: u64,
    pub graph_generation: u64,
    pub tasks_generation: u64,

    // ── Search debounce ──
    pub last_search_dispatch: Option<Instant>,
}

/// Which modal overlay is currently being animated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    Palette,
    Search,
    Keybindings,
    NewNote,
}

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub original_content: String,
    pub modified: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll: f32,
}

impl MaldApp {
    pub fn new() -> (Self, IcedTask<Message>) {
        let (mut editor_preview_panes, source_pane) =
            pane_grid::State::new(EditorPreviewPane::Source);
        if let Some((_preview_pane, split)) = editor_preview_panes.split(
            pane_grid::Axis::Vertical,
            source_pane,
            EditorPreviewPane::Preview,
        ) {
            editor_preview_panes.resize(split, 0.5);
        }

        let current_kb = load_default_kb_name();
        let known_kbs = workspace_kbs();
        let detected_editors = crate::commands::launch::detected_editors();
        let mald_shell_available = crate::commands::setup::mald_on_path();
        let settings_form = load_settings_form();
        let palette_commands = Self::all_commands(&current_kb, &known_kbs);

        let app = Self {
            mald_theme: MaldTheme::default(),
            layout: LayoutState::default_editor_layout(),
            active_view: ActiveView::Home,

            activity_mode: ActivityMode::Files,
            sidebar_visible: true,
            sidebar_width: SIDEBAR_DEFAULT_WIDTH,
            sidebar_resizing: false,

            feature_panel_visible: false,
            feature_panel_width: FEATURE_PANEL_DEFAULT_WIDTH,
            feature_panel_content: FeaturePanelContent::Backlinks,

            top_search_query: String::new(),
            top_search_focused: false,

            terminal_visible: false,
            terminal_height: TERMINAL_DEFAULT_HEIGHT,
            terminal_input: String::new(),
            terminal_lines: Vec::new(),
            terminal_partial_line: String::new(),
            terminal_session: None,

            file_tree_entries: Vec::new(),
            expanded_dirs: HashSet::new(),

            open_tabs: Vec::new(),
            active_tab: 0,
            editor_content: text_editor::Content::new(),
            editor_preview_panes,
            markdown_preview: markdown_view::parse_markdown(""),
            cursor_line: 1,
            cursor_col: 1,

            autocomplete_visible: false,
            autocomplete_query: String::new(),
            autocomplete_results: Vec::new(),
            autocomplete_selected: 0,

            ai_chat_messages: Vec::new(),
            ai_chat_input: String::new(),
            ai_streaming: false,
            ai_stream_receiver: None,

            search_visible: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,

            palette_visible: false,
            palette_query: String::new(),
            palette_commands,
            palette_filtered: Vec::new(),
            palette_selected: 0,

            graph_nodes: Vec::new(),
            graph_edges: Vec::new(),
            graph_zoom: 1.0,
            graph_pan: (0.0, 0.0),

            tasks: Vec::new(),
            tasks_kanban: false,

            backlinks: Vec::new(),
            outline: Vec::new(),

            daemon_status: DaemonStatus::Unknown,
            index_stats: None,
            keybindings_visible: false,
            new_note_visible: false,
            new_note_title: String::new(),
            new_note_path: String::new(),
            new_note_kb: current_kb.clone(),
            current_kb,
            known_kbs,
            detected_editors,
            mald_shell_available,
            settings_form,

            sidebar_animation: None,
            terminal_animation: None,
            feature_panel_animation: None,

            graph_simulation: PhysicsSimulation::default(),
            graph_settings_visible: false,
            graph_hide_orphans: false,
            pending_close_tab: None,

            code_cells: Vec::new(),
            syntax_highlighter: SyntaxHighlighter::new(),

            hovered_button: None,

            toasts: Vec::new(),
            toast_counter: 0,

            modal_animation: None,
            modal_closing_kind: None,

            activity_pulse_mode: None,
            activity_pulse_start: None,

            palette_prev_selected: 0,
            palette_select_time: None,

            search_generation: 0,
            backlinks_generation: 0,
            graph_generation: 1,
            tasks_generation: 1,

            last_search_dispatch: None,
        };

        // Load file tree + icon font on startup
        let file_tree_task = IcedTask::perform(
            load_file_tree_for(Some(app.current_kb.clone())),
            Message::FileTreeLoaded,
        );
        let graph_generation = app.graph_generation;
        let graph_task = IcedTask::perform(
            load_graph_for(Some(app.current_kb.clone())),
            move |(nodes, edges)| Message::GraphLoaded(nodes, edges, graph_generation),
        );
        let tasks_generation = app.tasks_generation;
        let tasks_task =
            IcedTask::perform(load_tasks_for(Some(app.current_kb.clone())), move |tasks| {
                Message::TasksLoaded(tasks, tasks_generation)
            });
        let font_task = font::load(BOOTSTRAP_FONT_BYTES).map(|_| Message::Noop);
        let daemon_task = IcedTask::perform(load_daemon_status(), Message::DaemonStatusUpdate);
        let index_stats_task = IcedTask::perform(load_index_stats(), Message::IndexStatsLoaded);
        let task = IcedTask::batch([
            file_tree_task,
            graph_task,
            tasks_task,
            font_task,
            daemon_task,
            index_stats_task,
        ]);

        (app, task)
    }

    pub fn title(state: &Self) -> String {
        let tab_title = state
            .open_tabs
            .get(state.active_tab)
            .map(|t| t.title.as_str())
            .unwrap_or("Home");
        format!("MALD — {tab_title}")
    }

    pub fn theme(&self) -> Theme {
        self.mald_theme.iced_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                handle_key_press(key, modifiers)
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::SidebarResize(position.x))
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::SidebarResizeEnd)
            }
            _ => None,
        });
        let mut subscriptions = vec![keys];

        if self.has_active_animation() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(16))
                    .map(|_| Message::AnimationTick(Instant::now())),
            );
        }

        if self.needs_runtime_tick() {
            subscriptions.push(iced::time::every(Duration::from_millis(50)).map(|_| Message::Tick));
        }

        subscriptions
            .push(iced::time::every(Duration::from_secs(2)).map(|_| Message::DaemonStatusRefresh));

        Subscription::batch(subscriptions)
    }

    pub fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            // ── Navigation ──
            Message::SwitchView(view) => {
                self.active_view = view;
            }
            Message::GoHome => {
                self.active_view = ActiveView::Home;
                self.activity_mode = ActivityMode::Files;
                self.search_visible = false;
                self.modal_animation = None;
                self.modal_closing_kind = None;
            }

            // ── Activity Bar ──
            Message::ActivityBarSelect(mode) => {
                // Trigger pulse animation on mode switch
                self.activity_pulse_mode = Some(mode);
                self.activity_pulse_start = Some(Instant::now());

                self.activity_mode = mode;
                // Auto-show sidebar when clicking activity bar
                if !self.sidebar_visible {
                    self.sidebar_visible = true;
                }
                // Switch view for some modes
                match mode {
                    ActivityMode::Graph => {
                        self.active_view = ActiveView::Graph;
                        self.graph_generation += 1;
                        let gen = self.graph_generation;
                        let kb_name = self.current_kb.clone();
                        return IcedTask::perform(
                            load_graph_for(Some(kb_name)),
                            move |(nodes, edges)| Message::GraphLoaded(nodes, edges, gen),
                        );
                    }
                    ActivityMode::Tasks => {
                        self.active_view = ActiveView::Tasks;
                        self.tasks_generation += 1;
                        let gen = self.tasks_generation;
                        let kb_name = self.current_kb.clone();
                        return IcedTask::perform(load_tasks_for(Some(kb_name)), move |tasks| {
                            Message::TasksLoaded(tasks, gen)
                        });
                    }
                    ActivityMode::Search => {
                        self.search_visible = true;
                        self.modal_animation =
                            Some(ModalAnimation::open(theme::animation::MODAL_FADE_IN));
                        self.modal_closing_kind = None;
                    }
                    _ => {}
                }
            }

            // ── Sidebar ──
            Message::SidebarToggle => {
                let current = self.sidebar_draw_width();
                let target = if current > 1.0 {
                    0.0
                } else {
                    self.sidebar_width
                };
                // Velocity-aware: duration scales with distance
                self.sidebar_animation = Some(AnimationState::velocity_aware(
                    current,
                    target,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                if target > 0.0 {
                    self.sidebar_visible = true;
                }
            }
            Message::SidebarResizeStart => {
                self.sidebar_resizing = true;
                self.sidebar_animation = None;
                self.sidebar_visible = true;
            }
            Message::SidebarResize(width) => {
                if self.sidebar_resizing {
                    let adjusted_width =
                        width - layout::ACTIVITY_BAR_WIDTH - SIDEBAR_RESIZE_HANDLE_WIDTH / 2.0;
                    self.sidebar_width =
                        adjusted_width.clamp(layout::SIDEBAR_MIN_WIDTH, layout::SIDEBAR_MAX_WIDTH);
                }
            }
            Message::SidebarResizeEnd => {
                self.sidebar_resizing = false;
            }

            // ── Feature Panel ──
            Message::FeaturePanelToggle => {
                let current = self.feature_panel_draw_width();
                let target = if current > 1.0 {
                    0.0
                } else {
                    self.feature_panel_width
                };
                // Velocity-aware: duration scales with distance
                self.feature_panel_animation = Some(AnimationState::velocity_aware(
                    current,
                    target,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                if target > 0.0 {
                    self.feature_panel_visible = true;
                }
            }
            Message::FeaturePanelSetContent(content) => {
                self.feature_panel_content = content;
                if !self.feature_panel_visible {
                    let current = self.feature_panel_draw_width();
                    self.feature_panel_animation = Some(AnimationState::velocity_aware(
                        current,
                        self.feature_panel_width,
                        theme::animation::PANEL_PIXELS_PER_SEC,
                    ));
                    self.feature_panel_visible = true;
                }
            }
            Message::FeaturePanelResize(width) => {
                self.feature_panel_width = width.clamp(
                    layout::FEATURE_PANEL_MIN_WIDTH,
                    layout::FEATURE_PANEL_MAX_WIDTH,
                );
            }

            // ── Top Search ──
            Message::TopSearchFocus => {
                self.top_search_focused = true;
            }
            Message::TopSearchBlur => {
                self.top_search_focused = false;
            }
            Message::TopSearchQueryChanged(q) => {
                self.top_search_query = q.clone();
                self.search_query = q.clone();
                if q.len() >= 2 {
                    // Debounce: skip if less than 150ms since last dispatch
                    let now = Instant::now();
                    let should_dispatch = self
                        .last_search_dispatch
                        .map(|t| now.duration_since(t) >= theme::animation::SEARCH_DEBOUNCE)
                        .unwrap_or(true);
                    if should_dispatch {
                        self.last_search_dispatch = Some(now);
                        self.search_generation += 1;
                        let gen = self.search_generation;
                        return IcedTask::perform(perform_search(q), move |results| {
                            Message::SearchResults(results, gen)
                        });
                    }
                }
            }
            Message::TopSearchSubmit => {
                if !self.top_search_query.is_empty() {
                    return IcedTask::done(Message::SearchOpen);
                }
            }

            // ── File tree ──
            Message::FileTreeToggle => {
                let current = self.sidebar_draw_width();
                let target = if current > 1.0 {
                    0.0
                } else {
                    self.sidebar_width
                };
                self.sidebar_animation = Some(AnimationState::velocity_aware(
                    current,
                    target,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                if target > 0.0 {
                    self.sidebar_visible = true;
                }
            }
            Message::FileTreeLoaded(mut entries) => {
                // Restore expanded state from our persistent set
                for entry in &mut entries {
                    if entry.is_dir {
                        entry.expanded = self.expanded_dirs.contains(&entry.path);
                    }
                }
                // Auto-expand root KB dirs on first load
                if self.expanded_dirs.is_empty() {
                    for entry in &mut entries {
                        if entry.is_dir && entry.depth == 0 {
                            entry.expanded = true;
                            self.expanded_dirs.insert(entry.path.clone());
                        }
                    }
                }
                self.file_tree_entries = entries;
            }
            Message::FileTreeSelect(path) => {
                return IcedTask::done(Message::EditorOpen(path));
            }
            Message::FileTreeExpand(path) => {
                self.expanded_dirs.insert(path.clone());
                for entry in &mut self.file_tree_entries {
                    if entry.path == path {
                        entry.expanded = true;
                    }
                }
            }
            Message::FileTreeCollapse(path) => {
                self.expanded_dirs.remove(&path);
                for entry in &mut self.file_tree_entries {
                    if entry.path == path {
                        entry.expanded = false;
                    }
                }
            }
            Message::FileTreeRefresh => {
                let kb_name = self.current_kb.clone();
                return IcedTask::perform(load_file_tree_for(Some(kb_name)), |entries| {
                    Message::FileTreeLoaded(entries)
                });
            }

            // ── Editor ──
            Message::EditorOpen(path) => {
                let mut follow_up = Vec::new();
                if let Some(kb_name) = kb_name_for_path(&path) {
                    let space_changed = kb_name != self.current_kb;
                    self.sync_current_kb(kb_name);
                    if space_changed {
                        let kb_name = self.current_kb.clone();
                        follow_up.push(IcedTask::perform(
                            load_file_tree_for(Some(kb_name.clone())),
                            Message::FileTreeLoaded,
                        ));
                        self.graph_generation += 1;
                        let graph_gen = self.graph_generation;
                        follow_up.push(IcedTask::perform(
                            load_graph_for(Some(kb_name.clone())),
                            move |(nodes, edges)| Message::GraphLoaded(nodes, edges, graph_gen),
                        ));
                        self.tasks_generation += 1;
                        let tasks_gen = self.tasks_generation;
                        follow_up.push(IcedTask::perform(
                            load_tasks_for(Some(kb_name)),
                            move |tasks| Message::TasksLoaded(tasks, tasks_gen),
                        ));
                    }
                }
                // Check if already open
                if let Some(idx) = self.open_tabs.iter().position(|t| t.path == path) {
                    self.active_tab = idx;
                    let content_str = self.open_tabs[idx].content.clone();
                    self.editor_content = text_editor::Content::with_text(&content_str);
                    self.restore_editor_tab_state();
                    self.refresh_code_cells(&content_str);
                    self.refresh_markdown_preview(&content_str);
                    self.outline = extract_outline(&content_str);
                    self.active_view = ActiveView::Editor;
                    if !follow_up.is_empty() {
                        return IcedTask::batch(follow_up);
                    }
                } else {
                    // Async file read — prevents blocking GUI on large/slow files
                    let p = path.clone();
                    follow_up.push(IcedTask::perform(
                        async move {
                            match tokio::fs::read_to_string(&p).await {
                                Ok(content) => (p, Ok(content)),
                                Err(e) => {
                                    let msg = format!("Failed to open {}: {}", p.display(), e);
                                    (p, Err(msg))
                                }
                            }
                        },
                        |(path, result)| Message::EditorFileLoaded(path, result),
                    ));
                    return IcedTask::batch(follow_up);
                }
            }
            Message::EditorFileLoaded(path, result) => {
                match result {
                    Ok(content) => {
                        let title = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "untitled".into());

                        // Extract outline from content
                        self.outline = extract_outline(&content);

                        self.open_tabs.push(EditorTab {
                            path: path.clone(),
                            title,
                            content: content.clone(),
                            original_content: content.clone(),
                            modified: false,
                            cursor_line: 1,
                            cursor_col: 1,
                            scroll: 0.0,
                        });
                        self.active_tab = self.open_tabs.len() - 1;
                        self.editor_content = text_editor::Content::with_text(&content);
                        self.restore_editor_tab_state();
                        self.refresh_code_cells(&content);
                        self.refresh_markdown_preview(&content);
                        self.active_view = ActiveView::Editor;

                        // Load backlinks for this file
                        self.backlinks_generation += 1;
                        let gen = self.backlinks_generation;
                        return IcedTask::perform(load_backlinks(path), move |backlinks| {
                            Message::BacklinksLoaded(backlinks, gen)
                        });
                    }
                    Err(e) => {
                        // Show error toast instead of silently opening empty tab
                        return IcedTask::done(Message::ErrorOccurred(e));
                    }
                }
            }
            Message::EditorClose(idx) => {
                if idx < self.open_tabs.len() {
                    // If modified, show confirmation modal instead of closing
                    if self.open_tabs[idx].modified {
                        self.pending_close_tab = Some(idx);
                        return IcedTask::none();
                    }
                    self.close_tab(idx);
                }
            }
            Message::EditorCloseConfirmSave => {
                if let Some(idx) = self.pending_close_tab.take() {
                    if idx < self.open_tabs.len() {
                        let tab = &mut self.open_tabs[idx];
                        let path = tab.path.clone();
                        let content = tab.content.clone();
                        tab.original_content = content.clone();
                        tab.modified = false;
                        let close_idx = idx;
                        return IcedTask::perform(save_file(path, content), move |result| {
                            match result {
                                Ok(()) => Message::EditorCloseAfterSave(close_idx),
                                Err(e) => Message::EditorSaved(Err(e)),
                            }
                        });
                    }
                }
            }
            Message::EditorCloseConfirmDiscard => {
                if let Some(idx) = self.pending_close_tab.take() {
                    self.close_tab(idx);
                }
            }
            Message::EditorCloseConfirmCancel => {
                self.pending_close_tab = None;
            }
            Message::EditorCloseAfterSave(idx) => {
                self.close_tab(idx);
            }
            Message::EditorSwitchTab(idx) => {
                if idx < self.open_tabs.len() {
                    let mut follow_up = Vec::new();
                    // Save current editor text back to tab
                    if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                        tab.content = self.editor_content.text();
                    }
                    self.active_tab = idx;
                    if let Some(kb_name) = kb_name_for_path(&self.open_tabs[idx].path) {
                        let space_changed = kb_name != self.current_kb;
                        self.sync_current_kb(kb_name);
                        if space_changed {
                            let kb_name = self.current_kb.clone();
                            follow_up.push(IcedTask::perform(
                                load_file_tree_for(Some(kb_name.clone())),
                                Message::FileTreeLoaded,
                            ));
                            self.graph_generation += 1;
                            let graph_gen = self.graph_generation;
                            follow_up.push(IcedTask::perform(
                                load_graph_for(Some(kb_name.clone())),
                                move |(nodes, edges)| Message::GraphLoaded(nodes, edges, graph_gen),
                            ));
                            self.tasks_generation += 1;
                            let tasks_gen = self.tasks_generation;
                            follow_up.push(IcedTask::perform(
                                load_tasks_for(Some(kb_name)),
                                move |tasks| Message::TasksLoaded(tasks, tasks_gen),
                            ));
                        }
                    }
                    let content_str = self.open_tabs[idx].content.clone();
                    self.editor_content = text_editor::Content::with_text(&content_str);
                    self.restore_editor_tab_state();
                    self.refresh_code_cells(&content_str);
                    self.refresh_markdown_preview(&content_str);

                    // Update outline
                    self.outline = extract_outline(&content_str);
                    if !follow_up.is_empty() {
                        return IcedTask::batch(follow_up);
                    }
                }
            }
            Message::EditorContentChanged(action) => {
                let is_edit = action.is_edit();
                self.editor_content.perform(action);
                self.sync_editor_cursor_state();
                // Only sync content on actual edits (not cursor moves)
                if is_edit {
                    let content = self.editor_content.text();
                    if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                        tab.content = content.clone();
                        tab.modified = tab.content != tab.original_content;
                    }
                    self.refresh_code_cells(&content);
                    self.refresh_markdown_preview(&content);
                    // Update outline
                    self.outline = extract_outline(&content);
                }
            }
            Message::EditorSave => {
                // Sync latest editor text to tab before saving
                if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                    tab.content = self.editor_content.text();
                }
                if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                    let path = tab.path.clone();
                    let content = tab.content.clone();
                    tab.original_content = content.clone();
                    tab.modified = false;
                    return IcedTask::perform(save_file(path, content), |result| match result {
                        Ok(()) => Message::EditorSaved(Ok(())),
                        Err(e) => Message::EditorSaved(Err(e)),
                    });
                }
            }
            Message::EditorSaved(result) => {
                if let Err(e) = result {
                    return IcedTask::done(Message::ErrorOccurred(format!("Save failed: {e}")));
                }
            }
            Message::EditorExternalOpen => {
                if let Some(tab) = self.open_tabs.get(self.active_tab) {
                    let _ =
                        std::process::Command::new(if cfg!(windows) { "cmd" } else { "xdg-open" })
                            .args(if cfg!(windows) {
                                vec!["/C", "start", ""]
                            } else {
                                vec![]
                            })
                            .arg(&tab.path)
                            .spawn();
                }
            }

            // ── Autocomplete ──
            Message::AutocompleteTriggered(query) => {
                self.autocomplete_visible = true;
                self.autocomplete_query = query.clone();
                return IcedTask::perform(fuzzy_match_notes(query), |results| {
                    Message::AutocompleteResults(results)
                });
            }
            Message::AutocompleteResults(results) => {
                self.autocomplete_results = results;
                self.autocomplete_selected = 0;
            }
            Message::AutocompleteSelect(name) => {
                self.autocomplete_visible = false;
                let insertion = format!("{name}]]");
                // Insert text at current cursor position via Edit actions
                for ch in insertion.chars() {
                    self.editor_content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Insert(ch)));
                }
                let content = self.editor_content.text();
                if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                    tab.content = content.clone();
                    tab.modified = tab.content != tab.original_content;
                }
                self.refresh_code_cells(&content);
            }
            Message::AutocompleteDismiss => {
                self.autocomplete_visible = false;
            }

            // ── Terminal ──
            Message::TerminalToggle => {
                let current = self.terminal_draw_height();
                let target = if current > 1.0 {
                    0.0
                } else {
                    self.terminal_height
                };
                self.terminal_animation = Some(AnimationState::velocity_aware(
                    current,
                    target,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                if target > 0.0 {
                    self.terminal_visible = true;
                    if let Err(error) = self.ensure_terminal_session() {
                        return IcedTask::done(Message::ErrorOccurred(error));
                    }
                }
            }
            Message::TerminalInput(input) => {
                self.terminal_input = input;
            }
            Message::TerminalSubmit => {
                let command = self.terminal_input.trim().to_string();
                if command.is_empty() {
                    return IcedTask::none();
                }

                self.show_terminal_panel();
                if let Err(error) = self.ensure_terminal_session() {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
                if let Some(session) = self.terminal_session.as_mut() {
                    if let Err(error) = session.handle.submit(&command) {
                        return IcedTask::done(Message::ErrorOccurred(error.to_string()));
                    }
                }
                self.terminal_input.clear();
            }
            Message::TerminalClear => {
                self.terminal_lines.clear();
                self.terminal_partial_line.clear();
            }
            Message::TerminalRestart => {
                if let Err(error) = self.restart_terminal_session() {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            }
            Message::TerminalInterrupt => {
                if let Some(session) = self.terminal_session.as_mut() {
                    if let Err(error) = session.handle.interrupt() {
                        return IcedTask::done(Message::ErrorOccurred(error.to_string()));
                    }
                }
            }
            Message::TerminalResized { cols, rows } => {
                if let Some(session) = self.terminal_session.as_ref() {
                    if let Err(error) = session.handle.resize(cols, rows) {
                        return IcedTask::done(Message::ErrorOccurred(error.to_string()));
                    }
                }
            }
            Message::TerminalHeightChanged(h) => {
                self.terminal_height =
                    h.clamp(layout::TERMINAL_MIN_HEIGHT, layout::TERMINAL_MAX_HEIGHT);
            }

            // â”€â”€ Code Execution â”€â”€
            Message::RunCodeBlock(block_id) => {
                if let Some(cell) = self.code_cells.iter_mut().find(|c| c.id == block_id) {
                    cell.status = CellStatus::Running;
                    cell.output = None;
                    let lang = cell.language.clone();
                    let code = cell.code.clone();
                    return IcedTask::perform(
                        crate::commands::run::run_inline_block(lang, code),
                        move |result| match result {
                            Ok(output) => Message::CodeBlockOutput { block_id, output },
                            Err(error) => Message::CodeBlockError { block_id, error },
                        },
                    );
                }
            }
            Message::CodeBlockOutput { block_id, output } => {
                if let Some(cell) = self.code_cells.iter_mut().find(|c| c.id == block_id) {
                    cell.output = Some(output);
                    cell.status = CellStatus::Success;
                }
            }
            Message::CodeBlockError { block_id, error } => {
                if let Some(cell) = self.code_cells.iter_mut().find(|c| c.id == block_id) {
                    cell.output = Some(error);
                    cell.status = CellStatus::Error;
                }
            }
            Message::CodeBlockComplete(block_id) => {
                if let Some(cell) = self.code_cells.iter_mut().find(|c| c.id == block_id) {
                    cell.status = CellStatus::Idle;
                }
            }

            // ── AI Chat ──
            Message::AiChatToggle => {
                if self.feature_panel_visible
                    && self.feature_panel_content == FeaturePanelContent::AIChat
                {
                    let current = self.feature_panel_draw_width();
                    self.feature_panel_animation = Some(AnimationState::velocity_aware(
                        current,
                        0.0,
                        theme::animation::PANEL_PIXELS_PER_SEC,
                    ));
                } else {
                    self.feature_panel_content = FeaturePanelContent::AIChat;
                    if !self.feature_panel_visible {
                        let current = self.feature_panel_draw_width();
                        self.feature_panel_animation = Some(AnimationState::velocity_aware(
                            current,
                            self.feature_panel_width,
                            theme::animation::PANEL_PIXELS_PER_SEC,
                        ));
                    }
                    self.feature_panel_visible = true;
                }
            }
            Message::AiChatInputChanged(input) => {
                self.ai_chat_input = input;
            }
            Message::AiChatSend(msg) => {
                let trimmed = msg.trim().to_string();
                if trimmed.is_empty() || self.ai_streaming {
                    return IcedTask::none();
                }

                self.ai_chat_messages.push(("user".into(), trimmed.clone()));
                self.ai_chat_input.clear();
                self.ai_streaming = true;
                self.ai_stream_receiver =
                    Some(spawn_ai_chat_stream(trimmed, self.current_kb.clone()));
                self.ai_chat_messages
                    .push(("assistant".into(), String::new()));
                // Cap chat history to prevent unbounded memory growth
                if self.ai_chat_messages.len() > theme::limits::AI_CHAT_MESSAGES_MAX {
                    let excess = self.ai_chat_messages.len() - theme::limits::AI_CHAT_MESSAGES_MAX;
                    self.ai_chat_messages.drain(..excess);
                }
            }
            Message::AiChatCitationClick(path) => {
                return IcedTask::done(Message::EditorOpen(path));
            }
            Message::MarkdownLinkClick(target) => {
                let current_path = self
                    .open_tabs
                    .get(self.active_tab)
                    .map(|tab| tab.path.as_path());

                if let Some(path) =
                    resolve_markdown_link_target(current_path, &self.current_kb, &target)
                {
                    return IcedTask::done(Message::EditorOpen(path));
                }

                if is_external_url(&target) {
                    if let Err(error) = open_external_target(&target) {
                        return IcedTask::done(Message::ErrorOccurred(error));
                    }
                } else {
                    return IcedTask::done(Message::ErrorOccurred(format!(
                        "Could not resolve link target: {target}"
                    )));
                }
            }

            // ── Search ──
            Message::SearchOpen => {
                self.search_visible = true;
                self.activity_mode = ActivityMode::Search;
                self.sidebar_visible = true;
                self.modal_animation = Some(ModalAnimation::open(theme::animation::MODAL_FADE_IN));
                self.modal_closing_kind = None;
            }
            Message::SearchClose => {
                self.search_visible = false;
                self.activity_mode = ActivityMode::Files;
                self.modal_animation = None;
                self.modal_closing_kind = None;
            }
            Message::SearchQueryChanged(q) => {
                self.search_query = q.clone();
                if q.len() >= 2 {
                    // Debounce: skip if less than 150ms since last dispatch
                    let now = Instant::now();
                    let should_dispatch = self
                        .last_search_dispatch
                        .map(|t| now.duration_since(t) >= theme::animation::SEARCH_DEBOUNCE)
                        .unwrap_or(true);
                    if should_dispatch {
                        self.last_search_dispatch = Some(now);
                        self.search_generation += 1;
                        let gen = self.search_generation;
                        return IcedTask::perform(perform_search(q), move |results| {
                            Message::SearchResults(results, gen)
                        });
                    }
                } else {
                    self.search_results.clear();
                }
            }
            Message::SearchResults(results, gen) => {
                // Discard stale results from superseded search tasks
                if gen == self.search_generation {
                    self.search_results = results;
                    self.search_selected = 0;
                }
            }
            Message::SearchResultSelect(idx) => {
                if let Some(result) = self.search_results.get(idx) {
                    self.search_visible = false;
                    self.activity_mode = ActivityMode::Files;
                    self.modal_animation = None;
                    self.modal_closing_kind = None;
                    let path = result.path.clone();
                    return IcedTask::done(Message::EditorOpen(path));
                }
            }

            // ── Command palette ──
            Message::CommandPaletteOpen => {
                self.rebuild_palette_commands();
                self.palette_visible = true;
                self.palette_query.clear();
                self.palette_filtered = (0..self.palette_commands.len()).collect();
                self.palette_selected = 0;
                self.modal_animation = Some(ModalAnimation::open(theme::animation::MODAL_FADE_IN));
                self.modal_closing_kind = None;
            }
            Message::CommandPaletteClose => {
                // If any modal is open, start close animation
                if self.palette_visible {
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::Palette);
                } else if self.search_visible {
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::Search);
                } else if self.keybindings_visible {
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::Keybindings);
                } else if self.new_note_visible {
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::NewNote);
                }
            }
            Message::CommandPaletteQueryChanged(q) => {
                self.palette_query = q.clone();
                self.palette_filtered = self
                    .palette_commands
                    .iter()
                    .enumerate()
                    .filter(|(_, cmd)| {
                        let q_lower = q.to_lowercase();
                        cmd.label.to_lowercase().contains(&q_lower)
                            || cmd.description.to_lowercase().contains(&q_lower)
                    })
                    .map(|(i, _)| i)
                    .collect();
                self.palette_selected = 0;
            }
            Message::CommandPaletteSelect(idx) => {
                if let Some(&cmd_idx) = self.palette_filtered.get(idx) {
                    let cmd = self.palette_commands[cmd_idx].clone();
                    self.palette_visible = false;
                    // Start close animation
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::Palette);
                    return IcedTask::done(Message::CommandPaletteExecute(cmd));
                }
            }
            Message::CommandPaletteExecute(cmd) => {
                return self.execute_palette_command(&cmd);
            }
            Message::CommandPaletteUp => {
                if self.palette_visible && !self.palette_filtered.is_empty() {
                    self.palette_prev_selected = self.palette_selected;
                    self.palette_select_time = Some(Instant::now());
                    if self.palette_selected > 0 {
                        self.palette_selected -= 1;
                    } else {
                        // Wrap to bottom
                        self.palette_selected = self.palette_filtered.len().min(15) - 1;
                    }
                }
            }
            Message::CommandPaletteDown => {
                if self.palette_visible && !self.palette_filtered.is_empty() {
                    self.palette_prev_selected = self.palette_selected;
                    self.palette_select_time = Some(Instant::now());
                    let max = self.palette_filtered.len().min(15) - 1;
                    if self.palette_selected < max {
                        self.palette_selected += 1;
                    } else {
                        // Wrap to top
                        self.palette_selected = 0;
                    }
                }
            }
            Message::CommandPaletteSubmit => {
                if self.new_note_visible {
                    return IcedTask::done(Message::NewNoteCreate(self.new_note_title.clone()));
                }
                if self.palette_visible {
                    return IcedTask::done(Message::CommandPaletteSelect(self.palette_selected));
                }
            }

            // ── Graph ──
            Message::GraphToggle => {
                if self.active_view == ActiveView::Graph {
                    self.active_view = ActiveView::Home;
                } else {
                    self.active_view = ActiveView::Graph;
                    self.activity_mode = ActivityMode::Graph;
                    self.graph_generation += 1;
                    let gen = self.graph_generation;
                    let kb_name = self.current_kb.clone();
                    return IcedTask::perform(
                        load_graph_for(Some(kb_name)),
                        move |(nodes, edges)| Message::GraphLoaded(nodes, edges, gen),
                    );
                }
            }
            Message::GraphLoaded(nodes, edges, gen) => {
                if gen == self.graph_generation {
                    self.graph_nodes = nodes;
                    self.graph_edges = edges;
                    self.graph_zoom = 1.0;
                    self.graph_pan = (0.0, 0.0);
                    self.graph_simulation.alpha = 1.0;
                }
            }
            Message::GraphNodeClick(path) => {
                return IcedTask::done(Message::EditorOpen(path));
            }
            Message::GraphZoom(delta) => {
                self.graph_zoom = (self.graph_zoom + delta * 0.1).clamp(0.1, 5.0);
                // Gently restart simulation so nodes settle into new view
                if self.graph_simulation.alpha < 0.1 {
                    self.graph_simulation.alpha = 0.1;
                }
            }
            Message::GraphPan { dx, dy } => {
                self.graph_pan.0 += dx;
                self.graph_pan.1 += dy;
                // Soft alpha bump: nodes gently readjust during pan
                if self.graph_simulation.alpha < 0.05 {
                    self.graph_simulation.alpha = 0.05;
                }
            }
            Message::GraphSettingsToggle => {
                self.graph_settings_visible = !self.graph_settings_visible;
            }
            Message::GraphRepelForceChanged(v) => {
                self.graph_simulation.charge_strength = v;
                self.graph_simulation.alpha = 1.0; // full restart for responsive controls
            }
            Message::EditorPreviewSplitChanged(event) => {
                self.editor_preview_panes
                    .resize(event.split, event.ratio.clamp(0.2, 0.8));
            }
            Message::GraphLinkForceChanged(v) => {
                self.graph_simulation.link_strength = v;
                self.graph_simulation.alpha = 1.0;
            }
            Message::GraphLinkDistanceChanged(v) => {
                self.graph_simulation.link_distance = v;
                self.graph_simulation.alpha = 1.0;
            }
            Message::GraphCenterForceChanged(v) => {
                self.graph_simulation.center_strength = v;
                self.graph_simulation.alpha = 1.0;
            }
            Message::GraphPhysicsReset => {
                self.graph_simulation = PhysicsSimulation::default();
                self.graph_simulation.alpha = 1.0;
            }
            Message::GraphViewReset => {
                self.graph_zoom = 1.0;
                self.graph_pan = (0.0, 0.0);
                self.graph_simulation.alpha = 1.0;
            }
            Message::GraphHideOrphansToggle => {
                self.graph_hide_orphans = !self.graph_hide_orphans;
            }

            // ── Tasks ──
            Message::TasksToggle => {
                if self.active_view == ActiveView::Tasks {
                    self.active_view = ActiveView::Home;
                } else {
                    self.active_view = ActiveView::Tasks;
                    self.activity_mode = ActivityMode::Tasks;
                    self.tasks_generation += 1;
                    let gen = self.tasks_generation;
                    let kb_name = self.current_kb.clone();
                    return IcedTask::perform(load_tasks_for(Some(kb_name)), move |tasks| {
                        Message::TasksLoaded(tasks, gen)
                    });
                }
            }
            Message::TasksLoaded(tasks, gen) => {
                if gen == self.tasks_generation {
                    self.tasks = tasks;
                }
            }
            Message::TaskClick(idx) => {
                if let Some(task) = self.tasks.get(idx) {
                    let path = task.path.clone();
                    return IcedTask::done(Message::EditorOpen(path));
                }
            }
            Message::TaskToggleView => {
                self.tasks_kanban = !self.tasks_kanban;
            }

            // ── Backlinks ──
            Message::BacklinksLoaded(entries, gen) => {
                if gen == self.backlinks_generation {
                    self.backlinks = entries;
                }
            }
            Message::BacklinkClick(path) => {
                return IcedTask::done(Message::EditorOpen(path));
            }

            // ── Outline ──
            Message::OutlineLoaded(entries) => {
                self.outline = entries;
            }
            Message::OutlineClick(idx) => {
                if let Some(entry) = self.outline.get(idx) {
                    self.jump_editor_to_line(entry.line);
                }
            }

            // ── Layout ──
            Message::PaneSplitVertical => {
                self.layout
                    .split(Direction::Horizontal, PaneContent::Editor);
            }
            Message::PaneSplitHorizontal => {
                self.layout.split(Direction::Vertical, PaneContent::Editor);
            }
            Message::PaneClose => {
                self.layout.close_focused();
            }
            Message::PaneResize { ratio, .. } => {
                self.layout.resize_focused(ratio);
            }
            Message::PaneFocus(id) => {
                self.layout.focused = crate::gui::layout::PaneId(id);
            }

            // ── Theme ──
            Message::ThemeToggle => {
                self.mald_theme.toggle();
            }

            // ── Keybindings help ──
            Message::KeybindingsToggle => {
                if self.keybindings_visible {
                    // Start close animation
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::Keybindings);
                } else {
                    self.keybindings_visible = true;
                    self.modal_animation =
                        Some(ModalAnimation::open(theme::animation::MODAL_FADE_IN));
                    self.modal_closing_kind = None;
                }
            }

            // ── Daemon ──
            Message::DaemonStatusRefresh => {
                return IcedTask::perform(load_daemon_status(), Message::DaemonStatusUpdate);
            }
            Message::DaemonStatusUpdate(status) => {
                self.daemon_status = status;
            }

            // ── Settings ──
            Message::SettingChanged(key, value) => {
                match key.as_str() {
                    "editor" => self.settings_form.editor = value,
                    "default_kb" => self.settings_form.default_kb = value,
                    "ai.default_model" => self.settings_form.ai_model = value,
                    "ai.ollama_url" => self.settings_form.ollama_url = value,
                    "ai.embedding_model" => self.settings_form.embedding_model = value,
                    "session.shell" => self.settings_form.shell = value,
                    _ => {}
                }
                self.settings_form.dirty = true;
            }
            Message::SettingToggle(key) => {
                if key == "daemon.auto_start" {
                    self.settings_form.daemon_auto_start = !self.settings_form.daemon_auto_start;
                    self.settings_form.dirty = true;
                }
            }
            Message::SettingsSave => {
                self.settings_form.saving = true;
                return IcedTask::perform(
                    save_settings_form(self.settings_form.clone()),
                    Message::SettingsSaved,
                );
            }
            Message::SettingsReset => {
                self.settings_form = load_settings_form();
                self.refresh_workspace_cache();
                self.refresh_runtime_cache();
                self.rebuild_palette_commands();
            }
            Message::SettingsSaved(result) => match result {
                Ok(saved) => {
                    let old_shell = self.settings_form.shell.clone();
                    self.settings_form = saved;
                    self.refresh_runtime_cache();

                    if self.open_tabs.is_empty() {
                        self.sync_current_kb(self.settings_form.default_kb.clone());
                    } else {
                        self.rebuild_palette_commands();
                    }

                    if self.settings_form.shell != old_shell && self.terminal_session.is_some() {
                        if let Err(error) = self.restart_terminal_session() {
                            return IcedTask::done(Message::ErrorOccurred(error));
                        }
                    }

                    return IcedTask::done(Message::ToastShow {
                        level: ToastLevel::Success,
                        message: "Settings saved".into(),
                    });
                }
                Err(error) => {
                    self.settings_form.saving = false;
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            },
            Message::PathSetupRun => {
                return IcedTask::perform(repair_shell_path(), Message::PathSetupCompleted);
            }
            Message::PathSetupCompleted(result) => match result {
                Ok(_) => {
                    self.mald_shell_available = true;
                    return IcedTask::done(Message::ToastShow {
                        level: ToastLevel::Success,
                        message:
                            "MALD is ready for Command Prompt and PowerShell. Open a new terminal to use `mald` everywhere."
                                .into(),
                    });
                }
                Err(error) => {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            },
            Message::CurrentKbSwitch(name) => {
                let trimmed = name.trim().to_string();
                if trimmed.is_empty() {
                    return IcedTask::done(Message::ErrorOccurred(
                        "Space name cannot be empty".into(),
                    ));
                }
                if trimmed == self.current_kb {
                    return IcedTask::done(Message::ToastShow {
                        level: ToastLevel::Info,
                        message: format!("Already working in `{trimmed}`"),
                    });
                }
                return IcedTask::perform(
                    save_default_kb_name(trimmed),
                    Message::CurrentKbSwitched,
                );
            }
            Message::CurrentKbSwitched(result) => match result {
                Ok(kb_name) => {
                    self.refresh_workspace_cache();
                    self.sync_current_kb(kb_name.clone());
                    self.graph_generation += 1;
                    let graph_gen = self.graph_generation;
                    self.tasks_generation += 1;
                    let tasks_gen = self.tasks_generation;
                    return IcedTask::batch([
                        IcedTask::perform(
                            load_file_tree_for(Some(kb_name.clone())),
                            Message::FileTreeLoaded,
                        ),
                        IcedTask::perform(load_graph_for(Some(kb_name.clone())), move |(nodes, edges)| {
                            Message::GraphLoaded(nodes, edges, graph_gen)
                        }),
                        IcedTask::perform(load_tasks_for(Some(kb_name.clone())), move |tasks| {
                            Message::TasksLoaded(tasks, tasks_gen)
                        }),
                        IcedTask::done(Message::ToastShow {
                            level: ToastLevel::Success,
                            message: format!(
                                "Working space set to `{kb_name}`. Files, tasks, graph, and new notes now follow it."
                            ),
                        }),
                    ]);
                }
                Err(error) => {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            },
            Message::DemoSpaceOpen => {
                return IcedTask::perform(open_demo_space(), Message::DemoSpaceOpened);
            }
            Message::DemoSpaceOpened(result) => match result {
                Ok(path) => {
                    self.refresh_workspace_cache();
                    self.sync_current_kb("demo".into());
                    self.graph_generation += 1;
                    let graph_gen = self.graph_generation;
                    self.tasks_generation += 1;
                    let tasks_gen = self.tasks_generation;
                    return IcedTask::batch([
                        IcedTask::perform(load_file_tree_for(Some("demo".into())), Message::FileTreeLoaded),
                        IcedTask::perform(load_graph_for(Some("demo".into())), move |(nodes, edges)| {
                            Message::GraphLoaded(nodes, edges, graph_gen)
                        }),
                        IcedTask::perform(load_tasks_for(Some("demo".into())), move |tasks| {
                            Message::TasksLoaded(tasks, tasks_gen)
                        }),
                        IcedTask::done(Message::EditorOpen(path)),
                        IcedTask::done(Message::ToastShow {
                            level: ToastLevel::Success,
                            message: "Demo space loaded. Explore freely and switch back whenever you want."
                                .into(),
                        }),
                    ]);
                }
                Err(error) => {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            },

            // ── New Note ──
            Message::NewNotePrompt => {
                self.new_note_visible = true;
                self.new_note_title.clear();
                self.new_note_path.clear();
                self.new_note_kb = self.current_kb.clone();
                self.modal_animation = Some(ModalAnimation::open(theme::animation::MODAL_FADE_IN));
                self.modal_closing_kind = None;
            }
            Message::NewNoteTitleChanged(title) => {
                self.new_note_title = title;
            }
            Message::NewNotePathChanged(path) => {
                self.new_note_path = path;
            }
            Message::NewNoteKbSelected(kb_name) => {
                self.new_note_kb = kb_name;
            }
            Message::NewNoteCreate(title) => {
                let trimmed = title.trim().to_string();
                if trimmed.is_empty() {
                    return IcedTask::done(Message::ErrorOccurred(
                        "Note title cannot be empty".into(),
                    ));
                }
                let target_kb = self.new_note_kb.trim().to_string();
                if target_kb.is_empty() {
                    return IcedTask::done(Message::ErrorOccurred(
                        "Choose a space before creating the note".into(),
                    ));
                }
                let target_path = self.new_note_path.trim().to_string();
                return IcedTask::perform(
                    create_new_note(
                        trimmed,
                        target_kb,
                        (!target_path.is_empty()).then_some(target_path),
                    ),
                    Message::NewNoteCreated,
                );
            }
            Message::NewNoteCreated(result) => match result {
                Ok(path) => {
                    let target_kb =
                        kb_name_for_path(&path).unwrap_or_else(|| self.current_kb.clone());
                    self.sync_current_kb(target_kb.clone());
                    self.new_note_title.clear();
                    self.new_note_path.clear();
                    self.new_note_kb = target_kb.clone();
                    self.modal_animation =
                        Some(ModalAnimation::close(theme::animation::MODAL_FADE_OUT));
                    self.modal_closing_kind = Some(ModalKind::NewNote);
                    self.graph_generation += 1;
                    let graph_gen = self.graph_generation;
                    self.tasks_generation += 1;
                    let tasks_gen = self.tasks_generation;
                    return IcedTask::batch([
                        IcedTask::perform(
                            load_file_tree_for(Some(target_kb.clone())),
                            Message::FileTreeLoaded,
                        ),
                        IcedTask::perform(
                            load_graph_for(Some(target_kb.clone())),
                            move |(nodes, edges)| Message::GraphLoaded(nodes, edges, graph_gen),
                        ),
                        IcedTask::perform(load_tasks_for(Some(target_kb.clone())), move |tasks| {
                            Message::TasksLoaded(tasks, tasks_gen)
                        }),
                        IcedTask::done(Message::EditorOpen(path)),
                        IcedTask::done(Message::ToastShow {
                            level: ToastLevel::Success,
                            message: format!("New note created in `{target_kb}`"),
                        }),
                    ]);
                }
                Err(error) => {
                    return IcedTask::done(Message::ErrorOccurred(error));
                }
            },
            Message::ReindexCompleted(result) => {
                self.show_terminal_panel();
                match result {
                    Ok(count) => {
                        self.push_terminal_line(format!(
                            "Reindexed search store: {count} file(s)."
                        ));

                        let mut tasks = vec![
                            IcedTask::done(Message::ToastShow {
                                level: ToastLevel::Success,
                                message: format!("Reindexed {count} file(s)"),
                            }),
                            IcedTask::done(Message::IndexStatsRefresh),
                        ];

                        if !self.search_query.trim().is_empty() {
                            self.search_generation += 1;
                            let generation = self.search_generation;
                            let query = self.search_query.clone();
                            tasks.push(IcedTask::perform(perform_search(query), move |results| {
                                Message::SearchResults(results, generation)
                            }));
                        }

                        return IcedTask::batch(tasks);
                    }
                    Err(error) => {
                        self.push_terminal_line(format!("Reindex failed: {error}"));
                        return IcedTask::done(Message::ErrorOccurred(error));
                    }
                }
            }
            Message::IndexStatsRefresh => {
                return IcedTask::perform(load_index_stats(), Message::IndexStatsLoaded);
            }
            Message::IndexStatsLoaded(stats) => {
                self.index_stats = stats;
                return IcedTask::none();
            }
            Message::DoctorCompleted(result) => {
                self.show_terminal_panel();
                match result {
                    Ok(summary) => {
                        self.push_terminal_output(&summary.output);

                        let (level, message) = if summary.issues > 0 {
                            (
                                ToastLevel::Error,
                                format!(
                                    "Doctor found {} issue(s) and {} warning(s)",
                                    summary.issues, summary.warnings
                                ),
                            )
                        } else if summary.warnings > 0 {
                            (
                                ToastLevel::Warning,
                                format!("Doctor found {} warning(s)", summary.warnings),
                            )
                        } else {
                            (ToastLevel::Success, "Doctor passed all checks".into())
                        };

                        return IcedTask::done(Message::ToastShow { level, message });
                    }
                    Err(error) => {
                        self.push_terminal_line(format!("Doctor failed: {error}"));
                        return IcedTask::done(Message::ErrorOccurred(error));
                    }
                }
            }

            // ── Misc ──
            Message::Tick => {
                self.poll_runtime_channels();
            }
            Message::AnimationTick(_) => {
                if let Some(anim) = &self.sidebar_animation {
                    if anim.is_complete() {
                        if anim.to <= 1.0 {
                            self.sidebar_visible = false;
                        }
                        self.sidebar_animation = None;
                    }
                }
                if let Some(anim) = &self.feature_panel_animation {
                    if anim.is_complete() {
                        if anim.to <= 1.0 {
                            self.feature_panel_visible = false;
                        }
                        self.feature_panel_animation = None;
                    }
                }
                if let Some(anim) = &self.terminal_animation {
                    if anim.is_complete() {
                        if anim.to <= 1.0 {
                            self.terminal_visible = false;
                        }
                        self.terminal_animation = None;
                    }
                }

                if self.active_view == ActiveView::Graph && self.graph_simulation.is_active() {
                    self.graph_simulation
                        .tick(&mut self.graph_nodes, &self.graph_edges);
                }

                // Activity bar pulse processing (300ms duration)
                if let Some(start) = self.activity_pulse_start {
                    if start.elapsed().as_millis() > 300 {
                        self.activity_pulse_mode = None;
                        self.activity_pulse_start = None;
                    }
                }

                // Palette selection animation cleanup (150ms)
                if let Some(start) = self.palette_select_time {
                    if start.elapsed().as_millis() > 150 {
                        self.palette_select_time = None;
                    }
                }

                // Modal animation processing
                if let Some(ref anim) = self.modal_animation {
                    if anim.is_complete() {
                        // Close animation finished: clear visibility
                        if !anim.opening {
                            if let Some(kind) = self.modal_closing_kind.take() {
                                match kind {
                                    ModalKind::Palette => self.palette_visible = false,
                                    ModalKind::Search => self.search_visible = false,
                                    ModalKind::Keybindings => self.keybindings_visible = false,
                                    ModalKind::NewNote => self.new_note_visible = false,
                                }
                            }
                        }
                        self.modal_animation = None;
                    }
                }

                // Toast animation processing
                // 1. Check for auto-dismiss (start exit animation)
                for toast in &mut self.toasts {
                    if toast.should_auto_dismiss() {
                        toast.start_exit();
                    }
                }
                // 2. Tick animations and remove completed exit animations
                self.toasts.retain_mut(|toast| !toast.tick());
            }
            Message::SidebarAnimateCollapse => {
                let current = self.sidebar_draw_width();
                self.sidebar_animation = Some(AnimationState::velocity_aware(
                    current,
                    0.0,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
            }
            Message::SidebarAnimateExpand => {
                let current = self.sidebar_draw_width();
                self.sidebar_animation = Some(AnimationState::velocity_aware(
                    current,
                    self.sidebar_width,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                self.sidebar_visible = true;
            }
            Message::TerminalAnimateCollapse => {
                let current = self.terminal_draw_height();
                self.terminal_animation = Some(AnimationState::velocity_aware(
                    current,
                    0.0,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
            }
            Message::TerminalAnimateExpand => {
                let current = self.terminal_draw_height();
                self.terminal_animation = Some(AnimationState::velocity_aware(
                    current,
                    self.terminal_height,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                self.terminal_visible = true;
            }
            Message::FeaturePanelAnimateCollapse => {
                let current = self.feature_panel_draw_width();
                self.feature_panel_animation = Some(AnimationState::velocity_aware(
                    current,
                    0.0,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
            }
            Message::FeaturePanelAnimateExpand => {
                let current = self.feature_panel_draw_width();
                self.feature_panel_animation = Some(AnimationState::velocity_aware(
                    current,
                    self.feature_panel_width,
                    theme::animation::PANEL_PIXELS_PER_SEC,
                ));
                self.feature_panel_visible = true;
            }
            Message::Noop => {}
            Message::ErrorOccurred(e) => {
                // Show error toast (cap at max to prevent memory exhaustion)
                if self.toasts.len() >= theme::limits::TOASTS_MAX {
                    self.toasts.remove(0); // Evict oldest
                }
                self.toast_counter += 1;
                self.toasts.push(crate::gui::widgets::toast::Toast::error(
                    self.toast_counter,
                    e,
                ));
            }
            Message::ToastShow { level, message } => {
                self.toast_counter += 1;
                let toast = match level {
                    crate::gui::message::ToastLevel::Info => {
                        crate::gui::widgets::toast::Toast::info(self.toast_counter, message)
                    }
                    crate::gui::message::ToastLevel::Success => {
                        crate::gui::widgets::toast::Toast::success(self.toast_counter, message)
                    }
                    crate::gui::message::ToastLevel::Warning => {
                        crate::gui::widgets::toast::Toast::warning(self.toast_counter, message)
                    }
                    crate::gui::message::ToastLevel::Error => {
                        crate::gui::widgets::toast::Toast::error(self.toast_counter, message)
                    }
                };
                // Cap toasts to prevent memory exhaustion from rapid errors
                if self.toasts.len() >= theme::limits::TOASTS_MAX {
                    self.toasts.remove(0); // Evict oldest
                }
                self.toasts.push(toast);
            }
            Message::ToastDismiss(id) => {
                // Start exit animation instead of immediate removal
                if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
                    toast.start_exit();
                }
            }
            Message::ToastTimeout(id) => {
                self.toasts.retain(|t| t.id != id);
            }
        }

        IcedTask::none()
    }

    fn has_active_animation(&self) -> bool {
        self.sidebar_animation.is_some()
            || self.feature_panel_animation.is_some()
            || self.terminal_animation.is_some()
            || self.modal_animation.is_some()
            || self.activity_pulse_start.is_some()
            || self.palette_select_time.is_some()
            || (self.active_view == ActiveView::Graph && self.graph_simulation.is_active())
            || self.toasts.iter().any(|t| t.animation.is_some())
    }

    fn needs_runtime_tick(&self) -> bool {
        self.terminal_session.is_some() || self.ai_stream_receiver.is_some()
    }

    /// Get the current modal overlay opacity based on animation state
    fn modal_overlay_opacity(&self) -> f32 {
        match &self.modal_animation {
            Some(anim) => {
                let (overlay_opacity, _scale, _content_opacity) = anim.values();
                overlay_opacity
            }
            None => colors::OVERLAY_BG.a, // Full opacity when no animation
        }
    }

    fn sidebar_draw_width(&self) -> f32 {
        if let Some(anim) = &self.sidebar_animation {
            anim.value()
        } else if self.sidebar_visible {
            self.sidebar_width
        } else {
            0.0
        }
    }

    fn feature_panel_draw_width(&self) -> f32 {
        if let Some(anim) = &self.feature_panel_animation {
            anim.value()
        } else if self.feature_panel_visible {
            self.feature_panel_width
        } else {
            0.0
        }
    }

    fn terminal_draw_height(&self) -> f32 {
        if let Some(anim) = &self.terminal_animation {
            anim.value()
        } else if self.terminal_visible {
            self.terminal_height
        } else {
            0.0
        }
    }

    fn close_tab(&mut self, idx: usize) {
        if idx < self.open_tabs.len() {
            self.open_tabs.remove(idx);
            if self.active_tab >= self.open_tabs.len() && !self.open_tabs.is_empty() {
                self.active_tab = self.open_tabs.len() - 1;
            }
            if self.open_tabs.is_empty() {
                self.editor_content = text_editor::Content::new();
                self.active_view = ActiveView::Home;
                self.outline.clear();
                self.backlinks.clear();
                self.code_cells.clear();
                self.markdown_preview = markdown_view::parse_markdown("");
            } else {
                if let Some(kb_name) = kb_name_for_path(&self.open_tabs[self.active_tab].path) {
                    self.sync_current_kb(kb_name);
                }
                let content_str = self.open_tabs[self.active_tab].content.clone();
                self.editor_content = text_editor::Content::with_text(&content_str);
                self.restore_editor_tab_state();
                self.refresh_code_cells(&content_str);
                self.refresh_markdown_preview(&content_str);
                self.outline = extract_outline(&content_str);
            }
        }
    }

    fn refresh_code_cells(&mut self, content: &str) {
        let doc = MarkdownDocument::parse(content);
        let mut new_cells: Vec<CodeCell> = Vec::new();

        for (i, block) in doc.code_blocks.iter().enumerate() {
            let mut cell = CodeCell::new(i, block.language.clone(), block.content.clone());
            if let Some(existing) = self.code_cells.get(i) {
                if existing.language == cell.language && existing.code == cell.code {
                    cell.output = existing.output.clone();
                    cell.status = existing.status;
                }
            }
            new_cells.push(cell);
        }

        self.code_cells = new_cells;
    }

    fn refresh_markdown_preview(&mut self, content: &str) {
        let is_markdown = self
            .open_tabs
            .get(self.active_tab)
            .and_then(|tab| tab.path.extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);

        self.markdown_preview = if is_markdown {
            markdown_view::parse_markdown(content)
        } else {
            markdown_view::parse_markdown("")
        };
    }

    // ══════════════════════════════════════════════════════════════════════════
    // View Composition
    // ══════════════════════════════════════════════════════════════════════════

    pub fn view(&self) -> Element<'_, Message> {
        // Build the main layout row
        let mut main_row = Row::new().spacing(0);

        // 1. Activity bar (always visible, 48px)
        main_row = main_row.push(activity_bar::view(
            self.activity_mode,
            self.active_view == ActiveView::Home,
            self.activity_pulse_mode,
            self.activity_pulse_start,
            self.mald_theme.is_dark,
        ));

        // 2. Sidebar (collapsible, 250px default)
        let sidebar_width = self.sidebar_draw_width();
        if self.sidebar_visible || self.sidebar_animation.is_some() {
            let sidebar = self.view_sidebar();
            main_row = main_row.push(
                container(sidebar)
                    .width(Length::Fixed(sidebar_width))
                    .height(Length::Fill),
            );

            let handle_color = if self.sidebar_resizing {
                iced::Color {
                    a: 0.45,
                    ..theme::themed(
                        &self.mald_theme.iced_theme(),
                        colors::ACCENT,
                        colors::latte::ACCENT,
                    )
                }
            } else {
                iced::Color {
                    a: 0.14,
                    ..theme::themed(
                        &self.mald_theme.iced_theme(),
                        colors::SURFACE2,
                        colors::latte::SURFACE2,
                    )
                }
            };

            let resize_handle = mouse_area(
                container(Space::new())
                    .width(Length::Fixed(SIDEBAR_RESIZE_HANDLE_WIDTH))
                    .height(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(handle_color)),
                        ..Default::default()
                    }),
            )
            .on_press(Message::SidebarResizeStart)
            .interaction(mouse::Interaction::ResizingHorizontally);

            main_row = main_row.push(resize_handle);
        }

        // 3. Main content area (fills remaining space)
        let main_content = self.view_main_content();
        main_row = main_row.push(
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
        );

        // 4. Feature panel (collapsible, 300px default)
        let feature_panel_width = self.feature_panel_draw_width();
        if self.feature_panel_visible || self.feature_panel_animation.is_some() {
            let panel = feature_panel::view(
                self.feature_panel_content,
                &self.backlinks,
                &self.outline,
                &self.ai_chat_messages,
                &self.ai_chat_input,
                self.ai_streaming,
                self.mald_theme.is_dark,
            );
            main_row = main_row.push(
                container(panel)
                    .width(Length::Fixed(feature_panel_width))
                    .height(Length::Fill),
            );
        }

        // 5. Status bar (always at bottom, 24px)
        let word_count = self
            .open_tabs
            .get(self.active_tab)
            .map(|t| t.content.split_whitespace().count())
            .unwrap_or(0);

        let note_count = self.file_tree_entries.iter().filter(|e| !e.is_dir).count();

        let status = status_bar::view(
            self.daemon_status,
            &self.current_kb,
            self.cursor_line,
            self.cursor_col,
            word_count,
            note_count,
            self.mald_theme.is_dark,
        );

        let main = column![main_row, status].into();

        // Handle overlays
        if self.palette_visible {
            return self.view_with_palette(main);
        }
        if self.search_visible {
            return self.view_with_search_overlay(main);
        }
        if self.new_note_visible {
            return self.view_with_new_note(main);
        }
        if self.keybindings_visible {
            return self.view_with_keybindings(main);
        }
        if self.pending_close_tab.is_some() {
            return self.view_with_unsaved_close(main);
        }

        // Add toast overlay if there are toasts
        if !self.toasts.is_empty() {
            return self.view_with_toasts(main);
        }

        main
    }

    fn view_with_toasts<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let toasts = crate::gui::widgets::toast::view_toasts(&self.toasts, self.mald_theme.is_dark);

        // Position toasts at bottom-right
        let toast_layer = container(toasts)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom);

        iced::widget::stack![base, toast_layer].into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let theme = self.mald_theme.iced_theme();
        let modified_paths: std::collections::HashSet<std::path::PathBuf> = self
            .open_tabs
            .iter()
            .filter(|t| t.modified)
            .map(|t| t.path.clone())
            .collect();
        sidebar_content::view(sidebar_content::SidebarData {
            mode: self.activity_mode,
            file_entries: &self.file_tree_entries,
            search_query: &self.search_query,
            search_results: &self.search_results,
            graph_nodes: &self.graph_nodes,
            tasks: &self.tasks,
            ai_messages: &self.ai_chat_messages,
            ai_input: &self.ai_chat_input,
            settings: &self.settings_form,
            known_kbs: &self.known_kbs,
            detected_editors: &self.detected_editors,
            mald_shell_available: self.mald_shell_available,
            theme,
            modified_paths,
            index_stats: self.index_stats,
        })
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        // Top search bar
        let search_bar = top_search::view(
            &self.top_search_query,
            self.top_search_focused,
            "Search notes... (Ctrl+Shift+F)",
            self.mald_theme.is_dark,
        );
        let space_toolbar = self.view_space_toolbar();

        // Tab bar
        let tabs: Vec<tab_bar::TabInfo> = self
            .open_tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| tab_bar::TabInfo {
                title: tab.title.clone(),
                modified: tab.modified,
                index: i,
            })
            .collect();
        let tab_bar = tab_bar::view(tabs, self.active_tab, self.mald_theme.is_dark);

        // Main editor content or view
        let content: Element<Message> = match self.active_view {
            ActiveView::Home => self.view_home(),
            ActiveView::Editor => self.view_editor_content(),
            ActiveView::Graph => self.view_graph_content(),
            ActiveView::Search => self.view_search_content(),
            ActiveView::Tasks => self.view_tasks_content(),
        };

        // Terminal panel (at bottom, collapsible)
        let terminal_height = self.terminal_draw_height();
        if self.terminal_visible || self.terminal_animation.is_some() {
            column![
                search_bar,
                space_toolbar,
                tab_bar,
                container(content).height(Length::Fill),
                terminal_panel::view(
                    &self.terminal_lines,
                    &self.terminal_input,
                    terminal_height,
                    self.mald_theme.is_dark
                ),
            ]
            .spacing(0)
            .into()
        } else {
            column![
                search_bar,
                space_toolbar,
                tab_bar,
                container(content).height(Length::Fill),
            ]
            .spacing(0)
            .into()
        }
    }

    fn view_space_toolbar(&self) -> Element<'_, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let lavender = theme::themed(&iced_theme, colors::LAVENDER, colors::latte::LAVENDER);

        let buttons: Vec<Element<Message>> = self
            .known_kbs
            .iter()
            .take(5)
            .map(|kb_name| self.space_toolbar_button(kb_name.as_str(), text_color, lavender))
            .collect();

        let hint = if self.known_kbs.len() > 5 {
            "Ctrl+P shows the rest"
        } else {
            "Switch space or create a note"
        };

        container(
            row![
                text("Working space")
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
                self.signal_badge(self.current_kb.clone(), lavender),
                row(buttons).spacing(theme::spacing::XS),
                text(hint).size(theme::type_scale::CAPTION).color(sub0),
                Space::new().width(Length::Fill),
                self.action_button("Demo space", Message::DemoSpaceOpen, false),
                self.action_button("New note", Message::NewNotePrompt, true),
            ]
            .spacing(theme::spacing::SM)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
        .style(theme::card_style)
        .into()
    }

    fn view_home(&self) -> Element<'_, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let sub1 = theme::themed(&iced_theme, colors::SUBTEXT1, colors::latte::SUBTEXT1);
        let dim = theme::themed(&iced_theme, colors::SURFACE2, colors::latte::SURFACE2);
        let teal = theme::themed(&iced_theme, colors::TEAL, colors::latte::TEAL);
        let green = theme::themed(&iced_theme, colors::GREEN, colors::latte::GREEN);
        let red = theme::themed(&iced_theme, colors::RED, colors::latte::RED);
        let blue = theme::themed(&iced_theme, colors::ACCENT, colors::latte::ACCENT);
        let yellow = theme::themed(&iced_theme, colors::YELLOW, colors::latte::YELLOW);
        let lavender = theme::themed(&iced_theme, colors::LAVENDER, colors::latte::LAVENDER);

        let note_count = self
            .file_tree_entries
            .iter()
            .filter(|entry| !entry.is_dir)
            .count();
        let open_task_count = self.tasks.iter().filter(|task| !task.done).count();
        let done_task_count = self.tasks.iter().filter(|task| task.done).count();
        let link_count = self.graph_edges.len();
        let connected_count = self
            .graph_nodes
            .iter()
            .filter(|node| node.degree > 0)
            .count();
        let orphan_count = note_count.saturating_sub(connected_count);
        let modified_count = self.open_tabs.iter().filter(|tab| tab.modified).count();
        let connected_ratio = if note_count == 0 {
            0
        } else {
            ((connected_count as f32 / note_count as f32) * 100.0).round() as i32
        };
        let active_focus = self
            .open_tabs
            .get(self.active_tab)
            .map(|tab| tab.title.clone())
            .unwrap_or_else(|| "No note open yet".into());
        let graph_hub = self
            .graph_nodes
            .iter()
            .max_by_key(|node| node.degree)
            .map(|node| format!("{} · {} links", node.label, node.degree))
            .unwrap_or_else(|| "Start linking notes to grow the graph".into());

        let recent_files: Vec<&FileEntry> = self
            .file_tree_entries
            .iter()
            .filter(|entry| !entry.is_dir)
            .take(6)
            .collect();
        let open_tasks: Vec<(usize, &TaskItem)> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| !task.done)
            .take(5)
            .collect();
        let spaces_content: Element<Message> = if self.known_kbs.is_empty() {
            column![
                text("No spaces detected yet.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                text("Create one from the CLI or load the demo space to explore safely.")
                    .size(theme::type_scale::UI)
                    .color(sub0),
            ]
            .spacing(theme::spacing::XS)
            .into()
        } else {
            column(
                self.known_kbs
                    .iter()
                    .take(6)
                    .cloned()
                    .map(|kb_name| self.home_kb_button(kb_name, text_color, sub0, lavender))
                    .collect::<Vec<_>>(),
            )
            .spacing(theme::spacing::XS)
            .into()
        };
        let recent_content: Element<Message> = if recent_files.is_empty() {
            column![
                text("No notes yet. Start with one note and one real question.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                text("Use [[wikilinks]] early so the graph grows around actual work, not theory.")
                    .size(theme::type_scale::UI)
                    .color(sub0),
            ]
            .spacing(theme::spacing::XS)
            .into()
        } else {
            let items: Vec<Element<Message>> = recent_files
                .into_iter()
                .map(|entry| {
                    self.home_note_button(entry.name.as_str(), entry.path.clone(), lavender, dim)
                })
                .collect();
            column(items).spacing(theme::spacing::SM).into()
        };
        let tasks_content: Element<Message> = if open_tasks.is_empty() {
            column![
                text("No open loops right now.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                text("Capture tasks with `- [ ]` inside notes and MALD will surface them here.")
                    .size(theme::type_scale::UI)
                    .color(sub0),
            ]
            .spacing(theme::spacing::XS)
            .into()
        } else {
            let items: Vec<Element<Message>> = open_tasks
                .into_iter()
                .map(|(idx, task)| self.home_task_button(idx, task, text_color, sub0, yellow))
                .collect();
            column(items).spacing(theme::spacing::SM).into()
        };

        let focus_card = self.panel_card(
            column![
                row![
                    text("Work now").size(theme::type_scale::CAPTION).color(sub0),
                    Space::new().width(Length::Fill),
                    self.signal_badge(
                        match self.daemon_status {
                            DaemonStatus::Running => "Daemon running".into(),
                            DaemonStatus::Stopped => "Daemon stopped".into(),
                            DaemonStatus::Unknown => "Daemon checking".into(),
                        },
                        match self.daemon_status {
                            DaemonStatus::Running => green,
                            DaemonStatus::Stopped => red,
                            DaemonStatus::Unknown => dim,
                        },
                    ),
                ]
                .align_y(iced::alignment::Vertical::Center),
                Space::new().height(theme::spacing::SM),
                text(format!("Working in {}", self.current_kb))
                    .size(theme::type_scale::H1)
                    .color(text_color),
                Space::new().height(theme::spacing::XS),
                text("The dashboard should help you decide the next move fast: continue the current note, create a new one, or jump into search.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                Space::new().height(theme::spacing::LG),
                container(
                    column![
                        text("In focus").size(theme::type_scale::CAPTION).color(sub0),
                        text(active_focus).size(theme::type_scale::H3).color(text_color),
                        text("Use Ctrl+P for commands, Ctrl+Shift+F for search, and Ctrl+N for a new note.")
                            .size(theme::type_scale::UI)
                            .color(sub0),
                    ]
                    .spacing(theme::spacing::XS),
                )
                .padding(theme::spacing::LG as u16)
                .style(theme::card_style),
                Space::new().height(theme::spacing::LG),
                row![
                    self.action_button("New note", Message::NewNotePrompt, true),
                    self.action_button("Search", Message::SearchOpen, false),
                    self.action_button("Review tasks", Message::TasksToggle, false),
                    self.action_button("Demo space", Message::DemoSpaceOpen, false),
                ]
                .spacing(theme::spacing::SM)
                .wrap(),
                Space::new().height(theme::spacing::MD),
                row![
                    self.signal_badge(format!("{modified_count} modified"), lavender),
                    self.signal_badge(format!("{done_task_count} completed"), green),
                ]
                .spacing(theme::spacing::SM)
                .wrap(),
            ]
            .spacing(0),
            Some(teal),
        );

        let state_card = self.panel_card(
            column![
                text("Workspace state")
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
                Space::new().height(theme::spacing::XS),
                text("Health, structure, and space selection")
                    .size(theme::type_scale::H3)
                    .color(text_color),
                Space::new().height(theme::spacing::SM),
                text("This region is about the archive itself: scale, graph quality, and which space you want to work in.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                Space::new().height(theme::spacing::MD),
                row![
                    container(self.stat_card_colored("Notes", note_count.to_string(), blue, sub0))
                        .width(Length::Fill),
                    container(self.stat_card_colored(
                        "Open tasks",
                        open_task_count.to_string(),
                        yellow,
                        sub0
                    ))
                    .width(Length::Fill),
                ]
                .spacing(theme::spacing::SM),
                row![
                    container(self.stat_card_colored(
                        "Connected",
                        format!("{connected_ratio}%"),
                        teal,
                        sub0
                    ))
                    .width(Length::Fill),
                    container(self.stat_card_colored(
                        "Links",
                        link_count.to_string(),
                        lavender,
                        sub0
                    ))
                    .width(Length::Fill),
                ]
                .spacing(theme::spacing::SM),
                Space::new().height(theme::spacing::LG),
                text("Most connected note")
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
                text(graph_hub).size(theme::type_scale::UI).color(text_color),
                text(format!("{orphan_count} notes still need links to become part of the graph."))
                    .size(theme::type_scale::CAPTION)
                    .color(sub1),
                Space::new().height(theme::spacing::LG),
                text("Spaces").size(theme::type_scale::CAPTION).color(sub0),
                spaces_content,
            ]
            .spacing(0),
            Some(lavender),
        );

        let recent_card = self.panel_card(
            column![
                row![
                    text("Continue")
                        .size(theme::type_scale::CAPTION)
                        .color(sub0),
                    Space::new().width(Length::Fill),
                    text("Recent notes")
                        .size(theme::type_scale::CAPTION)
                        .color(blue),
                ]
                .align_y(iced::alignment::Vertical::Center),
                Space::new().height(theme::spacing::XS),
                text("The quickest way back into flow is usually the note you touched most recently.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                Space::new().height(theme::spacing::MD),
                recent_content,
            ]
            .spacing(0),
            Some(blue),
        );

        let review_card = self.panel_card(
            column![
                row![
                    text("Review").size(theme::type_scale::CAPTION).color(sub0),
                    Space::new().width(Length::Fill),
                    self.signal_badge(format!("{open_task_count} active"), yellow),
                ]
                .align_y(iced::alignment::Vertical::Center),
                Space::new().height(theme::spacing::XS),
                text("Tasks, graph actions, and the safe next step")
                    .size(theme::type_scale::H3)
                    .color(text_color),
                Space::new().height(theme::spacing::SM),
                row![
                    self.action_button("Review tasks", Message::TasksToggle, false),
                    self.action_button("Inspect graph", Message::GraphToggle, false),
                    self.action_button("New note", Message::NewNotePrompt, true),
                ]
                .spacing(theme::spacing::SM)
                .wrap(),
                Space::new().height(theme::spacing::LG),
                tasks_content,
                Space::new().height(theme::spacing::LG),
                text("Safe ways to learn").size(theme::type_scale::CAPTION).color(sub0),
                text("Use the demo space if you want sample notes, tasks, and wikilinks without touching your real archive.")
                    .size(theme::type_scale::UI)
                    .color(sub1),
                text("If you already know what you want, Ctrl+P gets there faster than browsing.")
                    .size(theme::type_scale::UI)
                    .color(sub1),
            ]
            .spacing(0),
            Some(yellow),
        );

        let layout = column![
            row![
                container(focus_card).width(Length::FillPortion(1)),
                container(state_card).width(Length::FillPortion(1)),
            ]
            .spacing(theme::spacing::LG),
            row![
                container(recent_card).width(Length::FillPortion(1)),
                container(review_card).width(Length::FillPortion(1)),
            ]
            .spacing(theme::spacing::LG),
        ]
        .spacing(theme::spacing::LG)
        .max_width(1180);

        container(scrollable(layout).style(theme::scrollable_style))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(theme::spacing::XXL as u16)
            .center_x(Length::Fill)
            .style(theme::editor_style)
            .into()
    }

    fn stat_card_colored(
        &self,
        label: &'static str,
        value: String,
        accent: iced::Color,
        sub_color: iced::Color,
    ) -> Element<'static, Message> {
        let tint = iced::Color { a: 0.10, ..accent };
        container(
            column![
                text(value).size(theme::type_scale::H2).color(accent),
                text(label).size(theme::type_scale::UI).color(sub_color),
            ]
            .spacing(theme::spacing::XS)
            .padding(theme::spacing::LG as u16),
        )
        .style(move |theme_ctx| {
            let mut style = theme::card_style(theme_ctx);
            style.background = Some(iced::Background::Color(tint));
            style.border = iced::Border {
                color: iced::Color { a: 0.24, ..accent },
                width: 1.0,
                radius: 14.0.into(),
            };
            style
        })
        .into()
    }

    fn panel_card<'a>(
        &'a self,
        content: impl Into<Element<'a, Message>>,
        accent: Option<iced::Color>,
    ) -> Element<'a, Message> {
        container(content)
            .padding(theme::spacing::XL as u16)
            .style(move |theme_ctx| {
                let mut style = theme::card_style(theme_ctx);
                if let Some(accent) = accent {
                    style.background =
                        Some(iced::Background::Color(iced::Color { a: 0.08, ..accent }));
                    style.border = iced::Border {
                        color: iced::Color { a: 0.22, ..accent },
                        width: 1.0,
                        radius: 18.0.into(),
                    };
                } else {
                    style.border.radius = 18.0.into();
                }
                style
            })
            .into()
    }

    fn signal_badge(&self, label: String, accent: iced::Color) -> Element<'static, Message> {
        container(text(label).size(theme::type_scale::CAPTION).color(accent))
            .padding([4, 10])
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.10, ..accent })),
                border: iced::Border {
                    color: iced::Color { a: 0.22, ..accent },
                    width: 1.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn action_button(
        &self,
        label: &'static str,
        message: Message,
        primary: bool,
    ) -> Element<'static, Message> {
        let button = button(text(label).size(theme::type_scale::UI))
            .on_press(message)
            .padding([theme::spacing::XS as u16, theme::spacing::SM as u16]);

        if primary {
            button.style(theme::primary_button_style).into()
        } else {
            button.style(theme::secondary_button_style).into()
        }
    }

    fn home_note_button(
        &self,
        label: &str,
        path: PathBuf,
        accent: iced::Color,
        dim: iced::Color,
    ) -> Element<'static, Message> {
        button(
            row![
                icons::files().color(dim),
                text(label.to_string())
                    .size(theme::type_scale::UI)
                    .color(accent),
                Space::new().width(Length::Fill),
                text("Open").size(theme::type_scale::CAPTION).color(dim),
            ]
            .spacing(theme::spacing::SM)
            .align_y(iced::alignment::Vertical::Center),
        )
        .on_press(Message::FileTreeSelect(path))
        .padding([theme::spacing::SM as u16, theme::spacing::MD as u16])
        .style(theme::list_item_style(false))
        .into()
    }

    fn home_task_button(
        &self,
        index: usize,
        task: &TaskItem,
        text_color: iced::Color,
        sub0: iced::Color,
        accent: iced::Color,
    ) -> Element<'static, Message> {
        button(
            column![
                row![
                    icons::empty_box().color(accent),
                    text(task.text.clone())
                        .size(theme::type_scale::UI)
                        .color(text_color),
                ]
                .spacing(theme::spacing::SM)
                .align_y(iced::alignment::Vertical::Center),
                text(format!("in {}", task.note))
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
            ]
            .spacing(theme::spacing::XS),
        )
        .on_press(Message::TaskClick(index))
        .padding([theme::spacing::SM as u16, theme::spacing::MD as u16])
        .style(theme::list_item_style(false))
        .into()
    }

    fn home_kb_button(
        &self,
        kb_name: String,
        text_color: iced::Color,
        sub0: iced::Color,
        accent: iced::Color,
    ) -> Element<'static, Message> {
        let is_active = kb_name == self.current_kb;
        let target = kb_name.clone();
        let label_color = if is_active { accent } else { text_color };

        button(
            row![
                icons::files().color(label_color),
                text(kb_name).size(theme::type_scale::UI).color(label_color),
                Space::new().width(Length::Fill),
                if is_active {
                    self.signal_badge("Active".into(), accent)
                } else {
                    text("Switch")
                        .size(theme::type_scale::CAPTION)
                        .color(sub0)
                        .into()
                },
            ]
            .spacing(theme::spacing::SM)
            .align_y(iced::alignment::Vertical::Center),
        )
        .on_press(Message::CurrentKbSwitch(target))
        .padding([theme::spacing::SM as u16, theme::spacing::MD as u16])
        .style(theme::list_item_style(false))
        .into()
    }

    fn space_toolbar_button(
        &self,
        kb_name: &str,
        text_color: iced::Color,
        accent: iced::Color,
    ) -> Element<'static, Message> {
        let is_active = kb_name == self.current_kb;
        let label = kb_name.to_string();
        let button_text = text(label.clone())
            .size(theme::type_scale::CAPTION)
            .color(if is_active { accent } else { text_color });

        button(button_text)
            .on_press(Message::CurrentKbSwitch(label))
            .padding([4, 10])
            .style(theme::ghost_button_style(is_active))
            .into()
    }

    fn rebuild_palette_commands(&mut self) {
        self.palette_commands = Self::all_commands(&self.current_kb, &self.known_kbs);
    }

    fn refresh_workspace_cache(&mut self) {
        self.known_kbs = workspace_kbs();
    }

    fn refresh_runtime_cache(&mut self) {
        self.detected_editors = crate::commands::launch::detected_editors();
        self.mald_shell_available = crate::commands::setup::mald_on_path();
    }

    fn sync_current_kb(&mut self, kb_name: String) {
        self.current_kb = kb_name.clone();
        self.settings_form.default_kb = kb_name;
        if !self.new_note_visible {
            self.new_note_kb = self.current_kb.clone();
        }
        self.rebuild_palette_commands();
    }

    fn view_editor_content(&self) -> Element<'_, Message> {
        use crate::gui::widgets::empty_state;

        if self.open_tabs.is_empty() {
            return container(empty_state::presets::no_editor(self.mald_theme.is_dark))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(theme::editor_style)
                .into();
        }

        let is_markdown = self
            .open_tabs
            .get(self.active_tab)
            .and_then(|t| t.path.extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);

        if is_markdown {
            let iced_theme = self.mald_theme.iced_theme();

            let editor_preview = pane_grid(&self.editor_preview_panes, |_, pane, _| match pane {
                EditorPreviewPane::Source => pane_grid::Content::new(
                    container(
                        text_editor(&self.editor_content)
                            .on_action(Message::EditorContentChanged)
                            .padding(theme::spacing::LG)
                            .style(theme::text_editor_style),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::editor_style),
                ),
                EditorPreviewPane::Preview => pane_grid::Content::new(
                    container(markdown_view::render_markdown(
                        &self.markdown_preview,
                        &self.code_cells,
                        &self.syntax_highlighter,
                        iced_theme.clone(),
                        self.mald_theme.is_dark,
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::editor_style),
                ),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_resize(12, Message::EditorPreviewSplitChanged);

            container(editor_preview)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::editor_style)
                .into()
        } else {
            container(
                text_editor(&self.editor_content)
                    .on_action(Message::EditorContentChanged)
                    .padding(theme::spacing::LG)
                    .style(theme::text_editor_style),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::editor_style)
            .into()
        }
    }

    fn view_graph_content(&self) -> Element<'_, Message> {
        use crate::gui::widgets::empty_state;
        use iced::widget::slider;

        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub_color = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let sub1_color = theme::themed(&iced_theme, colors::SUBTEXT1, colors::latte::SUBTEXT1);
        let dim_color = theme::themed(&iced_theme, colors::SURFACE2, colors::latte::SURFACE2);
        let blue = theme::themed(&iced_theme, colors::ACCENT, colors::latte::ACCENT);
        let teal = theme::themed(&iced_theme, colors::TEAL, colors::latte::TEAL);
        let lavender = theme::themed(&iced_theme, colors::LAVENDER, colors::latte::LAVENDER);
        let yellow = theme::themed(&iced_theme, colors::YELLOW, colors::latte::YELLOW);

        if self.graph_nodes.is_empty() {
            return container(empty_state::presets::no_graph(self.mald_theme.is_dark))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::editor_style)
                .into();
        }

        // Filter orphans if enabled
        let display_nodes: Vec<GraphNode> = if self.graph_hide_orphans {
            self.graph_nodes
                .iter()
                .filter(|n| n.degree > 0)
                .cloned()
                .collect()
        } else {
            self.graph_nodes.clone()
        };
        let visible_count = display_nodes.len();
        let visible_ids: std::collections::HashSet<&str> =
            display_nodes.iter().map(|node| node.id.as_str()).collect();
        let visible_edge_count = self
            .graph_edges
            .iter()
            .filter(|edge| {
                visible_ids.contains(edge.from.as_str()) && visible_ids.contains(edge.to.as_str())
            })
            .count();
        let orphan_count = self.graph_nodes.len().saturating_sub(
            self.graph_nodes
                .iter()
                .filter(|node| node.degree > 0)
                .count(),
        );
        let hub_label = display_nodes
            .iter()
            .max_by_key(|node| node.degree)
            .map(|node| format!("{} · {} links", node.label, node.degree))
            .unwrap_or_else(|| "Waiting for the first connection".into());
        let zoom_label = format!("{:.0}%", self.graph_zoom * 100.0);

        let header = self.panel_card(
            column![
                row![
                    self.signal_badge(format!("{visible_count} notes"), blue),
                    self.signal_badge(format!("{visible_edge_count} links"), teal),
                    self.signal_badge(format!("Zoom {zoom_label}"), lavender),
                ]
                .spacing(theme::spacing::SM),
                Space::new().height(theme::spacing::LG),
                text("Knowledge graph").size(theme::type_scale::DISPLAY).color(text_color),
                Space::new().height(theme::spacing::XS),
                text("See how ideas attach before they drift. Scroll to zoom, drag to pan, click a note to open it.")
                    .size(theme::type_scale::BODY)
                    .color(sub1_color),
                Space::new().height(theme::spacing::LG),
                row![
                    self.action_button(
                        if self.graph_settings_visible {
                            "Hide controls"
                        } else {
                            "Tune physics"
                        },
                        Message::GraphSettingsToggle,
                        false,
                    ),
                    self.action_button("Reset view", Message::GraphViewReset, false),
                    self.action_button("Reset forces", Message::GraphPhysicsReset, false),
                    self.action_button("New note", Message::NewNotePrompt, true),
                    Space::new().width(Length::Fill),
                    text(format!("Hub: {hub_label}"))
                        .size(theme::type_scale::UI)
                        .color(sub_color),
                ]
                .spacing(theme::spacing::SM)
                .align_y(iced::alignment::Vertical::Center),
            ]
            .spacing(0),
            Some(blue),
        );

        let metrics = row![
            container(self.stat_card_colored(
                "Visible",
                visible_count.to_string(),
                blue,
                sub_color
            ))
            .width(Length::FillPortion(1)),
            container(self.stat_card_colored(
                "Linked",
                visible_edge_count.to_string(),
                teal,
                sub_color
            ))
            .width(Length::FillPortion(1)),
            container(self.stat_card_colored(
                "Orphans",
                orphan_count.to_string(),
                yellow,
                sub_color
            ))
            .width(Length::FillPortion(1)),
            container(self.stat_card_colored("Focus", zoom_label.clone(), lavender, sub_color))
                .width(Length::FillPortion(1)),
        ]
        .spacing(theme::spacing::MD);

        let graph_body: Element<Message> = if display_nodes.is_empty() {
            empty_state::presets::no_graph(self.mald_theme.is_dark)
        } else {
            container(graph::view(
                &display_nodes,
                &self.graph_edges,
                self.graph_zoom,
                self.graph_pan,
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        let graph_area = container(
            column![
                row![
                    text("Canvas").size(theme::type_scale::H3).color(text_color),
                    Space::new().width(Length::Fill),
                    text("Wheel: zoom · drag: pan · click: open note")
                        .size(theme::type_scale::CAPTION)
                        .color(dim_color),
                ]
                .align_y(iced::alignment::Vertical::Center),
                Space::new().height(theme::spacing::MD),
                container(graph_body)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .height(Length::Fill),
        )
        .padding(theme::spacing::XL as u16)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme_ctx| {
            let mut style = theme::card_style(theme_ctx);
            style.border.radius = 20.0.into();
            style
        });

        let controls = if self.graph_settings_visible {
            let sim = &self.graph_simulation;
            Some(
                self.panel_card(
                    column![
                        text("Physics controls")
                            .size(theme::type_scale::H3)
                            .color(text_color),
                        Space::new().height(theme::spacing::SM),
                        text("Small changes are enough. The goal is legibility, not chaos.")
                            .size(theme::type_scale::UI)
                            .color(sub_color),
                        Space::new().height(theme::spacing::MD),
                        row![
                            text("Repel")
                                .size(theme::type_scale::UI)
                                .color(sub1_color)
                                .width(Length::Fixed(96.0)),
                            slider(
                                -2000.0..=-200.0,
                                sim.charge_strength,
                                Message::GraphRepelForceChanged
                            )
                            .step(10.0)
                            .width(Length::Fill),
                            text(format!("{:.0}", sim.charge_strength))
                                .size(theme::type_scale::CAPTION)
                                .color(dim_color)
                                .width(Length::Fixed(56.0)),
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::alignment::Vertical::Center),
                        row![
                            text("Link strength")
                                .size(theme::type_scale::UI)
                                .color(sub1_color)
                                .width(Length::Fixed(96.0)),
                            slider(
                                0.02..=0.30,
                                sim.link_strength,
                                Message::GraphLinkForceChanged
                            )
                            .step(0.01)
                            .width(Length::Fill),
                            text(format!("{:.2}", sim.link_strength))
                                .size(theme::type_scale::CAPTION)
                                .color(dim_color)
                                .width(Length::Fixed(56.0)),
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::alignment::Vertical::Center),
                        row![
                            text("Link distance")
                                .size(theme::type_scale::UI)
                                .color(sub1_color)
                                .width(Length::Fixed(96.0)),
                            slider(
                                50.0..=300.0,
                                sim.link_distance,
                                Message::GraphLinkDistanceChanged
                            )
                            .step(5.0)
                            .width(Length::Fill),
                            text(format!("{:.0}", sim.link_distance))
                                .size(theme::type_scale::CAPTION)
                                .color(dim_color)
                                .width(Length::Fixed(56.0)),
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::alignment::Vertical::Center),
                        row![
                            text("Center pull")
                                .size(theme::type_scale::UI)
                                .color(sub1_color)
                                .width(Length::Fixed(96.0)),
                            slider(
                                0.0..=0.1,
                                sim.center_strength,
                                Message::GraphCenterForceChanged
                            )
                            .step(0.005)
                            .width(Length::Fill),
                            text(format!("{:.3}", sim.center_strength))
                                .size(theme::type_scale::CAPTION)
                                .color(dim_color)
                                .width(Length::Fixed(56.0)),
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::alignment::Vertical::Center),
                        row![
                            iced::widget::toggler(self.graph_hide_orphans)
                                .on_toggle(|_| Message::GraphHideOrphansToggle)
                                .size(theme::type_scale::UI),
                            text("Hide orphan notes")
                                .size(theme::type_scale::UI)
                                .color(sub1_color),
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::alignment::Vertical::Center),
                    ]
                    .spacing(theme::spacing::SM),
                    Some(teal),
                ),
            )
        } else {
            None
        };

        let mut content = column![header, metrics]
            .spacing(theme::spacing::LG)
            .padding(theme::spacing::XXL as u16)
            .height(Length::Fill);

        if let Some(controls) = controls {
            content = content.push(controls);
        }

        content = content.push(graph_area);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::editor_style)
            .into()
    }

    fn view_search_content(&self) -> Element<'_, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let sub1 = theme::themed(&iced_theme, colors::SUBTEXT1, colors::latte::SUBTEXT1);
        let blue = theme::themed(&iced_theme, colors::ACCENT, colors::latte::ACCENT);
        let lavender = theme::themed(&iced_theme, colors::LAVENDER, colors::latte::LAVENDER);
        let yellow = theme::themed(&iced_theme, colors::YELLOW, colors::latte::YELLOW);
        let query = self.search_query.trim();
        let query_label = if query.is_empty() {
            "Waiting for a search".to_string()
        } else {
            format!("Query: {query}")
        };

        let header = self.panel_card(
            column![
                row![
                    self.signal_badge(format!("Working space {}", self.current_kb), lavender),
                    self.signal_badge(format!("{} results", self.search_results.len()), blue),
                    if query.is_empty() {
                        self.signal_badge("Type above to search".into(), yellow)
                    } else {
                        self.signal_badge("Search is live".into(), yellow)
                    },
                ]
                .spacing(theme::spacing::SM),
                Space::new().height(theme::spacing::LG),
                text("Search across your notes")
                    .size(theme::type_scale::DISPLAY)
                    .color(text_color),
                Space::new().height(theme::spacing::XS),
                text("Use the search bar above. Results stay visible here so you can scan them without losing the rest of the app.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                Space::new().height(theme::spacing::LG),
                row![
                    self.action_button("Focus search", Message::SearchOpen, false),
                    self.action_button("New note", Message::NewNotePrompt, true),
                    self.action_button("Open demo space", Message::DemoSpaceOpen, false),
                    Space::new().width(Length::Fill),
                    text(query_label)
                        .size(theme::type_scale::UI)
                        .color(sub0),
                ]
                .spacing(theme::spacing::SM)
                .align_y(iced::alignment::Vertical::Center),
            ]
            .spacing(0),
            Some(blue),
        );

        let body: Element<Message> = if query.is_empty() {
            self.panel_card(
                column![
                    text("Try one of these")
                        .size(theme::type_scale::H3)
                        .color(text_color),
                    Space::new().height(theme::spacing::SM),
                    text("nebula lattice").size(theme::type_scale::BODY).color(lavender),
                    text("amber harbor review")
                        .size(theme::type_scale::BODY)
                        .color(lavender),
                    text("meeting rhythm reset")
                        .size(theme::type_scale::BODY)
                        .color(lavender),
                    Space::new().height(theme::spacing::MD),
                    text("Search works across titles and note bodies. The sidebar stays useful for narrow drilling, while this screen gives you room to scan results comfortably.")
                        .size(theme::type_scale::BODY)
                        .color(sub1),
                ]
                .spacing(0),
                Some(lavender),
            )
        } else if self.search_results.is_empty() {
            crate::gui::widgets::empty_state::presets::no_search_results(self.mald_theme.is_dark)
        } else {
            let items: Vec<Element<Message>> = self
                .search_results
                .iter()
                .enumerate()
                .map(|(i, result)| {
                    button(
                        column![
                            text(result.title.clone())
                                .size(theme::type_scale::BODY)
                                .color(text_color),
                            text(result.snippet.clone())
                                .size(theme::type_scale::UI)
                                .color(sub0),
                        ]
                        .spacing(theme::spacing::XS)
                        .padding(theme::spacing::MD as u16),
                    )
                    .on_press(Message::SearchResultSelect(i))
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
                })
                .collect();

            self.panel_card(
                column![
                    row![
                        text("Results")
                            .size(theme::type_scale::H3)
                            .color(text_color),
                        Space::new().width(Length::Fill),
                        text("Open a result to jump straight into the note.")
                            .size(theme::type_scale::CAPTION)
                            .color(sub0),
                    ]
                    .align_y(iced::alignment::Vertical::Center),
                    Space::new().height(theme::spacing::MD),
                    column(items).spacing(theme::spacing::SM),
                ]
                .spacing(0),
                Some(lavender),
            )
        };

        container(
            scrollable(
                column![header, body]
                    .spacing(theme::spacing::LG)
                    .padding(theme::spacing::XXL as u16)
                    .max_width(1080),
            )
            .style(theme::scrollable_style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .style(theme::editor_style)
        .into()
    }

    fn view_tasks_content(&self) -> Element<'_, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let sub1 = theme::themed(&iced_theme, colors::SUBTEXT1, colors::latte::SUBTEXT1);
        let dim = theme::themed(&iced_theme, colors::SURFACE2, colors::latte::SURFACE2);
        let green = theme::themed(&iced_theme, colors::GREEN, colors::latte::GREEN);
        let yellow = theme::themed(&iced_theme, colors::YELLOW, colors::latte::YELLOW);
        let blue = theme::themed(&iced_theme, colors::ACCENT, colors::latte::ACCENT);
        let open_count = self.tasks.iter().filter(|task| !task.done).count();
        let done_count = self.tasks.iter().filter(|task| task.done).count();

        let header = self.panel_card(
            column![
                row![
                    self.signal_badge(format!("Working space {}", self.current_kb), blue),
                    self.signal_badge(format!("{open_count} open"), yellow),
                    self.signal_badge(format!("{done_count} done"), green),
                ]
                .spacing(theme::spacing::SM),
                Space::new().height(theme::spacing::LG),
                text("Tasks from ordinary notes")
                    .size(theme::type_scale::DISPLAY)
                    .color(text_color),
                Space::new().height(theme::spacing::XS),
                text("MALD pulls `- [ ]` tasks out of your notes so work stays close to context instead of living in a separate system.")
                    .size(theme::type_scale::BODY)
                    .color(sub1),
                Space::new().height(theme::spacing::LG),
                row![
                    self.action_button(
                        if self.tasks_kanban { "List view" } else { "Kanban view" },
                        Message::TaskToggleView,
                        false,
                    ),
                    self.action_button("New note", Message::NewNotePrompt, true),
                    self.action_button("Open demo space", Message::DemoSpaceOpen, false),
                    Space::new().width(Length::Fill),
                    text("Open a task to jump back to the note that owns it.")
                        .size(theme::type_scale::UI)
                        .color(sub0),
                ]
                .spacing(theme::spacing::SM)
                .align_y(iced::alignment::Vertical::Center),
            ]
            .spacing(0),
            Some(yellow),
        );

        let body: Element<Message> = if self.tasks.is_empty() {
            crate::gui::widgets::empty_state::presets::no_tasks(self.mald_theme.is_dark)
        } else if self.tasks_kanban {
            let open_items: Vec<Element<Message>> = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| !task.done)
                .map(|(i, task)| {
                    button(
                        column![
                            text(task.text.clone())
                                .size(theme::type_scale::BODY)
                                .color(text_color),
                            text(format!("in {}", task.note))
                                .size(theme::type_scale::CAPTION)
                                .color(dim),
                        ]
                        .spacing(theme::spacing::XS)
                        .padding(theme::spacing::MD as u16),
                    )
                    .on_press(Message::TaskClick(i))
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
                })
                .collect();

            let done_items: Vec<Element<Message>> = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| task.done)
                .map(|(i, task)| {
                    button(
                        column![
                            text(task.text.clone())
                                .size(theme::type_scale::BODY)
                                .color(sub0),
                            text(format!("in {}", task.note))
                                .size(theme::type_scale::CAPTION)
                                .color(dim),
                        ]
                        .spacing(theme::spacing::XS)
                        .padding(theme::spacing::MD as u16),
                    )
                    .on_press(Message::TaskClick(i))
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
                })
                .collect();

            row![
                container(
                    self.panel_card(
                        column![
                            row![
                                text("Open").size(theme::type_scale::H3).color(text_color),
                                Space::new().width(Length::Fill),
                                self.signal_badge(open_count.to_string(), yellow),
                            ]
                            .align_y(iced::alignment::Vertical::Center),
                            Space::new().height(theme::spacing::MD),
                            column(open_items).spacing(theme::spacing::SM),
                        ]
                        .spacing(0),
                        Some(yellow),
                    )
                )
                .width(Length::FillPortion(1)),
                container(
                    self.panel_card(
                        column![
                            row![
                                text("Done").size(theme::type_scale::H3).color(text_color),
                                Space::new().width(Length::Fill),
                                self.signal_badge(done_count.to_string(), green),
                            ]
                            .align_y(iced::alignment::Vertical::Center),
                            Space::new().height(theme::spacing::MD),
                            column(done_items).spacing(theme::spacing::SM),
                        ]
                        .spacing(0),
                        Some(green),
                    )
                )
                .width(Length::FillPortion(1)),
            ]
            .spacing(theme::spacing::LG)
            .into()
        } else {
            let items: Vec<Element<Message>> = self
                .tasks
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    let checkbox = if task.done {
                        icons::check_box().color(green)
                    } else {
                        icons::empty_box().color(yellow)
                    };

                    button(
                        row![
                            checkbox,
                            text(task.text.clone())
                                .size(theme::type_scale::BODY)
                                .color(if task.done { sub0 } else { text_color }),
                            Space::new().width(Length::Fill),
                            text(task.note.clone())
                                .size(theme::type_scale::CAPTION)
                                .color(dim),
                        ]
                        .spacing(theme::spacing::SM)
                        .padding(theme::spacing::SM),
                    )
                    .on_press(Message::TaskClick(i))
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
                })
                .collect();

            self.panel_card(
                column![
                    row![
                        text("Task list")
                            .size(theme::type_scale::H3)
                            .color(text_color),
                        Space::new().width(Length::Fill),
                        text("Click any item to open the source note.")
                            .size(theme::type_scale::CAPTION)
                            .color(sub0),
                    ]
                    .align_y(iced::alignment::Vertical::Center),
                    Space::new().height(theme::spacing::MD),
                    column(items).spacing(theme::spacing::XS),
                ]
                .spacing(0),
                Some(blue),
            )
        };

        container(
            scrollable(
                column![header, body]
                    .spacing(theme::spacing::LG)
                    .padding(theme::spacing::XXL as u16)
                    .max_width(1080),
            )
            .style(theme::scrollable_style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .style(theme::editor_style)
        .into()
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Overlays
    // ══════════════════════════════════════════════════════════════════════════

    fn view_with_palette<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);

        let input = text_input("Type a command...", &self.palette_query)
            .on_input(Message::CommandPaletteQueryChanged)
            .padding(theme::spacing::MD)
            .width(Length::Fill)
            .style(theme::search_input_style);

        let items: Vec<Element<Message>> = self
            .palette_filtered
            .iter()
            .take(15)
            .enumerate()
            .map(|(i, &cmd_idx)| {
                let cmd = &self.palette_commands[cmd_idx];
                let is_selected = i == self.palette_selected;
                button(
                    column![
                        text(&cmd.label)
                            .size(theme::type_scale::BODY)
                            .color(text_color),
                        text(&cmd.description)
                            .size(theme::type_scale::CAPTION)
                            .color(sub0),
                    ]
                    .spacing(theme::spacing::XS)
                    .padding(theme::spacing::SM),
                )
                .on_press(Message::CommandPaletteSelect(i))
                .width(Length::Fill)
                .style(theme::list_item_style(is_selected))
                .into()
            })
            .collect();

        let palette_content = column![
            input,
            scrollable(column(items).spacing(theme::spacing::XS))
                .height(Length::Fixed(300.0))
                .style(theme::scrollable_style),
        ]
        .spacing(theme::spacing::SM);

        let palette = container(palette_content)
            .padding(theme::spacing::MD)
            .width(Length::Fixed(theme::layout::MODAL_WIDTH))
            .style(theme::modal_style);

        // Animated overlay
        let overlay_opacity = self.modal_overlay_opacity();
        let overlay = container(
            container(palette)
                .center_x(Length::Fill)
                .padding(theme::layout::MODAL_VERTICAL_PADDING),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: overlay_opacity,
                ..colors::OVERLAY_BG
            })),
            ..Default::default()
        });

        // Stack overlay on base
        iced::widget::stack![base, overlay].into()
    }

    fn view_with_search_overlay<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);

        let input = text_input("Search notes...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .padding(theme::spacing::MD)
            .width(Length::Fill)
            .style(theme::search_input_style);

        let items: Vec<Element<Message>> = self
            .search_results
            .iter()
            .enumerate()
            .take(20)
            .map(|(i, result)| {
                button(
                    column![
                        text(&result.title)
                            .size(theme::type_scale::BODY)
                            .color(text_color),
                        text(&result.snippet)
                            .size(theme::type_scale::CAPTION)
                            .color(sub0),
                    ]
                    .spacing(theme::spacing::XS)
                    .padding(theme::spacing::SM),
                )
                .on_press(Message::SearchResultSelect(i))
                .width(Length::Fill)
                .style(theme::list_item_style(false))
                .into()
            })
            .collect();

        let search_content = column![
            input,
            scrollable(column(items).spacing(theme::spacing::XS))
                .height(Length::Fixed(300.0))
                .style(theme::scrollable_style),
        ]
        .spacing(theme::spacing::SM);

        let search_box = container(search_content)
            .padding(theme::spacing::MD)
            .width(Length::Fixed(theme::layout::MODAL_WIDTH))
            .style(theme::modal_style);

        // Animated overlay
        let overlay_opacity = self.modal_overlay_opacity();
        let overlay = container(
            container(search_box)
                .center_x(Length::Fill)
                .padding(theme::layout::MODAL_VERTICAL_PADDING),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: overlay_opacity,
                ..colors::OVERLAY_BG
            })),
            ..Default::default()
        });

        iced::widget::stack![base, overlay].into()
    }

    fn view_with_new_note<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let sub1 = theme::themed(&iced_theme, colors::SUBTEXT1, colors::latte::SUBTEXT1);
        let teal = theme::themed(&iced_theme, colors::TEAL, colors::latte::TEAL);
        let lavender = theme::themed(&iced_theme, colors::LAVENDER, colors::latte::LAVENDER);

        let title_input = text_input("Give the note a clear title...", &self.new_note_title)
            .on_input(Message::NewNoteTitleChanged)
            .on_submit(Message::NewNoteCreate(self.new_note_title.clone()))
            .padding(theme::spacing::MD)
            .width(Length::Fill)
            .style(theme::search_input_style);

        let path_input = text_input(
            "Optional folder inside the space, e.g. inbox or projects/api",
            &self.new_note_path,
        )
        .on_input(Message::NewNotePathChanged)
        .on_submit(Message::NewNoteCreate(self.new_note_title.clone()))
        .padding(theme::spacing::MD)
        .width(Length::Fill)
        .style(theme::search_input_style);

        let space_buttons: Vec<Element<Message>> = self
            .known_kbs
            .iter()
            .cloned()
            .map(|kb_name| {
                let is_active = kb_name == self.new_note_kb;
                button(
                    text(kb_name.clone())
                        .size(theme::type_scale::CAPTION)
                        .color(if is_active { lavender } else { text_color }),
                )
                .on_press(Message::NewNoteKbSelected(kb_name))
                .padding([4, 10])
                .style(theme::ghost_button_style(is_active))
                .into()
            })
            .collect();

        let destination_label = if self.new_note_path.trim().is_empty() {
            format!("Space: {} · Folder: /", self.new_note_kb)
        } else {
            format!(
                "Space: {} · Folder: /{}",
                self.new_note_kb,
                self.new_note_path.trim().replace('\\', "/")
            )
        };

        let content = column![
            text("Create a new note")
                .size(theme::type_scale::H2)
                .color(text_color),
            text("Choose the space and folder up front. MALD will create the note, index it, and open it immediately.")
                .size(theme::type_scale::BODY)
                .color(sub0),
            Space::new().height(theme::spacing::SM),
            title_input,
            path_input,
            column![
                text("Target space")
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
                row(space_buttons).spacing(theme::spacing::XS),
                text(destination_label)
                    .size(theme::type_scale::UI)
                    .color(sub1),
            ]
            .spacing(theme::spacing::XS),
            text("Use a concrete working title. Rename later if the idea sharpens.")
                .size(theme::type_scale::CAPTION)
                .color(teal),
            Space::new().height(theme::spacing::SM),
            row![
                button(text(format!("Create in {}", self.new_note_kb)).size(theme::type_scale::UI))
                    .on_press(Message::NewNoteCreate(self.new_note_title.clone()))
                    .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
                    .style(theme::primary_button_style),
                button(text("Cancel").size(theme::type_scale::UI))
                    .on_press(Message::CommandPaletteClose)
                    .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
                    .style(theme::secondary_button_style),
            ]
            .spacing(theme::spacing::SM),
        ]
        .spacing(theme::spacing::SM);

        let dialog = container(content)
            .padding(theme::spacing::XL)
            .width(Length::Fixed(theme::layout::MODAL_WIDTH))
            .style(theme::modal_style);

        let overlay_opacity = self.modal_overlay_opacity();
        let overlay = container(
            container(dialog)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: overlay_opacity,
                ..colors::OVERLAY_BG
            })),
            ..Default::default()
        });

        iced::widget::stack![base, overlay].into()
    }

    fn view_with_keybindings<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let blue = theme::themed(&iced_theme, colors::ACCENT, colors::latte::ACCENT);

        let bindings = vec![
            ("Ctrl+P", "Command palette"),
            ("Ctrl+N", "New note"),
            ("Ctrl+Shift+F", "Global search"),
            ("Ctrl+B", "Toggle sidebar"),
            ("Ctrl+J", "Toggle terminal"),
            ("Ctrl+Shift+B", "Toggle feature panel"),
            ("Ctrl+Shift+A", "Toggle AI chat"),
            ("Ctrl+Shift+G", "Graph view"),
            ("Ctrl+Shift+T", "Tasks view"),
            ("Ctrl+\\", "Split vertical"),
            ("Ctrl+Shift+\\", "Split horizontal"),
            ("Ctrl+W", "Close pane/tab"),
            ("Ctrl+Tab", "Next tab"),
            ("Ctrl+S", "Save"),
            ("Ctrl+E", "External editor"),
            ("?", "This help"),
            ("Esc", "Close overlays"),
        ];

        let items: Vec<Element<Message>> = bindings
            .into_iter()
            .map(|(key, desc)| {
                row![
                    container(text(key).size(theme::type_scale::UI).color(blue))
                        .width(Length::Fixed(140.0)),
                    text(desc).size(theme::type_scale::UI).color(text_color),
                ]
                .spacing(theme::spacing::SM)
                .into()
            })
            .collect();

        let help_content = column![
            text("Keyboard Shortcuts")
                .size(theme::type_scale::H2)
                .color(text_color),
            column(items).spacing(theme::spacing::XS),
            button(text("Close").size(theme::type_scale::UI))
                .on_press(Message::KeybindingsToggle)
                .padding([theme::spacing::SM, theme::spacing::LG])
                .style(theme::primary_button_style),
        ]
        .spacing(theme::spacing::LG);

        let help_box = container(help_content)
            .padding(theme::spacing::XL)
            .width(Length::Fixed(theme::layout::MODAL_WIDTH))
            .style(theme::modal_style);

        // Animated overlay
        let overlay_opacity = self.modal_overlay_opacity();
        let overlay = container(
            container(help_box)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: overlay_opacity,
                ..colors::OVERLAY_BG
            })),
            ..Default::default()
        });

        iced::widget::stack![base, overlay].into()
    }

    fn view_with_unsaved_close<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let iced_theme = self.mald_theme.iced_theme();
        let text_color = theme::themed(&iced_theme, colors::TEXT, colors::latte::TEXT);
        let sub0 = theme::themed(&iced_theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
        let yellow = theme::themed(&iced_theme, colors::YELLOW, colors::latte::YELLOW);

        let tab_title = self
            .pending_close_tab
            .and_then(|idx| self.open_tabs.get(idx))
            .map(|t| t.title.as_str())
            .unwrap_or("this file");

        let content = column![
            row![
                icons::warning().size(theme::type_scale::H2).color(yellow),
                text("Unsaved Changes")
                    .size(theme::type_scale::H2)
                    .color(text_color),
            ]
            .spacing(theme::spacing::SM)
            .align_y(iced::Alignment::Center),
            text(format!(
                "\"{tab_title}\" has unsaved changes. What would you like to do?"
            ))
            .size(theme::type_scale::BODY)
            .color(sub0),
            row![
                button(text("Save & Close").size(theme::type_scale::UI))
                    .on_press(Message::EditorCloseConfirmSave)
                    .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
                    .style(theme::primary_button_style),
                button(text("Discard").size(theme::type_scale::UI))
                    .on_press(Message::EditorCloseConfirmDiscard)
                    .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
                    .style(theme::secondary_button_style),
                button(text("Cancel").size(theme::type_scale::UI))
                    .on_press(Message::EditorCloseConfirmCancel)
                    .padding([theme::spacing::SM as u16, theme::spacing::LG as u16])
                    .style(theme::secondary_button_style),
            ]
            .spacing(theme::spacing::SM),
        ]
        .spacing(theme::spacing::LG);

        let dialog = container(content)
            .padding(theme::spacing::XL)
            .width(Length::Fixed(theme::layout::MODAL_WIDTH))
            .style(theme::modal_style);

        let overlay = container(
            container(dialog)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: colors::OVERLAY_BG.a,
                ..colors::OVERLAY_BG
            })),
            ..Default::default()
        });

        iced::widget::stack![base, overlay].into()
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Commands & Helpers
    // ══════════════════════════════════════════════════════════════════════════

    fn show_terminal_panel(&mut self) {
        let current = self.terminal_draw_height();
        self.terminal_animation = Some(AnimationState::velocity_aware(
            current,
            self.terminal_height,
            theme::animation::PANEL_PIXELS_PER_SEC,
        ));
        self.terminal_visible = true;
    }

    fn ensure_terminal_session(&mut self) -> Result<(), String> {
        if self.terminal_session.is_some() {
            return Ok(());
        }

        let configured_shell = self.settings_form.shell.trim();
        let shell = (!configured_shell.is_empty()).then_some(configured_shell);
        let (handle, output) = crate::gui::util::pty::PtyHandle::spawn(shell, 120, 32)
            .map_err(|error| format!("Failed to start terminal shell: {error}"))?;

        self.terminal_session = Some(TerminalSession { handle, output });
        self.push_terminal_line(format!(
            "[session] started {}",
            shell.unwrap_or("default shell")
        ));
        Ok(())
    }

    fn restart_terminal_session(&mut self) -> Result<(), String> {
        if let Some(session) = self.terminal_session.as_mut() {
            let _ = session.handle.kill();
        }
        self.terminal_session = None;
        self.terminal_partial_line.clear();
        self.push_terminal_line("[session] restarting terminal".to_string());
        self.ensure_terminal_session()
    }

    fn poll_runtime_channels(&mut self) {
        self.poll_terminal_output();
        self.poll_ai_stream();
    }

    fn poll_terminal_output(&mut self) {
        let mut chunks = Vec::new();
        let mut session_finished = false;

        if let Some(session) = self.terminal_session.as_mut() {
            chunks.extend(session.output.try_iter());

            if !session.handle.is_alive() {
                session_finished = true;
            }
        }

        for bytes in chunks {
            self.push_terminal_bytes(&bytes);
        }

        if session_finished {
            self.push_terminal_line("[session] shell exited".to_string());
            self.terminal_session = None;
            self.terminal_partial_line.clear();
        }
    }

    fn poll_ai_stream(&mut self) {
        let mut clear_receiver = false;

        if let Some(receiver) = self.ai_stream_receiver.as_ref() {
            for event in receiver.try_iter() {
                match event {
                    AiStreamEvent::Chunk(chunk) => {
                        if let Some((role, content)) = self.ai_chat_messages.last_mut() {
                            if role == "assistant" {
                                content.push_str(&chunk);
                            }
                        }
                    }
                    AiStreamEvent::Finished(citations) => {
                        self.ai_streaming = false;
                        if !citations.is_empty() {
                            if let Some((role, content)) = self.ai_chat_messages.last_mut() {
                                if role == "assistant" {
                                    content.push_str(&citations);
                                }
                            }
                        }
                        clear_receiver = true;
                    }
                    AiStreamEvent::Error(error) => {
                        self.ai_streaming = false;
                        if let Some((role, content)) = self.ai_chat_messages.last_mut() {
                            if role == "assistant" {
                                *content = format!("Error: {error}");
                            }
                        }
                        clear_receiver = true;
                    }
                }
            }
        }

        if clear_receiver {
            self.ai_stream_receiver = None;
        }
    }

    fn push_terminal_line(&mut self, line: impl Into<String>) {
        self.terminal_lines.push(line.into());
        if self.terminal_lines.len() > theme::limits::TERMINAL_LINES_MAX {
            let excess = self.terminal_lines.len() - theme::limits::TERMINAL_LINES_MAX;
            self.terminal_lines.drain(..excess);
        }
    }

    fn push_terminal_bytes(&mut self, bytes: &[u8]) {
        let chunk = strip_terminal_ansi(&String::from_utf8_lossy(bytes));
        self.terminal_partial_line.push_str(&chunk);

        while let Some(newline_idx) = self.terminal_partial_line.find(['\n', '\r']) {
            let line = self.terminal_partial_line[..newline_idx].to_string();
            self.push_terminal_line(line);

            let mut drain_end = newline_idx + 1;
            while self
                .terminal_partial_line
                .as_bytes()
                .get(drain_end)
                .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
            {
                drain_end += 1;
            }
            self.terminal_partial_line.drain(..drain_end);
        }
    }

    fn push_terminal_output(&mut self, output: &str) {
        for line in output.lines() {
            self.push_terminal_line(line.to_string());
        }
    }

    fn sync_editor_cursor_state(&mut self) {
        let cursor = self.editor_content.cursor();
        let line = cursor.position.line + 1;
        let col = cursor.position.column + 1;

        self.cursor_line = line;
        self.cursor_col = col;

        if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
            tab.cursor_line = line;
            tab.cursor_col = col;
        }
    }

    fn move_editor_cursor_to(&mut self, line: usize, col: usize) {
        if self.open_tabs.is_empty() {
            return;
        }

        let target_line = line
            .saturating_sub(1)
            .min(self.editor_content.line_count().saturating_sub(1));
        let target_col = self
            .editor_content
            .line(target_line)
            .map(|line| col.saturating_sub(1).min(line.text.chars().count()))
            .unwrap_or(0);

        self.editor_content.move_to(text_editor::Cursor {
            position: text_editor::Position {
                line: target_line,
                column: target_col,
            },
            selection: None,
        });
        self.sync_editor_cursor_state();
    }

    fn restore_editor_tab_state(&mut self) {
        if let Some((line, col)) = self
            .open_tabs
            .get(self.active_tab)
            .map(|tab| (tab.cursor_line, tab.cursor_col))
        {
            self.move_editor_cursor_to(line, col);
        }
    }

    fn jump_editor_to_line(&mut self, line: usize) {
        self.move_editor_cursor_to(line, 1);
    }

    fn all_commands(current_kb: &str, kbs: &[String]) -> Vec<PaletteCommand> {
        let mut commands = vec![
            PaletteCommand {
                id: "new_note".into(),
                label: "New Note".into(),
                description: "Create a new markdown note".into(),
            },
            PaletteCommand {
                id: "search".into(),
                label: "Search".into(),
                description: "Full-text search across all notes".into(),
            },
            PaletteCommand {
                id: "graph".into(),
                label: "Graph View".into(),
                description: "Visualize wikilink connections".into(),
            },
            PaletteCommand {
                id: "tasks".into(),
                label: "Tasks".into(),
                description: "View open tasks across notes".into(),
            },
            PaletteCommand {
                id: "toggle_sidebar".into(),
                label: "Toggle Sidebar".into(),
                description: "Show/hide file explorer".into(),
            },
            PaletteCommand {
                id: "toggle_terminal".into(),
                label: "Toggle Terminal".into(),
                description: "Show/hide embedded terminal".into(),
            },
            PaletteCommand {
                id: "toggle_ai".into(),
                label: "Toggle AI Chat".into(),
                description: "Show/hide AI assistant".into(),
            },
            PaletteCommand {
                id: "toggle_feature_panel".into(),
                label: "Toggle Feature Panel".into(),
                description: "Show/hide right panel".into(),
            },
            PaletteCommand {
                id: "split_v".into(),
                label: "Split Vertical".into(),
                description: "Split current pane vertically".into(),
            },
            PaletteCommand {
                id: "split_h".into(),
                label: "Split Horizontal".into(),
                description: "Split current pane horizontally".into(),
            },
            PaletteCommand {
                id: "theme".into(),
                label: "Toggle Theme".into(),
                description: "Switch between dark and light mode".into(),
            },
            PaletteCommand {
                id: "save".into(),
                label: "Save".into(),
                description: "Save current file".into(),
            },
            PaletteCommand {
                id: "close_tab".into(),
                label: "Close Tab".into(),
                description: "Close current editor tab".into(),
            },
            PaletteCommand {
                id: "home".into(),
                label: "Go Home".into(),
                description: "Return to dashboard".into(),
            },
            PaletteCommand {
                id: "keybindings".into(),
                label: "Keybindings".into(),
                description: "Show keyboard shortcuts".into(),
            },
            PaletteCommand {
                id: "reindex".into(),
                label: "Reindex".into(),
                description: "Rebuild search index".into(),
            },
            PaletteCommand {
                id: "doctor".into(),
                label: "Doctor".into(),
                description: "Run self-diagnostics".into(),
            },
            PaletteCommand {
                id: "demo".into(),
                label: "Open Demo Space".into(),
                description: "Load the safe sample notes and switch the working space to demo."
                    .into(),
            },
        ];

        if kbs.len() > 1 {
            for kb_name in kbs {
                if kb_name == current_kb {
                    continue;
                }
                commands.push(PaletteCommand {
                    id: format!("switch_kb:{kb_name}"),
                    label: format!("Switch Space: {kb_name}"),
                    description: "Change the working space for new notes, AI, and launch context."
                        .into(),
                });
            }
        }

        commands
    }

    fn execute_palette_command(&mut self, cmd: &PaletteCommand) -> IcedTask<Message> {
        if let Some(kb_name) = cmd.id.strip_prefix("switch_kb:") {
            return IcedTask::done(Message::CurrentKbSwitch(kb_name.to_string()));
        }

        match cmd.id.as_str() {
            "new_note" => IcedTask::done(Message::NewNotePrompt),
            "search" => IcedTask::done(Message::SearchOpen),
            "graph" => IcedTask::done(Message::GraphToggle),
            "tasks" => IcedTask::done(Message::TasksToggle),
            "toggle_sidebar" => IcedTask::done(Message::SidebarToggle),
            "toggle_terminal" => IcedTask::done(Message::TerminalToggle),
            "toggle_ai" => IcedTask::done(Message::AiChatToggle),
            "toggle_feature_panel" => IcedTask::done(Message::FeaturePanelToggle),
            "split_v" => IcedTask::done(Message::PaneSplitVertical),
            "split_h" => IcedTask::done(Message::PaneSplitHorizontal),
            "theme" => IcedTask::done(Message::ThemeToggle),
            "save" => IcedTask::done(Message::EditorSave),
            "close_tab" => {
                let idx = self.active_tab;
                IcedTask::done(Message::EditorClose(idx))
            }
            "home" => IcedTask::done(Message::GoHome),
            "keybindings" => IcedTask::done(Message::KeybindingsToggle),
            "reindex" => {
                self.show_terminal_panel();
                self.push_terminal_line("> mald reindex");
                IcedTask::perform(rebuild_search_index(), Message::ReindexCompleted)
            }
            "doctor" => {
                self.show_terminal_panel();
                self.push_terminal_line("> mald doctor");
                IcedTask::perform(run_doctor_report(), Message::DoctorCompleted)
            }
            "demo" => IcedTask::done(Message::DemoSpaceOpen),
            _ => IcedTask::none(),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Keyboard Handler
// ══════════════════════════════════════════════════════════════════════════════

fn handle_key_press(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    use keyboard::key::Named;
    use keyboard::Key;

    match (modifiers.control(), modifiers.shift(), &key) {
        // Ctrl+P → command palette
        (true, false, Key::Character(c)) if c.as_str() == "p" => Some(Message::CommandPaletteOpen),
        // Ctrl+N → new note
        (true, false, Key::Character(c)) if c.as_str() == "n" => Some(Message::NewNotePrompt),
        // Ctrl+Shift+F → search
        (true, true, Key::Character(c)) if c.as_str() == "f" => Some(Message::SearchOpen),
        // Ctrl+B → toggle sidebar
        (true, false, Key::Character(c)) if c.as_str() == "b" => Some(Message::SidebarToggle),
        // Ctrl+J → toggle terminal
        (true, false, Key::Character(c)) if c.as_str() == "j" => Some(Message::TerminalToggle),
        // Ctrl+Shift+B → toggle feature panel
        (true, true, Key::Character(c)) if c.as_str() == "b" => Some(Message::FeaturePanelToggle),
        // Ctrl+Shift+A → toggle AI
        (true, true, Key::Character(c)) if c.as_str() == "a" => Some(Message::AiChatToggle),
        // Ctrl+Shift+G → graph
        (true, true, Key::Character(c)) if c.as_str() == "g" => Some(Message::GraphToggle),
        // Ctrl+Shift+T → tasks
        (true, true, Key::Character(c)) if c.as_str() == "t" => Some(Message::TasksToggle),
        // Ctrl+S → save
        (true, false, Key::Character(c)) if c.as_str() == "s" => Some(Message::EditorSave),
        // Ctrl+W → close tab/pane
        (true, false, Key::Character(c)) if c.as_str() == "w" => Some(Message::PaneClose),
        // Ctrl+\ → split vertical
        (true, false, Key::Character(c)) if c.as_str() == "\\" => Some(Message::PaneSplitVertical),
        // Ctrl+Shift+\ → split horizontal
        (true, true, Key::Character(c)) if c.as_str() == "\\" => Some(Message::PaneSplitHorizontal),
        // Ctrl+E → external editor
        (true, false, Key::Character(c)) if c.as_str() == "e" => Some(Message::EditorExternalOpen),
        // Ctrl+Tab → next tab
        (true, false, Key::Named(Named::Tab)) => Some(Message::Noop),
        // ? → keybindings help
        (false, true, Key::Character(c)) if c.as_str() == "/" => Some(Message::KeybindingsToggle),
        // Arrow Up → palette navigate up
        (false, false, Key::Named(Named::ArrowUp)) => Some(Message::CommandPaletteUp),
        // Arrow Down → palette navigate down
        (false, false, Key::Named(Named::ArrowDown)) => Some(Message::CommandPaletteDown),
        // Enter → palette submit
        (false, false, Key::Named(Named::Enter)) => Some(Message::CommandPaletteSubmit),
        // Escape → close overlays
        (false, false, Key::Named(Named::Escape)) => Some(Message::CommandPaletteClose),
        _ => None,
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Async Data Loaders (with tracing)
// ══════════════════════════════════════════════════════════════════════════════

fn strip_terminal_ansi(text: &str) -> String {
    static ANSI_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let ansi = ANSI_REGEX
        .get_or_init(|| regex::Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").expect("valid ansi regex"));
    ansi.replace_all(text, "").into_owned()
}

fn spawn_ai_chat_stream(message: String, kb_name: String) -> Receiver<AiStreamEvent> {
    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let _ = perform_ai_chat_stream(message, kb_name, tx).await;
    });
    rx
}

fn load_settings_form() -> GuiSettingsForm {
    let defaults = crate::config::manager::TypedConfig::default();
    let config_path = crate::fs::mald_home().join("config").join("config.json");
    if let Ok(config) = crate::config::manager::ConfigManager::load(&config_path) {
        let typed = config.typed();
        GuiSettingsForm {
            editor: typed.editor,
            default_kb: typed.default_kb,
            ai_model: typed.ai.default_model,
            ollama_url: typed.ai.ollama_url,
            embedding_model: typed.ai.embedding_model,
            shell: typed.session.shell,
            daemon_auto_start: typed.daemon.auto_start,
            dirty: false,
            saving: false,
        }
    } else {
        GuiSettingsForm {
            editor: defaults.editor,
            default_kb: defaults.default_kb,
            ai_model: defaults.ai.default_model,
            ollama_url: defaults.ai.ollama_url,
            embedding_model: defaults.ai.embedding_model,
            shell: defaults.session.shell,
            daemon_auto_start: defaults.daemon.auto_start,
            dirty: false,
            saving: false,
        }
    }
}

async fn save_settings_form(mut form: GuiSettingsForm) -> Result<GuiSettingsForm, String> {
    use serde_json::Value;

    let defaults = crate::config::manager::TypedConfig::default();
    form.editor = normalize_settings_value(&form.editor, &defaults.editor);
    form.default_kb = normalize_settings_value(&form.default_kb, &defaults.default_kb);
    form.ai_model = normalize_settings_value(&form.ai_model, &defaults.ai.default_model);
    form.ollama_url = normalize_settings_value(&form.ollama_url, &defaults.ai.ollama_url);
    form.embedding_model =
        normalize_settings_value(&form.embedding_model, &defaults.ai.embedding_model);
    form.shell = normalize_settings_value(&form.shell, &defaults.session.shell);

    let config_path = crate::fs::mald_home().join("config").join("config.json");
    let mut config = crate::config::manager::ConfigManager::load(&config_path)
        .map_err(|error| format!("Failed to load config: {error}"))?;

    let updates = [
        ("editor", Value::String(form.editor.clone())),
        ("default_kb", Value::String(form.default_kb.clone())),
        ("ai.default_model", Value::String(form.ai_model.clone())),
        ("ai.ollama_url", Value::String(form.ollama_url.clone())),
        (
            "ai.embedding_model",
            Value::String(form.embedding_model.clone()),
        ),
        ("session.shell", Value::String(form.shell.clone())),
        ("daemon.auto_start", Value::Bool(form.daemon_auto_start)),
    ];

    for (key, value) in updates {
        config
            .set(key, value)
            .map_err(|error| format!("Failed to save `{key}`: {error}"))?;
    }

    form.dirty = false;
    form.saving = false;
    Ok(form)
}

async fn repair_shell_path() -> Result<String, String> {
    crate::commands::setup::ensure_shell_command().map_err(|error| error.to_string())
}

fn normalize_settings_value(value: &str, default: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_default_kb_name() -> String {
    let config_path = crate::fs::mald_home().join("config").join("config.json");
    if config_path.exists() {
        if let Ok(config) = crate::config::manager::ConfigManager::load(&config_path) {
            return config.typed().default_kb;
        }
    }
    "personal".into()
}

fn workspace_kbs() -> Vec<String> {
    crate::commands::kb::available_kbs().unwrap_or_default()
}

fn kb_name_for_path(path: &std::path::Path) -> Option<String> {
    let kb_root = crate::fs::mald_home().join("kb");
    path.strip_prefix(&kb_root)
        .ok()?
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
}

fn is_external_url(target: &str) -> bool {
    matches!(
        target.split('#').next().unwrap_or(target),
        link if link.starts_with("http://")
            || link.starts_with("https://")
            || link.starts_with("mailto:")
    )
}

fn open_external_target(target: &str) -> Result<(), String> {
    if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", target])
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Failed to open link: {error}"))
    } else {
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Failed to open link: {error}"))
    }
}

fn decode_markdown_target(target: &str) -> String {
    target
        .replace("%20", " ")
        .replace("%23", "#")
        .replace("%5B", "[")
        .replace("%5D", "]")
}

fn resolve_note_in_kb(kb_name: &str, note_name: &str) -> Option<PathBuf> {
    let target_key = kb_note_key(kb_name, note_name);

    load_kb_files(None)
        .into_iter()
        .find(|file| kb_note_key(&file.kb_name, &file.name) == target_key)
        .map(|file| file.path)
}

fn resolve_markdown_link_target(
    current_path: Option<&std::path::Path>,
    current_kb: &str,
    raw_target: &str,
) -> Option<PathBuf> {
    let target_without_anchor = raw_target.split('#').next().unwrap_or(raw_target);
    if target_without_anchor.is_empty() {
        return None;
    }

    if let Some(note) = target_without_anchor.strip_prefix("mald-note://") {
        return resolve_note_in_kb(current_kb, &decode_markdown_target(note));
    }

    let decoded = decode_markdown_target(target_without_anchor);
    let link_path = std::path::Path::new(&decoded);

    if link_path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        if let Some(current_dir) = current_path.and_then(std::path::Path::parent) {
            let candidate = current_dir.join(link_path);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        let kb_candidate = crate::fs::mald_home()
            .join("kb")
            .join(current_kb)
            .join(link_path);
        if kb_candidate.exists() {
            return Some(kb_candidate);
        }

        if let Some(stem) = link_path.file_stem().and_then(|stem| stem.to_str()) {
            return resolve_note_in_kb(current_kb, stem);
        }
    }

    resolve_note_in_kb(current_kb, &decoded)
}

async fn create_new_note(
    title: String,
    kb_name: String,
    path: Option<String>,
) -> Result<PathBuf, String> {
    crate::commands::new::create_note(&title, Some(&kb_name), path.as_deref())
        .await
        .map_err(|error| error.to_string())
}

async fn open_demo_space() -> Result<PathBuf, String> {
    crate::commands::demo::activate_demo_space(false)
        .map(|space| space.entry_note)
        .map_err(|error| error.to_string())
}

async fn save_default_kb_name(kb_name: String) -> Result<String, String> {
    crate::commands::kb::set_default_kb_sync(&kb_name)
        .map(|_| kb_name)
        .map_err(|error| error.to_string())
}

async fn rebuild_search_index() -> Result<usize, String> {
    crate::commands::reindex::rebuild()
        .await
        .map_err(|error| error.to_string())
}

async fn run_doctor_report() -> Result<DoctorSummary, String> {
    let report = crate::commands::doctor::collect_report()
        .await
        .map_err(|error| error.to_string())?;

    Ok(DoctorSummary {
        output: report.render_plain(),
        issues: report.issues,
        warnings: report.warnings,
    })
}

async fn load_daemon_status() -> DaemonStatus {
    if !crate::commands::daemon::is_running() {
        return DaemonStatus::Stopped;
    }

    match tokio::time::timeout(
        Duration::from_millis(900),
        crate::commands::daemon::query_health(),
    )
    .await
    {
        Ok(Some(health)) if health.healthy => DaemonStatus::Running,
        Ok(Some(_)) => DaemonStatus::Unknown,
        Ok(None) | Err(_) => DaemonStatus::Unknown,
    }
}

async fn perform_ai_chat_stream(
    message: String,
    kb_name: String,
    tx: std::sync::mpsc::Sender<AiStreamEvent>,
) -> Result<(), String> {
    let config_path = crate::fs::mald_home().join("config").join("config.json");
    let config = crate::config::manager::ConfigManager::load(&config_path)
        .map_err(|error| format!("Failed to load config: {error}"))?;
    let client = crate::ai::ollama::OllamaClient::from_config(&config);

    if !client.is_running().await {
        let _ = tx.send(AiStreamEvent::Error(
            "Ollama is not running. Start it or run `mald ai setup`.".into(),
        ));
        return Ok(());
    }

    let mut session = crate::ai::history::latest_session(&kb_name)
        .unwrap_or_else(|| crate::ai::history::ChatSession::new(&kb_name));

    let prepared =
        crate::ai::chat::prepare_rag_chat(&client, &config, &message, &kb_name, Some(&session))
            .await
            .map_err(|error| error.to_string())?;

    let response = client
        .chat_streaming_with_callback(&prepared.model, &prepared.messages, |chunk| {
            let _ = tx.send(AiStreamEvent::Chunk(chunk.to_string()));
        })
        .await
        .map_err(|error| error.to_string());

    match response {
        Ok(full_response) => {
            session.add("user", &message);
            session.add("assistant", &full_response);
            session.save().map_err(|error| error.to_string())?;
            let citations = crate::ai::chat::format_citations(&prepared.sources);
            let _ = tx.send(AiStreamEvent::Finished(citations));
        }
        Err(error) => {
            let _ = tx.send(AiStreamEvent::Error(error));
        }
    }

    Ok(())
}

async fn load_file_tree_for(kb_name: Option<String>) -> Vec<FileEntry> {
    tracing::debug!("Loading file tree");
    let kb_dir = crate::fs::mald_home().join("kb");
    if !kb_dir.exists() {
        tracing::debug!("KB directory does not exist");
        return Vec::new();
    }
    let root = kb_name
        .as_ref()
        .map(|name| kb_dir.join(name))
        .filter(|path| path.exists())
        .unwrap_or(kb_dir);
    let mut entries = Vec::new();
    collect_entries(&root, 0, &mut entries);
    tracing::debug!("Loaded {} file tree entries", entries.len());
    entries
}

fn collect_entries(dir: &std::path::Path, depth: usize, out: &mut Vec<FileEntry>) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
        items.sort_by(|a, b| {
            let a_dir = a.path().is_dir();
            let b_dir = b.path().is_dir();
            b_dir
                .cmp(&a_dir)
                .then_with(|| a.file_name().cmp(&b.file_name()))
        });

        for entry in items {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_dir = path.is_dir();

            out.push(FileEntry {
                path: path.clone(),
                name,
                is_dir,
                depth,
                expanded: depth == 0,
            });

            if is_dir {
                collect_entries(&path, depth + 1, out);
            }
        }
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, String), String> {
    std::fs::read_to_string(&path)
        .map(|content| (path, content))
        .map_err(|e| e.to_string())
}

async fn save_file(path: PathBuf, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| e.to_string())
}

async fn fuzzy_match_notes(query: String) -> Vec<String> {
    let kb_dir = crate::fs::mald_home().join("kb");
    if !kb_dir.exists() {
        return Vec::new();
    }
    let mut results = Vec::new();
    if let Ok(files) = crate::fs::find_files(&kb_dir, "md") {
        let query_lower = query.to_lowercase();
        for f in files {
            if let Some(stem) = f.file_stem() {
                let name = stem.to_string_lossy().to_string();
                if name.to_lowercase().contains(&query_lower) {
                    results.push(name);
                }
            }
        }
    }
    results.sort();
    results.truncate(20);
    results
}

async fn perform_search(query: String) -> Vec<SearchResult> {
    tracing::debug!("Performing search: {}", query);
    let index_dir = crate::fs::mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");
    if !meta_path.exists() {
        tracing::debug!("Metadata DB does not exist");
        return Vec::new();
    }

    let meta = match crate::index::metadata::MetadataStore::open(&meta_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("Failed to open metadata store: {}", e);
            return Vec::new();
        }
    };

    match meta.fts_search(&query, 20) {
        Ok(results) => results
            .into_iter()
            .map(|r| SearchResult {
                path: PathBuf::from(&r.path),
                title: r.title,
                snippet: r.snippet,
                score: 1.0,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

async fn load_index_stats() -> Option<IndexStats> {
    tokio::task::spawn_blocking(|| {
        fn count_md(dir: &std::path::Path, total: &mut usize, depth: u32) {
            if depth > 16 {
                return;
            }
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                let path = entry.path();
                if file_type.is_dir() {
                    count_md(&path, total, depth + 1);
                } else if file_type.is_file()
                    && path
                        .extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                {
                    *total = total.saturating_add(1);
                }
            }
        }

        let kb_dir = crate::fs::mald_home().join("kb");
        let mut total: usize = 0;
        if kb_dir.exists() {
            count_md(&kb_dir, &mut total, 0);
        }

        let meta_path = crate::fs::mald_home().join("index").join("metadata.db");
        let indexed = if meta_path.exists() {
            crate::index::metadata::MetadataStore::open(&meta_path)
                .ok()
                .and_then(|m| m.document_count().ok())
                .unwrap_or(0)
        } else {
            0
        };

        Some(IndexStats { indexed, total })
    })
    .await
    .ok()
    .flatten()
}

/// Cached KB file data — read once, used by graph/tasks/backlinks.
/// Prevents redundant directory traversal and file I/O.
struct KbFile {
    path: PathBuf,
    name: String,
    kb_name: String,
    content: String,
    links: Vec<String>,
}

/// Read all markdown files from all KBs once. Returns the shared snapshot.
fn load_kb_files(kb_filter: Option<&str>) -> Vec<KbFile> {
    let kb_dir = crate::fs::mald_home().join("kb");
    if !kb_dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    if let Ok(dirs) = std::fs::read_dir(&kb_dir) {
        for dir_entry in dirs.filter_map(|e| e.ok()) {
            let dir = dir_entry.path();
            if !dir.is_dir() {
                continue;
            }
            let kb_name = dir
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(filter) = kb_filter {
                if kb_name != filter {
                    continue;
                }
            }

            if let Ok(md_files) = crate::fs::find_files(&dir, "md") {
                for f in md_files {
                    if let Ok(content) = std::fs::read_to_string(&f) {
                        let name = f
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let links = crate::parser::links::extract_wikilinks(&content);
                        files.push(KbFile {
                            path: f,
                            name,
                            kb_name: kb_name.clone(),
                            content,
                            links,
                        });
                    }
                }
            }
        }
    }
    files
}

fn normalize_note_key(name: &str) -> String {
    name.trim().to_lowercase().replace(' ', "-")
}

fn kb_note_key(kb_name: &str, note_name: &str) -> String {
    format!("{kb_name}::{}", normalize_note_key(note_name))
}

fn graph_node_id(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

async fn load_graph_for(kb_name: Option<String>) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let kb_files = load_kb_files(kb_name.as_deref());
    if kb_files.is_empty() {
        return (Vec::new(), Vec::new());
    }

    use std::collections::{HashMap, HashSet};

    let canonical_notes: HashMap<String, (&String, &PathBuf)> = kb_files
        .iter()
        .map(|kf| (kb_note_key(&kf.kb_name, &kf.name), (&kf.name, &kf.path)))
        .collect();

    let mut edges = Vec::new();
    let mut seen_edges = HashSet::new();
    for kf in &kb_files {
        let from_id = graph_node_id(&kf.path);
        for link in &kf.links {
            if let Some((_, target_path)) = canonical_notes.get(&kb_note_key(&kf.kb_name, link)) {
                let to_id = graph_node_id(target_path);
                if from_id == to_id || !seen_edges.insert((from_id.clone(), to_id.clone())) {
                    continue;
                }
                edges.push(GraphEdge {
                    from: from_id.clone(),
                    to: to_id,
                });
            }
        }
    }

    let mut degree_map: HashMap<String, usize> = HashMap::new();
    for edge in &edges {
        *degree_map.entry(edge.from.clone()).or_default() += 1;
        *degree_map.entry(edge.to.clone()).or_default() += 1;
    }

    let mut ordered_files: Vec<&KbFile> = kb_files.iter().collect();
    ordered_files.sort_by(|a, b| {
        degree_map
            .get(&graph_node_id(&b.path))
            .copied()
            .unwrap_or(0)
            .cmp(
                &degree_map
                    .get(&graph_node_id(&a.path))
                    .copied()
                    .unwrap_or(0),
            )
            .then_with(|| a.kb_name.cmp(&b.kb_name))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let mut positions: HashMap<String, (f32, f32)> = HashMap::new();
    for (i, file) in ordered_files.iter().enumerate() {
        let node_id = graph_node_id(&file.path);
        let degree = degree_map.get(&node_id).copied().unwrap_or(0) as f32;
        let angle = (i as f32) * 2.399_963_1;
        let radius = if i == 0 {
            0.0
        } else {
            (90.0 + (i as f32).sqrt() * 62.0 - degree * 8.0).max(36.0)
        };
        positions.insert(node_id, (radius * angle.cos(), radius * angle.sin()));
    }

    let mut nodes = Vec::new();
    for file in ordered_files {
        let node_id = graph_node_id(&file.path);
        let degree = degree_map.get(&node_id).copied().unwrap_or(0);
        let (x, y) = positions.get(&node_id).copied().unwrap_or((0.0, 0.0));
        nodes.push(GraphNode {
            id: node_id,
            label: file.name.clone(),
            path: file.path.clone(),
            kb: file.kb_name.clone(),
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass: 1.2 + (degree as f32) * 0.22,
            pinned: false,
            degree,
        });
    }

    (nodes, edges)
}

async fn load_tasks_for(kb_name: Option<String>) -> Vec<TaskItem> {
    let kb_files = load_kb_files(kb_name.as_deref());
    let mut tasks = Vec::new();

    for kf in &kb_files {
        for line in kf.content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                let text = rest.trim().to_string();
                if !text.is_empty() {
                    tasks.push(TaskItem {
                        text,
                        note: kf.name.clone(),
                        kb: kf.kb_name.clone(),
                        done: false,
                        path: kf.path.clone(),
                    });
                }
            } else if let Some(rest) = trimmed
                .strip_prefix("- [x] ")
                .or_else(|| trimmed.strip_prefix("- [X] "))
            {
                tasks.push(TaskItem {
                    text: rest.trim().to_string(),
                    note: kf.name.clone(),
                    kb: kf.kb_name.clone(),
                    done: true,
                    path: kf.path.clone(),
                });
            }
        }
    }

    tasks
}

async fn load_backlinks(path: PathBuf) -> Vec<BacklinkEntry> {
    let Some(target_kb) = kb_name_for_path(&path) else {
        return Vec::new();
    };
    let target_name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string().to_lowercase())
        .unwrap_or_default();
    let target_key = kb_note_key(&target_kb, &target_name);

    let kb_files = load_kb_files(Some(&target_kb));
    let mut backlinks = Vec::new();

    for kf in &kb_files {
        if kf.path == path || kf.kb_name != target_kb {
            continue;
        }
        // Links already extracted in load_kb_files — no redundant parsing
        for link in &kf.links {
            if kb_note_key(&kf.kb_name, link) == target_key {
                let context = kf
                    .content
                    .lines()
                    .find(|l| l.contains(&format!("[[{link}]]")))
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();
                backlinks.push(BacklinkEntry {
                    note: kf.name.clone(),
                    path: kf.path.clone(),
                    context,
                });
                break;
            }
        }
    }

    backlinks
}

fn extract_outline(content: &str) -> Vec<OutlineEntry> {
    let mut outline = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed.trim_start_matches('#').trim().to_string();
            if !text.is_empty() && level <= 6 {
                outline.push(OutlineEntry {
                    level,
                    text,
                    line: line_num + 1,
                });
            }
        }
    }
    outline
}
