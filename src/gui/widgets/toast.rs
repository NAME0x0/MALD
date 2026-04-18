//! Toast notification widget for transient messages.
//!
//! Provides feedback for errors, warnings, and success states.
//! Includes slide-up + fade-in animation on enter, fade-out on exit.

use std::time::Instant;

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};

use crate::gui::animations::ToastAnimation;
use crate::gui::message::Message;
use crate::gui::theme::{self, colors, spacing, type_scale};

/// Toast severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn color(&self, is_dark: bool) -> iced::Color {
        match self {
            ToastLevel::Info => {
                if is_dark {
                    colors::BLUE
                } else {
                    colors::latte::BLUE
                }
            }
            ToastLevel::Success => {
                if is_dark {
                    colors::GREEN
                } else {
                    colors::latte::GREEN
                }
            }
            ToastLevel::Warning => {
                if is_dark {
                    colors::YELLOW
                } else {
                    colors::latte::YELLOW
                }
            }
            ToastLevel::Error => {
                if is_dark {
                    colors::RED
                } else {
                    colors::latte::RED
                }
            }
        }
    }

    fn icon_char(&self) -> &'static str {
        match self {
            ToastLevel::Info => "\u{2139}",
            ToastLevel::Success => "\u{2713}",
            ToastLevel::Warning => "\u{26a0}",
            ToastLevel::Error => "\u{2715}",
        }
    }
}

/// Toast animation phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastPhase {
    Entering,
    Visible,
    Exiting,
}

/// Toast notification data
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: usize,
    pub level: ToastLevel,
    pub message: String,
    pub dismissible: bool,
    pub created_at: Instant,
    pub phase: ToastPhase,
    pub animation: Option<ToastAnimation>,
}

impl Toast {
    fn new(id: usize, level: ToastLevel, message: impl Into<String>) -> Self {
        Self {
            id,
            level,
            message: message.into(),
            dismissible: true,
            created_at: Instant::now(),
            phase: ToastPhase::Entering,
            animation: Some(ToastAnimation::enter(theme::animation::TOAST_ENTER)),
        }
    }

    pub fn error(id: usize, message: impl Into<String>) -> Self {
        Self::new(id, ToastLevel::Error, message)
    }

    pub fn success(id: usize, message: impl Into<String>) -> Self {
        Self::new(id, ToastLevel::Success, message)
    }

    pub fn warning(id: usize, message: impl Into<String>) -> Self {
        Self::new(id, ToastLevel::Warning, message)
    }

    pub fn info(id: usize, message: impl Into<String>) -> Self {
        Self::new(id, ToastLevel::Info, message)
    }

    /// Check if toast should auto-dismiss (after display time)
    pub fn should_auto_dismiss(&self) -> bool {
        self.phase == ToastPhase::Visible
            && self.created_at.elapsed() > theme::animation::TOAST_AUTO_DISMISS
    }

    /// Start exit animation
    pub fn start_exit(&mut self) {
        if self.phase != ToastPhase::Exiting {
            self.phase = ToastPhase::Exiting;
            self.animation = Some(ToastAnimation::exit(theme::animation::TOAST_EXIT));
        }
    }

    /// Update animation state, returns true if toast should be removed
    pub fn tick(&mut self) -> bool {
        if let Some(ref anim) = self.animation {
            if anim.is_complete() {
                match self.phase {
                    ToastPhase::Entering => {
                        self.phase = ToastPhase::Visible;
                        self.animation = None;
                    }
                    ToastPhase::Exiting => {
                        return true; // Remove toast
                    }
                    ToastPhase::Visible => {}
                }
            }
        }
        false
    }

    /// Get current animation values (opacity, y_offset)
    pub fn animation_values(&self) -> (f32, f32) {
        match &self.animation {
            Some(anim) => anim.values(),
            None => (1.0, 0.0), // Fully visible, no offset
        }
    }
}

/// Render a single toast notification with animation
pub fn view_toast<'a>(toast: &'a Toast, is_dark: bool) -> Element<'a, Message> {
    let accent = toast.level.color(is_dark);
    let (opacity, y_offset) = toast.animation_values();

    let base_text = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };
    let base_subtext = if is_dark {
        colors::SUBTEXT0
    } else {
        colors::latte::SUBTEXT0
    };
    let base_bg = if is_dark {
        colors::MANTLE
    } else {
        colors::latte::MANTLE
    };

    // Apply opacity to colors
    let accent_with_opacity = iced::Color {
        a: accent.a * opacity,
        ..accent
    };
    let text_color = iced::Color {
        a: base_text.a * opacity,
        ..base_text
    };
    let subtext_color = iced::Color {
        a: base_subtext.a * opacity,
        ..base_subtext
    };
    let bg_color = iced::Color {
        a: base_bg.a * opacity,
        ..base_bg
    };

    let icon = text(toast.level.icon_char())
        .size(type_scale::BODY)
        .color(accent_with_opacity);

    let message = text(&toast.message).size(type_scale::UI).color(text_color);

    let mut content = row![icon, message]
        .spacing(spacing::SM)
        .align_y(iced::Alignment::Center);

    if toast.dismissible {
        let close = button(text("\u{00d7}").size(type_scale::BODY).color(subtext_color))
            .on_press(Message::ToastDismiss(toast.id))
            .padding([0, spacing::XS as u16])
            .style(crate::gui::theme::close_button_style);

        content = content.push(Space::new().width(Length::Fill));
        content = content.push(close);
    }

    // Inner container with styling
    let styled_toast = container(content)
        .padding([spacing::SM as u16, spacing::MD as u16])
        .width(Length::Fixed(320.0))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                color: accent_with_opacity,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.25 * opacity,
                },
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

    // Slide animation: use column with top spacer for y_offset
    column![Space::new().height(Length::Fixed(y_offset)), styled_toast,]
        .spacing(0)
        .into()
}

/// Render toast container (bottom-right positioned)
pub fn view_toasts<'a>(toasts: &'a [Toast], is_dark: bool) -> Element<'a, Message> {
    if toasts.is_empty() {
        return Space::new().width(0).height(0).into();
    }

    let toast_elements: Vec<Element<Message>> = toasts
        .iter()
        .rev()
        .take(5) // Max 5 visible toasts
        .map(|t| view_toast(t, is_dark))
        .collect();

    container(
        column(toast_elements)
            .spacing(spacing::SM)
            .align_x(iced::Alignment::End),
    )
    .width(Length::Shrink)
    .padding(spacing::LG as u16)
    .into()
}
