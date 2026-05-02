//! Sidebar content component - mode-dependent content panel.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};
use iced_anim::widget::button;
use iced_aw::Badge;

use crate::gui::icons;
use crate::gui::message::{
    ActivityMode, FileEntry, GraphNode, GuiSettingsForm, IndexStats, Message, SearchResult,
    TaskItem,
};
use crate::gui::theme::{self, colors, spacing, themed, type_scale};
use crate::gui::widgets::empty_state;

pub struct SidebarData<'a> {
    pub mode: ActivityMode,
    pub file_entries: &'a [FileEntry],
    pub search_query: &'a str,
    pub search_results: &'a [SearchResult],
    pub graph_nodes: &'a [GraphNode],
    pub tasks: &'a [TaskItem],
    pub ai_messages: &'a [(String, String)],
    pub ai_input: &'a str,
    pub settings: &'a GuiSettingsForm,
    pub known_kbs: &'a [String],
    pub detected_editors: &'a [crate::commands::launch::DetectedEditor],
    pub mald_shell_available: bool,
    pub theme: iced::Theme,
    pub modified_paths: HashSet<PathBuf>,
    pub index_stats: Option<IndexStats>,
}

/// Render sidebar content based on the current activity mode.
pub fn view<'a>(data: SidebarData<'a>) -> Element<'a, Message> {
    let SidebarData {
        mode,
        file_entries,
        search_query,
        search_results,
        graph_nodes,
        tasks,
        ai_messages,
        ai_input,
        settings,
        known_kbs,
        detected_editors,
        mald_shell_available,
        theme,
        modified_paths,
        index_stats,
    } = data;

    // Sovereign layout — left rail always shows vault tree.
    // Other former-sidebar modes (Search/Graph/Tasks/AI/Settings) live in the
    // main pane or feature panel and are reached via the Commands footer.
    let _ = (
        mode,
        search_query,
        search_results,
        graph_nodes,
        tasks,
        ai_messages,
        ai_input,
        settings,
        known_kbs,
        detected_editors,
        mald_shell_available,
    );

    let header_badge = if file_entries.is_empty() {
        None
    } else {
        Some(file_entries.len().to_string())
    };

    let sub_color = themed(&theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let header = section_header("VAULT", header_badge, sub_color);

    let content: Element<Message> = view_files(file_entries, &theme, &modified_paths);

    let footer = indexer_footer(index_stats, &theme);
    let quick = quick_commands_footer(&theme);
    let inner = column![
        header,
        container(content).width(Length::Fill).height(Length::Fill),
        quick,
        footer,
    ]
    .spacing(0);

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::sidebar_style)
        .into()
}

fn quick_commands_footer<'a>(theme: &iced::Theme) -> Element<'a, Message> {
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let dim = themed(theme, colors::SURFACE2, colors::latte::SURFACE2);
    let accent = themed(theme, colors::ACCENT, colors::latte::ACCENT);

    let header = container(
        text("> COMMANDS")
            .size(type_scale::CAPTION)
            .color(sub_color)
            .font(iced::Font::MONOSPACE),
    )
    .padding([spacing::SM as u16, spacing::SM as u16]);

    let entry =
        |label: &'static str, shortcut: &'static str, msg: Message| -> Element<'a, Message> {
            iced_anim::widget::button::Button::new(
                row![
                    text(">")
                        .size(type_scale::CAPTION)
                        .color(accent)
                        .font(iced::Font::MONOSPACE),
                    text(label)
                        .size(type_scale::CAPTION)
                        .color(text_color)
                        .font(iced::Font::MONOSPACE),
                    Space::new().width(Length::Fill),
                    text(shortcut)
                        .size(type_scale::CAPTION)
                        .color(dim)
                        .font(iced::Font::MONOSPACE),
                ]
                .align_y(iced::Alignment::Center)
                .spacing(spacing::SM),
            )
            .on_press(msg)
            .padding([spacing::XS as u16, spacing::SM as u16])
            .width(Length::Fill)
            .style(theme::list_item_style(false))
            .into()
        };

    let body = column![
        entry("home", "", Message::GoHome),
        entry(
            "ask",
            "Ctrl+Shift+B",
            Message::FeaturePanelSetContent(crate::gui::message::FeaturePanelContent::AIChat),
        ),
        entry("search", "Ctrl+Shift+F", Message::SearchOpen),
        entry("new", "Ctrl+N", Message::NewNotePrompt),
        entry("graph", "", Message::ActivityBarSelect(ActivityMode::Graph),),
        entry("tasks", "", Message::ActivityBarSelect(ActivityMode::Tasks),),
        entry("index", "", Message::ReindexRequested),
        entry(
            "settings",
            "",
            Message::ActivityBarSelect(ActivityMode::Settings),
        ),
    ]
    .spacing(0);

    container(column![header, body].spacing(0))
        .width(Length::Fill)
        .style(theme::section_header_style)
        .into()
}

