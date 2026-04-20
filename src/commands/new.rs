use anyhow::{bail, Result};
use chrono::Local;
use std::path::{Component, Path, PathBuf};

use crate::fs::{ensure_directory, mald_home, slugify};

fn create_note_inner(title: &str, kb: Option<&str>, path: Option<&str>) -> Result<PathBuf> {
    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Space `{kb_name}` not found."),
            format!("Run `mald kb list` to inspect your workspace, or `mald kb create {kb_name}` to create it."),
        ));
    }
    if title.trim().is_empty() {
        bail!("Note title cannot be empty");
    }

    let now = Local::now();
    let slug = slugify(title);
    let filename = format!("{}-{}.md", now.format("%Y%m%d"), slug);
    let note_dir = resolve_note_dir(&kb_path, path)?;
    ensure_directory(&note_dir)?;
    let filepath = note_dir.join(&filename);

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

pub fn create_note_sync(title: &str, kb: Option<&str>, path: Option<&str>) -> Result<PathBuf> {
    create_note_inner(title, kb, path)
}

pub async fn create_note(title: &str, kb: Option<&str>, path: Option<&str>) -> Result<PathBuf> {
    create_note_inner(title, kb, path)
}

pub async fn run(title: &str, kb: Option<&str>, path: Option<&str>) -> Result<()> {
    let (_config, typed, _kb_name, _kb_path) = crate::config::resolve_kb(kb)?;
    let filepath = create_note(title, kb, path).await?;

    println!("{}", filepath.display());

    // Open in editor
    let editor = typed.editor.clone();
    crate::commands::launch::open_in_editor(&editor, filepath.as_os_str())?;

    Ok(())
}

pub(crate) fn resolve_note_dir(kb_path: &Path, path: Option<&str>) -> Result<PathBuf> {
    let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(kb_path.to_path_buf());
    };

    let requested = Path::new(path);
    if requested.is_absolute() {
        return Err(crate::errors::bail_ctx(
            "Note path must be relative to the active space.",
            "Try `mald new \"Title\" --path projects/api` instead of using an absolute path.",
        ));
    }
    if requested.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(crate::errors::bail_ctx(
            "Note path cannot escape the active space.",
            "Use a subdirectory like `--path inbox` or `--path projects/api`.",
        ));
    }

    Ok(kb_path.join(requested))
}

pub async fn today(kb: Option<&str>) -> Result<()> {
    let (_config, typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Space `{kb_name}` not found."),
            format!("Run `mald kb list` to inspect your workspace, or `mald kb create {kb_name}` to create it."),
        ));
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
    crate::commands::launch::open_in_editor(&editor, filepath.as_os_str())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_note_dir_defaults_to_kb_root() {
        let kb_path = Path::new("kb").join("personal");
        assert_eq!(resolve_note_dir(&kb_path, None).unwrap(), kb_path);
    }

    #[test]
    fn resolve_note_dir_appends_relative_subdirectory() {
        let kb_path = Path::new("kb").join("personal");
        assert_eq!(
            resolve_note_dir(&kb_path, Some("projects/api")).unwrap(),
            kb_path.join("projects").join("api")
        );
    }

    #[test]
    fn resolve_note_dir_rejects_absolute_paths() {
        let kb_path = Path::new("kb").join("personal");
        let absolute = if cfg!(windows) { r"C:\temp" } else { "/tmp" };
        assert!(resolve_note_dir(&kb_path, Some(absolute)).is_err());
    }

    #[test]
    fn resolve_note_dir_rejects_parent_traversal() {
        let kb_path = Path::new("kb").join("personal");
        assert!(resolve_note_dir(&kb_path, Some("../escape")).is_err());
    }
}
