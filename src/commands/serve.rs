use anyhow::{bail, Result};
use std::io::Write;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Start a read-only local HTTP server that renders the KB as HTML.
pub async fn run(kb: Option<&str>, port: u16) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());
    let kb_path = mald_home().join("kb").join(&kb_name);

    if !kb_path.exists() {
        bail!("Knowledge base '{}' not found", kb_name);
    }

    let addr = format!("127.0.0.1:{}", port);
    let listener = std::net::TcpListener::bind(&addr)?;
    println!("Serving '{}' at http://{}", kb_name, addr);
    println!("Press Ctrl+C to stop.\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let kb = kb_path.clone();
                let name = kb_name.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_request(stream, &kb, &name) {
                        tracing::debug!("Request error: {}", e);
                    }
                });
            }
            Err(e) => tracing::debug!("Accept error: {}", e),
        }
    }

    Ok(())
}

fn handle_request(
    mut stream: std::net::TcpStream,
    kb_path: &std::path::Path,
    kb_name: &str,
) -> Result<()> {
    let mut buf = [0u8; 4096];
    let n = std::io::Read::read(&mut stream, &mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (status, body, content_type) = if path == "/" || path == "/index" {
        let html = render_index(kb_path, kb_name)?;
        ("200 OK", html, "text/html; charset=utf-8")
    } else {
        let note_name = path.trim_start_matches('/').trim_end_matches(".html");
        match render_note(kb_path, note_name) {
            Some(html) => ("200 OK", html, "text/html; charset=utf-8"),
            None => (
                "404 Not Found",
                format!(
                    "<html><body><h1>404</h1><p>Note '{}' not found.</p><p><a href=\"/\">Back</a></p></body></html>",
                    note_name
                ),
                "text/html; charset=utf-8",
            ),
        }
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn render_index(kb_path: &std::path::Path, kb_name: &str) -> Result<String> {
    let files = crate::fs::find_files(kb_path, "md")?;
    let mut entries: Vec<(String, String)> = Vec::new();
    for f in &files {
        let stem = f
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let title = extract_title_from_file(f).unwrap_or_else(|| stem.clone());
        entries.push((stem, title));
    }
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut list = String::new();
    for (stem, title) in &entries {
        list.push_str(&format!(
            "<li><a href=\"/{}\">{}</a></li>\n",
            stem, title
        ));
    }

    Ok(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>{kb_name}</title>
<style>{CSS}</style></head>
<body><h1>{kb_name}</h1><p>{} notes</p><ul>{list}</ul>
<footer>Served by MALD</footer></body></html>"#,
        entries.len()
    ))
}

fn render_note(kb_path: &std::path::Path, name: &str) -> Option<String> {
    let files = crate::fs::find_files(kb_path, "md").ok()?;
    let name_lower = name.to_lowercase();
    let file = files.iter().find(|f| {
        f.file_stem()
            .map(|s| s.to_string_lossy().to_lowercase() == name_lower)
            .unwrap_or(false)
    })?;

    let content = std::fs::read_to_string(file).ok()?;
    let body = strip_frontmatter(&content);
    let title = extract_title_from_content(&content).unwrap_or_else(|| name.to_string());

    // Convert wikilinks to HTML links
    let body = regex::Regex::new(r"\[\[([^\]]+)\]\]")
        .unwrap()
        .replace_all(&body, |caps: &regex::Captures| {
            let target = &caps[1];
            let slug = target.to_lowercase().replace(' ', "-");
            format!("<a href=\"/{}\">{}</a>", slug, target)
        })
        .to_string();

    let parser = pulldown_cmark::Parser::new(&body);
    let mut html_body = String::new();
    pulldown_cmark::html::push_html(&mut html_body, parser);

    Some(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>{title}</title>
<style>{CSS}</style></head>
<body><nav><a href="/">← Back</a></nav>{html_body}
<footer>Served by MALD</footer></body></html>"#
    ))
}

fn strip_frontmatter(content: &str) -> String {
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

fn extract_title_from_file(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    extract_title_from_content(&content)
}

fn extract_title_from_content(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("\n---") {
            let yaml = &trimmed[3..3 + end];
            for line in yaml.lines() {
                if let Some(rest) = line.trim().strip_prefix("title:") {
                    let t = rest.trim().to_string();
                    if !t.is_empty() {
                        return Some(t);
                    }
                }
            }
        }
    }
    // Fall back to first heading
    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

const CSS: &str = "body{max-width:720px;margin:2rem auto;padding:0 1rem;\
font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",Roboto,sans-serif;\
line-height:1.6;color:#1a1a1a}\
h1,h2,h3{margin-top:1.5em}\
code{background:#f4f4f4;padding:0.2em 0.4em;border-radius:3px;font-size:0.9em}\
pre{background:#f4f4f4;padding:1em;border-radius:6px;overflow-x:auto}\
pre code{background:none;padding:0}\
a{color:#0066cc}\
nav{margin-bottom:1rem}\
ul{list-style:none;padding-left:0}\
li{padding:0.3em 0}\
footer{margin-top:3rem;padding-top:1rem;border-top:1px solid #eee;\
font-size:0.85em;color:#888}";
