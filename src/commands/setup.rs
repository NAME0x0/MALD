use anyhow::Result;
use crossterm::style::Stylize;
use serde_json::Value;
use std::io::{self, IsTerminal, Write};

#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

use crate::config::ConfigManager;
use crate::fs::ensure_directory;

pub async fn run(action: Option<crate::cli::SetupAction>) -> Result<()> {
    match action {
        None => crate::commands::wizard::run().await,
        Some(crate::cli::SetupAction::Editor { editor }) => {
            let choice = (!editor.is_empty()).then_some(editor.join(" "));
            let selected = configure_editor(choice.as_deref())?;
            println!(
                "{} {}",
                "Editor configured for MALD:".green().bold(),
                selected.as_str().bold()
            );
            println!(
                "  {}",
                "You can change it again anytime with `mald setup editor`.".dark_grey()
            );
            Ok(())
        }
        Some(crate::cli::SetupAction::Path) => {
            let summary = ensure_shell_command()?;
            println!("{summary}");
            Ok(())
        }
    }
}

pub fn mald_on_path() -> bool {
    crate::commands::launch::command_exists("mald")
}

pub fn configure_editor(choice: Option<&str>) -> Result<String> {
    let selected = select_editor(choice, io::stdout().is_terminal())?;
    let config_path = crate::fs::mald_home().join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;
    config.set("editor", Value::String(selected.clone()))?;
    Ok(selected)
}

pub fn select_editor(choice: Option<&str>, interactive: bool) -> Result<String> {
    if let Some(choice) = choice.map(str::trim).filter(|choice| !choice.is_empty()) {
        return Ok(resolve_editor_input(choice));
    }

    if interactive {
        return prompt_for_editor();
    }

    crate::commands::launch::auto_detect_editor().ok_or_else(|| {
        crate::errors::bail_ctx(
            "No supported editor was auto-detected.",
            "Run `mald setup editor` in a terminal to pick one manually, or use `mald config set editor <command>`.",
        )
    })
}

#[cfg(windows)]
pub fn maybe_offer_path_setup() -> Result<()> {
    if mald_on_path() {
        return Ok(());
    }

    println!("  {} MALD is not on your shell PATH yet.", "->".yellow());
    println!(
        "    Add it now so {} works in Command Prompt and PowerShell anywhere.",
        "mald".cyan()
    );

    if prompt_yes_no("  Add MALD to PATH", true)? {
        let summary = ensure_shell_command()?;
        for line in summary.lines() {
            println!("  {line}");
        }
        println!();
    } else {
        println!(
            "  {} Skipped PATH setup. Run {} later if you want the shell command.",
            "->".yellow(),
            "mald setup path".cyan()
        );
        println!();
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn maybe_offer_path_setup() -> Result<()> {
    Ok(())
}

fn resolve_editor_input(input: &str) -> String {
    crate::commands::launch::resolve_editor_choice(input)
        .unwrap_or_else(|| input.trim().to_string())
}

fn prompt_for_editor() -> Result<String> {
    let detected = crate::commands::launch::detected_editors();

    println!("  {}:", "Choose your editor".bold());
    if detected.is_empty() {
        println!("    No common editors were auto-detected.");
        println!("    Type one of these friendly names, or enter a command/path:");
        println!("    - VS Code: code");
        println!("    - Neovim: nvim");
        println!("    - Helix: hx");
        println!("    - Zed: zed");
    } else {
        for (index, editor) in detected.iter().enumerate() {
            println!(
                "    {}. {} ({}){}",
                index + 1,
                editor.label,
                editor.command,
                " detected".green()
            );
        }
        println!("    Or type a command/path manually.");
    }

    print!("  Choice [1]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        if let Some(editor) = detected.first() {
            return Ok(editor.command.clone());
        }
        return Ok(resolve_editor_input(default_editor_hint()));
    }

    if let Ok(index) = input.parse::<usize>() {
        if let Some(editor) = detected.get(index.saturating_sub(1)) {
            return Ok(editor.command.clone());
        }
    }

    Ok(resolve_editor_input(input))
}

fn default_editor_hint() -> &'static str {
    if cfg!(windows) {
        "code"
    } else {
        "nvim"
    }
}

fn prompt_yes_no(label: &str, default_yes: bool) -> Result<bool> {
    let prompt = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{label} {prompt}: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_ascii_lowercase();

    if input.is_empty() {
        return Ok(default_yes);
    }

    Ok(matches!(input.as_str(), "y" | "yes"))
}

