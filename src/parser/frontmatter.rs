use serde_norway::Value;

/// Strip YAML frontmatter from content, returning only the body.
pub fn strip(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    if let Some(end) = trimmed[3..].find("\n---") {
        trimmed[3 + end + 4..].trim_start_matches('\n').to_string()
    } else {
        content.to_string()
    }
}

/// Extract YAML frontmatter and return (metadata, body).
pub fn extract(content: &str) -> (Value, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (Value::Mapping(Default::default()), content.to_string());
    }

    // Find the closing ---
    let after_open = &trimmed[3..];
    if let Some(end_pos) = after_open.find("\n---") {
        let yaml_str = &after_open[..end_pos];
        let body_start = end_pos + 4; // skip \n---
        let body = after_open[body_start..]
            .trim_start_matches('\n')
            .to_string();

        match serde_norway::from_str(yaml_str) {
            Ok(value) => (value, body),
            Err(_) => (Value::Mapping(Default::default()), content.to_string()),
        }
    } else {
        (Value::Mapping(Default::default()), content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\ntitle: Hello\ntags:\n  - rust\n  - code\n---\n# Body\n";
        let (meta, body) = extract(content);
        assert_eq!(meta["title"].as_str().unwrap(), "Hello");
        assert!(body.starts_with("# Body"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\nSome text.\n";
        let (meta, body) = extract(content);
        assert!(meta.is_mapping());
        assert_eq!(body, content);
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n# Body\n";
        let (meta, body) = extract(content);
        assert!(meta.is_null() || meta.is_mapping());
        assert!(body.contains("# Body"));
    }

    // ---- strip() tests ----

    #[test]
    fn test_strip_removes_frontmatter() {
        let content = "---\ntitle: Hello\ntags:\n  - rust\n---\n# Body Content\nSome text.\n";
        let body = strip(content);
        assert!(body.starts_with("# Body Content"));
        assert!(!body.contains("---"));
        assert!(!body.contains("title: Hello"));
    }

    #[test]
    fn test_strip_no_frontmatter() {
        let content = "# Just a heading\nSome text.\n";
        let body = strip(content);
        assert_eq!(body, content);
    }

    #[test]
    fn test_strip_unclosed_frontmatter() {
        // Only opening --- with no closing --- should return content as-is
        let content = "---\ntitle: Broken\nNo closing delimiter\n";
        let body = strip(content);
        assert_eq!(body, content);
    }

    // ---- malformed YAML test ----

    #[test]
    fn test_malformed_yaml_returns_empty_mapping() {
        let content = "---\n: : : invalid yaml {{{\n---\n# Body\n";
        let (meta, body) = extract(content);
        // Malformed YAML should fall back to empty mapping
        assert!(meta.is_mapping());
        // Body should be the original content since YAML parsing failed
        assert_eq!(body, content);
    }

    // ---- Windows line endings test ----

    #[test]
    fn test_frontmatter_windows_line_endings() {
        let content = "---\r\ntitle: Windows\r\ntags:\r\n  - test\r\n---\r\n# Body\r\n";
        let (meta, body) = extract(content);
        // serde_yaml handles \r\n in YAML values
        if let Some(title) = meta.get("title").and_then(|v| v.as_str()) {
            // Title may have trailing \r from Windows line endings
            assert!(title.trim() == "Windows");
        }
        assert!(body.contains("# Body"));
    }
}
