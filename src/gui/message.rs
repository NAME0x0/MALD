use iced::widget::{pane_grid, text_editor};
use std::path::PathBuf;
use std::time::Instant;

/// All UI events in the application.
#[derive(Debug, Clone)]
pub enum Message {
    // ── Navigation ──
    SwitchView(ActiveView),
    GoHome,

    // ── Activity Bar ──
    ActivityBarSelect(ActivityMode),

    // ── Sidebar ──
    SidebarToggle,
    SidebarResizeStart,
    SidebarResize(f32),
    SidebarResizeEnd,

    // ── Feature Panel ──
    FeaturePanelToggle,
    FeaturePanelSetContent(FeaturePanelContent),
    FeaturePanelResize(f32),

    // ── Top Search ──
    TopSearchFocus,
    TopSearchBlur,
    TopSearchQueryChanged(String),
    TopSearchSubmit,

    // ── File tree ──
    FileTreeToggle,
    FileTreeSelect(PathBuf),
    FileTreeExpand(PathBuf),
    FileTreeCollapse(PathBuf),
    FileTreeRefresh,
    FileTreeLoaded(Vec<FileEntry>),

    // ── Editor ──
    EditorOpen(PathBuf),
    EditorFileLoaded(PathBuf, Result<String, String>), // (path, content_or_error)
    EditorClose(usize),
    EditorCloseConfirmSave,
    EditorCloseConfirmDiscard,
    EditorCloseConfirmCancel,
    EditorCloseAfterSave(usize),
    EditorSwitchTab(usize),
    EditorContentChanged(text_editor::Action),
    EditorSave,
    EditorSaved(Result<(), String>),
    EditorExternalOpen,
    EditorPreviewSplitChanged(pane_grid::ResizeEvent),

    // ── Wikilink autocomplete ──
    AutocompleteTriggered(String),
    AutocompleteSelect(String),
    AutocompleteDismiss,
    AutocompleteResults(Vec<String>),

    // ── Terminal ──
    TerminalToggle,
    TerminalInput(String),
    TerminalSubmit,
    TerminalClear,
    TerminalRestart,
    TerminalInterrupt,
    TerminalResized { cols: u16, rows: u16 },
    TerminalHeightChanged(f32),

    // â”€â”€ Code Execution â”€â”€
    RunCodeBlock(usize),
    CodeBlockOutput { block_id: usize, output: String },
    CodeBlockError { block_id: usize, error: String },
    CodeBlockComplete(usize),

    // ── AI Chat ──
    AiChatToggle,
    AiChatSend(String),
    AiChatInputChanged(String),
    AiChatCitationClick(PathBuf),
    MarkdownLinkClick(String),

    // ── Search ──
    SearchOpen,
    SearchClose,
    SearchQueryChanged(String),
    SearchResults(Vec<SearchResult>, u64), // (results, generation)
    SearchResultSelect(usize),

    // ── Command palette ──
    CommandPaletteOpen,
    CommandPaletteClose,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(usize),
    CommandPaletteExecute(PaletteCommand),
    CommandPaletteUp,
    CommandPaletteDown,
    CommandPaletteSubmit,

    // ── Graph ──
    GraphToggle,
    GraphLoaded(Vec<GraphNode>, Vec<GraphEdge>, u64), // (nodes, edges, generation)
    GraphNodeClick(PathBuf),
    GraphZoom(f32),
    GraphPan { dx: f32, dy: f32 },
    GraphSettingsToggle,
    GraphRepelForceChanged(f32),
    GraphLinkForceChanged(f32),
    GraphLinkDistanceChanged(f32),
    GraphCenterForceChanged(f32),
    GraphPhysicsReset,
    GraphViewReset,
    GraphHideOrphansToggle,

    // ── Tasks ──
    TasksToggle,
    TasksLoaded(Vec<TaskItem>, u64), // (tasks, generation)
    TaskClick(usize),
    TaskToggleView, // list ↔ kanban

    // ── Backlinks ──
    BacklinksLoaded(Vec<BacklinkEntry>, u64), // (entries, generation)
    BacklinkClick(PathBuf),

    // ── Outline ──
    OutlineLoaded(Vec<OutlineEntry>),
    OutlineClick(usize),