#[cfg(windows)]
pub fn ensure_shell_command() -> Result<String> {
    let current_exe = std::env::current_exe()?;
    let install_dir = windows_install_dir()?;
    let install_path = install_dir.join("mald.exe");

    ensure_directory(&install_dir)?;

    let copied = if same_windows_path(&current_exe, &install_path) {
        false
    } else {
        crate::fs::safe_copy(&current_exe, &install_path)?;
        true
    };

    let user_path = read_user_path().unwrap_or_default();
    let install_dir_str = install_dir.to_string_lossy().to_string();
    let added_to_path = if path_contains_entry(&user_path, &install_dir_str) {
        false
    } else {
        let updated = prepend_path_entry(&user_path, &install_dir_str);
        write_user_path(&updated)?;
        true
    };

    ensure_current_process_path(&install_dir_str);

    let mut lines = vec![format!(
        "{} {}",
        "Shell command ready at".green().bold(),
        install_path.display()
    )];

    if copied {
        lines.push(format!(
            "{} Copied the current MALD binary into {}.",
            "->".green(),
            install_dir.display()
        ));
    } else {
        lines.push(format!(
            "{} MALD is already installed in {}.",
            "->".green(),
            install_dir.display()
        ));
    }

    if added_to_path {
        lines.push(format!(
            "{} Added {} to your user PATH. Open a new terminal to use {} everywhere.",
            "->".green(),
            install_dir.display(),
            "mald".cyan()
        ));
    } else {
        lines.push(format!(
            "{} {} is already on your user PATH.",
            "->".green(),
            install_dir.display()
        ));
    }

    Ok(lines.join("\n"))
}

#[cfg(not(windows))]
pub fn ensure_shell_command() -> Result<String> {
    Err(crate::errors::bail_ctx(
        "Automatic shell PATH setup is currently only built into MALD on Windows.",
        "On macOS/Linux, add the MALD binary directory to PATH manually, or use `cargo install --path . --features gui`.",
    ))
}

#[cfg(windows)]
fn windows_install_dir() -> Result<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        crate::errors::bail_ctx(
            "Could not determine LOCALAPPDATA for PATH setup.",
            "Create the `%LOCALAPPDATA%` environment variable, or install MALD with `install.ps1`.",
        )
    })?;
    Ok(PathBuf::from(local_app_data).join("mald"))
}

#[cfg(windows)]
fn same_windows_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(windows)]
fn powershell_binary() -> String {
    crate::commands::launch::resolved_command_string("pwsh")
        .or_else(|| crate::commands::launch::resolved_command_string("powershell"))
        .unwrap_or_else(|| "powershell".to_string())
}

#[cfg(windows)]
fn read_user_path() -> Result<String> {
    let output = Command::new(powershell_binary())
        .args([
            "-NoProfile",
            "-Command",
            "[Environment]::GetEnvironmentVariable('Path','User')",
        ])
        .output()
        .map_err(|error| {
            crate::errors::bail_ctx(
                format!("Could not read your user PATH: {error}"),
                "Run `mald setup path` from PowerShell or Command Prompt on Windows.",
            )
        })?;

    if !output.status.success() {
        return Err(crate::errors::bail_ctx(
            "Could not read your user PATH.",
            "Run `mald setup path` from PowerShell or Command Prompt on Windows.",
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(windows)]
fn write_user_path(path: &str) -> Result<()> {
    let escaped = path.replace('\'', "''");
    let command = format!("[Environment]::SetEnvironmentVariable('Path','{escaped}','User')");
    let status = Command::new(powershell_binary())
        .args(["-NoProfile", "-Command", &command])
        .status()
        .map_err(|error| {
            crate::errors::bail_ctx(
                format!("Could not update your user PATH: {error}"),
                "Run `install.ps1`, or add `%LOCALAPPDATA%\\mald` to PATH manually.",
            )
        })?;

    if !status.success() {
        return Err(crate::errors::bail_ctx(
            "Could not update your user PATH.",
            "Run `install.ps1`, or add `%LOCALAPPDATA%\\mald` to PATH manually.",
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn path_contains_entry(path: &str, entry: &str) -> bool {
    path.split(';')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .any(|segment| segment.eq_ignore_ascii_case(entry))
}

#[cfg(windows)]
fn prepend_path_entry(path: &str, entry: &str) -> String {
    if path.trim().is_empty() {
        entry.to_string()
    } else {
        format!("{entry};{path}")
    }
}

#[cfg(windows)]
fn ensure_current_process_path(entry: &str) {
    let current = std::env::var("PATH").unwrap_or_default();
    if !path_contains_entry(&current, entry) {
        let updated = prepend_path_entry(&current, entry);
        std::env::set_var("PATH", updated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_editor_prefers_alias_resolution() {
        let configured = select_editor(Some("vscode"), false).unwrap();
        assert!(!configured.trim().is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn prepend_path_entry_puts_new_dir_first() {
        let updated = prepend_path_entry(r"C:\Windows", r"C:\Users\Test\AppData\Local\mald");
        assert!(updated.starts_with(r"C:\Users\Test\AppData\Local\mald;"));
    }

    #[cfg(windows)]
    #[test]
    fn path_contains_entry_is_case_insensitive() {
        assert!(path_contains_entry(
            r"C:\Users\Test\AppData\Local\MALD;C:\Windows",
            r"c:\users\test\appdata\local\mald"
        ));
    }
}
