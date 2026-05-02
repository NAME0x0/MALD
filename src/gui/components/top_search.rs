//! Top bar — brand wordmark, breadcrumb, status. Sovereign Terminal style.
//!
//! Layout: `MALD                    path/to/note.md            Local · model · ●`
//! No centered search input. Search is reached via the command palette
//! (`Ctrl+Shift+F`).

use iced::widget::{container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::gui::message::{DaemonStatus, Message};
use crate::gui::theme::{self, colors, layout, spacing, type_scale};

/// Render the top bar.
///
/// `breadcrumb` — current file path / section label (e.g. `vault/projects/note.md`).
/// `index_label` — short status (`Indexed`, `Indexing…`, `Idle`).
/// `model_label` — model identifier (e.g. `llama3`).
pub fn view<'a>(
    _query: &'a str,
    _focused: bool,
    breadcrumb: String,
    daemon_status: DaemonStatus,
    model_label: &'a str,
    index_label: String,
    is_dark: bool,
) -> Element<'a, Message> {
    let subtext0 = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let accent = if is_dark {
        colors::ACCENT
    } else {
        colors::latte::ACCENT
    };
    let red = if is_dark {
        colors::RED
    } else {
        colors::latte::RED
    };
    let dim = if is_dark {
        colors::SURFACE2
    } else {
        colors::latte::SURFACE2
    };

    // Brand block — green "MALD" wordmark, mono.
    let brand = row![text("MALD")
        .size(type_scale::H2)
        .color(accent)
        .font(iced::Font::MONOSPACE)]
    .align_y(Alignment::Center);

    // Breadcrumb middle — current file path or section
    let crumb_text = if breadcrumb.is_empty() {
        text("").size(type_scale::UI)
    } else {
        text(breadcrumb).size(type_scale::UI).color(subtext0)
    };
    let crumb_block = row![crumb_text].align_y(Alignment::Center);

    // Right block — daemon dot + Local · model · index_label
    let dot_color = match daemon_status {
        DaemonStatus::Running => accent,
        DaemonStatus::Stopped => red,
        DaemonStatus::Unknown => dim,
    };
    let sep = || {
        text("·")
            .size(type_scale::UI)
            .color(subtext0)
            .font(iced::Font::MONOSPACE)
    };
    let model_block = row![
        text("Local")
            .size(type_scale::UI)
            .color(text_color)
            .font(iced::Font::MONOSPACE),
        sep(),
        text(model_label)
            .size(type_scale::UI)
            .color(subtext0)
            .font(iced::Font::MONOSPACE),
        sep(),
        text(index_label)
            .size(type_scale::UI)
            .color(subtext0)
            .font(iced::Font::MONOSPACE),
        Space::new().width(spacing::XS),
        text("●").size(type_scale::UI).color(dot_color),
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center);

    let bar = row![
        container(brand).width(Length::FillPortion(1)),
        container(crumb_block)
            .width(Length::FillPortion(2))
            .center_x(Length::Fill),
        container(model_block)
            .width(Length::FillPortion(1))
            .align_x(iced::alignment::Horizontal::Right),
    ]
    .spacing(spacing::MD)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(layout::SEARCH_BAR_HEIGHT))
        .padding([spacing::SM as u16, spacing::LG as u16])
        .style(theme::top_search_container_style(false))
        .into()
}

/// Compact version (legacy callsites — narrow widget search). Kept as a no-op
/// passthrough so the callers compile until they migrate to the command palette.
#[allow(dead_code)]
pub fn view_compact<'a>(_query: &'a str, _placeholder: &'a str) -> Element<'a, Message> {
    Space::new().width(0).height(0).into()
}
