//! Top search bar component - integrated search input at top of editor area.

use iced::widget::{container, row, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::icons;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, icon_size, layout, spacing};

/// Render the top search bar.
pub fn view<'a>(
    query: &'a str,
    focused: bool,
    _placeholder: &'a str, // Ignored - using integrated placeholder with shortcut
    is_dark: bool,
) -> Element<'a, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let search_icon = icons::search()
        .size(icon_size::SECONDARY as f32)
        .color(subtext0);

    // Integrated placeholder with shortcut hint
    let placeholder_text = "Search notes... (Ctrl+Shift+F)";

    let input = text_input(placeholder_text, query)
        .on_input(Message::TopSearchQueryChanged)
        .on_submit(Message::TopSearchSubmit)
        .padding([spacing::SM as u16, spacing::MD as u16])
        .width(Length::Fixed(layout::SEARCH_INPUT_WIDTH))
        .style(theme::search_input_style);

    let search_bar = row![search_icon, input,]
        .spacing(spacing::SM)
        .align_y(Alignment::Center);

    // Single container - no double-wrapping
    container(search_bar)
        .width(Length::Fill)
        .height(Length::Fixed(layout::SEARCH_BAR_HEIGHT))
        .padding([spacing::SM as u16, spacing::LG as u16])
        .center_x(Length::Fill)
        .style(theme::top_search_container_style(focused))
        .into()
}

/// Compact version of search bar (no centering, for narrow spaces)
pub fn view_compact<'a>(query: &'a str, placeholder: &'a str) -> Element<'a, Message> {
    let input = text_input(placeholder, query)
        .on_input(Message::TopSearchQueryChanged)
        .on_submit(Message::TopSearchSubmit)
        .padding([spacing::SM as u16, spacing::MD as u16])
        .width(Length::Fill)
        .style(theme::search_input_style);

    container(input)
        .padding(spacing::SM as u16)
        .width(Length::Fill)
        .into()
}
