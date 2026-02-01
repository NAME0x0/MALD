use anyhow::{bail, Result};
use std::process::Command;

use crate::fs::mald_home;
use crate::parser::MarkdownDocument;

/// Execute code blocks from a note. Like Jupyter but in the terminal.
/// With --save, writes output back into the markdown after each code block.
pub async fn run(
    note: &str,
    kb: Option<&str>,
    block_index: Option<usize>,
    save: bool,
) -> Result<()> {
    let filepath = resolve_note(note, kb)?;
    let content = std::fs::read_to_string(&filepath)?;
    let doc = MarkdownDocument::parse(&content);

    if doc.code_blocks.is_empty() {
        println!("No code blocks found in '{}'", note);
        return Ok(());
    }

    // Safety confirmation
    let total = if block_index.is_some() {
        1
    } else {
        doc.code_blocks.len()
    };
    println!("About to execute {} code block(s) from '{}'", total, note);
    print!("Continue? [Y/n] ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut confirm = String::new();
    std::io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    if confirm == "n" || confirm == "no" {
        println!("Aborted.");
        return Ok(());
    }

    let blocks_to_run: Vec<(usize, &crate::parser::markdown::CodeBlock)> =
        if let Some(idx) = block_index {
            if idx >= doc.code_blocks.len() {
                bail!(
                    "Block index {} out of range (note has {} blocks)",
                    idx,
                    doc.code_blocks.len()
                );
            }
            vec![(idx, &doc.code_blocks[idx])]
        } else {
            doc.code_blocks.iter().enumerate().collect()
        };

    let mut outputs: Vec<(usize, String)> = Vec::new();

    for (i, block) in &blocks_to_run {
        let lang = if block.language.is_empty() {
            "text"
        } else {
            &block.language
        };

        println!("--- [{}] block {} ---", lang, i);

        match execute_block(lang, &block.content) {
            Ok(output) => {
                let mut combined = String::new();
                if !output.stdout.is_empty() {
                    print!("{}", output.stdout);
                    combined.push_str(&output.stdout);
                }
                if !output.stderr.is_empty() {
                    eprint!("{}", output.stderr);
                    combined.push_str(&output.stderr);
                }
                if output.exit_code != 0 {
                    let msg = format!("(exit code: {})\n", output.exit_code);
                    print!("{}", msg);
                    combined.push_str(&msg);
                }
                if save && !combined.is_empty() {
                    outputs.push((*i, combined));
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                if save {
                    outputs.push((*i, format!("Error: {}\n", e)));
                }
            }
        }
        println!();
    }

    // Save outputs back into the markdown file
    if save && !outputs.is_empty() {
        save_outputs_to_file(&filepath, &content, &doc, &outputs)?;
        println!("Output saved to {}", filepath.display());
    }

    Ok(())
}

/// Save execution outputs back into the markdown, inserting output blocks
/// after each code block.
fn save_outputs_to_file(
    filepath: &std::path::Path,
    original: &str,
    _doc: &MarkdownDocument,
    outputs: &[(usize, String)],
) -> Result<()> {
    let lines: Vec<&str> = original.lines().collect();
    let mut new_content = String::new();
    let mut i = 0;
    let mut block_counter = 0;

    while i < lines.len() {
        let line = lines[i];
        new_content.push_str(line);
        new_content.push('\n');

        // Detect opening code fence
        if line.starts_with("```") && line.len() > 3 {
            // We're entering a code block, find its closing fence
            i += 1;
            while i < lines.len() {
                new_content.push_str(lines[i]);
                new_content.push('\n');
                if lines[i].trim() == "```" {
                    // Check if there's output to insert for this block
                    if let Some((_, output)) = outputs.iter().find(|(idx, _)| *idx == block_counter)
                    {
                        // Skip any existing output block immediately after
                        if i + 1 < lines.len() && lines[i + 1].starts_with("```output") {
                            i += 1; // skip ```output
                            i += 1;
                            while i < lines.len() && lines[i].trim() != "```" {
                                i += 1;
                            }
                            // i now points to closing ``` of old output
                        }
                        // Insert new output block
                        new_content.push_str("```output\n");
                        new_content.push_str(output.trim_end());
                        new_content.push_str("\n```\n");
                    }
                    block_counter += 1;
                    break;
                }
                i += 1;
            }
        }
        i += 1;
    }

    std::fs::write(filepath, new_content)?;
    Ok(())
}

/// List code blocks in a note without executing.
pub async fn list_blocks(note: &str, kb: Option<&str>) -> Result<()> {
    let filepath = resolve_note(note, kb)?;
    let content = std::fs::read_to_string(&filepath)?;
    let doc = MarkdownDocument::parse(&content);

    if doc.code_blocks.is_empty() {
        println!("No code blocks in '{}'", note);
        return Ok(());
    }

    for (i, block) in doc.code_blocks.iter().enumerate() {
        let lang = if block.language.is_empty() {
            "text"
        } else {
            &block.language
        };
        let preview: String = block
            .content
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect();
        println!("  [{}] {} — {}", i, lang, preview);
    }

    Ok(())
}

struct ExecOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn execute_block(lang: &str, code: &str) -> Result<ExecOutput> {
    let (cmd, args, _use_stdin) = match lang {
        "python" | "py" | "python3" => ("python", vec!["-c", code], false),
        "javascript" | "js" | "node" => ("node", vec!["-e", code], false),
        "bash" | "sh" | "shell" => {
            if cfg!(windows) {
                ("cmd", vec!["/C", code], false)
            } else {
                ("bash", vec!["-c", code], false)
            }
        }
        "powershell" | "ps1" | "pwsh" => ("powershell", vec!["-Command", code], false),
        "rust" | "rs" => {
            return run_rust_block(code);
        }
        "sql" => {
            println!("(SQL blocks are not directly executable — use with a DB connection)");
            return Ok(ExecOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            });
        }
        _ => {
            return Ok(ExecOutput {
                stdout: format!("(no executor for language: {})\n", lang),
                stderr: String::new(),
                exit_code: 0,
            });
        }
    };

    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", cmd, e))?;

    Ok(ExecOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

fn run_rust_block(code: &str) -> Result<ExecOutput> {
    let tmp_dir = std::env::temp_dir().join("mald-run");
    std::fs::create_dir_all(&tmp_dir)?;
    let src_path = tmp_dir.join("block.rs");
    let exe_path = if cfg!(windows) {
        tmp_dir.join("block.exe")
    } else {
        tmp_dir.join("block")
    };

    let full_code = if code.contains("fn main") {
        code.to_string()
    } else {
        format!("fn main() {{\n{}\n}}", code)
    };

    std::fs::write(&src_path, &full_code)?;

    let compile = Command::new("rustc")
        .args([src_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()])
        .output()?;

    if !compile.status.success() {
        return Ok(ExecOutput {
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&compile.stderr).to_string(),
            exit_code: compile.status.code().unwrap_or(-1),
        });
    }

    let run = Command::new(&exe_path).output()?;

    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&exe_path);

    Ok(ExecOutput {
        stdout: String::from_utf8_lossy(&run.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        exit_code: run.status.code().unwrap_or(-1),
    })
}

pub fn resolve_note_pub(note: &str, kb: Option<&str>) -> Result<std::path::PathBuf> {
    resolve_note(note, kb)
}

fn resolve_note(note: &str, kb: Option<&str>) -> Result<std::path::PathBuf> {
    let config_path = mald_home().join("config").join("config.json");
    let config = crate::config::ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());
    let kb_path = mald_home().join("kb").join(&kb_name);

    let exact = kb_path.join(note);
    if exact.exists() {
        return Ok(exact);
    }
    let with_ext = kb_path.join(format!("{}.md", note));
    if with_ext.exists() {
        return Ok(with_ext);
    }
    let files = crate::fs::find_files(&kb_path, "md")?;
    for f in &files {
        if let Some(stem) = f.file_stem() {
            if stem
                .to_string_lossy()
                .to_lowercase()
                .contains(&note.to_lowercase())
            {
                return Ok(f.clone());
            }
        }
    }
    bail!("Note '{}' not found in kb '{}'", note, kb_name)
}
