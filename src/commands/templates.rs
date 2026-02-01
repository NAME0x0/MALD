use anyhow::{bail, Result};
use chrono::Local;

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

/// Create a note from a template. Templates live in ~/.mald/templates/.
/// Variables: {{title}}, {{date}}, {{time}}, {{datetime}}, {{kb}}, {{author}}
pub async fn create_from_template(
    template_name: &str,
    title: &str,
    kb: Option<&str>,
) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let template_dir = mald_home().join("templates");
    let template_path = template_dir.join(format!("{}.md", template_name));
    if !template_path.exists() {
        bail!(
            "Template '{}' not found. Available: {}",
            template_name,
            list_template_names().join(", ")
        );
    }

    let template = std::fs::read_to_string(&template_path)?;
    let content = expand_variables(&template, title, &kb_name);

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        bail!("Knowledge base '{}' not found", kb_name);
    }

    let now = Local::now();
    let slug = slugify(title);
    let filename = format!("{}-{}.md", now.format("%Y%m%d"), slug);
    let filepath = kb_path.join(&filename);

    if filepath.exists() {
        bail!("File already exists: {}", filepath.display());
    }

    std::fs::write(&filepath, &content)?;

    // FTS index
    let hash = crate::daemon::indexer::content_hash(&content);
    let index_dir = mald_home().join("index");
    ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let meta = crate::index::metadata::MetadataStore::open(&meta_path)?;
    meta.index_document_fts(&filepath.to_string_lossy(), title, &content, &hash)?;

    println!("{}", filepath.display());

    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(filepath.to_str().unwrap())
        .status()?;

    Ok(())
}

/// List available templates.
pub async fn list() -> Result<()> {
    let names = list_template_names();
    if names.is_empty() {
        println!("No templates found. Create them in ~/.mald/templates/");
        println!("\nExample: create ~/.mald/templates/meeting.md with:");
        println!("---");
        println!("title: {{{{title}}}}");
        println!("created: {{{{datetime}}}}");
        println!("tags: [meeting]");
        println!("---");
        println!("\n# {{{{title}}}}\n\n## Attendees\n\n- \n\n## Agenda\n\n- \n\n## Action Items\n\n- [ ] ");
    } else {
        for name in names {
            println!("  {}", name);
        }
    }
    Ok(())
}

/// Initialize default templates.
pub fn init_defaults() -> Result<()> {
    let dir = mald_home().join("templates");
    ensure_directory(&dir)?;

    let defaults = [
        (
            "meeting",
            "---\ntitle: {{title}}\ncreated: {{datetime}}\ntags: [meeting]\n---\n\n# {{title}}\n\n## Attendees\n\n- \n\n## Agenda\n\n1. \n\n## Discussion\n\n\n\n## Action Items\n\n- [ ] \n",
        ),
        (
            "project",
            "---\ntitle: {{title}}\ncreated: {{datetime}}\ntags: [project]\nstatus: active\n---\n\n# {{title}}\n\n## Overview\n\n\n\n## Goals\n\n- [ ] \n\n## Resources\n\n- \n\n## Log\n\n### {{date}}\n\n- \n",
        ),
        (
            "reference",
            "---\ntitle: {{title}}\ncreated: {{datetime}}\ntags: [reference]\nsource: \n---\n\n# {{title}}\n\n## Summary\n\n\n\n## Key Points\n\n- \n\n## Notes\n\n\n\n## Related\n\n- \n",
        ),
        (
            "decision",
            "---\ntitle: {{title}}\ncreated: {{datetime}}\ntags: [decision]\nstatus: proposed\n---\n\n# {{title}}\n\n## Context\n\n\n\n## Options\n\n### Option A\n\n\n\n### Option B\n\n\n\n## Decision\n\n\n\n## Consequences\n\n- \n",
        ),
        (
            "review",
            "---\ntitle: {{title}}\ncreated: {{datetime}}\ntags: [review]\nrating: \n---\n\n# {{title}}\n\n## Summary\n\n\n\n## Strengths\n\n- \n\n## Weaknesses\n\n- \n\n## Verdict\n\n\n",
        ),
    ];

    for (name, content) in defaults {
        let path = dir.join(format!("{}.md", name));
        if !path.exists() {
            std::fs::write(&path, content)?;
        }
    }
    Ok(())
}

fn expand_variables(template: &str, title: &str, kb: &str) -> String {
    let now = Local::now();
    template
        .replace("{{title}}", title)
        .replace("{{date}}", &now.format("%Y-%m-%d").to_string())
        .replace("{{time}}", &now.format("%H:%M").to_string())
        .replace("{{datetime}}", &now.format("%Y-%m-%d %H:%M").to_string())
        .replace("{{kb}}", kb)
        .replace("{{year}}", &now.format("%Y").to_string())
        .replace("{{month}}", &now.format("%m").to_string())
        .replace("{{day}}", &now.format("%d").to_string())
        .replace("{{weekday}}", &now.format("%A").to_string())
}

/// Create a new user template.
pub async fn create(name: &str) -> Result<()> {
    let dir = mald_home().join("templates");
    ensure_directory(&dir)?;

    let path = dir.join(format!("{}.md", name));
    if path.exists() {
        bail!("Template '{}' already exists", name);
    }

    let content = format!(
        "---\ntitle: {{{{title}}}}\ncreated: {{{{datetime}}}}\ntags: [{}]\n---\n\n# {{{{title}}}}\n\n",
        name
    );
    std::fs::write(&path, &content)?;

    println!("Created template: {}", path.display());
    println!(
        "Edit it, then use with: mald new \"Title\" --template {}",
        name
    );

    // Open in editor
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()?;

    Ok(())
}

/// Delete a user template.
pub async fn delete(name: &str) -> Result<()> {
    let path = mald_home().join("templates").join(format!("{}.md", name));
    if !path.exists() {
        bail!("Template '{}' not found", name);
    }
    std::fs::remove_file(&path)?;
    println!("Deleted template: {}", name);
    Ok(())
}

/// Edit an existing template.
pub async fn edit(name: &str) -> Result<()> {
    let path = mald_home().join("templates").join(format!("{}.md", name));
    if !path.exists() {
        bail!(
            "Template '{}' not found. Available: {}",
            name,
            list_template_names().join(", ")
        );
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    std::process::Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()?;
    Ok(())
}

fn list_template_names() -> Vec<String> {
    let dir = mald_home().join("templates");
    if !dir.exists() {
        return Vec::new();
    }
    std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect()
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_variables() {
        let template = "# {{title}}\nKB: {{kb}}\nDate: {{date}}";
        let result = expand_variables(template, "Test Note", "personal");
        assert!(result.contains("# Test Note"));
        assert!(result.contains("KB: personal"));
        assert!(result.contains("Date: 20")); // starts with year
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My Cool Note!"), "my-cool-note");
    }
}
