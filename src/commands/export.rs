use anyhow::{bail, Result};
use pulldown_cmark::{html, Parser};

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Export a note to HTML. Strips frontmatter, renders markdown to HTML,
/// wraps in a minimal styled document.
pub async fn run(note: &str, kb: Option<&str>, output: Option<&str>) -> Result<()> {
    let filepath = super::run::resolve_note_pub(note, kb)?;
    let content = std::fs::read_to_string(&filepath)?;
    let body = strip_frontmatter(&content);

    // Extract title from frontmatter or first heading
    let title = extract_title(&content, note);

    let parser = Parser::new(&body);
    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  body {{ max-width: 720px; margin: 2rem auto; padding: 0 1rem; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; color: #1a1a1a; }}
  h1,h2,h3 {{ margin-top: 1.5em; }}
  code {{ background: #f4f4f4; padding: 0.2em 0.4em; border-radius: 3px; font-size: 0.9em; }}
  pre {{ background: #f4f4f4; padding: 1em; border-radius: 6px; overflow-x: auto; }}
  pre code {{ background: none; padding: 0; }}
  blockquote {{ border-left: 3px solid #ddd; margin-left: 0; padding-left: 1em; color: #555; }}
  a {{ color: #0066cc; }}
  table {{ border-collapse: collapse; width: 100%; }}
  th, td {{ border: 1px solid #ddd; padding: 0.5em; text-align: left; }}
  img {{ max-width: 100%; }}
  .task-done {{ text-decoration: line-through; color: #888; }}
  footer {{ margin-top: 3rem; padding-top: 1rem; border-top: 1px solid #eee; font-size: 0.85em; color: #888; }}
</style>
</head>
<body>
{html_body}
<footer>Exported from MALD</footer>
</body>
</html>"#
    );

    let out_path = if let Some(o) = output {
        std::path::PathBuf::from(o)
    } else {
        let stem = filepath.file_stem().unwrap().to_string_lossy();
        filepath.parent().unwrap().join(format!("{}.html", stem))
    };

    std::fs::write(&out_path, &full_html)?;
    println!("{}", out_path.display());
    Ok(())
}

/// Export all notes from a KB to a directory of HTML files or plain markdown.
pub async fn export_all(kb: Option<&str>, output_dir: &str, format: &str) -> Result<()> {
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

    let out = std::path::Path::new(output_dir);
    crate::fs::ensure_directory(out)?;

    let files = crate::fs::find_files(&kb_path, "md")?;
    let mut count = 0;

    for f in &files {
        let content = std::fs::read_to_string(f)?;
        let stem = f.file_stem().unwrap().to_string_lossy();

        match format {
            "html" => {
                let body = strip_frontmatter(&content);
                let title = extract_title(&content, &stem);
                let parser = Parser::new(&body);
                let mut html_body = String::new();
                html::push_html(&mut html_body, parser);

                let full_html = format!(
                    "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{}</title>\
                    <style>body{{max-width:720px;margin:2rem auto;padding:0 1rem;\
                    font-family:sans-serif;line-height:1.6}}code{{background:#f4f4f4;\
                    padding:0.2em 0.4em;border-radius:3px}}pre{{background:#f4f4f4;\
                    padding:1em;border-radius:6px;overflow-x:auto}}</style></head>\
                    <body>{}</body></html>",
                    title, html_body
                );
                std::fs::write(out.join(format!("{}.html", stem)), full_html)?;
            }
            _ => {
                // Plain markdown copy (portable export)
                let relative = f.strip_prefix(&kb_path).unwrap_or(f);
                let dest = out.join(relative);
                if let Some(parent) = dest.parent() {
                    crate::fs::ensure_directory(parent)?;
                }
                std::fs::copy(f, &dest)?;
            }
        }
        count += 1;
    }

    println!("Exported {} files to {}", count, output_dir);
    Ok(())
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

fn extract_title(content: &str, fallback: &str) -> String {
    let trimmed = content.trim_start();
    if let Some(after_prefix) = trimmed.strip_prefix("---") {
        if let Some(end) = after_prefix.find("\n---") {
            let yaml = &after_prefix[..end];
            for line in yaml.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("title:") {
                    return rest.trim().to_string();
                }
            }
        }
    }
    fallback.to_string()
}
