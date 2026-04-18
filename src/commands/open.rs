use anyhow::{bail, Result};

/// Open the KB directory in the configured editor (or file manager).
pub async fn run(kb: Option<&str>) -> Result<()> {
    let (_config, typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;

    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found. Create it with: mald kb create {kb_name}");
    }

    let editor = typed.editor.clone();

    // Some editors (VS Code, Sublime) can open directories
    // For terminal editors, open the index.md or first file
    let target = if editor.contains("code") || editor.contains("subl") || editor.contains("zed") {
        kb_path.clone()
    } else {
        // Terminal editor — open index.md or first file found
        let index = kb_path.join("index.md");
        if index.exists() {
            index
        } else {
            let files = crate::fs::find_files(&kb_path, "md")?;
            files.into_iter().next().unwrap_or(kb_path.clone())
        }
    };

    std::process::Command::new(&editor)
        .arg(target.to_str().unwrap())
        .status()?;

    Ok(())
}
