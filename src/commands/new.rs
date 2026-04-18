use anyhow::{bail, Result};
use chrono::Local;
use std::path::PathBuf;

use crate::fs::{ensure_directory, mald_home, slugify};

pub async fn create_note(title: &str, kb: Option<&str>) -> Result<PathBuf> {
    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found. Create it with `mald kb create {kb_name}`");
    }
    if title.trim().is_empty() {
        bail!("Note title cannot be empty");
    }

    let now = Local::now();
    let slug = slugify(title);
    let filename = format!("{}-{}.md", now.format("%Y%m%d"), slug);
    let filepath = kb_path.join(&filename);

    if filepath.exists() {
        bail!("File already exists: {}", filepath.display());
    }

    let content = format!(
        "---\ntitle: {}\ncreated: {}\ntags: []\n---\n\n# {}\n\n",
        title,
        now.format("%Y-%m-%d %H:%M"),
        title,
    );

    std::fs::write(&filepath, &content)?;

    // Index into FTS immediately
    let hash = crate::daemon::indexer::content_hash(&content);
    let index_dir = mald_home().join("index");
    ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let meta = crate::index::metadata::MetadataStore::open(&meta_path)?;
    meta.index_document_fts(&filepath.to_string_lossy(), title, &content, &hash)?;

    super::hooks::run_hook("on_create", Some(&filepath));

    Ok(filepath)
}

pub async fn run(title: &str, kb: Option<&str>) -> Result<()> {
    let (_config, typed, _kb_name, _kb_path) = crate::config::resolve_kb(kb)?;
    let filepath = create_note(title, kb).await?;

    println!("{}", filepath.display());

    // Open in editor
    let editor = typed.editor.clone();
    std::process::Command::new(&editor)
        .arg(filepath.to_str().unwrap())
        .status()?;

    Ok(())
}

pub async fn today(kb: Option<&str>) -> Result<()> {
    let (_config, typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found. Create it with `mald kb create {kb_name}`");
    }

    let now = Local::now();
    let filename = format!("{}.md", now.format("%Y-%m-%d"));
    let filepath = kb_path.join(&filename);

    if !filepath.exists() {
        let content = format!(
            "---\ntitle: {}\ncreated: {}\ntags: [daily]\n---\n\n# {}\n\n## Tasks\n\n- [ ] \n\n## Notes\n\n",
            now.format("%A, %B %d, %Y"),
            now.format("%Y-%m-%d %H:%M"),
            now.format("%A, %B %d, %Y"),
        );
        std::fs::write(&filepath, &content)?;

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
    }

    println!("{}", filepath.display());

    let editor = typed.editor.clone();
    std::process::Command::new(&editor)
        .arg(filepath.to_str().unwrap())
        .status()?;

    Ok(())
}
