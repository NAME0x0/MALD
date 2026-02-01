use anyhow::Result;
use chrono::Local;
use std::path::Path;

/// Update the `modified` field in YAML frontmatter. If no frontmatter exists, do nothing.
/// Returns true if the file was modified.
pub fn update_modified_timestamp(path: &Path) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Ok(false);
    }

    let after_open = &trimmed[3..];
    let Some(end_pos) = after_open.find("\n---") else {
        return Ok(false);
    };

    let yaml_section = &after_open[..end_pos];
    let body = &after_open[end_pos + 4..];
    let now = Local::now().format("%Y-%m-%d %H:%M").to_string();

    // Check if modified: field already exists
    let new_yaml = if yaml_section.contains("\nmodified:") || yaml_section.starts_with("modified:")
    {
        // Replace existing modified line
        let lines: Vec<String> = yaml_section
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("modified:") {
                    format!("modified: {}", now)
                } else {
                    line.to_string()
                }
            })
            .collect();
        lines.join("\n")
    } else {
        // Append modified field
        format!("{}\nmodified: {}", yaml_section.trim_end(), now)
    };

    let new_content = format!("---{}---{}", new_yaml, body);

    // Only write if content actually changed (avoid infinite watcher loop)
    if new_content != content {
        std::fs::write(path, new_content)?;
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_adds_modified() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(
            &path,
            "---\ntitle: Test\ncreated: 2024-01-01\n---\n\n# Body\n",
        )
        .unwrap();

        assert!(update_modified_timestamp(&path).unwrap());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("modified: "));
        assert!(content.contains("title: Test"));
        assert!(content.contains("# Body"));
    }

    #[test]
    fn test_updates_existing_modified() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(
            &path,
            "---\ntitle: Test\nmodified: 2020-01-01 00:00\n---\n\n# Body\n",
        )
        .unwrap();

        assert!(update_modified_timestamp(&path).unwrap());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("2020-01-01"));
    }

    #[test]
    fn test_no_frontmatter_noop() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "# No frontmatter\n\nJust text.\n").unwrap();

        assert!(!update_modified_timestamp(&path).unwrap());
    }
}
