//! Tab bar component - styled editor tabs with active/inactive/modified states.

use iced::widget::{container, row, scrollable, text, Row};
use iced::{Alignment, Element, Length};
use iced_anim::widget::button;

use crate::gui::icons;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, icon_size, layout, spacing, type_scale};

/// Tab data for rendering
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub title: String,
    pub modified: bool,
    pub index: usize,
}

/// Render the tab bar.
pub fn view(tabs: Vec<TabInfo>, active_index: usize, is_dark: bool) -> Element<'static, Message> {
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };
    let dim = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };

    if tabs.is_empty() {
        return container(
            text("Open a file to start")
                .size(type_scale::CAPTION)
                .color(dim),
        )
        .height(Length::Fixed(layout::TAB_BAR_HEIGHT))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fixed(layout::TAB_BAR_HEIGHT))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(surface0)),
            ..Default::default()
        })
        .into();
    }

    let tab_buttons: Vec<Element<Message>> = tabs
        .into_iter()
        .map(|tab| {
            let is_active = tab.index == active_index;
            tab_button(tab, is_active, is_dark)
        })
        .collect();

    let tabs_row = Row::with_children(tab_buttons)
        .spacing(0)
        .align_y(Alignment::End);

    let scrollable_tabs = scrollable(tabs_row)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(3).scroller_width(3),
        ))
        .style(theme::scrollable_style);

    container(scrollable_tabs)
        .height(Length::Fixed(layout::TAB_BAR_HEIGHT))
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(surface0)),
            border: iced::Border {
                color: surface0,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn tab_button(tab: TabInfo, active: bool, is_dark: bool) -> Element<'static, Message> {
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let blue = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };
    let base_bg = if is_dark {
        colors::BASE
    } else {
        colors::latte::BASE
    };
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };

    let yellow = if is_dark {
        colors::YELLOW
    } else {
        colors::latte::YELLOW
    };

    // Modified indicator - yellow dot for unsaved changes (Gemini: replaces close X)
    let modified_indicator = if tab.modified {
        text("\u{25CF}").size(type_scale::CAPTION).color(yellow)
    } else {
        text(" ").size(type_scale::CAPTION)
    };

    let title_text =
        text(tab.title)
            .size(type_scale::UI)
            .color(if active { text_color } else { sub0 });

    let close_btn = button(icons::close().size(icon_size::SECONDARY as f32).color(sub0))
        .on_press(Message::EditorClose(tab.index))
        .width(Length::Fixed(layout::CLOSE_BUTTON_SIZE))
        .height(Length::Fixed(layout::CLOSE_BUTTON_SIZE))
        .padding(0)
        .style(theme::close_button_style);

    let tab_content = row![modified_indicator, title_text, close_btn,]
        .spacing(spacing::XS)
        .align_y(Alignment::Center)
        .padding([0, spacing::SM as u16]);

    let tab_index = tab.index;
    let mut tab_btn = button(tab_content)
        .height(Length::Fixed(layout::TAB_BAR_HEIGHT - 1.0))
        .padding([spacing::SM as u16, spacing::XS as u16])
        .style(theme::tab_button_style(active, tab.modified));

    if !active {
        tab_btn = tab_btn.on_press(Message::EditorSwitchTab(tab_index));
    }

    // Active tab has a bottom border accent only (VSCode pattern)
    if active {
        iced::widget::column![
            container(tab_btn).style(move |_theme| container::Style {
                background: Some(iced::Background::Color(base_bg)),
                ..Default::default()
            }),
            container(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(2.0))
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(blue)),
                    ..Default::default()
                }),
        ]
        .spacing(0)
        .into()
    } else {
        container(tab_btn)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(surface0)),
                ..Default::default()
            })
            .into()
    }
}

/// Render a minimal tab bar with just the count
pub fn view_minimal(
    tab_count: usize,
    active_index: usize,
    is_dark: bool,
) -> Element<'static, Message> {
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let sub0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };

    let label = if tab_count == 0 {
        text("No open files").size(type_scale::UI).color(sub0)
    } else {
        text(format!("Tab {} of {}", active_index + 1, tab_count))
            .size(type_scale::UI)
            .color(text_color)
    };

    container(label)
        .height(Length::Fixed(layout::TAB_BAR_HEIGHT))
        .padding([spacing::SM as u16, spacing::LG as u16])
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(surface0)),
            ..Default::default()
        })
        .into()
}
