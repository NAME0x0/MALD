use iced::widget::{column, container, text, text_editor};
use iced::{Element, Length};
use iced_anim::widget::button;

use crate::gui::app::EditorTab;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};

/// Render the editor content area for the active tab.
pub fn view<'a>(
    tab: Option<&'a EditorTab>,
    content: &'a text_editor::Content,
    autocomplete_visible: bool,
    autocomplete_results: &'a [String],
    is_dark: bool,
) -> Element<'a, Message> {
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

    if let Some(_tab) = tab {
        let editor = text_editor(content)
            .on_action(Message::EditorContentChanged)
            .style(theme::text_editor_style)
            .height(Length::Fill);

        let mut col = column![editor].spacing(0);

        // Autocomplete popup
        if autocomplete_visible && !autocomplete_results.is_empty() {
            let items: Vec<Element<Message>> = autocomplete_results
                .iter()
                .take(10)
                .map(|name| {
                    button(text(name).size(type_scale::UI).color(text_color))
                        .on_press(Message::AutocompleteSelect(name.clone()))
                        .style(theme::list_item_style(false))
                        .padding(spacing::XS as u16)
                        .width(Length::Fill)
                        .into()
                })
                .collect();
            col = col.push(
                container(column(items).spacing(spacing::XS))
                    .width(Length::Fixed(300.0))
                    .style(theme::card_style),
            );
        }

        col.width(Length::Fill).height(Length::Fill).into()
    } else {
        container(
            text("Open a file to start editing")
                .size(type_scale::BODY)
                .color(subtext0),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::editor_style)
        .into()
    }
}
