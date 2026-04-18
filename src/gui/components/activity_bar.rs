//! Activity bar component - 52px vertical icon strip with mode buttons.
//!
//! Includes a pulse animation on mode switch for tactile feedback.

use std::time::Instant;

use iced::widget::{column, container, row, text, tooltip, Text};
use iced::{Alignment, Background, Element, Length};
use iced_anim::widget::button;

use crate::gui::animations;
use crate::gui::icons;
use crate::gui::message::{ActivityMode, Message};
use crate::gui::theme::{self, colors, icon_size, layout, spacing, type_scale};

/// Render the activity bar (leftmost strip).
pub fn view(
    current_mode: ActivityMode,
    home_active: bool,
    pulse_mode: Option<ActivityMode>,
    pulse_start: Option<Instant>,
    is_dark: bool,
) -> Element<'static, Message> {
    let home_button = activity_action_button(
        icons::mald_home().size(icon_size::PRIMARY as f32),
        "Dashboard",
        Message::GoHome,
        home_active,
        0.0,
        is_dark,
    );

    let buttons: Vec<Element<Message>> = ActivityMode::all()
        .iter()
        .map(|mode| {
            let is_active = !home_active && *mode == current_mode;
            let pulse_intensity = if pulse_mode == Some(*mode) {
                pulse_start
                    .map(|start| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let t = (elapsed / 300.0).min(1.0);
                        // Quick flash, gentle fade: high at start, fades to 0
                        1.0 - animations::ease_out_quint(t)
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            activity_mode_button(*mode, is_active, pulse_intensity, is_dark)
        })
        .collect();

    let content = column(
        std::iter::once(home_button)
            .chain(std::iter::once(
                container(iced::widget::Space::new().height(Length::Fixed(spacing::SM))).into(),
            ))
            .chain(buttons)
            .collect::<Vec<_>>(),
    )
    .spacing(spacing::MD)
    .padding(spacing::SM as u16)
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fixed(layout::ACTIVITY_BAR_WIDTH))
        .height(Length::Fill)
        .style(theme::activity_bar_style)
        .into()
}

fn activity_mode_button(
    mode: ActivityMode,
    active: bool,
    pulse_intensity: f32,
    is_dark: bool,
) -> Element<'static, Message> {
    let icon = match mode {
        ActivityMode::Files => icons::files(),
        ActivityMode::Search => icons::search(),
        ActivityMode::Graph => icons::graph(),
        ActivityMode::Tasks => icons::tasks(),
        ActivityMode::AI => icons::ai(),
        ActivityMode::Settings => icons::settings(),
    };

    // Use standardized icon size
    let icon = icon.size(icon_size::PRIMARY as f32);

    activity_action_button(
        icon,
        mode.label(),
        Message::ActivityBarSelect(mode),
        active,
        pulse_intensity,
        is_dark,
    )
}

fn activity_action_button(
    icon: Text<'static, iced::Theme>,
    tooltip_label: &'static str,
    message: Message,
    active: bool,
    pulse_intensity: f32,
    is_dark: bool,
) -> Element<'static, Message> {
    let blue = if is_dark {
        colors::BLUE
    } else {
        colors::latte::BLUE
    };
    let surface0 = if is_dark {
        colors::SURFACE0
    } else {
        colors::latte::SURFACE0
    };
    let surface1 = if is_dark {
        colors::SURFACE1
    } else {
        colors::latte::SURFACE1
    };
    let text_color = if is_dark {
        colors::TEXT
    } else {
        colors::latte::TEXT
    };

    let btn = button(
        container(icon)
            .width(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
            .height(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
            .center_x(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
            .center_y(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE)),
    )
    .on_press(message)
    .width(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
    .height(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
    .style(move |theme, status| theme::activity_button_style(active)(theme, status));

    // Active indicator: 2px left edge bar in BLUE (VSCode pattern)
    // Pulse: indicator flashes bright then fades to steady state
    let indicator_color = if pulse_intensity > 0.01 {
        // During pulse: bright accent flash that fades
        iced::Color {
            r: blue.r,
            g: blue.g,
            b: blue.b,
            a: 0.4 + 0.6 * pulse_intensity, // 1.0 → 0.4 (bright flash to steady)
        }
    } else if active {
        blue
    } else {
        iced::Color::TRANSPARENT
    };

    let indicator_width = 2.0;

    let indicator = container(iced::widget::Space::new())
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(layout::ACTIVITY_BUTTON_SIZE))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(indicator_color)),
            ..Default::default()
        });

    let btn_with_indicator = row![indicator, btn].spacing(0);

    // Wrap with tooltip showing the mode label
    tooltip(
        btn_with_indicator,
        text(tooltip_label).size(type_scale::UI),
        tooltip::Position::Right,
    )
    .gap(spacing::XS)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(surface0)),
        border: iced::Border {
            color: surface1,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        text_color: Some(text_color),
        snap: false,
    })
    .into()
}
