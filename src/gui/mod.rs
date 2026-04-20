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
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const GUI_LOCK_STARTUP_GRACE: Duration = Duration::from_secs(5);

pub fn run() -> anyhow::Result<()> {
    let Some(_instance_guard) = try_acquire_instance_guard()? else {
        tracing::warn!("MALD GUI is already running for this workspace");
        if std::io::stdout().is_terminal() {
            println!("MALD GUI is already running for this workspace.");
        }
        return Ok(());
    };

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

fn try_acquire_instance_guard() -> anyhow::Result<Option<GuiInstanceGuard>> {
    let home = crate::fs::mald_home();
    crate::fs::ensure_directory(&home)?;
    acquire_instance_guard_at_with_grace(&home, GUI_LOCK_STARTUP_GRACE)
}

fn acquire_instance_guard_at(home: &Path) -> anyhow::Result<Option<GuiInstanceGuard>> {
    acquire_instance_guard_at_with_grace(home, GUI_LOCK_STARTUP_GRACE)
}

fn acquire_instance_guard_at_with_grace(
    home: &Path,
    grace: Duration,
) -> anyhow::Result<Option<GuiInstanceGuard>> {
    let pid_file = home.join("gui.pid");
    let current_pid = std::process::id();

    for _ in 0..2 {
        match std::fs::OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&pid_file)
        {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(current_pid.to_string().as_bytes())?;
                let _ = file.sync_data();
                return Ok(Some(GuiInstanceGuard {
                    pid_file,
                    pid: current_pid,
                    _file: file,
                }));
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if existing_gui_lock_is_live(&pid_file, current_pid, grace) {
                    return Ok(None);
                }
                let _ = std::fs::remove_file(&pid_file);
            }
            Err(err) => return Err(err.into()),
        }
    }

    Ok(None)
}

struct GuiInstanceGuard {
    pid_file: PathBuf,
    pid: u32,
    _file: std::fs::File,
}

impl Drop for GuiInstanceGuard {
    fn drop(&mut self) {
        let current = std::fs::read_to_string(&self.pid_file)
            .ok()
            .and_then(|value| value.trim().parse::<u32>().ok());
        if current == Some(self.pid) {
            let _ = std::fs::remove_file(&self.pid_file);
        }
    }
}

fn existing_gui_lock_is_live(pid_file: &Path, current_pid: u32, grace: Duration) -> bool {
    if let Ok(existing_pid) = std::fs::read_to_string(pid_file) {
        let existing_pid = existing_pid.trim();
        if let Ok(existing_pid) = existing_pid.parse::<u32>() {
            return existing_pid == current_pid || process_alive(existing_pid);
        }
    }

    pid_file_recently_updated(pid_file, grace)
}

fn pid_file_recently_updated(pid_file: &Path, grace: Duration) -> bool {
    std::fs::metadata(pid_file)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .map(|age| age <= grace)
        .unwrap_or(false)
}

fn process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                !stdout.contains("INFO:") && stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn second_gui_guard_is_blocked_for_same_workspace() {
        let temp = tempfile::tempdir().expect("tempdir");

        let first = acquire_instance_guard_at_with_grace(temp.path(), Duration::from_millis(50))
            .expect("guard result")
            .expect("first guard");
        let second = acquire_instance_guard_at_with_grace(temp.path(), Duration::from_millis(50))
            .expect("guard result");

        assert!(second.is_none(), "second GUI guard should be blocked");

        drop(first);

        let third = acquire_instance_guard_at_with_grace(temp.path(), Duration::from_millis(50))
            .expect("guard result")
            .expect("guard after drop");
        drop(third);
    }

    #[test]
    fn stale_empty_lock_file_is_recovered_after_grace() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("gui.pid"), "").expect("write pid file");
        std::thread::sleep(Duration::from_millis(30));

        let guard = acquire_instance_guard_at_with_grace(temp.path(), Duration::from_millis(10))
            .expect("guard result");

        assert!(guard.is_some(), "stale empty lock should be recoverable");
    }

    #[test]
    fn gui_instance_guard_replaces_stale_pid_file() {
        let temp = TempDir::new().unwrap();
        let pid_file = temp.path().join("gui.pid");
        std::fs::write(&pid_file, "999999").unwrap();

        let guard = acquire_instance_guard_at(temp.path()).unwrap();
        assert!(guard.is_some());

        let current_pid = std::process::id().to_string();
        let stored = std::fs::read_to_string(pid_file).unwrap();
        assert_eq!(stored, current_pid);
    }
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
