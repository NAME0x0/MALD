//! Catppuccin Mocha + Latte theme for MALD GUI.
//!
//! Implements a VSCode+Obsidian hybrid design with consistent styling.
//! Supports both dark (Mocha) and light (Latte) themes with full color palettes.

use iced::theme::Palette;
use iced::widget::{button, container, scrollable, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use iced_aw::style::{badge as aw_badge, card as aw_card, Status as AwStatus};

// ══════════════════════════════════════════════════════════════════════════════
// Catppuccin Mocha Palette (Dark Theme)
// ══════════════════════════════════════════════════════════════════════════════

pub mod colors {
    use iced::Color;

    // ── Sovereign Terminal palette ──────────────────────────────────────────
    // Backgrounds (terminal black → graphite panels)
    pub const CRUST: Color = Color::from_rgb(0.0314, 0.0353, 0.0392); // #08090A - status bar / outermost
    pub const BASE: Color = Color::from_rgb(0.0431, 0.0510, 0.0549); // #0B0D0E - editor / main bg
    pub const MANTLE: Color = Color::from_rgb(0.0667, 0.0784, 0.0863); // #111416 - sidebar / feature panel
    pub const SURFACE0: Color = Color::from_rgb(0.0902, 0.1020, 0.1137); // #171A1D - inactive tabs / borders
    pub const SURFACE1: Color = Color::from_rgb(0.1216, 0.1373, 0.1490); // #1F2326 - hover
    pub const SURFACE2: Color = Color::from_rgb(0.1647, 0.1843, 0.2000); // #2A2F33 - disabled bg

    // Text (off-white → graphite muted)
    pub const TEXT: Color = Color::from_rgb(0.9098, 0.8863, 0.8314); // #E8E2D4 - primary
    pub const SUBTEXT1: Color = Color::from_rgb(0.7216, 0.7098, 0.6745); // #B8B5AC - secondary
    pub const SUBTEXT0: Color = Color::from_rgb(0.5451, 0.5686, 0.5412); // #8B918A - muted

    // Accent system — single sage + warm gold pair (no rainbow)
    pub const GREEN: Color = Color::from_rgb(0.5608, 0.6863, 0.5608); // #8FAF8F - sage accent
    pub const YELLOW: Color = Color::from_rgb(0.7490, 0.6549, 0.4157); // #BFA76A - warm gold (tags)
    pub const PEACH: Color = Color::from_rgb(0.8196, 0.6039, 0.4000); // #D19A66 - warning
    pub const RED: Color = Color::from_rgb(0.7765, 0.4706, 0.4706); // #C67878 - error

    // Compat aliases — collapsed onto the sovereign two-accent system.
    // All non-primary "rainbow" Catppuccin slots fold into sage/gold/text variants
    // so widgets still compile but the visual identity stays monotone.
    pub const BLUE: Color = GREEN; // sage stand-in (was bright blue)
    pub const SAPPHIRE: Color = Color::from_rgb(0.4863, 0.5961, 0.4863); // darker sage
    pub const TEAL: Color = Color::from_rgb(0.6353, 0.7373, 0.6353); // lighter sage
    pub const SKY: Color = GREEN;
    pub const LAVENDER: Color = SUBTEXT1; // links == off-white
    pub const MAUVE: Color = YELLOW; // keywords == warm gold
    pub const PINK: Color = PEACH;
    pub const FLAMINGO: Color = PEACH;
    pub const ROSEWATER: Color = TEXT;

    // Brand accent — sovereign sage.
    pub const ACCENT: Color = GREEN;

    // Semantic colors (aliases for clarity — dark theme only, use themed() for light)
    pub const ACTIVITY_BAR_BG: Color = CRUST;
    pub const STATUS_BAR_BG: Color = CRUST;
    pub const SIDEBAR_BG: Color = MANTLE;
    pub const FEATURE_PANEL_BG: Color = MANTLE;
    pub const EDITOR_BG: Color = BASE;
    pub const TAB_ACTIVE_BG: Color = BASE;
    pub const TAB_INACTIVE_BG: Color = SURFACE0;
    pub const TERMINAL_BG: Color = MANTLE;
    pub const BORDER: Color = SURFACE0;
    pub const HOVER: Color = SURFACE1;
    pub const SELECTION: Color = SURFACE1;

    pub const OVERLAY_BG: Color = Color {
        r: 0.067,
        g: 0.067,
        b: 0.106,
        a: 0.85,
    };

    // Syntax highlighting
    pub const HEADING: Color = TEAL;
    pub const LINK: Color = LAVENDER;
    pub const TAG: Color = YELLOW;
    pub const STRING: Color = PEACH;
    pub const KEYWORD: Color = MAUVE;
    pub const COMMENT: Color = SURFACE2;
    pub const LINE_NUMBER: Color = SURFACE2;
    pub const TASK_OPEN: Color = YELLOW;
    pub const TASK_DONE: Color = GREEN;

    // ══════════════════════════════════════════════════════════════════════════
    // Catppuccin Latte Palette (Light Theme)
    // ══════════════════════════════════════════════════════════════════════════

    // Dark-only build — `latte` module exists so all `is_light_theme()` callsites
    // still compile, but every constant mirrors the dark sovereign palette so
    // toggling theme produces no visible change. Light-mode is intentionally dead.
    pub mod latte {
        use iced::Color;

        pub const ROSEWATER: Color = Color::from_rgb(0.9098, 0.8863, 0.8314);
        pub const FLAMINGO: Color = Color::from_rgb(0.8196, 0.6039, 0.4000);
        pub const PINK: Color = Color::from_rgb(0.8196, 0.6039, 0.4000);
        pub const MAUVE: Color = Color::from_rgb(0.7490, 0.6549, 0.4157);
        pub const RED: Color = Color::from_rgb(0.7765, 0.4706, 0.4706);
        pub const MAROON: Color = Color::from_rgb(0.7765, 0.4706, 0.4706);
        pub const PEACH: Color = Color::from_rgb(0.8196, 0.6039, 0.4000);
        pub const YELLOW: Color = Color::from_rgb(0.7490, 0.6549, 0.4157);
        pub const GREEN: Color = Color::from_rgb(0.5608, 0.6863, 0.5608);
        pub const TEAL: Color = Color::from_rgb(0.6353, 0.7373, 0.6353);
        pub const SKY: Color = Color::from_rgb(0.5608, 0.6863, 0.5608);
        pub const SAPPHIRE: Color = Color::from_rgb(0.4863, 0.5961, 0.4863);
        pub const BLUE: Color = Color::from_rgb(0.5608, 0.6863, 0.5608);
        pub const LAVENDER: Color = Color::from_rgb(0.7216, 0.7098, 0.6745);

        pub const ACCENT: Color = GREEN;

        pub const TEXT: Color = Color::from_rgb(0.9098, 0.8863, 0.8314);
        pub const SUBTEXT1: Color = Color::from_rgb(0.7216, 0.7098, 0.6745);
        pub const SUBTEXT0: Color = Color::from_rgb(0.5451, 0.5686, 0.5412);

        pub const OVERLAY2: Color = Color::from_rgb(0.5451, 0.5686, 0.5412);
        pub const OVERLAY1: Color = Color::from_rgb(0.5451, 0.5686, 0.5412);
        pub const OVERLAY0: Color = Color::from_rgb(0.5451, 0.5686, 0.5412);

        pub const SURFACE2: Color = Color::from_rgb(0.1647, 0.1843, 0.2000);
        pub const SURFACE1: Color = Color::from_rgb(0.1216, 0.1373, 0.1490);
        pub const SURFACE0: Color = Color::from_rgb(0.0902, 0.1020, 0.1137);

        pub const BASE: Color = Color::from_rgb(0.0431, 0.0510, 0.0549);
        pub const MANTLE: Color = Color::from_rgb(0.0667, 0.0784, 0.0863);
        pub const CRUST: Color = Color::from_rgb(0.0314, 0.0353, 0.0392);

        pub const OVERLAY_BG: Color = Color {
            r: 0.0314,
            g: 0.0353,
            b: 0.0392,
            a: 0.85,
        };

        pub const SELECTION: Color = SURFACE1;
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Design System Tokens
// ══════════════════════════════════════════════════════════════════════════════

/// Typography scale — constrained to 7 sizes with clear roles
pub mod type_scale {
    pub const DISPLAY: f32 = 32.0; // App title only
    pub const H1: f32 = 24.0; // Page headings
    pub const H2: f32 = 20.0; // Section headings
    pub const H3: f32 = 16.0; // Subsection headings
    pub const BODY: f32 = 14.0; // Primary content
    pub const UI: f32 = 12.0; // Interface elements
    pub const CAPTION: f32 = 10.0; // Hints, badges, metadata
}

/// Spacing scale — consistent rhythm throughout the app
pub mod spacing {
    pub const XS: f32 = 4.0; // Tight spacing (icons, inline)
    pub const SM: f32 = 8.0; // Small gaps (list items)
    pub const MD: f32 = 12.0; // Medium gaps (sections)
    pub const LG: f32 = 16.0; // Large gaps (panels)
    pub const XL: f32 = 24.0; // Extra large (page sections)
    pub const XXL: f32 = 32.0; // Page margins
    pub const XXXL: f32 = 48.0; // Hero spacing
}

/// Icon sizes — standardized weights
pub mod icon_size {
    pub const PRIMARY: u16 = 16; // Activity bar, primary actions
    pub const SECONDARY: u16 = 14; // Close buttons, chevrons
    pub const INLINE: u16 = 12; // Inline indicators
}

/// Layout constants — refined dimensions
pub mod layout {
    pub const ACTIVITY_BAR_WIDTH: f32 = 52.0;
    pub const ACTIVITY_BUTTON_SIZE: f32 = 40.0;
    pub const TAB_BAR_HEIGHT: f32 = 40.0;
    pub const SEARCH_BAR_HEIGHT: f32 = 44.0;
    pub const STATUS_BAR_HEIGHT: f32 = 24.0;
    pub const SIDEBAR_DEFAULT_WIDTH: f32 = 250.0;
    pub const FEATURE_PANEL_DEFAULT_WIDTH: f32 = 380.0;
    pub const TERMINAL_DEFAULT_HEIGHT: f32 = 200.0;
    pub const MODAL_WIDTH: f32 = 600.0;
    pub const MODAL_VERTICAL_PADDING: f32 = 100.0;
    pub const PANEL_HEADER_HEIGHT: f32 = 32.0;
    pub const SEARCH_INPUT_WIDTH: f32 = 400.0;
    pub const CLOSE_BUTTON_SIZE: f32 = 16.0;

    // Resize bounds
    pub const SIDEBAR_MIN_WIDTH: f32 = 150.0;
    pub const SIDEBAR_MAX_WIDTH: f32 = 500.0;
    pub const FEATURE_PANEL_MIN_WIDTH: f32 = 200.0;
    pub const FEATURE_PANEL_MAX_WIDTH: f32 = 600.0;
    pub const TERMINAL_MIN_HEIGHT: f32 = 100.0;
    pub const TERMINAL_MAX_HEIGHT: f32 = 500.0;

    // Content dimensions
    pub const TOAST_WIDTH: f32 = 320.0;
    pub const MERMAID_DIAGRAM_HEIGHT: f32 = 260.0;

    // Resource limits
    pub const OPEN_TABS_MAX: usize = 50;
}

// Geometry tokens
const RADIUS_SM: f32 = 4.0;
const RADIUS_MD: f32 = 8.0;
const RADIUS_LG: f32 = 12.0;

// Focus state tokens
pub const FOCUS_RING_WIDTH: f32 = 2.0;
pub const FOCUS_RING_OFFSET: f32 = 2.0;

fn shadow_soft() -> Shadow {
    Shadow {
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.25,
        },
        offset: Vector::new(0.0, 2.0),
        blur_radius: 12.0,
    }
}

fn shadow_subtle() -> Shadow {
    Shadow {
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.15,
        },
        offset: Vector::new(0.0, 1.0),
        blur_radius: 6.0,
    }
}

// Light theme uses softer shadows
fn shadow_soft_light() -> Shadow {
    Shadow {
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.12,
        },
        offset: Vector::new(0.0, 2.0),
        blur_radius: 8.0,
    }
}

