use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::ai::ollama::OllamaClient;
use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::chunker;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;
use crate::parser::MarkdownDocument;

use super::server::DaemonState;

pub fn content_hash(content: &str) -> String {
    // Deterministic hash using FNV-1a: stable across process restarts so we
    // don't unnecessarily re-index documents after daemon restart.
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in content.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    format!("{hash:x}")
}

/// Index all markdown files into FTS. No AI required. Fast.
pub fn fts_index_kb(kb_path: &Path) -> Result<usize> {
    let index_dir = mald_home().join("index");
    crate::fs::ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let meta = MetadataStore::open(&meta_path)?;

    let files = crate::fs::find_files(kb_path, "md")?;
    let mut indexed = 0;

    for file in &files {
        let content = std::fs::read_to_string(file)?;
        let hash = content_hash(&content);
        let path_str = file.to_string_lossy().to_string();
        let doc = MarkdownDocument::parse(&content);
        meta.index_document_fts(&path_str, &doc.title, &content, &hash)?;
        indexed += 1;
    }

    Ok(indexed)
}

/// Index a single file into FTS only. Called by watcher.
pub fn fts_index_file(path: &Path) -> Result<()> {
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }
    let index_dir = mald_home().join("index");
    crate::fs::ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let meta = MetadataStore::open(&meta_path)?;

    let content = std::fs::read_to_string(path)?;
    let hash = content_hash(&content);
    let path_str = path.to_string_lossy().to_string();
    let doc = MarkdownDocument::parse(&content);
    meta.index_document_fts(&path_str, &doc.title, &content, &hash)?;
    Ok(())
}

/// Full vector+FTS index. Requires Ollama running.
pub async fn full_index(kb_path: &Path, config: &ConfigManager) -> Result<()> {
    let index_dir = mald_home().join("index");
    crate::fs::ensure_directory(&index_dir)?;

    let hnsw_path = index_dir.join("hnsw.bin");
    let meta_path = index_dir.join("metadata.db");

    let meta = MetadataStore::open(&meta_path)?;
    let typed = config.typed();
    let client = OllamaClient::from_config(config);
    let embedding_model = typed.ai.embedding_model.clone();

    let test_vec = client.embeddings(&embedding_model, "test").await?;
    let dim = test_vec.len();

    let mut index = if hnsw_path.exists() {
        HnswIndex::load(&hnsw_path).unwrap_or_else(|_| HnswIndex::new(dim))
    } else {
        HnswIndex::new(dim)
    };

    let files = crate::fs::find_files(kb_path, "md")?;
    let mut chunk_id = index.len() as u32;

    let progress = indicatif::ProgressBar::new(files.len() as u64);
    progress.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:30.cyan/dim}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    for file in &files {
        let file_name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        progress.set_message(file_name);
        let content = std::fs::read_to_string(file)?;
        let hash = content_hash(&content);
        let path_str = file.to_string_lossy().to_string();

        if !meta.needs_reindex(&path_str, &hash)? {
            tracing::debug!("Skipping unchanged: {}", path_str);
            continue;
        }

        tracing::info!("Indexing: {}", path_str);

        let doc = MarkdownDocument::parse(&content);
        let doc_id = meta.upsert_document(&path_str, &doc.title, &content, &hash)?;
        meta.delete_doc_chunks(&path_str)?;

        let chunks = chunker::chunk_document(&content);
        for chunk in &chunks {
            let embedding = client.embeddings(&embedding_model, &chunk.content).await?;
            let id = meta.insert_chunk(
                doc_id,
                &chunk.content,
                chunk.start_line,
                chunk.end_line,
                chunk_id,
            )?;
            index.insert(id, embedding);
            chunk_id += 1;
        }
        progress.inc(1);
    }

    progress.finish_with_message("done");
    index.save(&hnsw_path)?;
    tracing::info!("Index saved: {} vectors", index.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        let input = "Hello, world! This is a test document.";
        let hash1 = content_hash(input);
        let hash2 = content_hash(input);
        assert_eq!(hash1, hash2, "Same input must produce identical hashes");
    }

    #[test]
    fn test_content_hash_different_inputs() {
        let hash1 = content_hash("document alpha");
        let hash2 = content_hash("document beta");
        assert_ne!(
            hash1, hash2,
            "Different inputs should produce different hashes"
        );
    }

    #[test]
    fn test_content_hash_empty_string() {
        let hash = content_hash("");
        assert!(
            !hash.is_empty(),
            "Empty input should still produce a valid hash"
        );
        let hash2 = content_hash("");
        assert_eq!(hash, hash2, "Empty string hash should be deterministic");
    }

    #[test]
    fn test_content_hash_is_hex() {
        let hash = content_hash("test content");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be a valid hex string, got: {}",
            hash
        );
    }

    #[test]
    fn test_fts_index_kb_with_temp_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        // Set MALD_HOME so fts_index_kb can find the index directory
        let mald_dir = dir.path().join("mald_home");
        std::fs::create_dir_all(mald_dir.join("index")).unwrap();
        std::env::set_var("MALD_HOME", &mald_dir);

        // Create a small KB directory with markdown files
        let kb_dir = dir.path().join("kb");
        std::fs::create_dir_all(&kb_dir).unwrap();
        std::fs::write(
            kb_dir.join("note1.md"),
            "# Note One\nContent of note one.\n",
        )
        .unwrap();
        std::fs::write(
            kb_dir.join("note2.md"),
            "# Note Two\nContent of note two.\n",
        )
        .unwrap();

        let count = fts_index_kb(&kb_dir).unwrap();
        assert_eq!(count, 2, "Should index 2 markdown files");

        // Cleanup env
        std::env::remove_var("MALD_HOME");
    }
}

