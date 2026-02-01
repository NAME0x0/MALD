use anyhow::Result;

const REPO: &str = "NAME0x0/MALD";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
    html_url: String,
}

#[derive(serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn run() -> Result<()> {
    println!("Current version: {}", CURRENT_VERSION);
    println!("Checking for updates...");

    let client = reqwest::Client::builder()
        .user_agent("mald-updater")
        .build()?;

    let release: GithubRelease = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            REPO
        ))
        .send()
        .await?
        .error_for_status()
        .map_err(|_| {
            crate::errors::bail_ctx(
                "Could not check for updates",
                "Check https://github.com/NAME0x0/MALD/releases manually",
            )
        })?
        .json()
        .await?;

    let latest = release.tag_name.trim_start_matches('v');

    if !is_newer(latest, CURRENT_VERSION) {
        println!("You are on the latest version ({}).", CURRENT_VERSION);
        return Ok(());
    }

    println!("New version available: {} -> {}", CURRENT_VERSION, latest);
    println!("Release: {}\n", release.html_url);

    let binary_name = get_binary_name();
    let asset = release.assets.iter().find(|a| a.name == binary_name);

    match asset {
        Some(asset) => {
            println!("Downloading {}...", asset.name);
            let bytes = client
                .get(&asset.browser_download_url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            let current_exe = std::env::current_exe()?;
            let backup = current_exe.with_extension("bak");

            // Backup current binary
            if let Err(e) = std::fs::rename(&current_exe, &backup) {
                println!("Warning: could not create backup: {}", e);
            }

            // Write new binary
            if let Err(e) = std::fs::write(&current_exe, &bytes) {
                // Try to restore backup
                let _ = std::fs::rename(&backup, &current_exe);
                return Err(crate::errors::bail_ctx(
                    format!("Failed to write new binary: {}", e),
                    "Try downloading manually from the release page",
                ));
            }

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
            }

            // Clean up backup
            let _ = std::fs::remove_file(&backup);

            println!("Updated to version {}.", latest);
        }
        None => {
            println!("No pre-built binary for your platform ({}).", binary_name);
            println!("Update manually:");
            println!("  cargo install mald");
            println!("  Or download from: {}", release.html_url);
        }
    }

    Ok(())
}

fn get_binary_name() -> String {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    let ext = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    format!("mald-{}-{}{}", os, arch, ext)
}

/// Compare semver strings. Returns true if `new` is newer than `current`.
fn is_newer(new: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(new) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn test_get_binary_name() {
        let name = get_binary_name();
        assert!(name.starts_with("mald-"));
        if cfg!(target_os = "windows") {
            assert!(name.ends_with(".exe"));
        }
    }

    #[test]
    fn test_current_version() {
        assert!(!CURRENT_VERSION.is_empty());
    }
}