fn shadow_subtle_light() -> Shadow {
    Shadow {
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.08,
        },
        offset: Vector::new(0.0, 1.0),
        blur_radius: 4.0,
    }
}

// Animation timings
pub mod animation {
    use std::time::Duration;

    // Core transitions
    pub const HOVER: Duration = Duration::from_millis(150);
    pub const COLLAPSE: Duration = Duration::from_millis(200);
    pub const PRESS: Duration = Duration::from_millis(100);
    pub const TAB_CLOSE: Duration = Duration::from_millis(150);

    // Phase 1 Motion: Toast animations
    pub const TOAST_ENTER: Duration = Duration::from_millis(200);
    pub const TOAST_EXIT: Duration = Duration::from_millis(150);
    pub const TOAST_AUTO_DISMISS: Duration = Duration::from_millis(4000); // 4s display time

    // Phase 1 Motion: Modal transitions
    pub const MODAL_FADE_IN: Duration = Duration::from_millis(150);
    pub const MODAL_FADE_OUT: Duration = Duration::from_millis(100);

    // Phase 1 Motion: Velocity-aware panel animation
    pub const PANEL_PIXELS_PER_SEC: f32 = 1200.0; // Smooth but responsive

    // Phase 2 Motion: List stagger (future)
    pub const LIST_STAGGER_DELAY: Duration = Duration::from_millis(30);

