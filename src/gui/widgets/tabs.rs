use iced::widget::{button, row, text};
use iced::Element;

use crate::gui::app::EditorTab;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, type_scale, spacing};

/// Render the editor tab bar.
pub fn view<'a>(tabs: &'a [EditorTab], active: usize, is_dark: bool) -> Element<'a, Message> {
    let text_color = if is_dark { colors::TEXT } else { colors::latte::TEXT };
    let subtext0 = if is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };
    let subtext1 = if is_dark { colors::SUBTEXT1 } else { colors::latte::SUBTEXT1 };

    let tab_buttons: Vec<Element<Message>> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let modified = if tab.modified { " \u{2022}" } else { "" };
            let label = format!("{}{modified}", tab.title);
            let is_active = i == active;
            let mut btn = button(text(label.clone())
                .size(type_scale::UI)
                .color(if is_active { text_color } else { subtext0 }));
            if !is_active {
                btn = btn.on_press(Message::EditorSwitchTab(i));
            }
            row![
                btn.style(theme::tab_button_style(is_active, tab.modified)),
                button(text("\u{00d7}").size(type_scale::CAPTION).color(subtext1))
                    .on_press(Message::EditorClose(i))
                    .style(theme::close_button_style),
            ]
            .spacing(spacing::XS)
            .into()
        })
        .collect();

    row(tab_buttons).spacing(spacing::XS).padding(spacing::XS as u16).into()
}
