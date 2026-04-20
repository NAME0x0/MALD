use anyhow::{bail, Result};

use crate::parser::graph;

fn resolve_kb_path(kb: Option<&str>) -> Result<std::path::PathBuf> {
    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        bail!("Space '{kb_name}' not found");
    }
    Ok(kb_path)
}

pub async fn links(note: &str, kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;

    let target_lower = note.to_lowercase();
    let doc = docs.iter().find(|d| {
        d.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_lowercase() == target_lower)
            .unwrap_or(false)
            || d.title.to_lowercase() == target_lower
    });

    match doc {
        Some(doc) => {
            let all = doc.all_links();
            if all.is_empty() {
                println!("No outgoing links from '{note}'");
            } else {
                println!("Links from '{note}':\n");
                for link in all {
                    // Check if target exists
                    let exists = docs.iter().any(|d| {
                        d.path
                            .as_ref()
                            .and_then(|p| p.file_stem())
                            .map(|s| s.to_string_lossy().to_lowercase() == link.to_lowercase())
                            .unwrap_or(false)
                    });
                    let marker = if exists { " " } else { "?" };
                    println!("  {marker} {link}");
                }
            }
        }
        None => {
            bail!("Note '{note}' not found in space");
        }
    }
    Ok(())
}

pub async fn backlinks(note: &str, kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;
    let results = graph::find_backlinks(&docs, note);

    if results.is_empty() {
        println!("No backlinks to '{note}'");
    } else {
        println!("Backlinks to '{note}':\n");
        for doc in results {
            let name = doc
                .path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| doc.title.clone());
            let path = doc
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            println!("  {name} ({path})");
        }
    }
    Ok(())
}

pub async fn orphans(kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;
    let orphaned = graph::find_orphaned_files(&docs);

    if orphaned.is_empty() {
        println!("No orphaned files.");
    } else {
        println!("Orphaned files (no incoming links):\n");
        for doc in orphaned {
            let name = doc
                .path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| doc.title.clone());
            let path = doc
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            println!("  {name} ({path})");
        }
    }
    Ok(())
}

/// Show broken wikilinks (links to non-existent notes).
pub async fn broken_links(kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;
    let broken = crate::commands::review::find_broken_links(&docs);

    if broken.is_empty() {
        println!("No broken links.");
    } else {
        println!("Broken links ({}):\n", broken.len());
        for (source, target) in &broken {
            println!("  {source} -> [[{target}]]");
        }
    }
    Ok(())
}

/// Output a Mermaid graph of the KB link structure.
pub async fn view(kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;

    println!("```mermaid");
    println!("graph LR");

    // Collect unique node IDs
    let mut seen = std::collections::HashSet::new();
    for doc in &docs {
        let name = doc
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| doc.title.clone());
        let id = mermaid_id(&name);
        if seen.insert(id.clone()) {
            println!("    {id}[{name}]");
        }
        for link in doc.all_links() {
            let link_id = mermaid_id(link);
            // Check if target exists
            let exists = docs.iter().any(|d| {
                d.path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().to_lowercase() == link.to_lowercase())
                    .unwrap_or(false)
            });
            if !exists && seen.insert(link_id.clone()) {
                println!("    {link_id}[{link}]:::broken");
            }
            println!("    {id} --> {link_id}");
        }
    }
    println!("    classDef broken fill:#f99,stroke:#f00");
    println!("```");
    println!("\nPaste into any Mermaid renderer (GitHub, Obsidian, mermaid.live).");
    Ok(())
}

/// Stats about the knowledge graph.
pub async fn stats(kb: Option<&str>) -> Result<()> {
    let kb_path = resolve_kb_path(kb)?;
    let docs = graph::parse_knowledge_base(&kb_path)?;

    let total_notes = docs.len();
    let total_links: usize = docs.iter().map(|d| d.all_links().len()).sum();
    let orphans = graph::find_orphaned_files(&docs).len();
    let broken = crate::commands::review::find_broken_links(&docs).len();

    let mut tag_count = std::collections::HashMap::new();
    for doc in &docs {
        for tag in &doc.tags {
            *tag_count.entry(tag.clone()).or_insert(0usize) += 1;
        }
    }

    println!("Graph stats:\n");
    println!("  Notes:        {total_notes}");
    println!("  Links:        {total_links}");
    println!("  Unique tags:  {}", tag_count.len());
    println!("  Orphans:      {orphans}");
    println!("  Broken links: {broken}");

    if total_notes > 0 {
        let avg_links = total_links as f64 / total_notes as f64;
        println!("  Avg links:    {avg_links:.1}");
    }

    Ok(())
}

fn mermaid_id(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
