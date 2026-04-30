//! AI chat panel widget.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

use crate::gui::theme::{self, colors, spacing, type_scale};
use crate::gui::{icons, message::Message};

/// Chat message with role and content.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// Render the AI chat panel widget.
pub fn view<'a>(
    messages: &'a [ChatMessage],
    input: &'a str,
    streaming: bool,
    visible: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    if !visible {
        return column![].width(Length::Shrink).into();
    }

    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let subtext1 = if is_dark {
        colors::SUBTEXT1
    } else {
        colors::latte::SUBTEXT1
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
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

    let header = row![
        text("AI CHAT").size(type_scale::UI).color(subtext0),
        Space::new().width(Length::Fill),
        button(text("×").size(type_scale::CAPTION).color(subtext1))
            .on_press(Message::AiChatToggle)
            .padding([2, 6])
            .style(theme::close_button_style),
    ];

    let msg_items: Vec<Element<Message>> = messages
        .iter()
        .map(|msg| {
            let (prefix, color) = match msg.role {
                ChatRole::User => ("You", blue),
                ChatRole::Assistant => ("AI", teal),
                ChatRole::System => ("System", yellow),
            };
            column![
                text(prefix).size(type_scale::CAPTION).color(color),
                text(&msg.content).size(type_scale::UI).color(text_color),
            ]
            .spacing(spacing::XS)
            .padding(spacing::XS as u16)
            .into()
        })
        .collect();

    let streaming_indicator: Element<Message> = if streaming {
        container(
            row![
                icons::loading().color(blue),
                text("Thinking...")
                    .size(type_scale::CAPTION)
                    .color(subtext0),
            ]
            .spacing(spacing::XS),
        )
        .padding(spacing::XS as u16)
        .into()
    } else {
        Space::new().height(0).into()
    };

    let input_field = text_input("Ask something...", input)
        .on_input(Message::AiChatInputChanged)
        .on_submit(Message::AiChatSend(input.to_string()))
        .padding(spacing::SM as u16)
        .width(Length::Fill)
        .style(theme::text_input_style);

    column![
        header,
        scrollable(column(msg_items).spacing(spacing::XS))
            .height(Length::Fill)
            .style(theme::scrollable_style),
        streaming_indicator,
        input_field,
    ]
    .spacing(spacing::XS)
    .padding(spacing::SM as u16)
    .width(Length::Fixed(350.0))
    .height(Length::Fill)
    .into()
}
