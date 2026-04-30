//! Status feedback widget for errors, warnings, and success states.
//!
//! Provides consistent status messaging across the application.

use iced::widget::{column, container, row, text};
use iced::Element;

use crate::gui::{icons, message::Message, theme::{self, colors, spacing, type_scale}};

/// Status feedback type.
#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
}

impl StatusType {
    /// Get the color for this status type.
    fn color(&self, is_dark: bool) -> iced::Color {
        match self {
            StatusType::Success => if is_dark { colors::GREEN } else { colors::latte::GREEN },
            StatusType::Warning => if is_dark { colors::YELLOW } else { colors::latte::YELLOW },
            StatusType::Error => if is_dark { colors::RED } else { colors::latte::RED },
            StatusType::Info => if is_dark { colors::ACCENT } else { colors::latte::ACCENT },
        }
    }
}

/// Status feedback struct.
pub struct StatusFeedback<'a> {
    status_type: StatusType,
    title: &'a str,
    message: Option<&'a str>,
    is_dark: bool,
}

impl<'a> StatusFeedback<'a> {
    /// Create a new status feedback.
    pub fn new(status_type: StatusType, title: &'a str, is_dark: bool) -> Self {
        Self {
            status_type,
            title,
            message: None,
            is_dark,
        }
    }

    /// Add a detailed message.
    pub fn message(mut self, msg: &'a str) -> Self {
        self.message = Some(msg);
        self
    }

    /// Render the status feedback.
    pub fn view(self) -> Element<'a, Message> {
        let accent = self.status_type.color(self.is_dark);
        let text_color = if self.is_dark { colors::TEXT } else { colors::latte::TEXT };
        let subtext0 = if self.is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };

        let icon: Element<Message> = match self.status_type {
            StatusType::Success => icons::success().color(accent).into(),
            StatusType::Warning => icons::warning().color(accent).into(),
            StatusType::Error => icons::error().color(accent).into(),
            StatusType::Info => icons::info().color(accent).into(),
        };

        let title_text = text(self.title)
            .size(type_scale::UI)
            .color(text_color);

        let content: Element<Message> = if let Some(msg) = self.message {
            column![
                row![icon, title_text].spacing(spacing::SM).align_y(iced::Alignment::Center),
                text(msg).size(type_scale::CAPTION).color(subtext0),
            ]
            .spacing(spacing::XS)
            .into()
        } else {
            row![icon, title_text]
                .spacing(spacing::SM)
                .align_y(iced::Alignment::Center)
                .into()
        };

        container(content)
            .padding(spacing::SM as u16)
            .style(theme::card_style)
            .into()
    }
}

/// Quick constructors for common status states.
pub mod presets {
    use super::*;

    /// Success feedback.
    pub fn success<'a>(title: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Success, title, is_dark).view()
    }

    /// Success feedback with message.
    pub fn success_with_message<'a>(title: &'a str, message: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Success, title, is_dark)
            .message(message)
            .view()
    }

    /// Warning feedback.
    pub fn warning<'a>(title: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Warning, title, is_dark).view()
    }

    /// Warning feedback with message.
    pub fn warning_with_message<'a>(title: &'a str, message: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Warning, title, is_dark)
            .message(message)
            .view()
    }

    /// Error feedback.
    pub fn error<'a>(title: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Error, title, is_dark).view()
    }

    /// Error feedback with message.
    pub fn error_with_message<'a>(title: &'a str, message: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Error, title, is_dark)
            .message(message)
            .view()
    }

    /// Info feedback.
    pub fn info<'a>(title: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Info, title, is_dark).view()
    }

    /// Info feedback with message.
    pub fn info_with_message<'a>(title: &'a str, message: &'a str, is_dark: bool) -> Element<'a, Message> {
        StatusFeedback::new(StatusType::Info, title, is_dark)
            .message(message)
            .view()
    }
}
