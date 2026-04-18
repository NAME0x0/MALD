//! Status bar widget with theme-aware Catppuccin styling.

use iced::widget::{container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::gui::icons;
use crate::gui::message::{DaemonStatus, Message};
use crate::gui::theme::{self, colors, icon_size, layout, spacing, type_scale};

/// Render the bottom status bar.
pub fn view<'a>(
    daemon_status: DaemonStatus,
    kb_name: &'a str,
    cursor_line: usize,
    cursor_col: usize,
    _word_count: usize, // Removed from default view, available via tooltip
    note_count: usize,
    is_dark: bool,
) -> Element<'a, Message> {
    // Theme-aware colors
    let green = if is_dark {
        colors::GREEN
    } else {
        colors::latte::GREEN
    };
    let red = if is_dark {
        colors::RED
    } else {
        colors::latte::RED
    };
    let surface2 = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };
    let surface1 = if is_dark {
        colors::SURFACE1
    } else {
        colors::latte::SURFACE1
    };
    let subtext1 = if is_dark {
        colors::SUBTEXT1
    } else {
        colors::latte::SUBTEXT1
    };
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };

    // Left group: Daemon + KB
    let (daemon_icon, daemon_color) = match daemon_status {
        DaemonStatus::Running => ("●", green),
        DaemonStatus::Stopped => ("●", red),
        DaemonStatus::Unknown => ("○", surface2),
    };

    let daemon_indicator = row![
        text(daemon_icon)
            .size(type_scale::CAPTION)
            .color(daemon_color),
        text(match daemon_status {
            DaemonStatus::Running => "Daemon",
            DaemonStatus::Stopped => "Daemon",
            DaemonStatus::Unknown => "Daemon",
        })
        .size(type_scale::UI)
        .color(subtext1),
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center);

    let kb_info = row![
        icons::folder_closed()
            .size(icon_size::INLINE as f32)
            .color(subtext0),
        text(kb_name).size(type_scale::UI).color(subtext1),
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center);

    let left_group = row![
        daemon_indicator,
        text("│").size(type_scale::UI).color(surface1),
        kb_info,
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center);

    // Right group: Notes count + cursor position
    let notes = text(format!("{note_count} notes"))
        .size(type_scale::UI)
        .color(subtext0);

    let cursor_pos = text(format!("Ln {cursor_line}, Col {cursor_col}"))
        .size(type_scale::UI)
        .color(subtext1);

    let right_group = row![
        notes,
        text("│").size(type_scale::UI).color(surface1),
        cursor_pos,
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center);

    container(
        row![left_group, Space::new().width(Length::Fill), right_group,]
            .spacing(0)
            .padding([0, spacing::MD as u16])
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(layout::STATUS_BAR_HEIGHT))
    .style(theme::status_bar_style)
    .into()
}

/// Minimal status bar for error/loading states
pub fn view_minimal<'a>(message: &'a str, is_dark: bool) -> Element<'a, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };

    container(text(message).size(type_scale::UI).color(subtext0))
        .width(Length::Fill)
        .height(Length::Fixed(layout::STATUS_BAR_HEIGHT))
        .padding([0, spacing::MD as u16])
        .center_y(Length::Fixed(layout::STATUS_BAR_HEIGHT))
        .style(theme::status_bar_style)
        .into()
}
