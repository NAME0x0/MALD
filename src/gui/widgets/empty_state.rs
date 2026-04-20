//! Reusable empty state component with icon, title, and hint.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use iced_anim::widget::button;

use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};

/// Configuration for an empty state view.
pub struct EmptyState<'a> {
    pub icon: Element<'a, Message>,
    pub title: &'a str,
    pub hint: &'a str,
    pub actions: Vec<Element<'a, Message>>,
}

impl<'a> EmptyState<'a> {
    pub fn new(icon: impl Into<Element<'a, Message>>, title: &'a str, hint: &'a str) -> Self {
        Self {
            icon: icon.into(),
            title,
            hint,
            actions: Vec::new(),
        }
    }

    pub fn with_actions(mut self, actions: Vec<Element<'a, Message>>) -> Self {
        self.actions = actions;
        self
    }

    pub fn view(self, is_dark: bool) -> Element<'a, Message> {
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
        let panel = if is_dark {
            colors::MANTLE
        } else {
            colors::latte::MANTLE
        };
        let text_color = if is_dark {
            colors::TEXT
        } else {
            colors::latte::TEXT
        };
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        let accent = if is_dark {
            colors::TEAL
        } else {
            colors::latte::TEAL
        };

        let has_actions = !self.actions.is_empty();
        let actions: Element<'a, Message> = if has_actions {
            row(self.actions)
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .into()
        } else {
            Space::new().height(Length::Shrink).into()
        };
        let action_spacer: Element<'a, Message> = if has_actions {
            Space::new().height(spacing::LG).into()
        } else {
            Space::new().height(Length::Shrink).into()
        };

        container(
            container(
                column![
                    container(self.icon)
                        .width(Length::Fixed(72.0))
                        .height(Length::Fixed(72.0))
                        .center_x(Length::Fixed(72.0))
                        .center_y(Length::Fixed(72.0))
                        .style(move |_theme| container::Style {
                            background: Some(iced::Background::Color(surface0)),
                            border: iced::Border {
                                color: iced::Color {
                                    a: 0.24,
                                    ..surface1
                                },
                                width: 1.0,
                                radius: 36.0.into(),
                            },
                            ..Default::default()
                        }),
                    Space::new().height(spacing::LG),
                    text("Nothing here yet")
                        .size(type_scale::CAPTION)
                        .color(accent),
                    Space::new().height(spacing::XS),
                    text(self.title).size(type_scale::H2).color(text_color),
                    Space::new().height(spacing::SM),
                    container(text(self.hint).size(type_scale::BODY).color(subtext0),)
                        .max_width(420.0),
                    action_spacer,
                    actions,
                ]
                .align_x(Alignment::Center)
                .spacing(0)
                .padding(spacing::XXL as u16),
            )
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(panel)),
                border: iced::Border {
                    color: iced::Color {
                        a: 0.18,
                        ..surface1
                    },
                    width: 1.0,
                    radius: 20.0.into(),
                },
                ..Default::default()
            })
            .max_width(520.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(spacing::XXL as u16)
        .into()
    }
}

/// Shorthand for common empty states with actionable guidance
pub mod presets {
    use super::*;
    use crate::gui::icons;

    fn primary_action<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
        button(text(label).size(type_scale::UI))
            .on_press(message)
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::primary_button_style)
            .into()
    }

    fn secondary_action<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
        button(text(label).size(type_scale::UI))
            .on_press(message)
            .padding([spacing::SM as u16, spacing::LG as u16])
            .style(theme::secondary_button_style)
            .into()
    }

    pub fn no_files<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_folder().color(subtext0),
            "No files yet",
            "Create a note, or open the demo space if you want to explore MALD first.",
        )
        .with_actions(vec![
            primary_action("New note", Message::NewNotePrompt),
            secondary_action("Open demo space", Message::DemoSpaceOpen),
        ])
        .view(is_dark)
    }

    pub fn no_search_results<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_search().color(subtext0),
            "No results found",
            "Try different keywords. Search supports titles and full-text content.",
        )
        .with_actions(vec![primary_action("New note", Message::NewNotePrompt)])
        .view(is_dark)
    }

    pub fn search_prompt<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_search().color(subtext0),
            "Search your notes",
            "Type to search titles and content. Use Ctrl+Shift+F for global search.",
        )
        .with_actions(vec![
            primary_action("New note", Message::NewNotePrompt),
            secondary_action("Open demo space", Message::DemoSpaceOpen),
        ])
        .view(is_dark)
    }

    pub fn no_graph<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_graph().color(subtext0),
            "No connections yet",
            "Create [[wikilinks]] between notes to build your knowledge graph.",
        )
        .with_actions(vec![primary_action("New note", Message::NewNotePrompt)])
        .view(is_dark)
    }

    pub fn no_tasks<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_tasks().color(subtext0),
            "No tasks found",
            "Add `- [ ]` task items to your notes and they'll appear here.",
        )
        .with_actions(vec![primary_action("New note", Message::NewNotePrompt)])
        .view(is_dark)
    }

    pub fn ai_prompt<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_ai().color(subtext0),
            "Ask the AI",
            "Ask questions about your notes. Requires Ollama running locally.",
        )
        .view(is_dark)
    }

    pub fn no_editor<'a>(is_dark: bool) -> Element<'a, Message> {
        let subtext0 = if is_dark {
            colors::SUBTEXT0
        } else {
            colors::latte::SUBTEXT0
        };
        EmptyState::new(
            icons::empty_editor().color(subtext0),
            "No file open",
            "Select a file from the sidebar, or create a new note and start from a clean page.",
        )
        .with_actions(vec![
            primary_action("New note", Message::NewNotePrompt),
            secondary_action("Open demo space", Message::DemoSpaceOpen),
        ])
        .view(is_dark)
    }
}
