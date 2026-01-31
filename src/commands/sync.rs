use anyhow::{bail, Result};
use std::process::Command;

use crate::fs::mald_home;

/// Initialize git in the MALD home directory for version history and sync.
pub async fn init() -> Result<()> {
    let home = mald_home();
    let git_dir = home.join(".git");

    if git_dir.exists() {
        println!("Git already initialized in {}", home.display());
        return Ok(());
    }

    run_git(&home, &["init"])?;

    // Create .gitignore
    let gitignore = home.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(
            &gitignore,
            "daemon.pid\ndaemon.token\ndaemon.sock\ncache/\n*.tmp\n",
        )?;
    }

    // Initial commit
    run_git(&home, &["add", "-A"])?;
    run_git(&home, &["commit", "-m", "Initial MALD snapshot"])?;

    println!("Git initialized in {}", home.display());
    println!("Your notes now have version history.");
    println!("\nTo enable remote sync:");
    println!("  cd {} && git remote add origin <url>", home.display());
    Ok(())
}

/// Auto-commit all changes with a timestamp message.
pub async fn commit() -> Result<()> {
    let home = mald_home();
    ensure_git(&home)?;

    // Check for changes
    let status = run_git_output(&home, &["status", "--porcelain"])?;
    if status.trim().is_empty() {
        println!("No changes to commit.");
        return Ok(());
    }

    let now = chrono::Local::now();
    let msg = format!("mald sync {}", now.format("%Y-%m-%d %H:%M"));

    run_git(&home, &["add", "-A"])?;
    run_git(&home, &["commit", "-m", &msg])?;

    println!("Changes committed: {}", msg);
    Ok(())
}

/// Full sync: commit + push + pull.
pub async fn sync() -> Result<()> {
    let home = mald_home();
    ensure_git(&home)?;

    // Auto-commit first
    let status = run_git_output(&home, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        let now = chrono::Local::now();
        let msg = format!("mald sync {}", now.format("%Y-%m-%d %H:%M"));
        run_git(&home, &["add", "-A"])?;
        run_git(&home, &["commit", "-m", &msg])?;
        println!("Changes committed.");
    }

    // Check if remote exists
    let remotes = run_git_output(&home, &["remote"])?;
    if remotes.trim().is_empty() {
        println!("No remote configured. Changes committed locally.");
        println!("Add a remote: cd {} && git remote add origin <url>", home.display());
        return Ok(());
    }

    // Pull (rebase to avoid merge commits)
    println!("Pulling...");
    match run_git(&home, &["pull", "--rebase", "--autostash"]) {
        Ok(_) => {}
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("CONFLICT") || err_str.contains("conflict") {
                println!("Rebase conflict detected. Aborting rebase and using merge instead.");
                let _ = run_git(&home, &["rebase", "--abort"]);
                match run_git(&home, &["pull", "--no-rebase"]) {
                    Ok(_) => println!("Merged with remote (may have merge commit)."),
                    Err(e2) => {
                        println!("Merge also failed: {}", e2);
                        println!("\nManual resolution needed:");
                        println!("  cd {}", home.display());
                        println!("  git status     # see conflicting files");
                        println!("  # edit conflicts, then: git add . && git commit");
                        return Ok(());
                    }
                }
            } else {
                println!("Pull failed (offline?): {}", e);
            }
        }
    }

    // Push
    println!("Pushing...");
    match run_git(&home, &["push"]) {
        Ok(_) => println!("Synced."),
        Err(e) => {
            println!("Push failed: {}", e);
        }
    }

    Ok(())
}

/// Show version history for a note or all notes.
pub async fn log(note: Option<&str>, count: usize) -> Result<()> {
    let home = mald_home();
    ensure_git(&home)?;

    let n_str = format!("-{}", count);
    let args = if let Some(note_name) = note {
        // Find the file
        let kb_dir = home.join("kb");
        let files = crate::fs::find_files(&kb_dir, "md")?;
        let matching: Vec<String> = files
            .iter()
            .filter(|f| {
                f.file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase().contains(&note_name.to_lowercase()))
                    .unwrap_or(false)
            })
            .filter_map(|f| f.strip_prefix(&home).ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if matching.is_empty() {
            println!("No matching note found for '{}'", note_name);
            return Ok(());
        }

        // Show log for first match
        let path = &matching[0];
        println!("History for {}:\n", path);
        vec![
            "log".to_string(),
            "--oneline".to_string(),
            "--no-decorate".to_string(),
            n_str,
            "--follow".to_string(),
            "--".to_string(),
            path.clone(),
        ]
    } else {
        vec![
            "log".to_string(),
            "--oneline".to_string(),
            "--no-decorate".to_string(),
            n_str,
        ]
    };

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_git_output(&home, &args_ref)?;
    if output.trim().is_empty() {
        println!("No history found.");
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Undo the last change (git revert or restore).
pub async fn undo() -> Result<()> {
    let home = mald_home();
    ensure_git(&home)?;

    // Check if there are uncommitted changes
    let status = run_git_output(&home, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        // Restore all uncommitted changes
        run_git(&home, &["checkout", "--", "."])?;
        run_git(&home, &["clean", "-fd"])?;
        println!("Uncommitted changes reverted.");
    } else {
        // Revert last commit
        run_git(&home, &["revert", "--no-edit", "HEAD"])?;
        println!("Last commit reverted.");
    }

    Ok(())
}

fn ensure_git(home: &std::path::Path) -> Result<()> {
    if !home.join(".git").exists() {
        bail!("Git not initialized. Run `mald sync init` first.");
    }
    Ok(())
}

fn run_git(dir: &std::path::Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(())
}

fn run_git_output(dir: &std::path::Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