fn indexer_footer<'a>(stats: Option<IndexStats>, theme: &iced::Theme) -> Element<'a, Message> {
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let dim_color = themed(theme, colors::SURFACE2, colors::latte::SURFACE2);
    let accent = themed(theme, colors::ACCENT, colors::latte::ACCENT);

    let label = match stats {
        Some(s) if s.total > 0 => {
            format!("Indexed: {} files ({}%)", s.indexed, s.percent())
        }
        Some(s) => format!("Indexed: {} files", s.indexed),
        None => "Indexed: —".to_string(),
    };

    let dot = text("●").size(type_scale::CAPTION).color(match stats {
        Some(s) if s.total > 0 && s.indexed >= s.total => accent,
        Some(_) => sub_color,
        None => dim_color,
    });

    let line = row![dot, text(label).size(type_scale::CAPTION).color(sub_color),]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

    container(line)
        .padding([spacing::XS as u16, spacing::SM as u16])
        .width(Length::Fill)
        .style(theme::section_header_style)
        .into()
}

fn section_header<'a>(
    title: &'a str,
    badge: Option<String>,
    sub_color: iced::Color,
) -> Element<'a, Message> {
    let mut header_row = row![
        text(title.to_uppercase())
            .size(type_scale::UI)
            .color(sub_color),
        Space::new().width(Length::Fill),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center);

    if let Some(badge_text) = badge {
        let badge = Badge::new(text(badge_text).size(type_scale::CAPTION))
            .padding(spacing::XS as u16)
            .style(theme::aw_badge_style);
        header_row = header_row.push(badge);
    }

    container(header_row.padding(spacing::SM as u16))
        .style(theme::section_header_style)
        .width(Length::Fill)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Files view
// ─────────────────────────────────────────────────────────────────────────────

fn view_files<'a>(
    entries: &'a [FileEntry],
    theme: &iced::Theme,
    modified_paths: &HashSet<PathBuf>,
) -> Element<'a, Message> {
    if entries.is_empty() {
        return empty_state::presets::no_files(!theme::is_light_theme(theme));
    }

    // Filter entries: only show children of expanded directories
    let mut visible: Vec<&FileEntry> = Vec::new();
    let mut collapsed_depth: Option<usize> = None;

    for entry in entries {
        if let Some(cd) = collapsed_depth {
            if entry.depth > cd {
                continue;
            } else {
                collapsed_depth = None;
            }
        }
        visible.push(entry);
        if entry.is_dir && !entry.expanded {
            collapsed_depth = Some(entry.depth);
        }
    }

    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let accent_color = themed(theme, colors::LAVENDER, colors::latte::LAVENDER);
    let yellow = themed(theme, colors::YELLOW, colors::latte::YELLOW);

    let items: Vec<Element<Message>> = visible
        .into_iter()
        .map(|entry| {
            let is_modified = modified_paths.contains(&entry.path);
            file_entry_row(
                entry,
                text_color,
                sub_color,
                accent_color,
                yellow,
                is_modified,
            )
        })
        .collect();

    scrollable(column(items).spacing(1).padding([0, spacing::XS as u16]))
        .height(Length::Fill)
        .style(theme::scrollable_style)
        .into()
}