    // Search debounce: minimum time between async search dispatches
    pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(150);
}

// Resource limits — prevent unbounded memory growth
pub mod limits {
    /// Maximum terminal output lines (ring buffer, oldest evicted)
    pub const TERMINAL_LINES_MAX: usize = 10_000;
    /// Maximum AI chat messages (oldest pairs evicted)
    pub const AI_CHAT_MESSAGES_MAX: usize = 500;
    /// Maximum concurrent toasts (newest replaces oldest when full)
    pub const TOASTS_MAX: usize = 10;
}

// ══════════════════════════════════════════════════════════════════════════════
// Theme Detection Helper
// ══════════════════════════════════════════════════════════════════════════════

/// Detect whether the current Iced theme is light by checking background luminance.
///
/// Returns `true` for Catppuccin Latte and other light themes.
/// This is the key function that enables all widget styles to be theme-aware.
pub fn is_light_theme(theme: &Theme) -> bool {
    let palette = theme.palette();
    let bg = palette.background;
    // Latte BASE is #eff1f5 (luminance ~0.94), Mocha BASE is #1e1e2e (luminance ~0.13)
    (0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b) > 0.5
}

/// Select a color based on the current theme: dark color for Mocha, light color for Latte.
#[inline]
pub fn themed(theme: &Theme, dark: Color, light: Color) -> Color {
    if is_light_theme(theme) {
        light
    } else {
        dark
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Theme Configuration
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct MaldTheme {
    pub is_dark: bool,
}

impl Default for MaldTheme {
    fn default() -> Self {
        Self { is_dark: true }
    }
}

impl MaldTheme {
    /// Theme toggle — sovereign dark is the only mode. No-op preserved for the
    /// existing message wiring; flipping `is_dark` in the field does nothing
    /// because the palette is identical and `iced_theme()` always returns dark.
    pub fn toggle(&mut self) {
        // Sovereign Terminal OS is dark-only. Toggling does nothing.
    }

    pub fn iced_theme(&self) -> Theme {
        Theme::custom("MALD Sovereign", sovereign_palette())
    }
}

fn sovereign_palette() -> Palette {
    Palette {
        background: colors::BASE,
        text: colors::TEXT,
        primary: colors::ACCENT,
        success: colors::GREEN,
        danger: colors::RED,
        warning: colors::PEACH,
    }
}

// Retained for source compatibility — both old constructors now resolve to the
// sovereign palette so any lingering reference still compiles.
#[allow(dead_code)]
fn catppuccin_mocha_palette() -> Palette {
    sovereign_palette()
}

#[allow(dead_code)]
fn catppuccin_latte_palette() -> Palette {
    sovereign_palette()
}

// ══════════════════════════════════════════════════════════════════════════════
// Widget Styles (Theme-Aware)
// ══════════════════════════════════════════════════════════════════════════════

/// Style for the activity bar (leftmost 48px strip)
pub fn activity_bar_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::CRUST
        } else {
            colors::CRUST
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for sidebar panels
pub fn sidebar_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the main editor area
pub fn editor_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::BASE
        } else {
            colors::BASE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the status bar
pub fn status_bar_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::CRUST
        } else {
            colors::CRUST
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for terminal panel
pub fn terminal_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: if light {
            shadow_subtle_light()
        } else {
            shadow_subtle()
        },
        ..Default::default()
    }
}

