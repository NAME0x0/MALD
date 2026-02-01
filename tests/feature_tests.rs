use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn mald_cmd() -> Command {
    let mut cmd = Command::cargo_bin("mald").unwrap();
    cmd
}

fn setup_mald_home() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::env::set_var("MALD_HOME", dir.path());
    // Run init
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("init")
        .assert()
        .success();
    dir
}

#[test]
fn test_dashboard_before_init() {
    let dir = TempDir::new().unwrap();
    // MALD_HOME exists (temp dir) but has no kb/ subdirectory
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No knowledge bases"));
}

#[test]
fn test_dashboard_after_init() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dashboard"))
        .stdout(predicate::str::contains("Knowledge bases"));
}

#[test]
fn test_tasks_default_kb() {
    let dir = setup_mald_home();
    // Init creates a sample note with tasks
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create your first note"));
}

#[test]
fn test_tasks_finds_checkboxes() {
    let dir = setup_mald_home();
    let note_path = dir.path().join("kb").join("personal").join("test-tasks.md");
    fs::write(
        &note_path,
        "---\ntitle: Tasks\n---\n\n- [ ] Buy milk\n- [x] Done thing\n- [ ] Fix bug\n",
    )
    .unwrap();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .assert()
        .success()
        .stdout(predicate::str::contains("Buy milk"))
        .stdout(predicate::str::contains("Fix bug"))
        // 5 open (3 from sample + 2 from test note), 1 done
        .stdout(predicate::str::contains("5 open, 1 done"));
}

#[test]
fn test_review() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("review")
        .assert()
        .success()
        .stdout(predicate::str::contains("Review for"))
        .stdout(predicate::str::contains("Stats"));
}

#[test]
fn test_reindex() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("reindex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rebuilt FTS index"));
}

#[test]
fn test_doctor_extended() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("MALD Doctor"))
        .stdout(predicate::str::contains("Broken wikilinks"))
        .stdout(predicate::str::contains("Daemon log"));
}

#[test]
fn test_help_topic_ai() {
    mald_cmd()
        .arg("help-topic")
        .arg("ai")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ollama"))
        .stdout(predicate::str::contains("SETUP"));
}

#[test]
fn test_help_topic_unknown() {
    mald_cmd()
        .arg("help-topic")
        .arg("foobar")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unknown topic"))
        .stdout(predicate::str::contains("Available topics"));
}

#[test]
fn test_graph_stats() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("graph")
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Graph stats"))
        .stdout(predicate::str::contains("Notes:"));
}

#[test]
fn test_graph_broken_links() {
    let dir = setup_mald_home();
    // Create a note with a broken wikilink
    let note_path = dir.path().join("kb").join("personal").join("test-broken.md");
    fs::write(
        &note_path,
        "---\ntitle: Test\n---\n\nLink to [[nonexistent-note]]\n",
    )
    .unwrap();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("graph")
        .arg("broken-links")
        .assert()
        .success()
        .stdout(predicate::str::contains("nonexistent-note"));
}

#[test]
fn test_graph_view_mermaid() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("graph")
        .arg("view")
        .assert()
        .success()
        .stdout(predicate::str::contains("```mermaid"));
}

#[test]
fn test_template_list() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("template")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("meeting"));
}

#[test]
fn test_tags_empty_kb() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tags")
        .assert()
        .success();
}

#[test]
fn test_bench() {
    mald_cmd()
        .arg("bench")
        .arg("--dim")
        .arg("32")
        .arg("--count")
        .arg("100")
        .assert()
        .success()
        .stdout(predicate::str::contains("Insert:"))
        .stdout(predicate::str::contains("Search:"))
        .stdout(predicate::str::contains("Save:"));
}

#[test]
fn test_plugin_list_no_dir() {
    let dir = TempDir::new().unwrap();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("plugin")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No plugins directory"));
}

#[test]
fn test_serve_missing_kb() {
    let dir = TempDir::new().unwrap();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("serve")
        .arg("--kb")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// --- JSON output tests ---

#[test]
fn test_search_json_output() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("search")
        .arg("getting started")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("\"type\""));
}

#[test]
fn test_tasks_json_output() {
    let dir = setup_mald_home();
    let note_path = dir.path().join("kb").join("personal").join("json-tasks.md");
    fs::write(&note_path, "- [ ] JSON task\n- [x] Done\n").unwrap();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"task\""))
        .stdout(predicate::str::contains("JSON task"));
}

#[test]
fn test_tags_json_output() {
    let dir = setup_mald_home();
    // The default init note has #mald and #guide tags
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tags")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"count\""));
}

#[test]
fn test_kb_list_json_output() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("kb")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains("personal"));
}

#[test]
fn test_init_creates_searchable_sample() {
    let dir = setup_mald_home();
    // The init sample note should be findable
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("search")
        .arg("getting started")
        .assert()
        .success()
        .stdout(predicate::str::contains("Getting Started"));
}

#[test]
fn test_init_sample_has_tasks() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create your first note"));
}

#[test]
fn test_ai_chat_no_ollama() {
    let dir = setup_mald_home();
    // Set Ollama URL to something that definitely won't connect
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("config")
        .arg("set")
        .arg("ai.ollama_url")
        .arg("http://127.0.0.1:1")
        .assert()
        .success();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("ai")
        .arg("chat")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot connect to Ollama"))
        .stderr(predicate::str::contains("ollama serve"));
}