fn file_entry_row(
    entry: &FileEntry,
    text_color: iced::Color,
    sub_color: iced::Color,
    accent_color: iced::Color,
    yellow: iced::Color,
    is_modified: bool,
) -> Element<'_, Message> {
    let indent = 16.0 * entry.depth as f32;

    let chevron: Element<Message> = if entry.is_dir {
        if entry.expanded {
            icons::chevron_down()
                .size(type_scale::UI)
                .color(sub_color)
                .into()
        } else {
            icons::chevron_right()
                .size(type_scale::UI)
                .color(sub_color)
                .into()
        }
    } else {
        Space::new()
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(12.0))
            .into()
    };

    let icon = if entry.is_dir {
        if entry.expanded {
            icons::folder_open().color(accent_color)
        } else {
            icons::folder_closed().color(sub_color)
        }
    } else {
        icons::file().color(sub_color)
    };

    let name_text = text(&entry.name).size(type_scale::UI).color(text_color);

    let mut content_row = row![
        Space::new().width(Length::Fixed(indent)),
        chevron,
        icon,
        name_text,
    ]
    .spacing(spacing::XS);

    // Modified file indicator - yellow dot on right side
    if is_modified {
        content_row = content_row.push(Space::new().width(Length::Fill));
        content_row = content_row.push(text("\u{25CF}").size(type_scale::CAPTION).color(yellow));
    }

    let msg = if entry.is_dir {
        if entry.expanded {
            Message::FileTreeCollapse(entry.path.clone())
        } else {
            Message::FileTreeExpand(entry.path.clone())
        }
    } else {
        Message::FileTreeSelect(entry.path.clone())
    };

    button(
        container(content_row)
            .padding([spacing::XS as u16, spacing::SM as u16])
            .width(Length::Fill),
    )
    .on_press(msg)
    .width(Length::Fill)
    .style(theme::list_item_style(false))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Search view
// ─────────────────────────────────────────────────────────────────────────────

fn view_search<'a>(
    query: &'a str,
    results: &'a [SearchResult],
    theme: &iced::Theme,
) -> Element<'a, Message> {
    let input = text_input("Search notes...", query)
        .on_input(Message::SearchQueryChanged)
        .padding(spacing::SM as u16)
        .style(theme::search_input_style);

    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);

    let results_list: Element<Message> = if results.is_empty() {
        let is_dark = !theme::is_light_theme(theme);
        if query.is_empty() {
            empty_state::presets::search_prompt(is_dark)
        } else {
            empty_state::presets::no_search_results(is_dark)
        }
    } else {
        let items: Vec<Element<Message>> = results
            .iter()
            .enumerate()
            .map(|(i, result)| search_result_row(i, result, text_color, sub_color))
            .collect();

        scrollable(column(items).spacing(spacing::XS))
            .height(Length::Fill)
            .style(theme::scrollable_style)
            .into()
    };

    column![
        container(input).padding(spacing::SM as u16),
        container(results_list)
            .padding([0, spacing::SM as u16])
            .height(Length::Fill),
    ]
    .spacing(spacing::XS)
    .into()
}

fn search_result_row(
    idx: usize,
    result: &SearchResult,
    text_color: iced::Color,
    sub_color: iced::Color,
) -> Element<'_, Message> {
    let title = text(&result.title).size(type_scale::UI).color(text_color);
    let snippet = text(&result.snippet)
        .size(type_scale::CAPTION)
        .color(sub_color);

    button(
        column![title, snippet]
            .spacing(spacing::XS)
            .padding(spacing::XS as u16),
    )
    .on_press(Message::SearchResultSelect(idx))
    .width(Length::Fill)
    .style(theme::list_item_style(false))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Graph view (node list)
// ─────────────────────────────────────────────────────────────────────────────