/// Style for feature panel (right side - backlinks, AI, outline)
pub fn feature_panel_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: if light {
            shadow_subtle_light()
        } else {
            shadow_subtle()
        },
        ..Default::default()
    }
}

/// Style for the top search bar container
pub fn top_search_container_style(focused: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme: &Theme| {
        let light = is_light_theme(theme);
        container::Style {
            background: Some(Background::Color(if light {
                colors::latte::BASE
            } else {
                colors::BASE
            })),
            border: Border {
                color: if focused {
                    if light {
                        colors::latte::ACCENT
                    } else {
                        colors::ACCENT
                    }
                } else if light {
                    colors::latte::SURFACE0
                } else {
                    colors::SURFACE0
                },
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            shadow: if light {
                shadow_soft_light()
            } else {
                shadow_soft()
            },
            ..Default::default()
        }
    }
}

/// Style for overlay/modal backgrounds
pub fn overlay_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::OVERLAY_BG
        } else {
            colors::OVERLAY_BG
        })),
        border: Border::default(),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Style for modal dialogs (palette, search modal)
pub fn modal_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE1
            } else {
                colors::SURFACE1
            },
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        shadow: if light {
            shadow_soft_light()
        } else {
            shadow_soft()
        },
        ..Default::default()
    }
}

/// Style for cards (home view quick actions)
pub fn card_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::SURFACE0
        } else {
            colors::SURFACE0
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE1
            } else {
                colors::SURFACE1
            },
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: if light {
            shadow_subtle_light()
        } else {
            shadow_subtle()
        },
        ..Default::default()
    }
}

