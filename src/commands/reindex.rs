use anyhow::Result;

use crate::fs::mald_home;

/// Rebuild the searchable indices from scratch.
pub async fn rebuild() -> Result<usize> {
    let index_dir = mald_home().join("index");
    let meta_path = index_dir.join("metadata.db");

    // Delete existing index
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)?;
    }

    let hnsw_path = index_dir.join("hnsw.bin");
    if hnsw_path.exists() {
        std::fs::remove_file(&hnsw_path)?;
    }

    // Rebuild FTS
    let kb_dir = mald_home().join("kb");
    if !kb_dir.exists() {
        return Ok(0);
    }

    let count = crate::daemon::indexer::fts_index_kb(&kb_dir)?;
    Ok(count)
}

/// Rebuild the FTS index from scratch. Useful when the index gets corrupted.
pub async fn run() -> Result<()> {
    let count = rebuild().await?;
    println!("Rebuilt FTS index: {count} files indexed.");
    println!("\nTo rebuild vector index, run: mald ai index <kb>");

    Ok(())
}
