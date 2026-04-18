//! Simple syntax highlighting helpers (tree-sitter-ready).

use std::collections::HashMap;

use iced::Color;
use regex::Regex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle as SyntectFontStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tree_sitter_highlight::{HighlightConfiguration, Highlighter};

use crate::gui::theme::colors;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub color: Color,
    pub style: FontStyle,
}

/// High-level syntax highlighter.
/// For now this uses a lightweight regex-based fallback and keeps hooks
/// for tree-sitter highlight configurations when available.
pub struct SyntaxHighlighter {
    #[allow(dead_code)]
    highlighter: Highlighter,
    #[allow(dead_code)]
    configs: HashMap<String, HighlightConfiguration>,
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .or_else(|| theme_set.themes.values().next().cloned())
            .unwrap_or_default();
        Self {
            highlighter: Highlighter::new(),
            configs: HashMap::new(),
            syntax_set,
            theme,
        }
    }

    pub fn highlight(&self, code: &str, lang: &str) -> Vec<HighlightSpan> {
        self.syntect_highlight(code, lang)
            .unwrap_or_else(|| self.basic_highlight(code, lang))
    }

    fn syntect_highlight(&self, code: &str, lang: &str) -> Option<Vec<HighlightSpan>> {
        let lang = lang.trim().to_lowercase();
        if lang.is_empty() || code.is_empty() {
            return None;
        }

        let token = match lang.as_str() {
            "rs" | "rust" => "Rust",
            "py" | "python" => "Python",
            "js" | "javascript" => "JavaScript",
            "ts" | "typescript" => "TypeScript",
            "json" => "JSON",
            "html" => "HTML",
            "css" => "CSS",
            "toml" => "TOML",
            "yaml" | "yml" => "YAML",
            "bash" | "sh" | "shell" | "zsh" => "Bash",
            "sql" => "SQL",
            "go" => "Go",
            "java" => "Java",
            "c" => "C",
            "cpp" | "c++" => "C++",
            _ => "",
        };

        let syntax = if !token.is_empty() {
            self.syntax_set.find_syntax_by_token(token)
        } else {
            None
        }
        .or_else(|| self.syntax_set.find_syntax_by_extension(&lang))
        .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut spans: Vec<HighlightSpan> = Vec::new();
        let mut cursor = 0usize;

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set).ok()?;
            for (style, text) in ranges {
                if text.is_empty() {
                    continue;
                }
                let len = text.len();
                spans.push(HighlightSpan {
                    start: cursor,
                    end: cursor + len,
                    color: syntect_color(style.foreground),
                    style: map_font_style(style.font_style),
                });
                cursor += len;
            }
        }

        Some(spans)
    }

    fn basic_highlight(&self, code: &str, lang: &str) -> Vec<HighlightSpan> {
        let lang = lang.trim().to_lowercase();
        if lang.is_empty() || code.is_empty() {
            return Vec::new();
        }

        let mut spans: Vec<HighlightSpan> = Vec::new();

        let (comment_patterns, string_patterns, keywords) = match lang.as_str() {
            "rust" | "rs" => (
                vec![r"(?m)//.*$", r"(?s)/\*.*?\*/"],
                vec![r#""([^"\\]|\\.)*""#, r#"'([^'\\]|\\.)*'"#],
                vec![
                    "fn", "let", "mut", "pub", "struct", "enum", "impl", "use", "mod", "match",
                    "if", "else", "for", "while", "loop", "return", "trait", "const", "static",
                    "ref", "as", "crate", "super", "self", "Self",
                ],
            ),
            "python" | "py" => (
                vec![r"(?m)#.*$"],
                vec![r#""([^"\\]|\\.)*""#, r#"'([^'\\]|\\.)*'"#],
                vec![
                    "def", "class", "import", "from", "as", "if", "elif", "else", "for", "while",
                    "return", "with", "try", "except", "finally", "lambda", "yield", "True",
                    "False", "None",
                ],
            ),
            "javascript" | "js" | "typescript" | "ts" => (
                vec![r"(?m)//.*$", r"(?s)/\*.*?\*/"],
                vec![
                    r#""([^"\\]|\\.)*""#,
                    r#"'([^'\\]|\\.)*'"#,
                    r"`([^`\\]|\\.)*`",
                ],
                vec![
                    "function", "const", "let", "var", "if", "else", "for", "while", "return",
                    "class", "import", "from", "export", "async", "await", "new", "try", "catch",
                    "finally", "switch", "case", "break", "default", "extends", "super", "this",
                ],
            ),
            "go" => (
                vec![r"(?m)//.*$", r"(?s)/\*.*?\*/"],
                vec![r#""([^"\\]|\\.)*""#, r#"'([^'\\]|\\.)*'"#],
                vec![
                    "func",
                    "package",
                    "import",
                    "if",
                    "else",
                    "for",
                    "range",
                    "return",
                    "struct",
                    "interface",
                    "type",
                    "var",
                    "const",
                    "go",
                    "defer",
                    "select",
                    "switch",
                    "case",
                    "default",
                ],
            ),
            "c" | "cpp" | "c++" | "java" => (
                vec![r"(?m)//.*$", r"(?s)/\*.*?\*/"],
                vec![r#""([^"\\]|\\.)*""#, r#"'([^'\\]|\\.)*'"#],
                vec![
                    "int",
                    "float",
                    "double",
                    "char",
                    "void",
                    "struct",
                    "class",
                    "public",
                    "private",
                    "protected",
                    "static",
                    "const",
                    "if",
                    "else",
                    "for",
                    "while",
                    "switch",
                    "case",
                    "return",
                    "new",
                    "this",
                ],
            ),
            "bash" | "sh" | "shell" | "zsh" => (
                vec![r"(?m)#.*$"],
                vec![r#""([^"\\]|\\.)*""#, r#"'([^'\\]|\\.)*'"#],
                vec![
                    "if", "then", "fi", "for", "in", "do", "done", "case", "esac", "function",
                    "local", "export", "return",
                ],
            ),
            "sql" => (
                vec![r"(?m)--.*$", r"(?s)/\*.*?\*/"],
                vec![r#"'([^'\\]|\\.)*'"#],
                vec![
                    "select", "from", "where", "join", "left", "right", "inner", "outer", "group",
                    "by", "order", "insert", "into", "update", "delete", "create", "table", "drop",
                    "alter", "limit", "values",
                ],
            ),
            _ => return spans,
        };

        for pattern in comment_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for m in re.find_iter(code) {
                    spans.push(HighlightSpan {
                        start: m.start(),
                        end: m.end(),
                        color: colors::COMMENT,
                        style: FontStyle::Italic,
                    });
                }
            }
        }

        for pattern in string_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for m in re.find_iter(code) {
                    if !overlaps(&spans, m.start(), m.end()) {
                        spans.push(HighlightSpan {
                            start: m.start(),
                            end: m.end(),
                            color: colors::STRING,
                            style: FontStyle::Normal,
                        });
                    }
                }
            }
        }

        if !keywords.is_empty() {
            let pattern = format!(r"\b({})\b", keywords.join("|"));
            if let Ok(re) = Regex::new(&pattern) {
                for m in re.find_iter(code) {
                    if !overlaps(&spans, m.start(), m.end()) {
                        spans.push(HighlightSpan {
                            start: m.start(),
                            end: m.end(),
                            color: colors::KEYWORD,
                            style: FontStyle::Bold,
                        });
                    }
                }
            }
        }

        spans.sort_by_key(|s| s.start);
        spans
    }
}

fn overlaps(spans: &[HighlightSpan], start: usize, end: usize) -> bool {
    spans
        .iter()
        .any(|span| start < span.end && end > span.start)
}

fn syntect_color(color: syntect::highlighting::Color) -> Color {
    Color {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: color.a as f32 / 255.0,
    }
}

fn map_font_style(style: SyntectFontStyle) -> FontStyle {
    if style.contains(SyntectFontStyle::BOLD) {
        FontStyle::Bold
    } else if style.contains(SyntectFontStyle::ITALIC) {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    }
}
