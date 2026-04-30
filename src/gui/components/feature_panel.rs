//! Feature panel component - right panel for backlinks, AI chat, or outline.

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};
use iced_anim::widget::button;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::gui::icons;
use crate::gui::message::{
    AskMode, BacklinkEntry, ContextSnapshot, FeaturePanelContent, Message, OutlineEntry,
};
use crate::gui::theme::{self, colors, layout};

/// Render the feature panel.
#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    content_type: FeaturePanelContent,
    backlinks: &'a [BacklinkEntry],
    outline: &'a [OutlineEntry],
    ai_messages: &'a [(String, String)],
    ai_input: &'a str,
    ai_streaming: bool,
    ask_mode: AskMode,
    context_snapshot: Option<&'a ContextSnapshot>,
    context_extracting: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    let header = panel_header(content_type, is_dark);
    let content: Element<Message> = match content_type {
        FeaturePanelContent::Context => view_context(context_snapshot, context_extracting, is_dark),
        FeaturePanelContent::Backlinks => view_backlinks(backlinks, is_dark),
        FeaturePanelContent::Outline => view_outline(outline, is_dark),
        FeaturePanelContent::AIChat => {
            view_ai_chat(ai_messages, ai_input, ai_streaming, ask_mode, is_dark)
        }
        FeaturePanelContent::Timeline => view_timeline(is_dark),
    };

    let inner = column![header, content].spacing(0);

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::feature_panel_style)
        .into()
}

fn panel_header(content_type: FeaturePanelContent, is_dark: bool) -> Element<'static, Message> {
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let blue = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };

    let tabs: Vec<Element<Message>> = FeaturePanelContent::all()
        .iter()
        .map(|ct| {
            let is_active = *ct == content_type;
            let label = text(ct.label())
                .size(crate::gui::theme::type_scale::UI)
                .color(if is_active { text_color } else { sub0 });

            button(label)
                .on_press(Message::FeaturePanelSetContent(*ct))
                .padding([
                    crate::gui::theme::spacing::SM as u16,
                    crate::gui::theme::spacing::MD as u16,
                ])
                .style(move |theme, status| {
                    let mut style = theme::list_item_style(is_active)(theme, status);
                    if is_active {
                        // Active tab indicated by bottom border only
                        style.border = iced::Border {
                            color: blue,
                            width: 2.0,
                            radius: 0.0.into(),
                        };
                    }
                    style
                })
                .into()
        })
        .collect();

    let close_btn = button(icons::close().color(sub0))
        .on_press(Message::FeaturePanelToggle)
        .padding([
            crate::gui::theme::spacing::XS as u16,
            crate::gui::theme::spacing::SM as u16,
        ])
        .style(theme::close_button_style);

    container(
        row![
            row(tabs).spacing(crate::gui::theme::spacing::XS),
            Space::new().width(Length::Fill),
            close_btn,
        ]
        .spacing(crate::gui::theme::spacing::SM)
        .padding([
            crate::gui::theme::spacing::XS as u16,
            crate::gui::theme::spacing::SM as u16,
        ])
        .align_y(Alignment::Center),
    )
    .height(Length::Fixed(layout::PANEL_HEADER_HEIGHT))
    .style(theme::section_header_style)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Backlinks view
// ─────────────────────────────────────────────────────────────────────────────

fn view_backlinks(backlinks: &[BacklinkEntry], is_dark: bool) -> Element<'_, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface2 = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };

    if backlinks.is_empty() {
        return container(
            column![
                text("No backlinks")
                    .size(crate::gui::theme::type_scale::UI)
                    .color(sub0),
                text("Other notes linking to this one will appear here")
                    .size(crate::gui::theme::type_scale::CAPTION)
                    .color(surface2),
            ]
            .spacing(crate::gui::theme::spacing::XS),
        )
        .padding(crate::gui::theme::spacing::MD as u16)
        .into();
    }

    let items: Vec<Element<Message>> = backlinks
        .iter()
        .map(|bl| backlink_row(bl, is_dark))
        .collect();

    scrollable(
        column(items)
            .spacing(crate::gui::theme::spacing::XS)
            .padding(crate::gui::theme::spacing::SM as u16),
    )
    .height(Length::Fill)
    .style(theme::scrollable_style)
    .into()
}

