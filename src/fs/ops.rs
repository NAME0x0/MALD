use anyhow::{Context, Result};
use std::path::Path;

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn safe_copy(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        ensure_directory(parent)?;
    }
    std::fs::copy(src, dst)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}

pub fn safe_move(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        ensure_directory(parent)?;
    }
    if std::fs::rename(src, dst).is_err() {
        safe_copy(src, dst)?;
        std::fs::remove_file(src)?;
    }
    Ok(())
}

pub fn find_files(dir: &Path, extension: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut results = Vec::new();
    if !dir.exists() {
        return Ok(results);
    }
    for entry in walkdir(dir)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|e| e == extension.trim_start_matches('.'))
        {
            results.push(entry.path().to_path_buf());
        }
    }
    Ok(results)
}

fn walkdir(
    dir: &Path,
) -> Result<Box<dyn Iterator<Item = Result<std::fs::DirEntry, std::io::Error>>>> {
    fn collect_entries(
        dir: &Path,
        entries: &mut Vec<Result<std::fs::DirEntry, std::io::Error>>,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            match entry {
                Ok(e) => {
                    if e.file_type()?.is_dir() {
                        collect_entries(&e.path(), entries)?;
                    }
                    entries.push(Ok(e));
                }
                Err(err) => entries.push(Err(err)),
            }
        }
        Ok(())
    }
    let mut entries = Vec::new();
    collect_entries(dir, &mut entries)
        .with_context(|| format!("Failed to walk directory: {}", dir.display()))?;
    Ok(Box::new(entries.into_iter()))
}

/// Move a file to ~/.mald/trash/ instead of deleting permanently.
pub fn trash(path: &Path) -> Result<()> {
    let trash_dir = mald_home().join("trash");
    ensure_directory(&trash_dir)?;
    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dest = trash_dir.join(format!("{}-{}", timestamp, filename));
    std::fs::rename(path, &dest)
        .or_else(|_| {
            std::fs::copy(path, &dest)?;
            std::fs::remove_file(path)
        })
        .with_context(|| format!("Failed to trash {}", path.display()))?;
    Ok(())
}

pub fn mald_home() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("MALD_HOME") {
        return std::path::PathBuf::from(home);
    }
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".mald")
}
