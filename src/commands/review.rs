use anyhow::Result;
use chrono::Local;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Weekly review: surfaces modified notes, stale notes, orphans, open tasks.
pub async fn run(kb: Option<&str>, days: u64) -> Result<()> {
    let home = mald_home();
    let config_path = home.join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());
    let kb_path = home.join("kb").join(&kb_name);

    if !kb_path.exists() {
        println!("Knowledge base '{}' not found.", kb_name);
        return Ok(());
    }

    let files = crate::fs::find_files(&kb_path, "md")?;
    let now_ts = std::time::SystemTime::now();
    let cutoff_secs = days * 24 * 3600;
    let stale_cutoff_secs = 90 * 24 * 3600;

    println!(
        "Review for '{}' — {}\n",
        kb_name,
        Local::now().format("%Y-%m-%d")
    );

    // Modified recently
    let mut recent: Vec<(String, u64)> = Vec::new();
    let mut stale: Vec<(String, u64)> = Vec::new();
    let mut open_tasks: Vec<(String, String)> = Vec::new();
    let mut total_words = 0usize;

    for f in &files {
        let name = f
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let age_secs = f
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| now_ts.duration_since(t).ok())
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);

        if age_secs < cutoff_secs {
            recent.push((name.clone(), age_secs));
        }
        if age_secs > stale_cutoff_secs {
            stale.push((name.clone(), age_secs / (24 * 3600)));
        }

        if let Ok(content) = std::fs::read_to_string(f) {
            total_words += content.split_whitespace().count();
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                    let task = rest.trim().to_string();
                    if !task.is_empty() {
                        open_tasks.push((name.clone(), task));
                    }
                }
            }
        }
    }

    recent.sort_by_key(|(_n, age)| *age);

    // Stats
    println!("Stats:");
    println!("  {} notes, ~{} words\n", files.len(), total_words);

    // Recent activity
    if recent.is_empty() {
        println!("No activity in the last {} days.\n", days);
    } else {
        println!("Modified (last {} days):", days);
        for (name, age) in &recent {
            let d = age / (24 * 3600);
            let label = if d == 0 {
                "today".to_string()
            } else {
                format!("{}d ago", d)
            };
            println!("  {} ({})", name, label);
        }
        println!();
    }

    // Open tasks
    if !open_tasks.is_empty() {
        println!("Open tasks ({}):", open_tasks.len());
        for (note, task) in open_tasks.iter().take(10) {
            println!("  [ ] {} ({})", task, note);
        }
        if open_tasks.len() > 10 {
            println!("  ... {} more", open_tasks.len() - 10);
        }
        println!();
    }

    // Stale notes
    if !stale.is_empty() {
        println!("Stale notes (>90 days untouched):");
        for (name, age_days) in stale.iter().take(10) {
            println!("  {} ({}d)", name, age_days);
        }
        if stale.len() > 10 {
            println!("  ... {} more", stale.len() - 10);
        }
        println!();
    }

    // Orphans
    let docs = crate::parser::graph::parse_knowledge_base(&kb_path)?;
    let orphans = crate::parser::graph::find_orphaned_files(&docs);
    if !orphans.is_empty() {
        println!("Orphaned notes ({}):", orphans.len());
        for doc in orphans.iter().take(10) {
            let name = doc
                .path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| doc.title.clone());
            println!("  {}", name);
        }
        if orphans.len() > 10 {
            println!("  ... {} more", orphans.len() - 10);
        }
        println!();
    }

    // Broken links
    let broken = find_broken_links(&docs);
    if !broken.is_empty() {
        println!("Broken links ({}):", broken.len());
        for (source, target) in broken.iter().take(10) {
            println!("  {} -> [[{}]]", source, target);
        }
        if broken.len() > 10 {
            println!("  ... {} more", broken.len() - 10);
        }
        println!();
    }

    Ok(())
}

/// Find all wikilinks that point to non-existent notes.
pub fn find_broken_links(
    docs: &[crate::parser::markdown::MarkdownDocument],
) -> Vec<(String, String)> {
    let names: std::collections::HashSet<String> = docs
        .iter()
        .filter_map(|d| {
            d.path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
        })
        .collect();

    let mut broken = Vec::new();
    for doc in docs {
        let source = doc
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| doc.title.clone());
        for link in &doc.wikilinks {
            if !names.contains(&link.to_lowercase()) {
                broken.push((source.clone(), link.clone()));
            }
        }
    }
    broken
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::markdown::MarkdownDocument;

    #[test]
    fn test_find_broken_links_detects_missing() {
        let mut doc1 = MarkdownDocument::parse("Link to [[nonexistent]]");
        doc1.path = Some(std::path::PathBuf::from("/kb/note1.md"));

        let mut doc2 = MarkdownDocument::parse("Link to [[note1]]");
        doc2.path = Some(std::path::PathBuf::from("/kb/note2.md"));

        let docs = vec![doc1, doc2];
        let broken = find_broken_links(&docs);
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].1, "nonexistent");
    }

    #[test]
    fn test_find_broken_links_no_broken() {
        let mut doc1 = MarkdownDocument::parse("Link to [[note2]]");
        doc1.path = Some(std::path::PathBuf::from("/kb/note1.md"));

        let mut doc2 = MarkdownDocument::parse("Link to [[note1]]");
        doc2.path = Some(std::path::PathBuf::from("/kb/note2.md"));

        let docs = vec![doc1, doc2];
        let broken = find_broken_links(&docs);
        assert!(broken.is_empty());
    }
}
