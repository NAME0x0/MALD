//! Reusable loading indicator widget.
//!
//! Provides consistent loading states across the application.

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::gui::{icons, message::Message, theme::{colors, spacing, type_scale}};

/// Loading indicator size variants.
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingSize {
    /// Small inline indicator (icon only).
    Small,
    /// Medium indicator with short text.
    #[default]
    Medium,
    /// Large centered indicator with text.
    Large,
}

/// Loading indicator struct.
pub struct Loading<'a> {
    size: LoadingSize,
    message: Option<&'a str>,
    is_dark: bool,
}

impl<'a> Loading<'a> {
    /// Create a new loading indicator.
    pub fn new(is_dark: bool) -> Self {
        Self {
            size: LoadingSize::default(),
            message: None,
            is_dark,
        }
    }

    /// Set the size variant.
    pub fn size(mut self, size: LoadingSize) -> Self {
        self.size = size;
        self
    }

    /// Set a custom message.
    pub fn message(mut self, msg: &'a str) -> Self {
        self.message = Some(msg);
        self
    }

    /// Render the loading indicator.
    pub fn view(self) -> Element<'a, Message> {
        let blue = if self.is_dark { colors::BLUE } else { colors::latte::BLUE };
        let subtext0 = if self.is_dark { colors::SUBTEXT0 } else { colors::latte::SUBTEXT0 };

        match self.size {
            LoadingSize::Small => {
                icons::loading().color(blue).into()
            }
            LoadingSize::Medium => {
                let msg = self.message.unwrap_or("Loading...");
                row![
                    icons::loading().color(blue),
                    text(msg).size(type_scale::CAPTION).color(subtext0),
                ]
                .spacing(spacing::XS)
                .into()
            }
            LoadingSize::Large => {
                let msg = self.message.unwrap_or("Loading...");
                container(
                    column![
                        icons::loading().color(blue),
                        text(msg).size(type_scale::UI).color(subtext0),
                    ]
                    .spacing(spacing::SM)
                    .align_x(iced::Alignment::Center),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .padding(spacing::LG as u16)
                .into()
            }
        }
    }
}

/// Quick constructors for common loading states.
pub mod presets {
    use super::*;

    /// Small inline loading indicator.
    pub fn inline<'a>(is_dark: bool) -> Element<'a, Message> {
        Loading::new(is_dark).size(LoadingSize::Small).view()
    }

    /// Medium loading indicator with default text.
    pub fn loading<'a>(is_dark: bool) -> Element<'a, Message> {
        Loading::new(is_dark).size(LoadingSize::Medium).view()
    }

    /// Loading indicator for AI thinking.
    pub fn thinking<'a>(is_dark: bool) -> Element<'a, Message> {
        Loading::new(is_dark)
            .size(LoadingSize::Medium)
            .message("Thinking...")
            .view()
    }

    /// Loading indicator for search operations.
    pub fn searching<'a>(is_dark: bool) -> Element<'a, Message> {
        Loading::new(is_dark)
            .size(LoadingSize::Medium)
            .message("Searching...")
            .view()
    }

    /// Large centered loading for full-page states.
    pub fn page_loading<'a>(is_dark: bool) -> Element<'a, Message> {
        Loading::new(is_dark).size(LoadingSize::Large).view()
    }
}
