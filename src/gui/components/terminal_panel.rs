//! Terminal panel component - bottom terminal with styled header.

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};
use iced_anim::widget::button;

use crate::gui::icons;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};

const TERMINAL_HEADER_HEIGHT: f32 = 28.0;
const DEFAULT_TERMINAL_HEIGHT: f32 = 200.0;

/// Render the terminal panel.
pub fn view<'a>(
    lines: &'a [String],
    input: &'a str,
    height: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let header = terminal_header(is_dark);

    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };

    let output_lines: Vec<Element<Message>> = lines
        .iter()
        .map(|line| {
            text(line)
                .size(type_scale::UI)
                .font(iced::Font::MONOSPACE)
                .color(text_color)
                .into()
        })
        .collect();

    let output_area = scrollable(
        column(output_lines)
            .spacing(spacing::XS)
            .padding([spacing::XS as u16, spacing::SM as u16]),
    )
    .height(Length::Fill)
    .style(theme::scrollable_style);

    let input_field = text_input("> ", input)
        .on_input(Message::TerminalInput)
        .on_submit(Message::TerminalSubmit)
        .padding([spacing::XS as u16, spacing::SM as u16])
        .font(iced::Font::MONOSPACE)
        .style(theme::terminal_input_style);

    let content = column![header, output_area, input_field].spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(theme::terminal_style)
        .into()
}

fn terminal_header<'a>(is_dark: bool) -> Element<'a, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let crust = if is_dark {
        colors::CRUST
    } else {
        colors::latte::CRUST
    };
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };

    let title = text("TERMINAL").size(type_scale::UI).color(subtext0);

    let clear_btn = button(text("Clear").size(type_scale::CAPTION).color(subtext0))
        .on_press(Message::TerminalClear)
        .padding([2, 6])
        .style(theme::secondary_button_style);

    let restart_btn = button(text("Restart").size(type_scale::CAPTION).color(subtext0))
        .on_press(Message::TerminalRestart)
        .padding([2, 6])
        .style(theme::secondary_button_style);

    let interrupt_btn = button(text("Ctrl+C").size(type_scale::CAPTION).color(subtext0))
        .on_press(Message::TerminalInterrupt)
        .padding([2, 6])
        .style(theme::secondary_button_style);

    let close_btn = button(icons::close().color(subtext0))
        .on_press(Message::TerminalToggle)
        .padding([2, 6])
        .style(theme::close_button_style);

    container(
        row![
            title,
            Space::new().width(Length::Fill),
            clear_btn,
            restart_btn,
            interrupt_btn,
            close_btn,
        ]
        .spacing(8)
        .padding([4, 8])
        .align_y(Alignment::Center),
    )
    .height(Length::Fixed(TERMINAL_HEADER_HEIGHT))
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(crust)),
        border: iced::Border {
            color: surface0,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Render a minimal terminal toggle button (for when terminal is hidden)
pub fn view_toggle_button<'a>(is_dark: bool) -> Element<'a, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };

    button(
        row![
            icons::terminal().size(type_scale::UI),
            text("Terminal").size(type_scale::UI).color(subtext0),
        ]
        .spacing(spacing::SM),
    )
    .on_press(Message::TerminalToggle)
    .padding([spacing::XS as u16, spacing::SM as u16])
    .style(theme::secondary_button_style)
    .into()
}
