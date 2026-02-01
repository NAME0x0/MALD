use anyhow::Result;
use crossterm::style::Stylize;

use crate::fs::mald_home;
use crate::parser::graph::parse_knowledge_base;
use crate::parser::links::extract_wikilinks;

/// Scan notes for broken wikilinks and offer to fix them using fuzzy matching.
pub async fn run(kb: Option<&str>, auto_fix: bool) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = crate::config::ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Knowledge base '{kb_name}' not found"),
            "Run `mald kb list` to see available KBs",
        ));
    }

    let docs = parse_knowledge_base(&kb_path)?;

    // Collect all valid note stems (lowercased)
    let valid_names: Vec<String> = docs
        .iter()
        .filter_map(|d| {
            d.path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect();

    let valid_lower: Vec<String> = valid_names.iter().map(|n| n.to_lowercase()).collect();

    let mut total_broken = 0usize;
    let mut total_fixed = 0usize;

    for doc in &docs {
        let Some(path) = doc.path.as_ref() else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let wikilinks = extract_wikilinks(&content);
        let mut fixes: Vec<(String, String)> = Vec::new();

        for link in &wikilinks {
            let link_lower = link.to_lowercase();
            // Check if target exists
            if valid_lower.contains(&link_lower) {
                continue;
            }

            total_broken += 1;

            // Find closest match
            if let Some((best_idx, distance)) = closest_match(&link_lower, &valid_lower) {
                let suggestion = &valid_names[best_idx];
                let confidence = 1.0 - (distance as f64 / link.len().max(suggestion.len()) as f64);

                let note_stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                if auto_fix && confidence > 0.6 {
                    fixes.push((link.clone(), suggestion.clone()));
                    println!(
                        "  {} {} [[{}]] → [[{}]] ({})",
                        "fix".green().bold(),
                        note_stem,
                        link.as_str().red(),
                        suggestion.as_str().green(),
                        format!("{:.0}%", confidence * 100.0).dark_grey()
                    );
                    total_fixed += 1;
                } else {
                    println!(
                        "  {} {} [[{}]] → did you mean [[{}]]? ({:.0}%)",
                        "broken".red().bold(),
                        note_stem,
                        link.as_str().red(),
                        suggestion.as_str().cyan(),
                        confidence * 100.0
                    );
                }
            } else {
                let note_stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                println!(
                    "  {} {} [[{}]] — no close match found",
                    "broken".red().bold(),
                    note_stem,
                    link.as_str().red()
                );
            }
        }

        // Apply fixes
        if !fixes.is_empty() {
            let mut new_content = content.clone();
            for (old, new) in &fixes {
                new_content = new_content.replace(&format!("[[{old}]]"), &format!("[[{new}]]"));
            }
            std::fs::write(path, new_content)?;
        }
    }

    println!();
    if total_broken == 0 {
        println!("{}", "No broken wikilinks found.".green());
    } else {
        println!("{total_broken} broken link(s) found, {total_fixed} fixed.");
        if !auto_fix && total_broken > total_fixed {
            println!(
                "Run {} to auto-fix high-confidence matches.",
                "mald fix-links --fix".cyan()
            );
        }
    }

    Ok(())
}

/// Collect all note stems from a KB for wikilink autocomplete.
pub fn collect_note_names(kb_path: &std::path::Path) -> Vec<String> {
    crate::fs::find_files(kb_path, "md")
        .unwrap_or_default()
        .iter()
        .filter_map(|f| f.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect()
}

/// Collect all note stems from ALL KBs.
pub fn collect_all_note_names() -> Vec<String> {
    let kb_dir = mald_home().join("kb");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&kb_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                names.extend(collect_note_names(&entry.path()));
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Find backlinks: notes that contain [[target]] for a given target stem.
pub fn find_backlinks(target_stem: &str) -> Vec<(String, String)> {
    let kb_dir = mald_home().join("kb");
    let target_lower = target_stem.to_lowercase();
    let mut backlinks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&kb_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                let kb_name = entry.file_name().to_string_lossy().to_string();
                if let Ok(files) = crate::fs::find_files(&entry.path(), "md") {
                    for f in &files {
                        if let Ok(content) = std::fs::read_to_string(f) {
                            let links = extract_wikilinks(&content);
                            for link in &links {
                                if link.to_lowercase() == target_lower {
                                    let stem = f
                                        .file_stem()
                                        .map(|s| s.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    // Find the line containing the link
                                    let context_line = content
                                        .lines()
                                        .find(|l| l.contains(&format!("[[{link}]]")))
                                        .unwrap_or("")
                                        .trim()
                                        .to_string();
                                    backlinks.push((format!("{kb_name}/{stem}"), context_line));
                                    break; // one per file
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    backlinks
}

/// Levenshtein edit distance.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(n + 1) {
        *val = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

/// Find closest match by edit distance. Returns (index, distance).
fn closest_match(target: &str, candidates: &[String]) -> Option<(usize, usize)> {
    if candidates.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_dist = usize::MAX;
    for (i, candidate) in candidates.iter().enumerate() {
        let dist = edit_distance(target, candidate);
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    // Only return if the distance is reasonable (less than half the string length)
    let max_len = target.len().max(candidates[best_idx].len());
    if best_dist <= max_len / 2 + 1 {
        Some((best_idx, best_dist))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("same", "same"), 0);
    }

    #[test]
    fn test_closest_match() {
        let candidates = vec![
            "meeting-notes".to_string(),
            "project-plan".to_string(),
            "rust-async".to_string(),
        ];
        let (idx, dist) = closest_match("meeting-note", &candidates).unwrap();
        assert_eq!(idx, 0); // meeting-notes
        assert_eq!(dist, 1);
    }

    #[test]
    fn test_closest_match_no_good_match() {
        let candidates = vec!["abc".to_string()];
        let result = closest_match("xyzxyzxyzxyz", &candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_collect_note_names() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("note-a.md"), "# A").unwrap();
        std::fs::write(dir.path().join("note-b.md"), "# B").unwrap();
        let names = collect_note_names(dir.path());
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"note-a".to_string()));
        assert!(names.contains(&"note-b".to_string()));
    }
}
