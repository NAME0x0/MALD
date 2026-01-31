use regex::Regex;
use std::sync::LazyLock;

static WIKILINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[([^\]]+)\]\]").unwrap());

static MD_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap());

/// Extract wikilink targets from text: [[Target]] → "Target"
pub fn extract_wikilinks(text: &str) -> Vec<String> {
    // Filter out lines inside code blocks
    let filtered = filter_code_blocks(text);
    WIKILINK_RE
        .captures_iter(&filtered)
        .map(|c| c[1].to_string())
        .collect()
}

/// Extract markdown link targets: [text](target) → "target"
pub fn extract_markdown_links(text: &str) -> Vec<String> {
    let filtered = filter_code_blocks(text);
    MD_LINK_RE
        .captures_iter(&filtered)
        .map(|c| c[2].to_string())
        .collect()
}

/// Remove fenced code blocks so we don't extract links from code.
fn filter_code_blocks(text: &str) -> String {
    let mut result = String::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wikilinks() {
        let text = "Link to [[Page A]] and [[Page B]].\n";
        let links = extract_wikilinks(text);
        assert_eq!(links, vec!["Page A", "Page B"]);
    }

    #[test]
    fn test_wikilinks_in_code_ignored() {
        let text = "Normal [[Yes]]\n```\n[[No]]\n```\n";
        let links = extract_wikilinks(text);
        assert_eq!(links, vec!["Yes"]);
    }

    #[test]
    fn test_extract_markdown_links() {
        let text = "See [docs](readme.md) and [site](https://example.com).\n";
        let links = extract_markdown_links(text);
        assert_eq!(links, vec!["readme.md", "https://example.com"]);
    }
}