fn backlink_row(bl: &BacklinkEntry, is_dark: bool) -> Element<'_, Message> {
    let lavender = if is_dark {
        colors::LAVENDER
    } else {
        colors::latte::LAVENDER
    };
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };

    let note_name = text(&bl.note)
        .size(crate::gui::theme::type_scale::UI)
        .color(lavender);

    let context = text(&bl.context)
        .size(crate::gui::theme::type_scale::CAPTION)
        .color(sub0);

    button(
        column![note_name, context]
            .spacing(crate::gui::theme::spacing::XS)
            .padding(crate::gui::theme::spacing::XS as u16),
    )
    .on_press(Message::BacklinkClick(bl.path.clone()))
    .width(Length::Fill)
    .style(theme::list_item_style(false))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Outline view
// ─────────────────────────────────────────────────────────────────────────────

fn view_outline(outline: &[OutlineEntry], is_dark: bool) -> Element<'_, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface2 = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };

    if outline.is_empty() {
        return container(
            column![
                text("No outline")
                    .size(crate::gui::theme::type_scale::UI)
                    .color(sub0),
                text("Headings from the current note will appear here")
                    .size(crate::gui::theme::type_scale::CAPTION)
                    .color(surface2),
            ]
            .spacing(crate::gui::theme::spacing::XS),
        )
        .padding(crate::gui::theme::spacing::MD as u16)
        .into();
    }

    let items: Vec<Element<Message>> = outline
        .iter()
        .enumerate()
        .map(|(i, entry)| outline_row(i, entry, is_dark))
        .collect();

    scrollable(
        column(items)
            .spacing(crate::gui::theme::spacing::XS)
            .padding(crate::gui::theme::spacing::SM as u16),
    )
    .height(Length::Fill)
    .style(theme::scrollable_style)
    .into()
}

fn outline_row(idx: usize, entry: &OutlineEntry, is_dark: bool) -> Element<'_, Message> {
    let teal = if is_dark {
        colors::TEAL
    } else {
        colors::latte::TEAL
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let subtext1 = if is_dark {
        colors::SUBTEXT1
    } else {
        colors::latte::SUBTEXT1
    };

    // Indent based on heading level (H1=0, H2=8px, H3=16px, etc.)
    let indent = crate::gui::theme::spacing::SM * (entry.level.saturating_sub(1)) as f32;
    // Use consistent size with color/weight differentiating levels
    let heading_color = match entry.level {
        1 => teal,
        2 => text_color,
        _ => subtext1,
    };

    let heading = text(&entry.text)
        .size(crate::gui::theme::type_scale::UI)
        .color(heading_color);

    // Apply indent via leading Space element
    let content = row![Space::new().width(Length::Fixed(indent)), heading,].spacing(0);

    button(container(content).padding([
        crate::gui::theme::spacing::XS as u16,
        crate::gui::theme::spacing::SM as u16,
    ]))
    .on_press(Message::OutlineClick(idx))
    .width(Length::Fill)
    .style(move |theme, status| theme::list_item_style(false)(theme, status))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// AI Chat view
// ─────────────────────────────────────────────────────────────────────────────

fn view_ai_chat<'a>(
    messages: &'a [(String, String)],
    input: &'a str,
    streaming: bool,
    ask_mode: AskMode,
    is_dark: bool,
) -> Element<'a, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let teal = if is_dark {
        colors::TEAL
    } else {
        colors::latte::TEAL
    };
    let blue = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };

    let history: Vec<Element<Message>> = messages
        .iter()
        .map(|(role, content)| chat_message(role, content, is_dark))
        .collect();

    let chat_area: Element<Message> = if messages.is_empty() {
        container(
            column![
                icons::empty_ai().color(sub0),
                text("AI Assistant").size(theme::type_scale::UI).color(teal),
                text("Ask questions about your notes.")
                    .size(theme::type_scale::CAPTION)
                    .color(sub0),
            ]
            .spacing(theme::spacing::SM)
            .align_x(Alignment::Center),
        )
        .padding(theme::spacing::LG as u16)
        .center_x(Length::Fill)
        .into()
    } else {
        scrollable(
            column(history)
                .spacing(theme::spacing::SM)
                .padding(theme::spacing::SM as u16),
        )
        .height(Length::Fill)
        .style(theme::scrollable_style)
        .into()
    };

    let mut input_field = text_input("Ask something...", input)
        .on_input(Message::AiChatInputChanged)
        .padding([theme::spacing::SM as u16, theme::spacing::MD as u16])
        .style(theme::text_input_style);

    if !streaming && !input.is_empty() {
        input_field = input_field.on_submit(Message::AiChatSend(input.to_string()));
    }

    let input_row = if streaming {
        row![input_field, icons::loading().color(blue),].spacing(theme::spacing::SM)
    } else {
        row![input_field]
    };

    let pills = mode_pill_row(ask_mode, is_dark);

    column![
        container(chat_area).height(Length::Fill),
        container(pills).padding([0, theme::spacing::SM as u16]),
        container(input_row).padding(theme::spacing::SM as u16),
    ]
    .into()
}

