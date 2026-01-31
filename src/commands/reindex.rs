use anyhow::Result;

use crate::fs::mald_home;

/// Rebuild the FTS index from scratch. Useful when the index gets corrupted.
pub async fn run() -> Result<()> {
    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");

    // Delete existing index
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)?;
        println!("Removed old FTS index.");
    }

    let hnsw_path = index_dir.join("hnsw.bin");
    if hnsw_path.exists() {
        std::fs::remove_file(&hnsw_path)?;
        println!("Removed old vector index.");
    }

    // Rebuild FTS
    let kb_dir = mald_home().join("kb");
    if !kb_dir.exists() {
        println!("No knowledge bases found.");
        return Ok(());
    }

    let count = crate::daemon::indexer::fts_index_kb(&kb_dir)?;
    println!("Rebuilt FTS index: {} files indexed.", count);
    println!("\nTo rebuild vector index, run: mald ai index <kb>");

    Ok(())
}
