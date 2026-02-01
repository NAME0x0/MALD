use anyhow::Result;
use std::collections::BTreeMap;

use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::parser::MarkdownDocument;

/// List all tags across all notes, with counts.
pub async fn list(kb: Option<&str>, json: bool) -> Result<()> {
    let tag_map = collect_tags(kb)?;

    if json {
        let obj: serde_json::Value = tag_map
            .iter()
            .map(|(tag, files)| {
                (
                    tag.clone(),
                    serde_json::json!({ "count": files.len(), "notes": files }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    if tag_map.is_empty() {
        println!("No tags found.");
        return Ok(());
    }

    // Sort by count descending
    let mut sorted: Vec<(&String, &Vec<String>)> = tag_map.iter().collect();
    sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (tag, files) in sorted {
        println!("  #{} ({})", tag, files.len());
    }

    Ok(())
}

/// Show all notes with a specific tag.
pub async fn filter(tag: &str, kb: Option<&str>, json: bool) -> Result<()> {
    let tag_map = collect_tags(kb)?;

    let clean_tag = tag.trim_start_matches('#');
    match tag_map.get(clean_tag) {
        Some(files) => {
            if json {
                println!("{}", serde_json::to_string_pretty(files)?);
            } else {
                println!("Notes tagged #{}:\n", clean_tag);
                for f in files {
                    println!("  {}", f);
                }
            }
        }
        None => {
            if json {
                println!("[]");
            } else {
                println!("No notes tagged #{}", clean_tag);

                // Suggest similar tags
                let suggestions: Vec<&String> = tag_map
                    .keys()
                    .filter(|t| t.contains(clean_tag) || clean_tag.contains(t.as_str()))
                    .collect();
                if !suggestions.is_empty() {
                    println!(
                        "Did you mean: {}?",
                        suggestions
                            .iter()
                            .map(|t| format!("#{}", t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }

    Ok(())
}

fn collect_tags(kb: Option<&str>) -> Result<BTreeMap<String, Vec<String>>> {
    let config_path = mald_home().join("config").join("config.json");
    let _config = ConfigManager::load(&config_path)?;

    let kb_names: Vec<String> = if let Some(k) = kb {
        vec![k.to_string()]
    } else {
        // All KBs
        let kb_dir = mald_home().join("kb");
        if !kb_dir.exists() {
            return Ok(BTreeMap::new());
        }
        std::fs::read_dir(&kb_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect()
    };

    let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for kb_name in &kb_names {
        let kb_path = mald_home().join("kb").join(kb_name);
        if !kb_path.exists() {
            continue;
        }
        let files = crate::fs::find_files(&kb_path, "md")?;
        for f in &files {
            let content = std::fs::read_to_string(f)?;
            let doc = MarkdownDocument::parse(&content);
            let display = f
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            for tag in &doc.tags {
                tag_map
                    .entry(tag.clone())
                    .or_default()
                    .push(display.clone());
            }
        }
    }

    Ok(tag_map)
}