/// Style for iced_aw cards
pub fn aw_card_style(theme: &Theme, status: AwStatus) -> aw_card::Style {
    let light = is_light_theme(theme);

    let (border_color, head_bg) = if light {
        match status {
            AwStatus::Hovered => (colors::latte::ACCENT, colors::latte::SURFACE1),
            AwStatus::Pressed | AwStatus::Focused | AwStatus::Selected => {
                (colors::latte::LAVENDER, colors::latte::SURFACE2)
            }
            AwStatus::Disabled => (colors::latte::SURFACE0, colors::latte::SURFACE0),
            AwStatus::Active => (colors::latte::SURFACE1, colors::latte::SURFACE0),
        }
    } else {
        match status {
            AwStatus::Hovered => (colors::ACCENT, colors::SURFACE1),
            AwStatus::Pressed | AwStatus::Focused | AwStatus::Selected => {
                (colors::LAVENDER, colors::SURFACE2)
            }
            AwStatus::Disabled => (colors::SURFACE0, colors::SURFACE0),
            AwStatus::Active => (colors::SURFACE1, colors::SURFACE0),
        }
    };

    let (bg, text_color, subtext) = if light {
        (
            colors::latte::MANTLE,
            colors::latte::TEXT,
            colors::latte::SUBTEXT0,
        )
    } else {
        (colors::MANTLE, colors::TEXT, colors::SUBTEXT0)
    };

    aw_card::Style {
        background: Background::Color(bg),
        border_radius: RADIUS_MD,
        border_width: 1.0,
        border_color,
        head_background: Background::Color(head_bg),
        head_text_color: text_color,
        body_background: Background::Color(bg),
        body_text_color: text_color,
        foot_background: Background::Color(bg),
        foot_text_color: text_color,
        close_color: subtext,
    }
}

/// Style for iced_aw badges - decorative only, no interactive states
pub fn aw_badge_style(theme: &Theme, _status: AwStatus) -> aw_badge::Style {
    let light = is_light_theme(theme);
    // Badges are decorative indicators, not interactive elements
    // Consistent styling regardless of status prevents false affordance
    aw_badge::Style {
        background: Background::Color(if light {
            colors::latte::SURFACE1
        } else {
            colors::SURFACE1
        }),
        border_radius: Some(RADIUS_SM),
        border_width: 0.0,
        border_color: None,
        text_color: if light {
            colors::latte::SUBTEXT0
        } else {
            colors::SUBTEXT0
        },
    }
}

/// Style for section headers
pub fn section_header_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for code blocks (markdown/code cells)
pub fn code_block_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::CRUST
        } else {
            colors::CRUST
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE1
            } else {
                colors::SURFACE1
            },
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        shadow: if light {
            shadow_subtle_light()
        } else {
            shadow_subtle()
        },
        ..Default::default()
    }
}

/// Style for code output blocks
pub fn output_block_style(theme: &Theme) -> container::Style {
    let light = is_light_theme(theme);
    container::Style {
        background: Some(Background::Color(if light {
            colors::latte::MANTLE
        } else {
            colors::MANTLE
        })),
        border: Border {
            color: if light {
                colors::latte::SURFACE1
            } else {
                colors::SURFACE1
            },
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        shadow: if light {
            shadow_subtle_light()
        } else {
            shadow_subtle()
        },
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Button Styles (Theme-Aware)
// ══════════════════════════════════════════════════════════════════════════════

/// Ghost button style function (transparent, for toolbar/activity bar)
pub fn ghost_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let light = is_light_theme(theme);
        let (bg, text_color) = match status {
            button::Status::Active => {
                if active {
                    if light {
                        (colors::latte::SURFACE1, colors::latte::ACCENT)
                    } else {
                        (colors::SURFACE1, colors::ACCENT)
                    }
                } else if light {
                    (Color::TRANSPARENT, colors::latte::SUBTEXT0)
                } else {
                    (Color::TRANSPARENT, colors::SUBTEXT0)
                }
            }
            button::Status::Hovered => {
                if light {
                    (colors::latte::SURFACE1, colors::latte::TEXT)
                } else {
                    (colors::SURFACE1, colors::TEXT)
                }
            }
            button::Status::Pressed => {
                if light {
                    (colors::latte::SURFACE0, colors::latte::TEXT)
                } else {
                    (colors::SURFACE0, colors::TEXT)
                }
            }
            button::Status::Disabled => {
                if light {
                    (Color::TRANSPARENT, colors::latte::SURFACE2)
                } else {
                    (Color::TRANSPARENT, colors::SURFACE2)
                }
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    }
}

/// Activity bar button style
pub fn activity_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let light = is_light_theme(theme);
        let hover_bg = {
            let base = if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            };
            Color {
                r: base.r,
                g: base.g,
                b: base.b,
                a: 0.5,
            }
        };
        let pressed_bg = if light {
            colors::latte::SURFACE0
        } else {
            colors::SURFACE0
        };
        let active_color = if light {
            colors::latte::ACCENT
        } else {
            colors::ACCENT
        };
        let inactive_color = if light {
            colors::latte::SUBTEXT0
        } else {
            colors::SUBTEXT0
        };
        let hover_color = if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        };
        let inactive_dim = Color {
            r: inactive_color.r,
            g: inactive_color.g,
            b: inactive_color.b,
            a: 0.6,
        };
        let disabled_color = if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        };
        let disabled_dim = Color {
            r: disabled_color.r,
            g: disabled_color.g,
            b: disabled_color.b,
            a: 0.6,
        };
        let (bg, text_color) = match status {
            button::Status::Active => {
                let text_color = if active { active_color } else { inactive_dim };
                (Color::TRANSPARENT, text_color)
            }
            button::Status::Hovered => {
                let text_color = if active { active_color } else { hover_color };
                (hover_bg, text_color)
            }
            button::Status::Pressed => {
                let text_color = if active { active_color } else { hover_color };
                (pressed_bg, text_color)
            }
            button::Status::Disabled => (Color::TRANSPARENT, disabled_dim),
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                width: 0.0,
                radius: RADIUS_SM.into(),
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

/// Primary button (blue accent)
pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let light = is_light_theme(theme);
    let (bg, text_color) = match status {
        button::Status::Active => {
            if light {
                (colors::latte::ACCENT, colors::latte::BASE)
            } else {
                (colors::ACCENT, colors::CRUST)
            }
        }
        button::Status::Hovered => {
            if light {
                (colors::latte::SAPPHIRE, colors::latte::BASE)
            } else {
                (colors::SAPPHIRE, colors::CRUST)
            }
        }
        button::Status::Pressed => {
            if light {
                (colors::latte::SKY, colors::latte::BASE)
            } else {
                (colors::SKY, colors::CRUST)
            }
        }
        button::Status::Disabled => {
            if light {
                (colors::latte::SURFACE0, colors::latte::SURFACE2)
            } else {
                (colors::SURFACE0, colors::SURFACE2)
            }
        }
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_SM.into(),
        },
        ..Default::default()
    }
}

