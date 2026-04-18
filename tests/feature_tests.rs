use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn mald_cmd() -> Command {
    cargo_bin_cmd!("mald")
}

fn setup_mald_home() -> TempDir {
    let dir = TempDir::new().unwrap();
    // Run init — MALD_HOME is passed per-command via .env(), not set globally
    // (set_var is unsound when tests run in parallel across threads)
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
    let note_path = dir
        .path()
        .join("kb")
        .join("personal")
        .join("test-broken.md");
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

// --- Unquoted capture tests ---

#[test]
fn test_capture_unquoted() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("buy")
        .arg("groceries")
        .arg("tomorrow")
        .assert()
        .success()
        .stdout(predicate::str::contains("Captured to"));
}

#[test]
fn test_capture_unquoted_with_tag() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("--tag")
        .arg("work")
        .arg("fix")
        .arg("the")
        .arg("auth")
        .arg("bug")
        .assert()
        .success()
        .stdout(predicate::str::contains("Captured to"));
}

#[test]
fn test_capture_quoted_still_works() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("this is one quoted string")
        .assert()
        .success()
        .stdout(predicate::str::contains("Captured to"));
}

#[test]
fn test_capture_content_written() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("unique-capture-test-string-xyz")
        .assert()
        .success();

    // Verify the text ended up in the daily note
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_path = dir
        .path()
        .join("kb")
        .join("personal")
        .join(format!("{today}.md"));
    let content = fs::read_to_string(&daily_path).unwrap();
    assert!(content.contains("unique-capture-test-string-xyz"));
}

#[test]
fn test_capture_with_tag_content() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("--tag")
        .arg("inbox")
        .arg("tagged-capture-test")
        .assert()
        .success();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_path = dir
        .path()
        .join("kb")
        .join("personal")
        .join(format!("{today}.md"));
    let content = fs::read_to_string(&daily_path).unwrap();
    assert!(content.contains("tagged-capture-test"));
    assert!(content.contains("#inbox"));
}

// --- Find command tests ---

#[test]
fn test_find_single_result_no_editor() {
    // find with --json doesn't exist, but we can test that it finds things
    // by checking search --json which find uses internally
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("search")
        .arg("Getting Started")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Getting Started"));
}

#[test]
fn test_find_no_results() {
    let dir = setup_mald_home();
    // find with a query that matches nothing should fail
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("find")
        .arg("xyznonexistent999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No results"));
}

// --- Help topic shortcuts ---

#[test]
fn test_help_topic_shortcuts() {
    mald_cmd()
        .arg("help-topic")
        .arg("shortcuts")
        .assert()
        .success()
        .stdout(predicate::str::contains("SHELL ALIASES"))
        .stdout(predicate::str::contains("FZF INTEGRATION"))
        .stdout(predicate::str::contains("NEOVIM PLUGIN"));
}

#[test]
fn test_help_topic_aliases_synonym() {
    mald_cmd()
        .arg("help-topic")
        .arg("aliases")
        .assert()
        .success()
        .stdout(predicate::str::contains("SHELL ALIASES"));
}

// --- JSON output edge cases ---

#[test]
fn test_search_json_empty_result() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("search")
        .arg("xyznonexistent999")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_tasks_json_empty_kb() {
    let dir = TempDir::new().unwrap();
    // Create minimal MALD home with empty KB
    fs::create_dir_all(dir.path().join("kb").join("personal")).unwrap();
    fs::create_dir_all(dir.path().join("config")).unwrap();
    fs::write(
        dir.path().join("config").join("config.json"),
        r#"{"default_kb":"personal"}"#,
    )
    .unwrap();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_tags_json_with_tagged_notes() {
    let dir = setup_mald_home();
    // Create a note with specific tags
    let note_path = dir
        .path()
        .join("kb")
        .join("personal")
        .join("tagged-note.md");
    fs::write(&note_path, "# Tagged\n\n#rust #programming some content\n").unwrap();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tags")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"rust\""))
        .stdout(predicate::str::contains("\"programming\""));
}

#[test]
fn test_tags_filter_json() {
    let dir = setup_mald_home();
    // The init sample has #mald tag
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tags")
        .arg("mald")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("index")); // the sample note stem
}

#[test]
fn test_tags_filter_json_no_match() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tags")
        .arg("nonexistenttag999")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_kb_list_json_structure() {
    let dir = setup_mald_home();
    // Create a second KB
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("kb")
        .arg("create")
        .arg("work")
        .assert()
        .success();

    let output = mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("kb")
        .arg("list")
        .arg("--json")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.len() >= 2);

    // Check that personal is marked as default
    let personal = parsed.iter().find(|v| v["name"] == "personal").unwrap();
    assert_eq!(personal["default"], true);
}

#[test]
fn test_search_json_valid_json() {
    let dir = setup_mald_home();
    let output = mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("search")
        .arg("getting")
        .arg("--json")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty());
    // Each result must have type and path
    for item in arr {
        assert!(item.get("type").is_some());
        assert!(item.get("path").is_some());
    }
}

#[test]
fn test_tasks_json_valid_structure() {
    let dir = setup_mald_home();
    let output = mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("tasks")
        .arg("--json")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(!parsed.is_empty());
    for item in &parsed {
        assert!(item.get("task").is_some());
        assert!(item.get("note").is_some());
        assert!(item.get("kb").is_some());
        assert_eq!(item["done"], false);
    }
}

// --- Multiple captures to same daily note ---

#[test]
fn test_multiple_captures_append() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("first-capture-aaa")
        .assert()
        .success();

    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .arg("second-capture-bbb")
        .assert()
        .success();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_path = dir
        .path()
        .join("kb")
        .join("personal")
        .join(format!("{today}.md"));
    let content = fs::read_to_string(&daily_path).unwrap();
    assert!(content.contains("first-capture-aaa"));
    assert!(content.contains("second-capture-bbb"));
}

// --- Find command visible in help ---

#[test]
fn test_find_in_help() {
    mald_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("find"));
}

// --- Capture requires at least one word ---

#[test]
fn test_capture_empty_fails() {
    let dir = setup_mald_home();
    mald_cmd()
        .env("MALD_HOME", dir.path())
        .arg("q")
        .assert()
        .failure();
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
        .stderr(predicate::str::contains("mald setup ai"));
}
