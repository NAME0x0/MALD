use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::gui::message::{FileEntry, Message};
use crate::gui::theme::{self, colors, spacing, type_scale};

/// Render the file tree sidebar widget.
pub fn view<'a>(entries: &'a [FileEntry], visible: bool, is_dark: bool) -> Element<'a, Message> {
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

    let header = row![
        text("EXPLORER").size(type_scale::UI).color(subtext0),
        Space::new().width(Length::Fill),
        button(text("↻").size(type_scale::CAPTION).color(subtext1))
            .on_press(Message::FileTreeRefresh)
            .style(theme::ghost_button_style(false)),
    ]
    .padding(spacing::XS as u16);

    let items: Vec<Element<Message>> = entries
        .iter()
        .map(|entry| {
            let indent = " ".repeat(entry.depth * 2);
            let icon = if entry.is_dir {
                if entry.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  " // file icon placeholder
            };
            let label = format!("{indent}{icon}{}", entry.name);

            if entry.is_dir {
                let path = entry.path.clone();
                let msg = if entry.expanded {
                    Message::FileTreeCollapse(path)
                } else {
                    Message::FileTreeExpand(path)
                };
                button(text(label).size(type_scale::UI).color(text_color))
                    .on_press(msg)
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
            } else {
                button(text(label).size(type_scale::UI).color(text_color))
                    .on_press(Message::FileTreeSelect(entry.path.clone()))
                    .width(Length::Fill)
                    .style(theme::list_item_style(false))
                    .into()
            }
        })
        .collect();

    let list = scrollable(column(items).spacing(spacing::XS))
        .height(Length::Fill)
        .style(theme::scrollable_style);

    column![header, list]
        .spacing(spacing::XS)
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .into()
}
