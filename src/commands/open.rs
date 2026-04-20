use anyhow::Result;

/// Open the active space directory in the configured editor (or file manager).
pub async fn run(kb: Option<&str>) -> Result<()> {
    let (_config, typed, kb_name, kb_path) = crate::config::resolve_kb(kb)?;

    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Space `{kb_name}` not found."),
            format!("Run `mald kb list` to inspect your workspace, or `mald kb create {kb_name}` to create it."),
        ));
    }

    let editor = typed.editor.clone();

    // Some editors (VS Code, Sublime) can open directories
    // For terminal editors, open the index.md or first file
    let target = if crate::commands::launch::supports_directory_target(&editor) {
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

    crate::commands::launch::open_in_editor(&editor, target.as_os_str())?;

    Ok(())
}
