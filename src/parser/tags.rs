use regex::Regex;
use std::sync::LazyLock;

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)#([a-zA-Z0-9_-]+)").unwrap());

/// Extract hashtags from text: #tag → "tag"
pub fn extract_tags(text: &str) -> Vec<String> {
    let mut in_code = false;
    let mut tags = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        // Skip headings (# is heading marker, not tag)
        let trimmed = line.trim();
        if trimmed.starts_with('#') && trimmed.chars().nth(1) == Some(' ') {
            // It's a heading — but could still contain tags after the heading text
            // Search from after the heading prefix
            if let Some(rest) = trimmed.strip_prefix("# ") {
                for cap in TAG_RE.captures_iter(rest) {
                    tags.push(cap[1].to_string());
                }
            }
            continue;
        }
        for cap in TAG_RE.captures_iter(line) {
            tags.push(cap[1].to_string());
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags() {
        let text = "Some text with #rust and #programming tags.\n";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["rust", "programming"]);
    }

    #[test]
    fn test_tags_ignore_headings() {
        // "# Heading" should not produce "Heading" as a tag
        let text = "# Heading\nText #real-tag here.\n";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["real-tag"]);
    }

    #[test]
    fn test_tags_in_code_ignored() {
        let text = "#outside\n```\n#inside\n```\n#after\n";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["outside", "after"]);
    }

    #[test]
    fn test_tags_with_hyphens_underscores() {
        let text = "Tags: #my-tag #my_tag #123\n";
        let tags = extract_tags(text);
        assert_eq!(tags, vec!["my-tag", "my_tag", "123"]);
    }
}
