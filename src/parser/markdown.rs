use anyhow::Result;
use std::path::Path;

use super::frontmatter;
use super::links;
use super::tags;

#[derive(Debug, Clone, Default)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    pub path: Option<std::path::PathBuf>,
    pub title: String,
    pub content: String,
    pub metadata: serde_yaml::Value,
    pub wikilinks: Vec<String>,
    pub markdown_links: Vec<String>,
    pub tags: Vec<String>,
    pub code_blocks: Vec<CodeBlock>,
    pub headings: Vec<(u32, String)>,
}

impl MarkdownDocument {
    pub fn parse(content: &str) -> Self {
        let (metadata, body) = frontmatter::extract(content);
        let title = Self::extract_title(&body, &metadata);
        let wikilinks = links::extract_wikilinks(&body);
        let markdown_links = links::extract_markdown_links(&body);
        let mut tags_list = tags::extract_tags(&body);
        // Also include tags from frontmatter `tags: [...]`
        if let Some(fm_tags) = metadata.get("tags").and_then(|v| v.as_sequence()) {
            for t in fm_tags {
                if let Some(s) = t.as_str() {
                    let s = s.to_string();
                    if !tags_list.contains(&s) {
                        tags_list.push(s);
                    }
                }
            }
        }
        let code_blocks = Self::extract_code_blocks(&body);
        let headings = Self::extract_headings(&body);

        Self {
            path: None,
            title,
            content: body,
            metadata,
            wikilinks,
            markdown_links,
            tags: tags_list,
            code_blocks,
            headings,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut doc = Self::parse(&content);
        doc.path = Some(path.to_path_buf());
        if doc.title.is_empty() {
            if let Some(stem) = path.file_stem() {
                doc.title = stem.to_string_lossy().to_string();
            }
        }
        Ok(doc)
    }

    pub fn all_links(&self) -> Vec<&str> {
        self.wikilinks
            .iter()
            .chain(self.markdown_links.iter())
            .map(|s| s.as_str())
            .collect()
    }

    fn extract_title(body: &str, metadata: &serde_yaml::Value) -> String {
        // Try frontmatter title first
        if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
            return title.to_string();
        }
        // Fall back to first H1
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                return trimmed.trim_start_matches("# ").trim().to_string();
            }
        }
        String::new()
    }

    fn extract_code_blocks(body: &str) -> Vec<CodeBlock> {
        use pulldown_cmark::{Event, Parser, Tag, TagEnd};
        let parser = Parser::new(body);
        let mut blocks = Vec::new();
        let mut in_code = false;
        let mut current_lang = String::new();
        let mut current_content = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code = true;
                    current_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        _ => String::new(),
                    };
                    current_content.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code = false;
                    blocks.push(CodeBlock {
                        language: current_lang.clone(),
                        content: current_content.clone(),
                    });
                }
                Event::Text(text) if in_code => {
                    current_content.push_str(&text);
                }
                _ => {}
            }
        }
        blocks
    }

    fn extract_headings(body: &str) -> Vec<(u32, String)> {
        use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
        let parser = Parser::new(body);
        let mut headings = Vec::new();
        let mut in_heading = false;
        let mut current_level = 0u32;
        let mut current_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    current_level = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;
                    headings.push((current_level, current_text.clone()));
                }
                Event::Text(text) if in_heading => {
                    current_text.push_str(&text);
                }
                _ => {}
            }
        }
        headings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_document() {
        let content = "# My Title\n\nSome content here.\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.title, "My Title");
    }

    #[test]
    fn test_parse_with_frontmatter_title() {
        let content = "---\ntitle: FM Title\n---\n# Heading\nContent\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.title, "FM Title");
    }

    #[test]
    fn test_extract_code_blocks() {
        let content = "# Test\n\n```python\nprint('hello')\n```\n\n```rust\nfn main() {}\n```\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.code_blocks.len(), 2);
        assert_eq!(doc.code_blocks[0].language, "python");
        assert_eq!(doc.code_blocks[1].language, "rust");
    }

    #[test]
    fn test_extract_headings() {
        let content = "# H1\n## H2\n### H3\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0], (1, "H1".to_string()));
        assert_eq!(doc.headings[1], (2, "H2".to_string()));
    }

    #[test]
    fn test_wikilinks_extracted() {
        let content = "# Test\nLink to [[Other Page]] and [[Another]].\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.wikilinks, vec!["Other Page", "Another"]);
    }

    #[test]
    fn test_tags_extracted() {
        let content = "# Test\nTagged #rust and #programming here.\n";
        let doc = MarkdownDocument::parse(content);
        assert!(doc.tags.contains(&"rust".to_string()));
        assert!(doc.tags.contains(&"programming".to_string()));
    }

    #[test]
    fn test_markdown_links_extracted() {
        let content = "# Test\nSee [link](target.md) for more.\n";
        let doc = MarkdownDocument::parse(content);
        assert_eq!(doc.markdown_links, vec!["target.md"]);
    }

    #[test]
    fn test_all_links() {
        let content = "# Test\n[[Wiki]] and [md](file.md)\n";
        let doc = MarkdownDocument::parse(content);
        let all = doc.all_links();
        assert!(all.contains(&"Wiki"));
        assert!(all.contains(&"file.md"));
    }
}
