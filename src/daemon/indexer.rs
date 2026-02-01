use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::ai::ollama::OllamaClient;
use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::chunker;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;
use crate::parser::MarkdownDocument;

pub fn content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
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
    let client = OllamaClient::from_config(config);
    let embedding_model = config
        .get_string("ai.embedding_model")
        .unwrap_or_else(|| "nomic-embed-text".into());

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
    let client = OllamaClient::from_config(config);

    if !client.is_running().await {
        return Ok(());
    }

    let embedding_model = config
        .get_string("ai.embedding_model")
        .unwrap_or_else(|| "nomic-embed-text".into());

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