fn view_graph<'a>(nodes: &'a [GraphNode], theme: &iced::Theme) -> Element<'a, Message> {
    if nodes.is_empty() {
        return empty_state::presets::no_graph(!theme::is_light_theme(theme));
    }

    let mut sorted_nodes: Vec<&GraphNode> = nodes.iter().collect();
    sorted_nodes.sort_by(|a, b| {
        b.degree
            .cmp(&a.degree)
            .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
    });

    let connected_count = nodes.iter().filter(|node| node.degree > 0).count();
    let orphan_count = nodes.len().saturating_sub(connected_count);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let accent_color = themed(theme, colors::LAVENDER, colors::latte::LAVENDER);
    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let teal = themed(theme, colors::TEAL, colors::latte::TEAL);
    let yellow = themed(theme, colors::YELLOW, colors::latte::YELLOW);

    let items: Vec<Element<Message>> = sorted_nodes
        .into_iter()
        .map(|node| {
            let degree_badge = container(
                text(format!("{}", node.degree))
                    .size(type_scale::CAPTION)
                    .color(if node.degree == 0 { sub_color } else { teal }),
            )
            .padding([2, 6])
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.10,
                    ..if node.degree == 0 { yellow } else { teal }
                })),
                border: iced::Border {
                    color: iced::Color {
                        a: 0.20,
                        ..if node.degree == 0 { yellow } else { teal }
                    },
                    width: 1.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            });

            button(
                row![
                    column![
                        text(&node.label).size(type_scale::UI).color(accent_color),
                        text(format!(
                            "{} · {}",
                            node.kb,
                            if node.degree == 0 {
                                "Orphan note"
                            } else {
                                "Connected note"
                            }
                        ))
                        .size(type_scale::CAPTION)
                        .color(sub_color),
                    ]
                    .spacing(2),
                    Space::new().width(Length::Fill),
                    degree_badge,
                ]
                .spacing(spacing::SM)
                .align_y(Alignment::Center),
            )
            .on_press(Message::GraphNodeClick(node.path.clone()))
            .width(Length::Fill)
            .padding([spacing::XS as u16, spacing::SM as u16])
            .style(theme::list_item_style(false))
            .into()
        })
        .collect();

    column![
        container(
            column![
                text("Graph index")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                text(format!(
                    "{connected_count} connected • {orphan_count} orphaned"
                ))
                .size(type_scale::UI)
                .color(text_color),
            ]
            .spacing(2),
        )
        .padding([spacing::XS as u16, 0]),
        scrollable(column(items).spacing(spacing::XS))
            .height(Length::Fill)
            .style(theme::scrollable_style),
    ]
    .spacing(spacing::SM)
    .padding(spacing::SM as u16)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tasks view
// ─────────────────────────────────────────────────────────────────────────────

fn view_tasks<'a>(tasks: &'a [TaskItem], theme: &iced::Theme) -> Element<'a, Message> {
    if tasks.is_empty() {
        return empty_state::presets::no_tasks(!theme::is_light_theme(theme));
    }

    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let dim_color = themed(theme, colors::SURFACE2, colors::latte::SURFACE2);
    let green = themed(theme, colors::GREEN, colors::latte::GREEN);
    let yellow = themed(theme, colors::YELLOW, colors::latte::YELLOW);

    let open_count = tasks.iter().filter(|t| !t.done).count();
    let done_count = tasks.iter().filter(|t| t.done).count();

    let stats = text(format!("{open_count} open, {done_count} done"))
        .size(type_scale::UI)
        .color(sub_color);

    let items: Vec<Element<Message>> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let checkbox = if task.done {
                icons::check_box().size(type_scale::BODY).color(green)
            } else {
                icons::empty_box().size(type_scale::BODY).color(yellow)
            };

            let task_text = text(&task.text).size(type_scale::UI).color(if task.done {
                sub_color
            } else {
                text_color
            });

            let note_ref = text(format!("in {}", task.note))
                .size(type_scale::CAPTION)
                .color(dim_color);

            button(
                column![row![checkbox, task_text].spacing(spacing::SM), note_ref,]
                    .spacing(spacing::XS)
                    .padding(spacing::XS as u16),
            )
            .on_press(Message::TaskClick(i))
            .width(Length::Fill)
            .style(theme::list_item_style(false))
            .into()
        })
        .collect();

    column![
        stats,
        scrollable(column(items).spacing(spacing::XS))
            .height(Length::Fill)
            .style(theme::scrollable_style),
    ]
    .spacing(spacing::SM)
    .padding(spacing::SM as u16)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// AI view
// ─────────────────────────────────────────────────────────────────────────────

