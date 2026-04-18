/// Syntax highlighting token types for markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Normal,
    Heading,
    Bold,
    Italic,
    Code,
    CodeBlock,
    Link,
    Wikilink,
    Tag,
    TaskOpen,
    TaskDone,
    FrontmatterKey,
    FrontmatterValue,
    ListMarker,
    BlockQuote,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

/// Simple regex-based markdown tokenizer for syntax highlighting.
/// For MVP — not using tree-sitter yet, will upgrade later.
pub fn tokenize_markdown(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut in_frontmatter = false;
    let mut in_code_block = false;
    let mut offset = 0;

    for line in text.lines() {
        let line_start = offset;
        let line_end = offset + line.len();

        if line.starts_with("---") && offset == 0 {
            in_frontmatter = true;
            tokens.push(Token {
                kind: TokenKind::FrontmatterKey,
                start: line_start,
                end: line_end,
            });
        } else if line.starts_with("---") && in_frontmatter {
            in_frontmatter = false;
            tokens.push(Token {
                kind: TokenKind::FrontmatterKey,
                start: line_start,
                end: line_end,
            });
        } else if in_frontmatter {
            if let Some(colon_pos) = line.find(':') {
                tokens.push(Token {
                    kind: TokenKind::FrontmatterKey,
                    start: line_start,
                    end: line_start + colon_pos + 1,
                });
                tokens.push(Token {
                    kind: TokenKind::FrontmatterValue,
                    start: line_start + colon_pos + 1,
                    end: line_end,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::FrontmatterValue,
                    start: line_start,
                    end: line_end,
                });
            }
        } else if line.starts_with("```") {
            in_code_block = !in_code_block;
            tokens.push(Token {
                kind: TokenKind::CodeBlock,
                start: line_start,
                end: line_end,
            });
        } else if in_code_block {
            tokens.push(Token {
                kind: TokenKind::CodeBlock,
                start: line_start,
                end: line_end,
            });
        } else if line.starts_with('#') {
            tokens.push(Token {
                kind: TokenKind::Heading,
                start: line_start,
                end: line_end,
            });
        } else if line.trim().starts_with("- [ ] ") {
            tokens.push(Token {
                kind: TokenKind::TaskOpen,
                start: line_start,
                end: line_end,
            });
        } else if line.trim().starts_with("- [x] ") || line.trim().starts_with("- [X] ") {
            tokens.push(Token {
                kind: TokenKind::TaskDone,
                start: line_start,
                end: line_end,
            });
        } else if line.trim().starts_with('>') {
            tokens.push(Token {
                kind: TokenKind::BlockQuote,
                start: line_start,
                end: line_end,
            });
        } else {
            // Inline tokens
            tokenize_inline(line, line_start, &mut tokens);
        }

        offset = line_end + 1; // +1 for newline
    }

    tokens
}

fn tokenize_inline(line: &str, base: usize, tokens: &mut Vec<Token>) {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Wikilinks: [[...]]
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            if let Some(end) = line[i + 2..].find("]]") {
                tokens.push(Token {
                    kind: TokenKind::Wikilink,
                    start: base + i,
                    end: base + i + 4 + end,
                });
                i += end + 4;
                continue;
            }
        }

        // Tags: #word
        if bytes[i] == b'#' && (i == 0 || bytes[i - 1] == b' ') {
            let tag_start = i;
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
            {
                i += 1;
            }
            if i > tag_start + 1 {
                tokens.push(Token {
                    kind: TokenKind::Tag,
                    start: base + tag_start,
                    end: base + i,
                });
                continue;
            }
        }

        // Inline code: `...`
        if bytes[i] == b'`' {
            if let Some(end) = line[i + 1..].find('`') {
                tokens.push(Token {
                    kind: TokenKind::Code,
                    start: base + i,
                    end: base + i + 2 + end,
                });
                i += end + 2;
                continue;
            }
        }

        // Bold: **...**
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'*' {
            if let Some(end) = line[i + 2..].find("**") {
                tokens.push(Token {
                    kind: TokenKind::Bold,
                    start: base + i,
                    end: base + i + 4 + end,
                });
                i += end + 4;
                continue;
            }
        }

        // Markdown links: [text](url)
        if bytes[i] == b'[' {
            if let Some(bracket_end) = line[i + 1..].find("](") {
                if let Some(paren_end) = line[i + bracket_end + 3..].find(')') {
                    tokens.push(Token {
                        kind: TokenKind::Link,
                        start: base + i,
                        end: base + i + bracket_end + paren_end + 4,
                    });
                    i += bracket_end + paren_end + 4;
                    continue;
                }
            }
        }

        i += 1;
    }
}
