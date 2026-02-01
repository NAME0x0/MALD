use anyhow::{bail, Result};
use chrono::Local;

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

pub async fn run(title: &str, kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found. Create it with `mald kb create {kb_name}`");
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

    println!("{}", filepath.display());

    // Run on_create hook
    super::hooks::run_hook("on_create", Some(&filepath));

    // Open in editor
    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(filepath.to_str().unwrap())
        .status()?;

    Ok(())
}

pub async fn today(kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
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

    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(filepath.to_str().unwrap())
        .status()?;

    Ok(())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}