/// Secondary button (surface bg)
pub fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let light = is_light_theme(theme);
    let (bg, text_color, border_color) = if light {
        match status {
            button::Status::Active => (
                colors::latte::SURFACE0,
                colors::latte::TEXT,
                colors::latte::SURFACE1,
            ),
            button::Status::Hovered => (
                colors::latte::SURFACE1,
                colors::latte::TEXT,
                colors::latte::SURFACE2,
            ),
            button::Status::Pressed => (
                colors::latte::SURFACE2,
                colors::latte::TEXT,
                colors::latte::SURFACE2,
            ),
            button::Status::Disabled => (
                colors::latte::SURFACE0,
                colors::latte::SURFACE2,
                colors::latte::SURFACE0,
            ),
        }
    } else {
        match status {
            button::Status::Active => (colors::SURFACE0, colors::TEXT, colors::SURFACE1),
            button::Status::Hovered => (colors::SURFACE1, colors::TEXT, colors::SURFACE2),
            button::Status::Pressed => (colors::SURFACE2, colors::TEXT, colors::SURFACE2),
            button::Status::Disabled => (colors::SURFACE0, colors::SURFACE2, colors::SURFACE0),
        }
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        ..Default::default()
    }
}

/// Tab button style (for editor tabs)
pub fn tab_button_style(
    active: bool,
    modified: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let light = is_light_theme(theme);

        let bg = if active {
            if light {
                colors::latte::BASE
            } else {
                colors::BASE
            }
        } else if light {
            colors::latte::SURFACE0
        } else {
            colors::SURFACE0
        };
        let text_color = if active {
            if light {
                colors::latte::TEXT
            } else {
                colors::TEXT
            }
        } else if light {
            colors::latte::SUBTEXT0
        } else {
            colors::SUBTEXT0
        };

        let bg = match status {
            button::Status::Active => bg,
            button::Status::Hovered => {
                if light {
                    colors::latte::SURFACE1
                } else {
                    colors::SURFACE1
                }
            }
            button::Status::Pressed => {
                if light {
                    colors::latte::SURFACE0
                } else {
                    colors::SURFACE0
                }
            }
            button::Status::Disabled => {
                if light {
                    colors::latte::SURFACE0
                } else {
                    colors::SURFACE0
                }
            }
        };

        let border_color = if modified {
            if light {
                colors::latte::ACCENT
            } else {
                colors::ACCENT
            }
        } else {
            Color::TRANSPARENT
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: border_color,
                width: if modified { 2.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Close button (small X)
pub fn close_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let light = is_light_theme(theme);
    let (bg, text_color) = match status {
        button::Status::Active => {
            if light {
                (Color::TRANSPARENT, colors::latte::SUBTEXT0)
            } else {
                (Color::TRANSPARENT, colors::SUBTEXT0)
            }
        }
        button::Status::Hovered => {
            if light {
                (colors::latte::RED, colors::latte::BASE)
            } else {
                (colors::RED, colors::CRUST)
            }
        }
        button::Status::Pressed => {
            if light {
                (colors::latte::RED, colors::latte::BASE)
            } else {
                (colors::RED, colors::CRUST)
            }
        }
        button::Status::Disabled => {
            if light {
                (Color::TRANSPARENT, colors::latte::SURFACE2)
            } else {
                (Color::TRANSPARENT, colors::SURFACE2)
            }
        }
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_SM.into(),
        },
        ..Default::default()
    }
}

/// List item button style (for file tree, search results, etc)
pub fn list_item_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let light = is_light_theme(theme);
        let (bg, text_color, border_color) = if light {
            match status {
                button::Status::Active => {
                    if selected {
                        (
                            colors::latte::SURFACE1,
                            colors::latte::TEXT,
                            Color::TRANSPARENT,
                        )
                    } else {
                        (Color::TRANSPARENT, colors::latte::TEXT, Color::TRANSPARENT)
                    }
                }
                button::Status::Hovered => (
                    colors::latte::SURFACE1,
                    colors::latte::TEXT,
                    Color::TRANSPARENT,
                ),
                button::Status::Pressed => (
                    colors::latte::SURFACE0,
                    colors::latte::TEXT,
                    colors::latte::ACCENT,
                ),
                button::Status::Disabled => (
                    Color::TRANSPARENT,
                    colors::latte::SURFACE2,
                    Color::TRANSPARENT,
                ),
            }
        } else {
            match status {
                button::Status::Active => {
                    if selected {
                        (colors::SURFACE1, colors::TEXT, Color::TRANSPARENT)
                    } else {
                        (Color::TRANSPARENT, colors::TEXT, Color::TRANSPARENT)
                    }
                }
                button::Status::Hovered => (colors::SURFACE1, colors::TEXT, Color::TRANSPARENT),
                button::Status::Pressed => (colors::SURFACE0, colors::TEXT, colors::ACCENT),
                button::Status::Disabled => {
                    (Color::TRANSPARENT, colors::SURFACE2, Color::TRANSPARENT)
                }
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: border_color,
                width: if matches!(status, button::Status::Pressed) {
                    FOCUS_RING_WIDTH
                } else {
                    0.0
                },
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Text Input Styles (Theme-Aware)
// ══════════════════════════════════════════════════════════════════════════════

/// Standard text input style
pub fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let light = is_light_theme(theme);
    let (bg, border_color, border_width) = if light {
        match status {
            text_input::Status::Active => (colors::latte::SURFACE0, colors::latte::SURFACE1, 1.0),
            text_input::Status::Hovered => (colors::latte::SURFACE0, colors::latte::SURFACE2, 1.0),
            text_input::Status::Focused { .. } => {
                (colors::latte::SURFACE0, colors::latte::ACCENT, 2.0)
            }
            text_input::Status::Disabled => (colors::latte::MANTLE, colors::latte::SURFACE0, 1.0),
        }
    } else {
        match status {
            text_input::Status::Active => (colors::SURFACE0, colors::SURFACE1, 1.0),
            text_input::Status::Hovered => (colors::SURFACE0, colors::SURFACE2, 1.0),
            text_input::Status::Focused { .. } => (colors::SURFACE0, colors::ACCENT, 2.0),
            text_input::Status::Disabled => (colors::MANTLE, colors::SURFACE0, 1.0),
        }
    };

    text_input::Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_SM.into(),
        },
        icon: if light {
            colors::latte::SUBTEXT0
        } else {
            colors::SUBTEXT0
        },
        placeholder: if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        },
        value: if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        },
        selection: if light {
            colors::latte::SELECTION
        } else {
            colors::SELECTION
        },
    }
}