pub async fn index_file(path: &Path, config: &ConfigManager) -> Result<()> {
    // Always do FTS
    fts_index_file(path)?;

    // Try vector index if Ollama available
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    let index_dir = mald_home().join("index");
    let hnsw_path = index_dir.join("hnsw.bin");
    let meta_path = index_dir.join("metadata.db");

    if !meta_path.exists() {
        return Ok(());
    }

    let meta = MetadataStore::open(&meta_path)?;
    let typed = config.typed();
    let client = OllamaClient::from_config(config);

    if !client.is_running().await {
        return Ok(());
    }

    let embedding_model = typed.ai.embedding_model.clone();

    let content = std::fs::read_to_string(path)?;
    let hash = content_hash(&content);
    let path_str = path.to_string_lossy().to_string();

    if !meta.needs_reindex(&path_str, &hash)? {
        return Ok(());
    }

    let doc = MarkdownDocument::parse(&content);
    let doc_id = meta.upsert_document(&path_str, &doc.title, &content, &hash)?;
    meta.delete_doc_chunks(&path_str)?;

    let test_vec = client.embeddings(&embedding_model, "test").await?;
    let dim = test_vec.len();

    let mut index = if hnsw_path.exists() {
        HnswIndex::load(&hnsw_path).unwrap_or_else(|_| HnswIndex::new(dim))
    } else {
        HnswIndex::new(dim)
    };

    let chunks = chunker::chunk_document(&content);
    for chunk in &chunks {
        let embedding = client.embeddings(&embedding_model, &chunk.content).await?;
        let id = meta.insert_chunk(
            doc_id,
            &chunk.content,
            chunk.start_line,
            chunk.end_line,
            index.len() as u32,
        )?;
        index.insert(id, embedding);
    }

    index.save(&hnsw_path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared-state variants for daemon use.
// These accept an existing DaemonState instead of opening new DB connections
// and loading/saving the HNSW index on every call.
// ---------------------------------------------------------------------------

/// FTS-index a single file using the shared MetadataStore.
/// Called from the daemon watcher loop.
pub async fn fts_index_file_shared(path: &Path, state: &DaemonState) -> Result<()> {
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let hash = content_hash(&content);
    let path_str = path.to_string_lossy().to_string();
    let doc = MarkdownDocument::parse(&content);

    let meta = state.meta.lock().await;
    meta.index_document_fts(&path_str, &doc.title, &content, &hash)?;
    Ok(())
}

/// FTS-index all markdown files in a KB directory using the shared MetadataStore.
/// Called once at daemon startup.
pub async fn fts_index_kb_shared(kb_path: &Path, state: &DaemonState) -> Result<usize> {
    let files = crate::fs::find_files(kb_path, "md")?;
    let mut indexed = 0;

    for file in &files {
        let content = std::fs::read_to_string(file)?;
        let hash = content_hash(&content);
        let path_str = file.to_string_lossy().to_string();
        let doc = MarkdownDocument::parse(&content);

        let meta = state.meta.lock().await;
        meta.index_document_fts(&path_str, &doc.title, &content, &hash)?;
        drop(meta); // Release lock between files
        indexed += 1;
    }

    Ok(indexed)
}

/// Vector+FTS index a single file using shared DaemonState.
/// Uses the in-memory HNSW index and MetadataStore instead of loading from disk.
pub async fn index_file_shared(path: &Path, state: &Arc<DaemonState>) -> Result<()> {
    // FTS first (via shared state)
    fts_index_file_shared(path, state).await?;

    // Try vector index if Ollama available
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    // Read config and extract typed values (drop guard before await points)
    let (client, embedding_model) = {
        let config = state.config.read().await;
        let typed = config.typed();
        let client = OllamaClient::new(&typed.ai.ollama_url);
        (client, typed.ai.embedding_model.clone())
    };

    if !client.is_running().await {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let hash = content_hash(&content);
    let path_str = path.to_string_lossy().to_string();

    // Check if reindex needed (hold meta lock briefly)
    {
        let meta = state.meta.lock().await;
        if !meta.needs_reindex(&path_str, &hash)? {
            return Ok(());
        }
    }

    let doc = MarkdownDocument::parse(&content);

    // Upsert document and delete old chunks (hold meta lock briefly)
    let doc_id = {
        let meta = state.meta.lock().await;
        let doc_id = meta.upsert_document(&path_str, &doc.title, &content, &hash)?;
        meta.delete_doc_chunks(&path_str)?;
        doc_id
    };

    // Get or create HNSW index with proper dimensions
    let test_vec = client.embeddings(&embedding_model, "test").await?;
    let dim = test_vec.len();

    // Ensure HNSW index exists with correct dimensions
    {
        let mut hnsw_guard = state.hnsw.write().await;
        if hnsw_guard.is_none() {
            *hnsw_guard = Some(HnswIndex::new(dim));
        }
    }

    let chunks = chunker::chunk_document(&content);
    for chunk in &chunks {
        let embedding = client.embeddings(&embedding_model, &chunk.content).await?;

        // Hold both locks briefly to insert chunk metadata and vector
        let meta = state.meta.lock().await;
        let mut hnsw_guard = state.hnsw.write().await;
        if let Some(ref mut index) = *hnsw_guard {
            let id = meta.insert_chunk(
                doc_id,
                &chunk.content,
                chunk.start_line,
                chunk.end_line,
                index.len() as u32,
            )?;
            index.insert(id, embedding);
        }
    }

    state.mark_hnsw_dirty();
    Ok(())
}
