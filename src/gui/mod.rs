pub mod animations;
pub mod app;
pub mod canvas;
pub mod components;
pub mod icons;
pub mod layout;
pub mod message;
pub mod syntax;
pub mod theme;
pub mod util;
pub mod widgets;

use app::MaldApp;

pub fn run() -> anyhow::Result<()> {
    tracing::info!("Starting MALD GUI");

    let window = iced::window::Settings {
        size: iced::Size::new(1400.0, 900.0),
        icon: build_window_icon(),
        ..Default::default()
    };

    let result = iced::application(MaldApp::new, MaldApp::update, MaldApp::view)
        .title(MaldApp::title)
        .theme(MaldApp::theme)
        .subscription(MaldApp::subscription)
        .window(window)
        .antialiasing(true)
        .run()
        .map_err(|e| anyhow::anyhow!("GUI error: {e}"));

    tracing::info!("MALD GUI closed");
    result
}

fn build_window_icon() -> Option<iced::window::Icon> {
    const SIZE: u32 = 64;
    const BG: [u8; 4] = [0, 0, 0, 255];
    const FG: [u8; 4] = [255, 255, 255, 255];
    const CUT: [u8; 4] = [0, 0, 0, 255];

    let mut pixels = vec![0; (SIZE * SIZE * 4) as usize];

    fill_rect(&mut pixels, SIZE, 0, 0, SIZE, SIZE, BG);
    fill_rect(&mut pixels, SIZE, 18, 12, 28, 40, FG);
    fill_rect(&mut pixels, SIZE, 24, 12, 3, 40, CUT);
    fill_rect(&mut pixels, SIZE, 30, 22, 10, 3, CUT);
    fill_rect(&mut pixels, SIZE, 30, 30, 10, 3, CUT);
    fill_rect(&mut pixels, SIZE, 30, 38, 8, 3, CUT);

    iced::window::icon::from_rgba(pixels, SIZE, SIZE).ok()
}

fn fill_rect(
    pixels: &mut [u8],
    canvas_size: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    for py in y..(y + height).min(canvas_size) {
        for px in x..(x + width).min(canvas_size) {
            let index = ((py * canvas_size + px) * 4) as usize;
            pixels[index..index + 4].copy_from_slice(&color);
        }
    }
}
