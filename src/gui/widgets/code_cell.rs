//! Executable code cell widget with inline output.

use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};
use iced_anim::widget::button;
use iced_aw::Card;

use crate::gui::icons;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};
use crate::gui::widgets::chart_view;

#[derive(Debug, Clone)]
pub struct CodeCell {
    pub id: usize,
    pub language: String,
    pub code: String,
    pub output: Option<String>,
    pub status: CellStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStatus {
    Idle,
    Running,
    Success,
    Error,
}

impl CodeCell {
    pub fn new(id: usize, language: String, code: String) -> Self {
        Self {
            id,
            language,
            code,
            output: None,
            status: CellStatus::Idle,
        }
    }
}

pub fn view(cell: &CodeCell, is_dark: bool) -> Element<'static, Message> {
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
    let yellow = if is_dark {
        colors::YELLOW
    } else {
        colors::latte::YELLOW
    };
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

    let status_color = match cell.status {
        CellStatus::Idle => subtext0,
        CellStatus::Running => yellow,
        CellStatus::Success => green,
        CellStatus::Error => red,
    };

    let header = row![
        icons::file().size(type_scale::UI),
        text(cell.language.clone())
            .size(type_scale::UI)
            .color(subtext0),
        Space::new().width(Length::Fill),
        text(match cell.status {
            CellStatus::Running => "Running",
            CellStatus::Success => "Success",
            CellStatus::Error => "Error",
            CellStatus::Idle => "Idle",
        })
        .size(type_scale::CAPTION)
        .color(status_color),
        if cell.status == CellStatus::Running {
            button(icons::stop().color(red)).style(theme::ghost_button_style(false))
        } else {
            button(icons::play().color(green))
                .on_press(Message::RunCodeBlock(cell.id))
                .style(theme::ghost_button_style(false))
        }
    ]
    .spacing(spacing::SM);

    let code_area = container(
        text(cell.code.clone())
            .font(iced::Font::MONOSPACE)
            .size(type_scale::UI)
            .color(text_color),
    )
    .style(theme::code_block_style)
    .padding(spacing::MD as u16)
    .width(Length::Fill);

    let output_area = cell.output.as_ref().map(|out| {
        container(
            text(out.clone())
                .font(iced::Font::MONOSPACE)
                .size(type_scale::UI)
                .color(text_color),
        )
        .style(theme::output_block_style)
        .padding(spacing::SM as u16)
        .width(Length::Fill)
    });

    let chart = if matches!(cell.language.as_str(), "plot" | "chart") {
        chart_view::view_from_code(&cell.code)
    } else if let Some(out) = &cell.output {
        chart_view::view_from_code(out)
    } else {
        None
    };

    let mut body = column![code_area].spacing(spacing::SM);
    if let Some(out) = output_area {
        body = body.push(out);
    }
    if let Some(chart) = chart {
        body = body.push(chart);
    }

    Card::new(header, body)
        .padding(spacing::SM.into())
        .style(theme::aw_card_style)
        .into()
}
