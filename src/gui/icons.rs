//! Bootstrap icon helpers (iced_fonts via iced_aw).

use iced::widget::Text;
use iced_aw::iced_fonts::bootstrap;

fn icon(text: Text<'static, iced::Theme>, size: u16) -> Text<'static, iced::Theme> {
    text.size(size as f32)
}

pub fn files() -> Text<'static, iced::Theme> {
    icon(bootstrap::folder_fill(), 18)
}

pub fn search() -> Text<'static, iced::Theme> {
    icon(bootstrap::search(), 18)
}

pub fn graph() -> Text<'static, iced::Theme> {
    icon(bootstrap::diagram_three_fill(), 18)
}

pub fn tasks() -> Text<'static, iced::Theme> {
    icon(bootstrap::check_square(), 18)
}

pub fn ai() -> Text<'static, iced::Theme> {
    icon(bootstrap::robot(), 18)
}

pub fn settings() -> Text<'static, iced::Theme> {
    icon(bootstrap::gear_fill(), 18)
}

pub fn mald_home() -> Text<'static, iced::Theme> {
    Text::new("M").size(18.0)
}

pub fn file() -> Text<'static, iced::Theme> {
    icon(bootstrap::file_earmark_text(), 16)
}

pub fn folder_open() -> Text<'static, iced::Theme> {
    icon(bootstrap::folder_fill(), 16)
}

pub fn folder_closed() -> Text<'static, iced::Theme> {
    icon(bootstrap::folder(), 16)
}

pub fn chevron_down() -> Text<'static, iced::Theme> {
    icon(bootstrap::chevron_down(), 12)
}

pub fn chevron_right() -> Text<'static, iced::Theme> {
    icon(bootstrap::chevron_right(), 12)
}

pub fn close() -> Text<'static, iced::Theme> {
    icon(bootstrap::x(), 14)
}

pub fn terminal() -> Text<'static, iced::Theme> {
    icon(bootstrap::terminal(), 16)
}

pub fn play() -> Text<'static, iced::Theme> {
    icon(bootstrap::play_fill(), 14)
}

pub fn stop() -> Text<'static, iced::Theme> {
    icon(bootstrap::stop_fill(), 14)
}

pub fn check_box() -> Text<'static, iced::Theme> {
    icon(bootstrap::check_square_fill(), 14)
}

pub fn empty_box() -> Text<'static, iced::Theme> {
    icon(bootstrap::square(), 14)
}

// ─────────────────────────────────────────────────────────────────────────────
// Empty state icons (larger, for placeholder views)
// ─────────────────────────────────────────────────────────────────────────────

pub fn empty_folder() -> Text<'static, iced::Theme> {
    icon(bootstrap::folder(), 32)
}

pub fn empty_search() -> Text<'static, iced::Theme> {
    icon(bootstrap::search(), 32)
}

pub fn empty_graph() -> Text<'static, iced::Theme> {
    icon(bootstrap::diagram_three(), 32)
}

pub fn empty_tasks() -> Text<'static, iced::Theme> {
    icon(bootstrap::clipboard(), 32)
}

pub fn empty_ai() -> Text<'static, iced::Theme> {
    icon(bootstrap::chat_dots(), 32)
}

pub fn empty_editor() -> Text<'static, iced::Theme> {
    icon(bootstrap::file_earmark(), 32)
}

// ─────────────────────────────────────────────────────────────────────────────
// Status icons
// ─────────────────────────────────────────────────────────────────────────────

pub fn loading() -> Text<'static, iced::Theme> {
    icon(bootstrap::arrow_repeat(), 16)
}

pub fn success() -> Text<'static, iced::Theme> {
    icon(bootstrap::check_circle_fill(), 16)
}

pub fn error() -> Text<'static, iced::Theme> {
    icon(bootstrap::exclamation_circle_fill(), 16)
}

pub fn warning() -> Text<'static, iced::Theme> {
    icon(bootstrap::exclamation_triangle_fill(), 16)
}

pub fn info() -> Text<'static, iced::Theme> {
    icon(bootstrap::info_circle_fill(), 16)
}

pub fn plus() -> Text<'static, iced::Theme> {
    icon(bootstrap::plus_lg(), 14)
}

pub fn new_file() -> Text<'static, iced::Theme> {
    icon(bootstrap::file_earmark_plus(), 16)
}

pub fn refresh() -> Text<'static, iced::Theme> {
    icon(bootstrap::arrow_clockwise(), 14)
}
