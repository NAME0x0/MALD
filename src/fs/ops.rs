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

/// Streaming directory walker using stack-based iteration.
/// Processes directories lazily instead of collecting all entries upfront.
struct WalkDir {
    stack: Vec<std::fs::ReadDir>,
}

impl WalkDir {
    fn new(dir: &Path) -> std::io::Result<Self> {
        let read_dir = std::fs::read_dir(dir)?;
        Ok(Self {
            stack: vec![read_dir],
        })
    }
}

impl Iterator for WalkDir {
    type Item = Result<std::fs::DirEntry, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(read_dir) = self.stack.last_mut() {
            match read_dir.next() {
                Some(Ok(entry)) => {
                    // If it's a directory, push it onto the stack for later processing
                    if let Ok(ft) = entry.file_type() {
                        if ft.is_dir() {
                            if let Ok(sub_read_dir) = std::fs::read_dir(entry.path()) {
                                self.stack.push(sub_read_dir);
                            }
                        }
                    }
                    return Some(Ok(entry));
                }
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    // Current directory exhausted, pop and continue with parent
                    self.stack.pop();
                }
            }
        }
        None
    }
}

fn walkdir(
    dir: &Path,
) -> Result<Box<dyn Iterator<Item = Result<std::fs::DirEntry, std::io::Error>>>> {
    let walker = WalkDir::new(dir)
        .with_context(|| format!("Failed to walk directory: {}", dir.display()))?;
    Ok(Box::new(walker))
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
    let dest = trash_dir.join(format!("{timestamp}-{filename}"));
    std::fs::rename(path, &dest)
        .or_else(|_| {
            std::fs::copy(path, &dest)?;
            std::fs::remove_file(path)
        })
        .with_context(|| format!("Failed to trash {}", path.display()))?;
    Ok(())
}

/// Convert a string to a URL/filename-safe slug.
/// Lowercases, replaces non-alphanumeric chars with hyphens, collapses runs of hyphens.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Returns the MALD home directory, panics if home cannot be determined.
/// Prefer `try_mald_home()` in contexts where graceful error handling is needed.
pub fn mald_home() -> std::path::PathBuf {
    try_mald_home().expect("Could not determine home directory. Set MALD_HOME env var.")
}

/// Fallible version of mald_home() for contexts requiring graceful error handling.
pub fn try_mald_home() -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var("MALD_HOME") {
        return Some(std::path::PathBuf::from(home));
    }
    dirs::home_dir().map(|h| h.join(".mald"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ---- ensure_directory tests ----

    #[test]
    fn test_ensure_directory_creates_nested() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        assert!(!nested.exists());

        ensure_directory(&nested).unwrap();
        assert!(nested.exists());
        assert!(nested.is_dir());
    }

    #[test]
    fn test_ensure_directory_idempotent() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("existing");
        std::fs::create_dir(&target).unwrap();

        // Should not error when dir already exists
        ensure_directory(&target).unwrap();
        assert!(target.is_dir());
    }

    // ---- find_files tests ----

    #[test]
    fn test_find_files_by_extension() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "markdown").unwrap();
        std::fs::write(dir.path().join("b.md"), "markdown2").unwrap();
        std::fs::write(dir.path().join("c.txt"), "text").unwrap();

        let md_files = find_files(dir.path(), "md").unwrap();
        assert_eq!(md_files.len(), 2);
        for f in &md_files {
            assert_eq!(f.extension().unwrap(), "md");
        }

        let txt_files = find_files(dir.path(), "txt").unwrap();
        assert_eq!(txt_files.len(), 1);
    }

    #[test]
    fn test_find_files_with_dot_prefix_extension() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.md"), "content").unwrap();

        // Extension with leading dot should also work
        let files = find_files(dir.path(), ".md").unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_find_files_nonexistent_dir() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("no_such_dir");
        let files = find_files(&missing, "md").unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_files_nested_directories() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("top.md"), "top").unwrap();
        std::fs::write(sub.join("nested.md"), "nested").unwrap();

        let files = find_files(dir.path(), "md").unwrap();
        assert_eq!(files.len(), 2);
    }

    // ---- safe_move tests ----

    #[test]
    fn test_safe_move_moves_file() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");

        std::fs::write(&src, "hello move").unwrap();
        safe_move(&src, &dst).unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "hello move");
    }

    #[test]
    fn test_safe_move_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("new_dir").join("sub").join("dest.txt");

        std::fs::write(&src, "content").unwrap();
        safe_move(&src, &dst).unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "content");
    }

    // ---- slugify tests ----

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Hello, World! #2024"), "hello-world-2024");
    }

    #[test]
    fn test_slugify_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_slugify_unicode() {
        // Unicode alphanumeric chars are kept, non-alphanumeric replaced
        let result = slugify("cafe\u{0301} au lait");
        // The combining accent is not alphanumeric, so it becomes a separator
        assert!(!result.contains(' '));
        assert!(result.starts_with("cafe"));
    }

    #[test]
    fn test_slugify_consecutive_special() {
        // Multiple consecutive non-alnum chars should collapse to single hyphen
        assert_eq!(slugify("a---b___c   d"), "a-b-c-d");
    }

    #[test]
    fn test_slugify_very_long_string() {
        let long = "a".repeat(1000);
        let result = slugify(&long);
        assert_eq!(result.len(), 1000);
        assert_eq!(result, long);
    }

    // ---- mald_home tests ----

    #[test]
    fn test_mald_home_returns_path() {
        // mald_home() should not panic — it either reads MALD_HOME env or home dir
        let path = mald_home();
        assert!(!path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_try_mald_home_with_env_var() {
        // Save/restore is not perfectly safe in parallel tests,
        // but try_mald_home is a pure env check so it's fine for a unit test.
        let dir = TempDir::new().unwrap();
        std::env::set_var("MALD_HOME", dir.path());
        let home = try_mald_home().unwrap();
        assert_eq!(home, dir.path());
        std::env::remove_var("MALD_HOME");
    }

    // ---- WalkDir iterator test ----

    #[test]
    fn test_walkdir_iterates_nested() {
        let dir = TempDir::new().unwrap();
        let sub1 = dir.path().join("sub1");
        let sub2 = sub1.join("sub2");
        std::fs::create_dir_all(&sub2).unwrap();
        std::fs::write(dir.path().join("root.txt"), "r").unwrap();
        std::fs::write(sub1.join("mid.txt"), "m").unwrap();
        std::fs::write(sub2.join("deep.txt"), "d").unwrap();

        let walker = WalkDir::new(dir.path()).unwrap();
        let entries: Vec<_> = walker.filter_map(|e| e.ok()).collect();
        // Should find at least the 3 files + 2 directories = 5 entries
        // (directories are also yielded by WalkDir)
        let file_count = entries
            .iter()
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .count();
        assert_eq!(file_count, 3);
    }
}
