use anyhow::{bail, Result};

use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::metadata::MetadataStore;

/// Search-and-open in one step.
///
/// - If fzf is in PATH: pipes results through fzf with preview, opens selection in $EDITOR.
/// - If no fzf: falls back to built-in TUI (same as `mald search` with no args).
/// - If query is empty: interactive mode (fzf or TUI).
/// - If query produces exactly one result: opens it directly.
pub async fn run(query: Vec<String>, _kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let editor = config.typed().editor.clone();

    // Ensure index exists
    let kb_dir = mald_home().join("kb");
    if kb_dir.exists() {
        crate::daemon::indexer::fts_index_kb(&kb_dir)?;
    }

    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");
    if !meta_path.exists() {
        return Err(crate::errors::bail_ctx(
            "No search index found",
            "Run `mald init` to create the index",
        ));
    }

    let joined = query.join(" ");
    let has_fzf = which_exists("fzf");

    // No query → interactive mode
    if joined.trim().is_empty() {
        if has_fzf {
            return fzf_interactive(&editor);
        } else {
            return crate::commands::tui::run_search_tui();
        }
    }

    // Query provided → search and open
    let meta = MetadataStore::open(&meta_path)?;

    // Build FTS query with prefix matching
    let fts_query: String = joined
        .split_whitespace()
        .map(|w| format!("{w}*"))
        .collect::<Vec<_>>()
        .join(" ");

    let results = meta.fts_search(&fts_query, 20)?;

    if results.is_empty() {
        return Err(crate::errors::bail_ctx(
            format!("No results for: {joined}"),
            "Try broader terms or run `mald kb list` to see available KBs",
        ));
    }

    // Single result → open directly
    if results.len() == 1 {
        let path = &results[0].path;
        std::process::Command::new(&editor).arg(path).status()?;
        return Ok(());
    }

    // Multiple results
    if has_fzf {
        fzf_select(&results, &editor)?;
    } else {
        // Fallback: numbered list
        println!("Results for '{joined}':\n");
        let show = results.len().min(15);
        for (i, r) in results.iter().take(show).enumerate() {
            let label = if r.title.is_empty() {
                &r.path
            } else {
                &r.title
            };
            println!("  [{i}] {label}");
        }
        println!();
        print!("Open [0-{}]: ", show - 1);
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let idx: usize = input.trim().parse().unwrap_or(0);
        if idx >= show {
            bail!("Invalid selection");
        }
        std::process::Command::new(&editor)
            .arg(&results[idx].path)
            .status()?;
    }

    Ok(())
}

/// Interactive fzf over all notes in all KBs.
fn fzf_interactive(editor: &str) -> Result<()> {
    let kb_dir = mald_home().join("kb");
    let files = crate::fs::find_files(&kb_dir, "md")?;

    let input: String = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let output = run_fzf(&input, editor)?;
    if let Some(path) = output {
        std::process::Command::new(editor).arg(&path).status()?;
    }
    Ok(())
}

/// Pipe search results through fzf and open the selection.
fn fzf_select(results: &[crate::index::metadata::FtsResult], editor: &str) -> Result<()> {
    // Format as "path\ttitle" for fzf, extract path after selection
    let input: String = results
        .iter()
        .map(|r| {
            let title = if r.title.is_empty() {
                std::path::Path::new(&r.path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else {
                r.title.clone()
            };
            format!("{}\t{}", r.path, title)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let output = run_fzf_with_columns(&input)?;
    if let Some(line) = output {
        let path = line.split('\t').next().unwrap_or(&line);
        std::process::Command::new(editor).arg(path).status()?;
    }
    Ok(())
}

/// Run fzf with the given input lines. Returns the selected line.
fn run_fzf(input: &str, _editor: &str) -> Result<Option<String>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let preview_cmd = if cfg!(windows) {
        "type {}"
    } else {
        "head -50 {}"
    };

    let mut child = Command::new("fzf")
        .args([
            "--preview",
            preview_cmd,
            "--preview-window",
            "right:50%:wrap",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !selection.is_empty() {
            return Ok(Some(selection));
        }
    }
    Ok(None)
}

/// Run fzf showing the second column (title) but returning the full line.
fn run_fzf_with_columns(input: &str) -> Result<Option<String>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let preview_cmd = if cfg!(windows) {
        "type {1}"
    } else {
        "head -50 {1}"
    };

    let mut child = Command::new("fzf")
        .args([
            "--delimiter",
            "\t",
            "--with-nth",
            "2",
            "--preview",
            preview_cmd,
            "--preview-window",
            "right:50%:wrap",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !selection.is_empty() {
            return Ok(Some(selection));
        }
    }
    Ok(None)
}

fn which_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}