fn mode_pill_row(active: AskMode, is_dark: bool) -> Element<'static, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let accent = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };

    let pills: Vec<Element<Message>> = AskMode::all()
        .iter()
        .map(|mode| {
            let is_active = *mode == active;
            let label = text(mode.label())
                .size(theme::type_scale::CAPTION)
                .color(if is_active { accent } else { sub0 });
            button(label)
                .on_press(Message::AskModeChanged(*mode))
                .padding([theme::spacing::XS as u16, theme::spacing::SM as u16])
                .style(move |theme, status| {
                    let mut style = theme::list_item_style(is_active)(theme, status);
                    if is_active {
                        style.border = iced::Border {
                            color: accent,
                            width: 1.0,
                            radius: theme::layout::PANEL_HEADER_HEIGHT.into(),
                        };
                    }
                    style
                })
                .into()
        })
        .collect();

    row(pills)
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .into()
}

fn chat_message<'a>(role: &'a str, content: &'a str, is_dark: bool) -> Element<'a, Message> {
    let blue = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };
    let teal = if is_dark {
        colors::TEAL
    } else {
        colors::latte::TEAL
    };
    let yellow = if is_dark {
        colors::YELLOW
    } else {
        colors::latte::YELLOW
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };
    let surface1 = if is_dark {
        colors::SURFACE1
    } else {
        colors::latte::SURFACE1
    };
    let mantle = if is_dark {
        colors::MANTLE
    } else {
        colors::latte::MANTLE
    };

    let (label, label_color, bg) = match role {
        "user" => ("You", blue, surface0),
        "assistant" => ("AI", teal, mantle),
        _ => ("System", yellow, surface0),
    };

    let parsed = parse_ai_message(content);
    let body = if parsed.body.trim().is_empty() && role == "assistant" {
        "Thinking...".to_string()
    } else {
        parsed.body
    };

    let sources_section: Element<'a, Message> = if parsed.sources.is_empty() {
        Space::new().height(Length::Shrink).into()
    } else {
        let source_items: Vec<Element<Message>> = parsed
            .sources
            .into_iter()
            .map(|citation| {
                button(
                    column![
                        text(citation.title)
                            .size(theme::type_scale::CAPTION)
                            .color(label_color),
                        text(citation.location)
                            .size(theme::type_scale::CAPTION)
                            .color(sub0),
                    ]
                    .spacing(2),
                )
                .on_press(Message::AiChatCitationClick(citation.path))
                .padding([theme::spacing::XS as u16, theme::spacing::SM as u16])
                .width(Length::Fill)
                .style(theme::list_item_style(false))
                .into()
            })
            .collect();

        column![
            text("Sources").size(theme::type_scale::CAPTION).color(sub0),
            column(source_items).spacing(theme::spacing::XS),
        ]
        .spacing(theme::spacing::XS)
        .into()
    };

    container(
        column![
            text(label)
                .size(theme::type_scale::CAPTION)
                .color(label_color),
            text(body).size(theme::type_scale::UI).color(text_color),
            sources_section,
        ]
        .spacing(theme::spacing::XS),
    )
    .padding(theme::spacing::SM as u16)
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            color: surface1,
            width: 0.0,
            radius: theme::spacing::XS.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

#[derive(Debug, Clone)]
struct ParsedAiMessage {
    body: String,
    sources: Vec<ParsedCitation>,
}

#[derive(Debug, Clone)]
struct ParsedCitation {
    title: String,
    location: String,
    path: PathBuf,
}

