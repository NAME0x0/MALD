use anyhow::{bail, Result};

use crate::fs::{mald_home, slugify};

/// Rename a note and update all wikilink references across the KB.
pub async fn run(old_name: &str, new_name: &str, kb: Option<&str>) -> Result<()> {
    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;

    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found");
    }

    let files = crate::fs::find_files(&kb_path, "md")?;

    // Find the source file
    let old_lower = old_name.to_lowercase();
    let source_file = files.iter().find(|f| {
        f.file_stem()
            .map(|s| s.to_string_lossy().to_lowercase() == old_lower)
            .unwrap_or(false)
    });

    let source_file = match source_file {
        Some(f) => f.clone(),
        None => bail!("Note '{old_name}' not found in KB '{kb_name}'"),
    };

    // Build the new filename, preserving date prefix if present
    let old_stem = source_file
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let new_stem = if old_stem.len() > 9
        && old_stem[..8].chars().all(|c| c.is_ascii_digit())
        && old_stem.as_bytes()[8] == b'-'
    {
        // Has YYYYMMDD- prefix, preserve it
        let prefix = &old_stem[..9];
        format!("{}{}", prefix, slugify(new_name))
    } else {
        slugify(new_name)
    };

    let new_file = source_file.with_file_name(format!("{new_stem}.md"));

    if new_file.exists() {
        bail!("Target file already exists: {}", new_file.display());
    }

    // Phase 1: Collect all changes in memory (atomic preparation)
    let old_link_lower = old_lower.clone();
    let mut file_updates: Vec<(std::path::PathBuf, String)> = Vec::new();

    // Prepare renamed file content
    let source_content = std::fs::read_to_string(&source_file)?;
    let renamed_content = update_title_in_frontmatter(&source_content, new_name);
    file_updates.push((new_file.clone(), renamed_content));

    // Prepare wikilink updates in other files
    for f in &files {
        if *f == source_file {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(f) {
            let new_content = update_wikilinks(&content, &old_link_lower, &new_stem);
            if new_content != content {
                file_updates.push((f.clone(), new_content));
            }
        }
    }

    let updated_count = file_updates.len() - 1; // -1 for the renamed file itself

    // Phase 2: Write all changes (write to .tmp first, then rename for atomicity)
    let mut temp_files: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    for (target, content) in &file_updates {
        let tmp = target.with_extension("md.mald-tmp");
        std::fs::write(&tmp, content)?;
        temp_files.push((tmp, target.clone()));
    }

    // Phase 3: Atomic commit — rename all temp files to final destinations
    // Remove old source file first
    std::fs::remove_file(&source_file)?;
    for (tmp, target) in &temp_files {
        if let Err(e) = std::fs::rename(tmp, target) {
            // Fallback: copy + delete (cross-device rename)
            std::fs::copy(tmp, target).map_err(|_| {
                anyhow::anyhow!("Failed to commit rename for {}: {}", target.display(), e)
            })?;
            let _ = std::fs::remove_file(tmp);
        }
    }

    // Update FTS index
    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");
    if meta_path.exists() {
        if let Ok(meta) = crate::index::metadata::MetadataStore::open(&meta_path) {
            // Remove old entry, re-index new
            let _ = meta.delete_doc_chunks(&source_file.to_string_lossy());
            if let Ok(content) = std::fs::read_to_string(&new_file) {
                let hash = crate::daemon::indexer::content_hash(&content);
                let _ =
                    meta.index_document_fts(&new_file.to_string_lossy(), new_name, &content, &hash);
            }
        }
    }

    println!("Renamed: {old_stem} -> {new_stem}");
    if updated_count > 0 {
        println!("Updated {updated_count} file(s) with new wikilink references.");
    }

    Ok(())
}

/// Replace `[[old_name]]` with `[[new_name]]` in content (case-insensitive matching).
fn update_wikilinks(content: &str, old_lower: &str, new_name: &str) -> String {
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let inner = &caps[1];
        if inner.to_lowercase() == *old_lower {
            format!("[[{new_name}]]")
        } else {
            caps[0].to_string()
        }
    })
    .to_string()
}

fn update_title_in_frontmatter(content: &str, new_title: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    if let Some(end) = trimmed[3..].find("\n---") {
        let yaml = &trimmed[3..3 + end];
        let mut new_yaml = String::new();
        for line in yaml.lines() {
            if line.trim().starts_with("title:") {
                new_yaml.push_str(&format!("title: {new_title}"));
            } else {
                new_yaml.push_str(line);
            }
            new_yaml.push('\n');
        }
        let rest = &trimmed[3 + end + 4..];
        format!("---\n{new_yaml}---{rest}")
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_wikilinks() {
        let content = "See [[Old Note]] and [[other]] for details.";
        let result = update_wikilinks(content, "old note", "new-note");
        assert_eq!(result, "See [[new-note]] and [[other]] for details.");
    }

    #[test]
    fn test_update_wikilinks_preserves_others() {
        let content = "[[keep-this]] and [[old note]] and [[also-keep]]";
        let result = update_wikilinks(content, "old note", "new-note");
        assert_eq!(result, "[[keep-this]] and [[new-note]] and [[also-keep]]");
    }

    #[test]
    fn test_update_title_in_frontmatter() {
        let content = "---\ntitle: Old Title\ntags: []\n---\n\n# Content";
        let result = update_title_in_frontmatter(content, "New Title");
        assert!(result.contains("title: New Title"));
        assert!(result.contains("# Content"));
    }

    #[test]
    fn test_update_title_no_frontmatter() {
        let content = "# Just Content";
        let result = update_title_in_frontmatter(content, "New Title");
        assert_eq!(result, content);
    }

    #[test]
    fn test_update_wikilinks_case_insensitive() {
        let content = "Link: [[OLD NOTE]]";
        let result = update_wikilinks(content, "old note", "new-note");
        assert_eq!(result, "Link: [[new-note]]");
    }
}
