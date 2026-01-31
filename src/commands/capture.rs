use anyhow::{bail, Result};
use chrono::Local;

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

/// Append a thought to today's daily note. Creates the note if it doesn't exist.
/// This is the fastest path from thought to disk — no editor, no friction.
pub async fn run(text: &str, kb: Option<&str>, tag: Option<&str>) -> Result<()> {
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

    let now = Local::now();
    let filename = format!("{}.md", now.format("%Y-%m-%d"));
    let filepath = kb_path.join(&filename);

    // Create daily note if it doesn't exist
    if !filepath.exists() {
        let content = format!(
            "---\ntitle: {}\ncreated: {}\ntags: [daily]\n---\n\n# {}\n\n## Captures\n\n",
            now.format("%A, %B %d, %Y"),
            now.format("%Y-%m-%d %H:%M"),
            now.format("%A, %B %d, %Y"),
        );
        std::fs::write(&filepath, &content)?;
    }

    // Append the capture
    let timestamp = now.format("%H:%M").to_string();
    let entry = if let Some(t) = tag {
        format!("- `{}` #{} {}\n", timestamp, t, text)
    } else {
        format!("- `{}` {}\n", timestamp, text)
    };

    let mut content = std::fs::read_to_string(&filepath)?;
    content.push_str(&entry);
    std::fs::write(&filepath, &content)?;

    // FTS re-index
    let hash = crate::daemon::indexer::content_hash(&content);
    let index_dir = mald_home().join("index");
    ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let meta = crate::index::metadata::MetadataStore::open(&meta_path)?;
    meta.index_document_fts(
        &filepath.to_string_lossy(),
        &now.format("%A, %B %d, %Y").to_string(),
        &content,
        &hash,
    )?;

    println!("Captured to {}", filepath.display());
    Ok(())
}