    // ── Layout ──
    PaneSplitVertical,
    PaneSplitHorizontal,
    PaneClose,
    PaneResize { pane_id: usize, ratio: f32 },
    PaneFocus(usize),

    // ── Theme ──
    ThemeToggle,

    // ── Keybindings help ──
    KeybindingsToggle,

    // ── Daemon ──
    DaemonStatusRefresh,
    DaemonStatusUpdate(DaemonStatus),

    // ── Settings ──
    SettingChanged(String, String),
    SettingToggle(String),
    SettingsSave,
    SettingsReset,
    SettingsSaved(Result<GuiSettingsForm, String>),

    // ── New Note ──
    NewNotePrompt,
    NewNoteTitleChanged(String),
    NewNoteCreate(String),
    NewNoteCreated(Result<PathBuf, String>),
    ReindexCompleted(Result<usize, String>),
    DoctorCompleted(Result<DoctorSummary, String>),

    // ── Misc ──
    Tick,
    AnimationTick(Instant),
    SidebarAnimateCollapse,
    SidebarAnimateExpand,
    TerminalAnimateCollapse,
    TerminalAnimateExpand,
    FeaturePanelAnimateCollapse,
    FeaturePanelAnimateExpand,
    Noop,
    ErrorOccurred(String),

    // ── Toast notifications ──
    ToastShow { level: ToastLevel, message: String },
    ToastDismiss(usize),
    ToastTimeout(usize),
}

/// Toast severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

// ══════════════════════════════════════════════════════════════════════════════
// Core Enums
// ══════════════════════════════════════════════════════════════════════════════

/// Activity bar mode selection (left-most icon strip)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivityMode {
    #[default]
    Files,
    Search,
    Graph,
    Tasks,
    AI,
    Settings,
}

impl ActivityMode {
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityMode::Files => "F",
            ActivityMode::Search => "S",
            ActivityMode::Graph => "G",
            ActivityMode::Tasks => "T",
            ActivityMode::AI => "AI",
            ActivityMode::Settings => "SET",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActivityMode::Files => "Explorer",
            ActivityMode::Search => "Search",
            ActivityMode::Graph => "Graph",
            ActivityMode::Tasks => "Tasks",
            ActivityMode::AI => "AI",
            ActivityMode::Settings => "Settings",
        }
    }

    pub fn all() -> &'static [ActivityMode] {
        &[
            ActivityMode::Files,
            ActivityMode::Search,
            ActivityMode::Graph,
            ActivityMode::Tasks,
            ActivityMode::AI,
            ActivityMode::Settings,
        ]
    }
}

/// Content shown in the right feature panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FeaturePanelContent {
    #[default]
    Backlinks,
    AIChat,
    Outline,
}

impl FeaturePanelContent {
    pub fn label(&self) -> &'static str {
        match self {
            FeaturePanelContent::Backlinks => "Backlinks",
            FeaturePanelContent::AIChat => "AI Chat",
            FeaturePanelContent::Outline => "Outline",
        }
    }

    pub fn all() -> &'static [FeaturePanelContent] {
        &[
            FeaturePanelContent::Backlinks,
            FeaturePanelContent::AIChat,
            FeaturePanelContent::Outline,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Home,
    Editor,
    Graph,
    Search,
    Tasks,
}

// ══════════════════════════════════════════════════════════════════════════════
// Supporting Types
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct PaletteCommand {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DoctorSummary {
    pub output: String,
    pub issues: u32,
    pub warnings: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GuiSettingsForm {
    pub editor: String,
    pub default_kb: String,
    pub ai_model: String,
    pub ollama_url: String,
    pub embedding_model: String,
    pub shell: String,
    pub daemon_auto_start: bool,
    pub dirty: bool,
    pub saving: bool,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
    pub kb: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub pinned: bool,
    pub degree: usize,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct TaskItem {
    pub text: String,
    pub note: String,
    pub kb: String,
    pub done: bool,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BacklinkEntry {
    pub note: String,
    pub path: PathBuf,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct OutlineEntry {
    pub level: usize,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonStatus {
    Running,
    Stopped,
    Unknown,
}
