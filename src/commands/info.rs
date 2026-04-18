use anyhow::Result;
use chrono::{DateTime, Local};

use crate::fs::mald_home;
use crate::parser::MarkdownDocument;

/// Show comprehensive metadata about a note.
pub async fn run(note: &str, kb: Option<&str>) -> Result<()> {
    let path = super::run::resolve_note_pub(note, kb)?;
    let content = std::fs::read_to_string(&path)?;
    let doc = MarkdownDocument::parse(&content);
    let meta = std::fs::metadata(&path)?;

    let word_count = content
        .split_whitespace()
        .filter(|w| !w.starts_with("---") && !w.starts_with('#'))
        .count();
    let reading_time = (word_count as f64 / 200.0).ceil() as usize; // 200 wpm
    let line_count = content.lines().count();

    println!("Note: {}", path.display());
    println!();

    // Title
    if !doc.title.is_empty() {
        println!("  Title:        {}", doc.title);
    }

    // Dates
    if let Ok(created) = meta.created() {
        let dt: DateTime<Local> = created.into();
        println!("  Created:      {}", dt.format("%Y-%m-%d %H:%M"));
    }
    if let Ok(modified) = meta.modified() {
        let dt: DateTime<Local> = modified.into();
        println!("  Modified:     {}", dt.format("%Y-%m-%d %H:%M"));
    }

    // Size
    println!("  Words:        {word_count}");
    println!("  Lines:        {line_count}");
    println!("  Reading time: {} min", reading_time.max(1));

    // Tags
    if !doc.tags.is_empty() {
        println!(
            "  Tags:         {}",
            doc.tags
                .iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    // Links
    let wikilinks = &doc.wikilinks;
    let md_links = &doc.markdown_links;
    if !wikilinks.is_empty() || !md_links.is_empty() {
        println!(
            "  Outgoing:     {} wikilinks, {} markdown links",
            wikilinks.len(),
            md_links.len()
        );
    }

    // Backlinks
    let (_config, _typed, _kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let files = crate::fs::find_files(&kb_path, "md")?;
    let mut backlink_count = 0;
    for f in &files {
        if f == &path {
            continue;
        }
        if let Ok(c) = std::fs::read_to_string(f) {
            let other = MarkdownDocument::parse(&c);
            if other.wikilinks.iter().any(|l| l.to_lowercase() == stem) {
                backlink_count += 1;
            }
        }
    }
    println!("  Backlinks:    {backlink_count}");

    // Code blocks
    if !doc.code_blocks.is_empty() {
        let langs: Vec<&str> = doc
            .code_blocks
            .iter()
            .map(|b| {
                if b.language.is_empty() {
                    "text"
                } else {
                    b.language.as_str()
                }
            })
            .collect();
        println!(
            "  Code blocks:  {} ({})",
            doc.code_blocks.len(),
            langs.join(", ")
        );
    }

    // Index status
    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");
    if meta_path.exists() {
        let db = crate::index::metadata::MetadataStore::open(&meta_path)?;
        let path_str = path.to_string_lossy().to_string();
        let hash = crate::daemon::indexer::content_hash(&content);
        let indexed = !db.needs_reindex(&path_str, &hash)?;
        println!(
            "  Indexed:      {}",
            if indexed { "yes" } else { "no (stale)" }
        );
    }

    Ok(())
}
