use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn mald_cmd_with_home(dir: &std::path::Path) -> Command {
    let mut cmd = cargo_bin_cmd!("mald");
    cmd.env("MALD_HOME", dir.join(".mald"));
    cmd
}

#[test]
fn test_cli_help() {
    cargo_bin_cmd!("mald")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Markdown Archive & Localized Daemon",
        ))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("today"))
        .stdout(predicate::str::contains("links"))
        .stdout(predicate::str::contains("backlinks"))
        .stdout(predicate::str::contains("orphans"));
}

#[test]
fn test_cli_init() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("MALD initialized"))
        .stdout(predicate::str::contains("Indexed"));
}

#[test]
fn test_cli_init_creates_fts_index() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    assert!(dir.path().join(".mald/index/metadata.db").exists());
}

#[test]
fn test_cli_kb_list_before_init() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .args(["kb", "list"])
        .assert()
        .success();
}

#[test]
fn test_cli_kb_create_and_list() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["kb", "create", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created space: work"));

    mald_cmd_with_home(dir.path())
        .args(["kb", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work"))
        .stdout(predicate::str::contains("personal"));
}

#[test]
fn test_cli_config_get() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    let expected_editor = if cfg!(windows) { "code" } else { "nvim" };
    mald_cmd_with_home(dir.path())
        .args(["config", "get", "editor"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_editor));
}

#[test]
fn test_cli_config_set_and_get() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["config", "set", "ai.backend", "gguf"])
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["config", "get", "ai.backend"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gguf"));
}

#[test]
fn test_cli_search_with_query() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["search", "amber"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Results for"));
}

#[test]
fn test_cli_orphans() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["orphans"])
        .assert()
        .success();
}

#[test]
fn test_cli_unknown_subcommand() {
    cargo_bin_cmd!("mald")
        .args(["kb", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mald kb list"));
}

#[test]
fn test_cli_setup_ai_hint() {
    cargo_bin_cmd!("mald")
        .args(["setup", "ai"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mald ai setup"));
}

#[test]
fn test_cli_daemon_status() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = mald_cmd_with_home(dir.path())
        .args(["daemon", "status"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("not running") || stdout.contains("Daemon is running"),
        "Unexpected daemon status output: {stdout}"
    );
}

#[test]
fn test_cli_kb_current_and_use() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["kb", "create", "work"])
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["space", "use", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default space set to: work"));

    mald_cmd_with_home(dir.path())
        .args(["space", "current"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Current space: work"));
}

#[test]
fn test_cli_kb_use_accepts_multiword_space_without_quotes() {
    let dir = TempDir::new().unwrap();
    mald_cmd_with_home(dir.path())
        .arg("init")
        .assert()
        .success();

    mald_cmd_with_home(dir.path())
        .args(["kb", "create", "client", "acme", "prod"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created space: client acme prod"));

    mald_cmd_with_home(dir.path())
        .args(["kb", "use", "client", "acme"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Default space set to: client acme prod",
        ));

    mald_cmd_with_home(dir.path())
        .args(["kb", "current"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Current space: client acme prod"));
}