fn view_ai<'a>(
    messages: &'a [(String, String)],
    input: &'a str,
    theme: &iced::Theme,
) -> Element<'a, Message> {
    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let blue = themed(theme, colors::ACCENT, colors::latte::ACCENT);
    let teal = themed(theme, colors::TEAL, colors::latte::TEAL);
    let yellow = themed(theme, colors::YELLOW, colors::latte::YELLOW);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);

    let history: Vec<Element<Message>> = messages
        .iter()
        .map(|(role, content)| {
            let (prefix, color) = match role.as_str() {
                "user" => ("You", blue),
                "assistant" => ("AI", teal),
                _ => ("System", yellow),
            };
            let parsed = parse_sidebar_ai_message(content);
            let body = if parsed.body.trim().is_empty() && role == "assistant" {
                "Thinking...".to_string()
            } else {
                parsed.body
            };
            let sources: Element<Message> = if parsed.sources.is_empty() {
                Space::new().height(Length::Shrink).into()
            } else {
                let items: Vec<Element<Message>> = parsed
                    .sources
                    .into_iter()
                    .map(|citation| {
                        button(
                            column![
                                text(citation.title).size(type_scale::CAPTION).color(color),
                                text(citation.location)
                                    .size(type_scale::CAPTION)
                                    .color(sub_color),
                            ]
                            .spacing(2),
                        )
                        .on_press(Message::AiChatCitationClick(citation.path))
                        .padding([spacing::XS as u16, spacing::SM as u16])
                        .width(Length::Fill)
                        .style(theme::list_item_style(false))
                        .into()
                    })
                    .collect();

                column![
                    text("Sources").size(type_scale::CAPTION).color(sub_color),
                    column(items).spacing(spacing::XS),
                ]
                .spacing(spacing::XS)
                .into()
            };
            column![
                text(prefix).size(type_scale::CAPTION).color(color),
                text(body).size(type_scale::UI).color(text_color),
                sources,
            ]
            .spacing(spacing::XS)
            .padding(spacing::XS as u16)
            .into()
        })
        .collect();

    let chat_area: Element<Message> = if messages.is_empty() {
        empty_state::presets::ai_prompt(!theme::is_light_theme(theme))
    } else {
        scrollable(column(history).spacing(spacing::SM))
            .height(Length::Fill)
            .style(theme::scrollable_style)
            .into()
    };

    let input_field = text_input("Ask something...", input)
        .on_input(Message::AiChatInputChanged)
        .on_submit(Message::AiChatSend(input.to_string()))
        .padding(spacing::SM as u16)
        .style(theme::text_input_style);

    column![
        container(chat_area)
            .height(Length::Fill)
            .padding(spacing::SM as u16),
        container(input_field).padding(spacing::SM as u16),
    ]
    .into()
}

#[derive(Debug, Clone)]
struct SidebarAiMessage {
    body: String,
    sources: Vec<SidebarAiCitation>,
}

#[derive(Debug, Clone)]
struct SidebarAiCitation {
    title: String,
    location: String,
    path: PathBuf,
}

fn parse_sidebar_ai_message(content: &str) -> SidebarAiMessage {
    let Some((body, sources_block)) = content.split_once("\n--- Sources ---\n") else {
        return SidebarAiMessage {
            body: content.trim().to_string(),
            sources: Vec::new(),
        };
    };

    SidebarAiMessage {
        body: body.trim().to_string(),
        sources: sources_block
            .lines()
            .filter_map(parse_sidebar_ai_citation)
            .collect(),
    }
}

