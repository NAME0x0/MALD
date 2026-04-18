use iced::widget::{column, container, scrollable, text, text_input, button};
use iced::{Element, Length};

use crate::gui::message::{Message, SearchResult};
use crate::gui::theme::{self, colors, type_scale, spacing};

/// Render the search overlay.
pub fn view<'a>(
    query: &'a str,
    results: &'a [SearchResult],
    visible: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    if !visible {
        return column![].into();
    }

    let text_color = if is_dark { colors::TEXT } else { colors::latte::TEXT };
    let subtext0 = if is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };
    let subtext1 = if is_dark { colors::SUBTEXT1 } else { colors::latte::SUBTEXT1 };

    let input = text_input("Search notes...", query)
        .on_input(Message::SearchQueryChanged)
        .padding(spacing::SM as u16)
        .style(theme::search_input_style)
        .width(Length::Fixed(500.0));

    let items: Vec<Element<Message>> = results
        .iter()
        .enumerate()
        .take(20)
        .map(|(i, result)| {
            button(
                column![
                    text(&result.title).size(type_scale::BODY).color(text_color),
                    text(&result.snippet).size(type_scale::CAPTION).color(subtext0),
                ]
                .spacing(spacing::XS),
            )
            .on_press(Message::SearchResultSelect(i))
            .width(Length::Fill)
            .style(theme::list_item_style(false))
            .into()
        })
        .collect();

    container(
        column![
            input,
            scrollable(column(items).spacing(spacing::XS))
                .height(Length::Fixed(400.0))
                .style(theme::scrollable_style),
            button(text("Close (Esc)").size(type_scale::UI).color(subtext1))
                .on_press(Message::SearchClose)
                .style(theme::ghost_button_style(false)),
        ]
        .spacing(spacing::SM)
        .padding(spacing::MD as u16),
    )
    .width(Length::Fixed(500.0))
    .center_x(Length::Fill)
    .style(theme::card_style)
    .into()
}
