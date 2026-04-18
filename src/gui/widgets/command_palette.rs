use iced::widget::{column, container, scrollable, text, text_input, button};
use iced::{Element, Length};

use crate::gui::message::{Message, PaletteCommand};
use crate::gui::theme::{self, colors, type_scale, spacing, layout};

/// Render the command palette overlay.
pub fn view<'a>(
    query: &'a str,
    commands: &'a [PaletteCommand],
    filtered: &'a [usize],
    visible: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    if !visible {
        return column![].into();
    }

    let text_color = if is_dark { colors::TEXT } else { colors::latte::TEXT };
    let subtext0 = if is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };

    let input = text_input("Type a command...", query)
        .on_input(Message::CommandPaletteQueryChanged)
        .padding(spacing::SM as u16)
        .style(theme::search_input_style)
        .width(Length::Fixed(layout::MODAL_WIDTH));

    let items: Vec<Element<Message>> = filtered
        .iter()
        .take(15)
        .enumerate()
        .map(|(i, &cmd_idx)| {
            let cmd = &commands[cmd_idx];
            button(
                column![
                    text(&cmd.label).size(type_scale::BODY).color(text_color),
                    text(&cmd.description).size(type_scale::CAPTION).color(subtext0),
                ]
                .spacing(spacing::XS),
            )
            .on_press(Message::CommandPaletteSelect(i))
            .width(Length::Fill)
            .style(theme::list_item_style(false))
            .into()
        })
        .collect();

    container(
        column![
            input,
            scrollable(column(items).spacing(spacing::XS)).height(Length::Fixed(400.0)).style(theme::scrollable_style),
        ]
        .spacing(spacing::SM)
        .padding(spacing::MD as u16),
    )
    .width(Length::Fixed(layout::MODAL_WIDTH))
    .center_x(Length::Fill)
    .style(theme::card_style)
    .into()
}
