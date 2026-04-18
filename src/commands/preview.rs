use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

pub async fn run(note: &str, kb: Option<&str>) -> Result<()> {
    let filepath = super::run::resolve_note_pub(note, kb)?;
    let content = std::fs::read_to_string(&filepath)?;

    // Strip frontmatter
    let body = strip_frontmatter(&content);

    // Resolve transclusions before rendering
    let resolved = resolve_transclusions(&body, kb, 0);

    render_markdown(&resolved);
    Ok(())
}

fn strip_frontmatter(content: &str) -> String {
    crate::parser::frontmatter::strip(content)
}

/// Resolve `![[note]]` transclusions by inlining the referenced note's body.
/// Max depth prevents infinite recursion.
fn resolve_transclusions(body: &str, kb: Option<&str>, depth: usize) -> String {
    if depth > 5 {
        return body.to_string();
    }

    let mut result = String::new();
    let mut remaining = body;

    while let Some(start) = remaining.find("![[") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 3..];
        if let Some(end) = after_open.find("]]") {
            let target = &after_open[..end];
            // Try to load the referenced note
            match load_note_body(target, kb) {
                Some(embedded) => {
                    result.push_str(&format!("\n\x1b[90m─── embedded: {target} ───\x1b[0m\n"));
                    let resolved = resolve_transclusions(&embedded, kb, depth + 1);
                    result.push_str(&resolved);
                    result.push_str(&format!("\n\x1b[90m─── end: {target} ───\x1b[0m\n"));
                }
                None => {
                    result.push_str(&format!("\x1b[31m![[{target} (not found)]]\x1b[0m"));
                }
            }
            remaining = &after_open[end + 2..];
        } else {
            result.push_str("![[");
            remaining = after_open;
        }
    }
    result.push_str(remaining);
    result
}

fn load_note_body(target: &str, kb: Option<&str>) -> Option<String> {
    let path = super::run::resolve_note_pub(target, kb).ok()?;
    let content = std::fs::read_to_string(&path).ok()?;
    Some(strip_frontmatter(&content))
}

fn render_markdown(body: &str) {
    let parser = Parser::new(body);

    let mut in_code = false;
    let mut _code_lang = String::new();
    let mut in_heading = false;
    let mut _heading_level = 0u32;
    let mut in_list = false;
    let mut _in_link = false;
    let mut link_url = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                _heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    _ => 4,
                };
                println!();
                let prefix = "#".repeat(_heading_level as usize);
                print!("\x1b[1;36m{prefix} ");
            }
            Event::End(TagEnd::Heading(_)) => {
                println!("\x1b[0m");
                in_heading = false;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                _code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    _ => String::new(),
                };
                if !_code_lang.is_empty() {
                    println!("\x1b[90m─── {_code_lang} ───\x1b[0m");
                } else {
                    println!("\x1b[90m───────────\x1b[0m");
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                println!("\x1b[90m───────────\x1b[0m");
                in_code = false;
            }
            Event::Start(Tag::List(_)) => {
                in_list = true;
            }
            Event::End(TagEnd::List(_)) => {
                in_list = false;
            }
            Event::Start(Tag::Item) => {
                print!("  • ");
            }
            Event::End(TagEnd::Item) => {
                println!();
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) if !in_list => {
                println!();
            }
            Event::Start(Tag::Emphasis) => {
                print!("\x1b[3m");
            }
            Event::End(TagEnd::Emphasis) => {
                print!("\x1b[23m");
            }
            Event::Start(Tag::Strong) => {
                print!("\x1b[1m");
            }
            Event::End(TagEnd::Strong) => {
                print!("\x1b[22m");
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                _in_link = true;
                link_url = dest_url.to_string();
            }
            Event::End(TagEnd::Link) => {
                if !link_url.is_empty() {
                    print!(" \x1b[90m({link_url})\x1b[0m");
                }
                _in_link = false;
                link_url.clear();
            }
            Event::Start(Tag::BlockQuote) => {
                print!("\x1b[33m  │ ");
            }
            Event::End(TagEnd::BlockQuote) => {
                println!("\x1b[0m");
            }
            Event::Code(text) => {
                print!("\x1b[33m{text}\x1b[0m");
            }
            Event::Text(text) => {
                if in_code {
                    print!("\x1b[37m  {text}\x1b[0m");
                } else if in_heading {
                    print!("{text}");
                } else {
                    let rendered = render_wikilinks(&text);
                    print!("{rendered}");
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                println!();
            }
            Event::Rule => {
                println!("\x1b[90m────────────────────────────────\x1b[0m");
            }
            Event::TaskListMarker(checked) => {
                if checked {
                    print!("\x1b[32m☑\x1b[0m ");
                } else {
                    print!("☐ ");
                }
            }
            _ => {}
        }
    }
    println!();
}

fn render_wikilinks(text: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("[[") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 2..];
        if let Some(end) = after_open.find("]]") {
            let link_target = &after_open[..end];
            result.push_str(&format!("\x1b[35m[[{link_target}]]\x1b[0m"));
            remaining = &after_open[end + 2..];
        } else {
            result.push_str("[[");
            remaining = after_open;
        }
    }
    result.push_str(remaining);
    result
}
