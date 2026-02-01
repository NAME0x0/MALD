use serde_yaml::Value;

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

        match serde_yaml::from_str(yaml_str) {
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
}
