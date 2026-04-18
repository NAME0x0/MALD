use iced::widget::{column, row, scrollable, text, text_input, button, Space};
use iced::{Element, Length};

use crate::gui::message::Message;
use crate::gui::theme::{self, colors, type_scale, spacing};

/// Render the embedded terminal panel.
pub fn view<'a>(
    lines: &'a [String],
    input: &'a str,
    visible: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    if !visible {
        return column![].height(Length::Shrink).into();
    }

    let text_color = if is_dark { colors::TEXT } else { colors::latte::TEXT };
    let subtext0 = if is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };
    let subtext1 = if is_dark { colors::SUBTEXT1 } else { colors::latte::SUBTEXT1 };

    let header = row![
        text("TERMINAL").size(type_scale::UI).color(subtext0),
        Space::new().width(Length::Fill),
        button(text("×").size(type_scale::CAPTION).color(subtext1))
            .on_press(Message::TerminalToggle)
            .style(theme::close_button_style),
    ];

    let output: Vec<Element<Message>> = lines
        .iter()
        .map(|line| text(line).size(type_scale::UI).font(iced::Font::MONOSPACE).color(text_color).into())
        .collect();

    let input_field = text_input("> ", input)
        .on_input(Message::TerminalInput)
        .on_submit(Message::TerminalSubmit)
        .font(iced::Font::MONOSPACE)
        .style(theme::terminal_input_style)
        .width(Length::Fill);

    column![
        header,
        scrollable(column(output).spacing(spacing::XS)).height(Length::Fill).style(theme::scrollable_style),
        input_field,
    ]
    .spacing(spacing::XS)
    .padding(spacing::XS as u16)
    .height(Length::Fixed(200.0))
    .into()
}