fn parse_ai_message(content: &str) -> ParsedAiMessage {
    let Some((body, sources_block)) = content.split_once("\n--- Sources ---\n") else {
        return ParsedAiMessage {
            body: content.trim().to_string(),
            sources: Vec::new(),
        };
    };

    ParsedAiMessage {
        body: body.trim().to_string(),
        sources: sources_block
            .lines()
            .filter_map(parse_ai_citation)
            .collect(),
    }
}

fn parse_ai_citation(line: &str) -> Option<ParsedCitation> {
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

    Some(ParsedCitation {
        title: format!("[{index}] {file_label}"),
        location,
        path,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Context tab — Phase 12 scaffold (Sources / Related / Extracted / Model)
// ─────────────────────────────────────────────────────────────────────────────

fn view_context<'a>(
    snapshot: Option<&'a ContextSnapshot>,
    extracting: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface2 = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let accent = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };

    let section_label = |label: &'static str| {
        text(label)
            .size(crate::gui::theme::type_scale::UI)
            .color(text_color)
    };
    let stub_label = |label: &'static str| {
        text(label)
            .size(crate::gui::theme::type_scale::CAPTION)
            .color(surface2)
    };

    let mut body = column![
        section_label("Sources"),
        stub_label(
            "Sources for the latest Ask MALD answer surface here once retrieval is wired in."
        ),
        Space::new().height(Length::Fixed(crate::gui::theme::spacing::MD)),
        section_label("Related Notes"),
        stub_label("Notes linked from the active document or answer."),
        Space::new().height(Length::Fixed(crate::gui::theme::spacing::MD)),
        section_label("Extracted"),
    ]
    .spacing(crate::gui::theme::spacing::XS);

    if extracting {
        body = body.push(
            text("Extracting concepts, tasks, and follow-ups…")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        );
    }

    let snap = snapshot.cloned();
    if let Some(snap) = snap.as_ref() {
        let extracted_section = |label: &'static str, items: &[String]| -> Element<'a, Message> {
            let label_el = text(label)
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0);

            if items.is_empty() {
                column![label_el, stub_label("—")]
                    .spacing(crate::gui::theme::spacing::XS)
                    .into()
            } else {
                let mut col = column![label_el].spacing(crate::gui::theme::spacing::XS);
                for item in items {
                    col = col.push(
                        text(format!("• {item}"))
                            .size(crate::gui::theme::type_scale::CAPTION)
                            .color(text_color),
                    );
                }
                col.into()
            }
        };

        body = body
            .push(extracted_section("Key Concepts", &snap.concepts))
            .push(Space::new().height(Length::Fixed(crate::gui::theme::spacing::SM)))
            .push(extracted_section("Tasks", &snap.tasks))
            .push(Space::new().height(Length::Fixed(crate::gui::theme::spacing::SM)))
            .push(extracted_section("Questions", &snap.questions));
    } else if !extracting {
        body = body.push(stub_label(
            "Key concepts, tasks, and follow-up questions auto-extract once an answer finishes streaming.",
        ));
    }

    body = body
        .push(Space::new().height(Length::Fixed(crate::gui::theme::spacing::MD)))
        .push(section_label("Model & Retrieval"));

    let _ = accent; // reserved for future retrieval-confidence highlight

    body = body
        .push(
            text("Model         —")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        )
        .push(
            text("Embedding     —")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        )
        .push(
            text("Index         —")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        )
        .push(
            text("Retrieved     —")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        )
        .push(
            text("Confidence    —")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(sub0),
        );

    container(scrollable(body).style(theme::scrollable_style))
        .padding(crate::gui::theme::spacing::MD as u16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Timeline tab — Phase 12 scaffold (placeholder until journal hook lands)
// ─────────────────────────────────────────────────────────────────────────────

fn view_timeline(is_dark: bool) -> Element<'static, Message> {
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface2 = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };

    container(
        column![
            text("Timeline")
                .size(crate::gui::theme::type_scale::UI)
                .color(sub0),
            text("Recent edits, captures, and answers will stream here.")
                .size(crate::gui::theme::type_scale::CAPTION)
                .color(surface2),
        ]
        .spacing(crate::gui::theme::spacing::XS),
    )
    .padding(crate::gui::theme::spacing::MD as u16)
    .into()
}
