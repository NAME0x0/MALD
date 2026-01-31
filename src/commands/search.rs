use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;

/// Search with optional date filter. `since` can be "YYYY-MM-DD" or "Nd" (e.g. "30d").
/// If `all_kbs` is true, rebuild/ensure the index covers all KBs first.
pub async fn run(query: &str, k: usize, since: Option<&str>) -> Result<()> {
    // Ensure all KBs are indexed
    let kb_dir = mald_home().join("kb");
    if kb_dir.exists() {
        crate::daemon::indexer::fts_index_kb(&kb_dir)?;
    }

    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");

    if !meta_path.exists() {
        println!("No knowledge bases indexed. Run `mald init` first.");
        return Ok(());
    }

    let meta = MetadataStore::open(&meta_path)?;

    // Resolve since to ISO date string
    let since_date = since.map(|s| parse_since(s));

    // Try vector search if no date filter and Ollama available
    if since_date.is_none() {
        let config_path = mald_home().join("config").join("config.json");
        let config = ConfigManager::load(&config_path)?;
        let hnsw_path = index_dir.join("hnsw.bin");

        if hnsw_path.exists() {
            let client = crate::ai::ollama::OllamaClient::from_config(&config);
            if client.is_running().await {
                let embedding_model = config
                    .get_string("ai.embedding_model")
                    .unwrap_or_else(|| "nomic-embed-text".into());
                if let Ok(query_vec) = client.embeddings(&embedding_model, query).await {
                    if let Ok(index) = HnswIndex::load(&hnsw_path) {
                        let results = index.search(&query_vec, k);
                        if !results.is_empty() {
                            println!("Semantic search results for: {}\n", query);
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

    if results.is_empty() {
        println!("No results for: {}", query);
    } else {
        let label = if since_date.is_some() {
            format!("Results for '{}' (since {}):", query, since.unwrap())
        } else {
            format!("Results for '{}':", query)
        };
        println!("{}\n", label);
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