/// Search input style (for top search bar)
pub fn search_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let light = is_light_theme(theme);
    let (bg, border_color, border_width) = if light {
        match status {
            text_input::Status::Active => (colors::latte::SURFACE0, colors::latte::SURFACE1, 1.0),
            text_input::Status::Hovered => (colors::latte::SURFACE0, colors::latte::ACCENT, 1.0),
            text_input::Status::Focused { .. } => {
                (colors::latte::SURFACE0, colors::latte::ACCENT, 2.0)
            }
            text_input::Status::Disabled => (colors::latte::MANTLE, colors::latte::SURFACE0, 1.0),
        }
    } else {
        match status {
            text_input::Status::Active => (colors::SURFACE0, colors::SURFACE1, 1.0),
            text_input::Status::Hovered => (colors::SURFACE0, colors::ACCENT, 1.0),
            text_input::Status::Focused { .. } => (colors::SURFACE0, colors::ACCENT, 2.0),
            text_input::Status::Disabled => (colors::MANTLE, colors::SURFACE0, 1.0),
        }
    };

    text_input::Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_MD.into(),
        },
        icon: if light {
            colors::latte::SUBTEXT0
        } else {
            colors::SUBTEXT0
        },
        placeholder: if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        },
        value: if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        },
        selection: if light {
            colors::latte::SELECTION
        } else {
            colors::SELECTION
        },
    }
}