fn parse_sidebar_ai_citation(line: &str) -> Option<SidebarAiCitation> {
    static CITATION_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = CITATION_RE.get_or_init(|| {
        regex::Regex::new(r"^\[(\d+)\]\s+(.+?)(?::L(\d+)(?:-(\d+))?)?$")
            .expect("valid citation regex")
    });

    let captures = re.captures(line.trim())?;
    let index = captures.get(1)?.as_str();
    let raw_path = captures.get(2)?.as_str();
    let path = PathBuf::from(raw_path);
    let file_label = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| raw_path.to_string());
    let location = if let Some(start) = captures.get(3) {
        let end = captures
            .get(4)
            .map(|line| line.as_str())
            .unwrap_or(start.as_str());
        format!("{} · L{}-{}", path.display(), start.as_str(), end)
    } else {
        path.display().to_string()
    };

    Some(SidebarAiCitation {
        title: format!("[{index}] {file_label}"),
        location,
        path,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Settings view
// ─────────────────────────────────────────────────────────────────────────────

/// Recommended Ollama models — current generation. Clickable rows that fill
/// the Model field.
fn model_recommendations<'a>(theme: &iced::Theme) -> Element<'a, Message> {
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let accent = themed(theme, colors::ACCENT, colors::latte::ACCENT);

    let pick = |name: &'static str| -> Element<'a, Message> {
        button(
            text(name)
                .size(type_scale::CAPTION)
                .color(accent)
                .font(iced::Font::MONOSPACE),
        )
        .on_press(Message::SettingChanged(
            "ai.default_model".into(),
            name.into(),
        ))
        .padding([2, spacing::SM as u16])
        .style(theme::secondary_button_style)
        .into()
    };

    column![
        text("Recommended (paste exact tag, then Save):")
            .size(type_scale::CAPTION)
            .color(sub_color),
        row![
            pick("gemma3:4b"),
            pick("gemma3:1b"),
            pick("qwen3:4b"),
            pick("qwen3:1.7b"),
            pick("llama3.2:3b"),
        ]
        .spacing(spacing::XS)
        .wrap(),
    ]
    .spacing(spacing::XS)
    .into()
}

fn settings_card<'a>(
    content: impl Into<Element<'a, Message>>,
    surface: iced::Color,
    surface_border: iced::Color,
) -> Element<'a, Message> {
    container(content)
        .padding(spacing::MD as u16)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border {
                color: surface_border,
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn view_settings<'a>(
    settings: &'a GuiSettingsForm,
    known_kbs: &'a [String],
    detected_editors: &'a [crate::commands::launch::DetectedEditor],
    mald_shell_available: bool,
    theme: &iced::Theme,
) -> Element<'a, Message> {
    let text_color = themed(theme, colors::TEXT, colors::latte::TEXT);
    let sub_color = themed(theme, colors::SUBTEXT0, colors::latte::SUBTEXT0);
    let accent = themed(theme, colors::LAVENDER, colors::latte::LAVENDER);
    let green = themed(theme, colors::GREEN, colors::latte::GREEN);
    let yellow = themed(theme, colors::YELLOW, colors::latte::YELLOW);
    let surface = themed(theme, colors::SURFACE0, colors::latte::SURFACE0);
    let surface_border = themed(theme, colors::SURFACE1, colors::latte::SURFACE1);

    let status_text = if settings.saving {
        text("Saving...").size(type_scale::CAPTION).color(accent)
    } else if settings.dirty {
        text("Unsaved changes")
            .size(type_scale::CAPTION)
            .color(yellow)
    } else {
        text("Config synced").size(type_scale::CAPTION).color(green)
    };

    let field = |label: &'static str, hint: &'static str, key: &'static str, value: &'a str| {
        column![
            text(label).size(type_scale::CAPTION).color(sub_color),
            text_input(hint, value)
                .on_input(move |next| Message::SettingChanged(key.into(), next))
                .padding(spacing::SM as u16)
                .style(theme::text_input_style),
        ]
        .spacing(spacing::XS)
    };

    let editor_quick_actions: Element<Message> = if detected_editors.is_empty() {
        text("No common editors detected yet. You can still type a command manually.")
            .size(type_scale::CAPTION)
            .color(sub_color)
            .into()
    } else {
        let editor_buttons = detected_editors.iter().take(3).fold(
            row![].spacing(spacing::XS).align_y(Alignment::Center),
            |row, editor| {
                row.push(
                    button(text(format!("Use {}", editor.label)).size(type_scale::CAPTION))
                        .on_press(Message::SettingChanged(
                            "editor".into(),
                            editor.command.clone(),
                        ))
                        .padding([spacing::XS as u16, spacing::SM as u16])
                        .style(theme::secondary_button_style),
                )
            },
        );

        column![
            text("Detected editors")
                .size(type_scale::CAPTION)
                .color(sub_color),
            editor_buttons,
        ]
        .spacing(spacing::XS)
        .into()
    };

    let space_quick_actions: Element<Message> = if known_kbs.is_empty() {
        text("No spaces found yet. Load the demo space or create one from the launcher.")
            .size(type_scale::CAPTION)
            .color(sub_color)
            .into()
    } else {
        let space_buttons = known_kbs.iter().take(6).fold(
            row![].spacing(spacing::XS).align_y(Alignment::Center),
            |row, kb| {
                let is_active = settings.default_kb == *kb;
                row.push(
                    button(
                        text(kb.clone())
                            .size(type_scale::CAPTION)
                            .color(if is_active { accent } else { text_color }),
                    )
                    .on_press(Message::CurrentKbSwitch(kb.clone()))
                    .padding([spacing::XS as u16, spacing::SM as u16])
                    .style(theme::ghost_button_style(is_active)),
                )
            },
        );

        column![
            text("Switch active space")
                .size(type_scale::CAPTION)
                .color(sub_color),
            space_buttons,
        ]
        .spacing(spacing::XS)
        .into()
    };

    let daemon_label = if settings.daemon_auto_start {
        "Auto-start daemon: On"
    } else {
        "Auto-start daemon: Off"
    };
    let daemon_color = if settings.daemon_auto_start {
        green
    } else {
        sub_color
    };

    let runtime_actions = row![
        button(text("Save").size(type_scale::UI))
            .on_press_maybe((!settings.saving && settings.dirty).then_some(Message::SettingsSave))
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::primary_button_style),
        button(text("Reload").size(type_scale::UI))
            .on_press_maybe((!settings.saving).then_some(Message::SettingsReset))
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::secondary_button_style),
        button(text("Theme").size(type_scale::UI))
            .on_press(Message::ThemeToggle)
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::secondary_button_style),
    ]
    .spacing(spacing::SM);

    #[cfg(windows)]
    let path_button: Option<Element<Message>> = (!mald_shell_available).then(|| {
        button(text("Add MALD to PATH").size(type_scale::UI))
            .on_press(Message::PathSetupRun)
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::secondary_button_style)
            .into()
    });

    #[cfg(not(windows))]
    let path_button: Option<Element<Message>> = None;

    let cards: Vec<Element<Message>> = vec![
        settings_card(
            column![
                row![
                    text("Settings").size(type_scale::BODY).color(text_color),
                    Space::new().width(Length::Fill),
                    status_text,
                ]
                .align_y(Alignment::Center),
                text("Change the working space, editor, and runtime behavior without touching config files.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                runtime_actions,
                if let Some(path_button) = path_button {
                    path_button
                } else {
                    Space::new().height(Length::Shrink).into()
                },
            ]
            .spacing(spacing::SM),
            surface,
            surface_border,
        ),
        settings_card(
            column![
                row![
                    text("Working space").size(type_scale::BODY).color(text_color),
                    Space::new().width(Length::Fill),
                    text(format!("Active: {}", settings.default_kb))
                        .size(type_scale::CAPTION)
                        .color(accent),
                ]
                .align_y(Alignment::Center),
                text("Switch where search, graph, AI, and new notes should work right now.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                space_quick_actions,
            ]
            .spacing(spacing::SM),
            surface,
            surface_border,
        ),
        settings_card(
            column![
                text("Editor & shell").size(type_scale::BODY).color(text_color),
                text("Pick the command MALD should use when opening notes or folders.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                field("Editor", "nvim", "editor", &settings.editor),
                editor_quick_actions,
                field("Shell", "powershell", "session.shell", &settings.shell),
                text(if mald_shell_available {
                    "MALD command is ready in new terminals."
                } else {
                    "MALD command is not on PATH yet."
                })
                .size(type_scale::CAPTION)
                .color(if mald_shell_available { green } else { yellow }),
            ]
            .spacing(spacing::SM),
            surface,
            surface_border,
        ),
        settings_card(
            column![
                text("AI backend").size(type_scale::BODY).color(text_color),
                text("These values stay local. MALD uses them when you ask AI to work against your active space.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                field("Model", "gemma3:4b", "ai.default_model", &settings.ai_model),
                model_recommendations(theme),
                field(
                    "Ollama URL",
                    "http://localhost:11434",
                    "ai.ollama_url",
                    &settings.ollama_url,
                ),
                field(
                    "Embedding model",
                    "nomic-embed-text",
                    "ai.embedding_model",
                    &settings.embedding_model,
                ),
            ]
            .spacing(spacing::SM),
            surface,
            surface_border,
        ),
        settings_card(
            column![
                text("Background service").size(type_scale::BODY).color(text_color),
                text("Keep indexing and automation ready in the background if you want faster follow-up actions.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
                button(text(daemon_label).size(type_scale::UI).color(daemon_color))
                    .on_press(Message::SettingToggle("daemon.auto_start".into()))
                    .padding([spacing::SM as u16, spacing::MD as u16])
                    .style(theme::list_item_style(false)),
                text("If new terminals still cannot find `mald`, run the PATH helper once.")
                    .size(type_scale::CAPTION)
                    .color(sub_color),
            ]
            .spacing(spacing::SM),
            surface,
            surface_border,
        ),
    ];

    scrollable(
        column(cards)
            .spacing(spacing::SM)
            .padding(spacing::SM as u16),
    )
    .height(Length::Fill)
    .style(theme::scrollable_style)
    .into()
}
