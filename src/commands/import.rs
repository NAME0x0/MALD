use anyhow::{bail, Result};
use std::path::Path;

use crate::fs::ensure_directory;

/// Import markdown files from an external directory (Obsidian vault, Logseq graph,
/// or any folder of .md files) into a MALD space.
pub async fn run(source: &str, kb: Option<&str>, flatten: bool) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        bail!("Source path does not exist: {source}");
    }

    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;
    if !kb_path.exists() {
        // Auto-create the space
        ensure_directory(&kb_path)?;
        println!("Created space: {kb_name}");
    }

    let md_files = collect_md_files(source_path)?;
    if md_files.is_empty() {
        println!("No .md files found in {source}");
        return Ok(());
    }

    println!("Found {} markdown files", md_files.len());

    let mut imported = 0u32;
    let mut skipped = 0u32;

    for src_file in &md_files {
        let dest = if flatten {
            // Flatten: all files go directly into kb root
            let filename = src_file.file_name().unwrap().to_string_lossy().to_string();
            kb_path.join(&filename)
        } else {
            // Preserve subdirectory structure
            let relative = src_file.strip_prefix(source_path).unwrap_or(src_file);
            kb_path.join(relative)
        };

        if dest.exists() {
            skipped += 1;
            continue;
        }

        if let Some(parent) = dest.parent() {
            ensure_directory(parent)?;
        }

        // Copy the file (don't move — source stays intact)
        std::fs::copy(src_file, &dest)?;
        imported += 1;
    }

    // Also copy non-md assets (images, PDFs) from common attachment dirs
    let asset_dirs = ["attachments", "assets", "images", "media", "files"];
    let mut assets_copied = 0u32;
    for dir_name in asset_dirs {
        let asset_src = source_path.join(dir_name);
        if asset_src.exists() && asset_src.is_dir() {
            let asset_dest = kb_path.join(dir_name);
            assets_copied += copy_dir_recursive(&asset_src, &asset_dest)?;
        }
    }

    // FTS index the new files
    println!("Indexing...");
    let count = crate::daemon::indexer::fts_index_kb(&kb_path)?;

    println!("\nImport complete:");
    println!("  {imported} notes imported");
    if skipped > 0 {
        println!("  {skipped} skipped (already exist)");
    }
    if assets_copied > 0 {
        println!("  {assets_copied} asset files copied");
    }
    println!("  {count} files indexed");

    Ok(())
}

fn collect_md_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut results = Vec::new();
    if dir.is_file() {
        if dir.extension().is_some_and(|e| e == "md") {
            results.push(dir.to_path_buf());
        }
        return Ok(results);
    }
    collect_md_recursive(dir, &mut results)?;
    Ok(results)
}

fn collect_md_recursive(dir: &Path, results: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    // Skip hidden dirs and common non-content dirs
    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if dir_name.starts_with('.') || dir_name == "node_modules" || dir_name == ".trash" {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_recursive(&path, results)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            results.push(path);
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<u32> {
    let mut count = 0;
    ensure_directory(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dest.join(entry.file_name());
        if path.is_dir() {
            count += copy_dir_recursive(&path, &target)?;
        } else if !target.exists() {
            std::fs::copy(&path, &target)?;
            count += 1;
        }
    }
    Ok(count)
}