/// Terminal input style
pub fn terminal_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let light = is_light_theme(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => {
            if light {
                colors::latte::ACCENT
            } else {
                colors::ACCENT
            }
        }
        _ => {
            if light {
                colors::latte::SURFACE0
            } else {
                colors::SURFACE0
            }
        }
    };

    text_input::Style {
        background: Background::Color(if light {
            colors::latte::CRUST
        } else {
            colors::CRUST
        }),
        border: Border {
            color: border_color,
            width: 0.0,
            radius: 0.0.into(),
        },
        icon: if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        },
        placeholder: if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        },
        value: if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        },
        selection: if light {
            colors::latte::SELECTION
        } else {
            colors::SELECTION
        },
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Text Editor Styles (Theme-Aware)
// ══════════════════════════════════════════════════════════════════════════════

/// Multi-line text editor style (for the main code/markdown editor)
pub fn text_editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let light = is_light_theme(theme);
    let (bg, border_color, border_width) = if light {
        match status {
            text_editor::Status::Active => (colors::latte::BASE, colors::latte::SURFACE0, 0.0),
            text_editor::Status::Hovered => (colors::latte::BASE, colors::latte::SURFACE1, 0.0),
            text_editor::Status::Focused { .. } => {
                (colors::latte::BASE, colors::latte::ACCENT, 0.0)
            }
            text_editor::Status::Disabled => (colors::latte::MANTLE, colors::latte::SURFACE0, 0.0),
        }
    } else {
        match status {
            text_editor::Status::Active => (colors::BASE, colors::SURFACE0, 0.0),
            text_editor::Status::Hovered => (colors::BASE, colors::SURFACE1, 0.0),
            text_editor::Status::Focused { .. } => (colors::BASE, colors::ACCENT, 0.0),
            text_editor::Status::Disabled => (colors::MANTLE, colors::SURFACE0, 0.0),
        }
    };

    text_editor::Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: border_width,
            radius: 0.0.into(),
        },
        placeholder: if light {
            colors::latte::SURFACE2
        } else {
            colors::SURFACE2
        },
        value: if light {
            colors::latte::TEXT
        } else {
            colors::TEXT
        },
        selection: if light {
            colors::latte::SELECTION
        } else {
            colors::SELECTION
        },
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Scrollable Styles (Theme-Aware)
// ══════════════════════════════════════════════════════════════════════════════

/// Standard scrollbar style
pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let light = is_light_theme(theme);

    let scroller_color = if light {
        match status {
            scrollable::Status::Active { .. } => colors::latte::SURFACE1,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: h,
                is_vertical_scrollbar_hovered: v,
                ..
            } => {
                if h || v {
                    colors::latte::SURFACE2
                } else {
                    colors::latte::SURFACE1
                }
            }
            scrollable::Status::Dragged { .. } => colors::latte::ACCENT,
        }
    } else {
        match status {
            scrollable::Status::Active { .. } => colors::SURFACE1,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: h,
                is_vertical_scrollbar_hovered: v,
                ..
            } => {
                if h || v {
                    colors::SURFACE2
                } else {
                    colors::SURFACE1
                }
            }
            scrollable::Status::Dragged { .. } => colors::ACCENT,
        }
    };

    // Subtle rail background for scroll affordance (30% opacity)
    let surface0 = if light {
        colors::latte::SURFACE0
    } else {
        colors::SURFACE0
    };
    let rail_bg = Color {
        r: surface0.r,
        g: surface0.g,
        b: surface0.b,
        a: 0.3,
    };

    let (auto_scroll_bg, auto_scroll_border, auto_scroll_icon) = if light {
        (
            colors::latte::SURFACE0,
            colors::latte::SURFACE2,
            colors::latte::TEXT,
        )
    } else {
        (colors::SURFACE0, colors::SURFACE2, colors::TEXT)
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(rail_bg)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(scroller_color),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: RADIUS_SM.into(),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(rail_bg)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(scroller_color),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: RADIUS_SM.into(),
                },
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(auto_scroll_bg),
            border: Border {
                color: auto_scroll_border,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            shadow: Shadow::default(),
            icon: auto_scroll_icon,
        },
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Utility Functions
// ══════════════════════════════════════════════════════════════════════════════

/// Get text color for daemon status
pub fn daemon_status_color(running: bool) -> Color {
    if running {
        colors::GREEN
    } else {
        colors::RED
    }
}

/// Get appropriate text color based on background
pub fn contrasting_text(bg: Color) -> Color {
    // Simple luminance check
    let luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    if luminance > 0.5 {
        colors::latte::TEXT
    } else {
        colors::TEXT
    }
}
