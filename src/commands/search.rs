use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;

/// Search with optional date filter. `since` can be "YYYY-MM-DD" or "Nd" (e.g. "30d").
pub async fn run(query: &str, k: usize, since: Option<&str>, json: bool) -> Result<()> {
    // Ensure all spaces are indexed
    let kb_dir = mald_home().join("kb");
    if kb_dir.exists() {
        crate::daemon::indexer::fts_index_kb(&kb_dir)?;
    }

    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");

    if !meta_path.exists() {
        if json {
            println!("[]");
        } else {
            println!("No spaces indexed yet. Run `mald init` first.");
        }
        return Ok(());
    }

    let meta = MetadataStore::open(&meta_path)?;

    // Resolve since to ISO date string
    let since_date = since.map(parse_since);

    // Try vector search if no date filter and Ollama available
    if since_date.is_none() {
        let config_path = mald_home().join("config").join("config.json");
        let config = ConfigManager::load(&config_path)?;
        let hnsw_path = index_dir.join("hnsw.bin");

        if hnsw_path.exists() {
            let typed = config.typed();
            let client = crate::ai::ollama::OllamaClient::from_config(&config);
            if client.is_running().await {
                let embedding_model = typed.ai.embedding_model.clone();
                if let Ok(query_vec) = client.embeddings(&embedding_model, query).await {
                    if let Ok(index) = HnswIndex::load(&hnsw_path) {
                        let results = index.search(&query_vec, k);
                        if !results.is_empty() {
                            if json {
                                let items: Vec<serde_json::Value> = results
                                    .iter()
                                    .filter_map(|(id, score)| {
                                        meta.get_chunk(*id).ok().flatten().map(|chunk| {
                                            serde_json::json!({
                                                "type": "semantic",
                                                "score": score,
                                                "path": chunk.doc_path,
                                                "start_line": chunk.start_line,
                                                "end_line": chunk.end_line,
                                                "snippet": chunk.content.chars().take(200).collect::<String>(),
                                            })
                                        })
                                    })
                                    .collect();
                                println!("{}", serde_json::to_string_pretty(&items)?);
                            } else {
                                println!("Semantic search results for: {query}\n");
                                for (i, (id, score)) in results.iter().enumerate() {
                                    if let Some(chunk) = meta.get_chunk(*id)? {
                                        println!(
                                            "{}. [score: {:.3}] {} (lines {}-{})",
                                            i + 1,
                                            score,
                                            chunk.doc_path,
                                            chunk.start_line,
                                            chunk.end_line,
                                        );
                                        let preview: String =
                                            chunk.content.chars().take(200).collect();
                                        println!("   {}\n", preview.replace('\n', " "));
                                    }
                                }
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    // FTS5 search with optional date filter
    let results = if let Some(ref date) = since_date {
        meta.fts_search_since(query, date, k)?
    } else {
        meta.fts_search(query, k)?
    };

    if json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "type": "fts",
                    "path": r.path,
                    "title": r.title,
                    "snippet": r.snippet,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else if results.is_empty() {
        println!("No results for: {query}");
    } else {
        let label = if since_date.is_some() {
            format!("Results for '{}' (since {}):", query, since.unwrap())
        } else {
            format!("Results for '{query}':")
        };
        println!("{label}\n");
        for (i, r) in results.iter().enumerate() {
            println!("{}. {} ({})", i + 1, r.title, r.path);
            if !r.snippet.is_empty() {
                println!("   {}\n", r.snippet.replace('\n', " "));
            }
        }
    }

    Ok(())
}

/// Parse "30d" to ISO date, or pass through "YYYY-MM-DD".
fn parse_since(s: &str) -> String {
    if s.ends_with('d') {
        if let Ok(days) = s.trim_end_matches('d').parse::<i64>() {
            let date = chrono::Local::now() - chrono::Duration::days(days);
            return date.format("%Y-%m-%d").to_string();
        }
    }
    // Assume ISO date
    s.to_string()
}

/// Interactive fuzzy search TUI.
pub fn interactive() -> Result<()> {
    crate::commands::tui::run_search_tui()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since_days() {
        let result = parse_since("7d");
        // Should be a YYYY-MM-DD string 7 days ago
        assert_eq!(result.len(), 10, "Date string should be YYYY-MM-DD format");
        assert!(result.contains('-'), "Date should contain dashes");

        // Verify it's actually ~7 days ago
        let expected = (chrono::Local::now() - chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_since_large_days() {
        let result = parse_since("30d");
        let expected = (chrono::Local::now() - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_since_iso_date_passthrough() {
        // A non-"Nd" string should be passed through as-is
        let result = parse_since("2024-01-15");
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_since_invalid_falls_through() {
        // "invalid" is not "Nd" format, so it's treated as an ISO date passthrough
        let result = parse_since("invalid");
        assert_eq!(result, "invalid");
    }

    #[test]
    fn test_parse_since_weeks_not_supported() {
        // "2w" is not "Nd", so it falls through to passthrough
        let result = parse_since("2w");
        assert_eq!(result, "2w");
    }
}
