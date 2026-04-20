use anyhow::Result;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEditor {
    pub id: &'static str,
    pub label: &'static str,
    pub command: String,
    pub directory_target: bool,
}

#[derive(Clone, Copy)]
struct EditorPreset {
    id: &'static str,
    label: &'static str,
    commands: &'static [&'static str],
    aliases: &'static [&'static str],
    directory_target: bool,
}

const EDITOR_PRESETS: &[EditorPreset] = &[
    EditorPreset {
        id: "code",
        label: "VS Code",
        commands: &["Code.exe", "code", "code.cmd"],
        aliases: &["code", "vscode", "vs code", "visual studio code"],
        directory_target: true,
    },
    EditorPreset {
        id: "nvim",
        label: "Neovim",
        commands: &["nvim", "nvim.exe"],
        aliases: &["nvim", "neovim"],
        directory_target: false,
    },
    EditorPreset {
        id: "vim",
        label: "Vim",
        commands: &["vim", "vim.exe"],
        aliases: &["vim"],
        directory_target: false,
    },
    EditorPreset {
        id: "hx",
        label: "Helix",
        commands: &["hx", "hx.exe"],
        aliases: &["hx", "helix"],
        directory_target: false,
    },
    EditorPreset {
        id: "zed",
        label: "Zed",
        commands: &["zed", "zed.exe"],
        aliases: &["zed"],
        directory_target: true,
    },
    EditorPreset {
        id: "subl",
        label: "Sublime Text",
        commands: &["subl", "subl.exe"],
        aliases: &["subl", "sublime", "sublime text"],
        directory_target: true,
    },
    EditorPreset {
        id: "nano",
        label: "Nano",
        commands: &["nano", "nano.exe"],
        aliases: &["nano"],
        directory_target: false,
    },
    EditorPreset {
        id: "emacs",
        label: "Emacs",
        commands: &["emacs", "emacs.exe"],
        aliases: &["emacs"],
        directory_target: false,
    },
];

pub fn command_exists(cmd: &str) -> bool {
    resolve_command(cmd).is_some()
}

pub fn command_for(cmd: &str) -> Command {
    Command::new(resolve_command(cmd).unwrap_or_else(|| OsString::from(cmd)))
}

pub fn resolved_command_string(cmd: &str) -> Option<String> {
    resolve_command(cmd).map(|command| command.to_string_lossy().into_owned())
}

pub fn detected_editors() -> Vec<DetectedEditor> {
    let mut detected = Vec::new();
    let mut seen = HashSet::new();

    for preset in EDITOR_PRESETS {
        if let Some(command) = preset
            .commands
            .iter()
            .find_map(|candidate| resolved_command_string(candidate))
        {
            let key = command.to_ascii_lowercase();
            if seen.insert(key) {
                detected.push(DetectedEditor {
                    id: preset.id,
                    label: preset.label,
                    command,
                    directory_target: preset.directory_target,
                });
            }
        }
    }

    detected
}

pub fn auto_detect_editor() -> Option<String> {
    detected_editors()
        .into_iter()
        .next()
        .map(|editor| editor.command)
}

pub fn resolve_editor_choice(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    if path.components().count() > 1 || path.is_absolute() {
        return Some(trimmed.to_string());
    }

    if let Some(preset) = matching_editor_preset(trimmed) {
        return preset
            .commands
            .iter()
            .find_map(|candidate| resolved_command_string(candidate))
            .or_else(|| Some(preset.id.to_string()));
    }

    resolved_command_string(trimmed).or_else(|| Some(trimmed.to_string()))
}

pub fn supports_directory_target(editor: &str) -> bool {
    if let Some(preset) = matching_editor_preset(editor) {
        return preset.directory_target;
    }

    let lower = editor.to_ascii_lowercase();
    lower.contains("code") || lower.contains("subl") || lower.contains("zed")
}

pub fn open_in_editor(editor: &str, target: impl AsRef<OsStr>) -> Result<()> {
    let mut command = command_for(editor);
    let status = command.arg(target.as_ref()).status().map_err(|_err| {
        if cfg!(windows) && editor.eq_ignore_ascii_case("code") {
            crate::errors::bail_ctx(
                "Could not launch VS Code from `code`.",
                "Run `mald setup editor` and choose VS Code, or set an explicit editor with `mald config set editor <command>`.",
            )
        } else {
            crate::errors::bail_ctx(
                format!("Could not launch editor `{editor}`."),
                "Run `mald setup editor`, or set an explicit editor with `mald config set editor <command>`.",
            )
        }
    })?;

    if !status.success() {
        return Err(crate::errors::bail_ctx(
            format!("Editor `{editor}` exited with a non-zero status."),
            "Try `mald setup editor` to switch editors, or set a different editor with `mald config set editor <command>`.",
        ));
    }

    Ok(())
}

fn matching_editor_preset(input: &str) -> Option<&'static EditorPreset> {
    let lower = input.trim().to_ascii_lowercase();
    let filename = Path::new(input)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase())
        .unwrap_or_else(|| lower.clone());

    EDITOR_PRESETS.iter().find(|preset| {
        preset.id.eq_ignore_ascii_case(&lower)
            || preset.id.eq_ignore_ascii_case(&filename)
            || preset.aliases.iter().any(|alias| {
                alias.eq_ignore_ascii_case(&lower) || alias.eq_ignore_ascii_case(&filename)
            })
            || preset.commands.iter().any(|command| {
                command.eq_ignore_ascii_case(&lower) || command.eq_ignore_ascii_case(&filename)
            })
    })
}

#[cfg(windows)]
fn resolve_command(cmd: &str) -> Option<OsString> {
    if cmd.trim().is_empty() {
        return None;
    }

    let path = Path::new(cmd);
    if path.components().count() > 1 || path.is_absolute() {
        return Some(OsString::from(cmd));
    }

    let output = Command::new("where").arg(cmd).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let mut fallback = None;
    let mut best_match: Option<(usize, OsString)> = None;
    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let candidate = OsString::from(line);
        if fallback.is_none() {
            fallback = Some(candidate.clone());
        }

        let ext = Path::new(line)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        let rank = match ext.as_deref() {
            Some("exe") => Some(0),
            Some("com") => Some(1),
            Some("cmd") => Some(2),
            Some("bat") => Some(3),
            _ => None,
        };

        if let Some(rank) = rank {
            let replace = match &best_match {
                Some((best_rank, _)) => rank < *best_rank,
                None => true,
            };
            if replace {
                best_match = Some((rank, candidate));
            }
        }
    }

    best_match.map(|(_, candidate)| candidate).or(fallback)
}

#[cfg(not(windows))]
fn resolve_command(cmd: &str) -> Option<OsString> {
    if cmd.trim().is_empty() {
        None
    } else {
        Some(OsString::from(cmd))
    }
}
