use anyhow::{bail, Result};

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Fuzzy-find a note and open it in the editor.
/// If the query matches exactly one note, opens it directly.
/// If multiple matches, shows a selection list.
pub async fn run(query: &str, kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        bail!("Knowledge base '{}' not found", kb_name);
    }

    let files = crate::fs::find_files(&kb_path, "md")?;
    let query_lower = query.to_lowercase();

    // Score matches: exact stem match > starts with > contains
    let mut matches: Vec<(u32, &std::path::PathBuf)> = files
        .iter()
        .filter_map(|f| {
            let stem = f.file_stem()?.to_string_lossy().to_lowercase();
            if stem == query_lower {
                Some((0, f)) // exact match
            } else if stem.starts_with(&query_lower) {
                Some((1, f)) // prefix match
            } else if stem.contains(&query_lower) {
                Some((2, f)) // substring match
            } else {
                // Fuzzy: check if all query chars appear in order
                let mut chars = query_lower.chars().peekable();
                for c in stem.chars() {
                    if chars.peek() == Some(&c) {
                        chars.next();
                    }
                }
                if chars.peek().is_none() {
                    Some((3, f)) // fuzzy match
                } else {
                    None
                }
            }
        })
        .collect();

    matches.sort_by_key(|(score, _)| *score);

    if matches.is_empty() {
        bail!("No notes matching '{}' in kb '{}'", query, kb_name);
    }

    let path = if matches.len() == 1 || matches[0].0 == 0 {
        // Single match or exact match — open directly
        matches[0].1
    } else {
        // Multiple matches — show list and let user pick
        println!("Multiple matches for '{}':\n", query);
        let show = matches.len().min(15);
        for (i, (_, f)) in matches.iter().take(show).enumerate() {
            let stem = f.file_stem().unwrap().to_string_lossy();
            println!("  [{}] {}", i, stem);
        }
        if matches.len() > show {
            println!("  ... and {} more", matches.len() - show);
        }
        println!();

        // Read selection
        print!("Open [0-{}]: ", show - 1);
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let idx: usize = input.trim().parse().unwrap_or(0);
        if idx >= show {
            bail!("Invalid selection");
        }
        matches[idx].1
    };

    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()?;

    Ok(())
}
