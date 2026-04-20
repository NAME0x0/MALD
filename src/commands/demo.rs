use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

#[derive(Debug, Clone)]
pub struct DemoSpace {
    pub kb_name: String,
    pub entry_note: PathBuf,
}

const DEMO_KB: &str = "demo";

const DEMO_NOTES: &[(&str, &str)] = &[
    (
        "index.md",
        r#"---
title: Demo Space
created: 2026-04-19
tags: [mald, demo, guide]
---

# Demo Space

This space exists so you can explore MALD safely and see what the product can do without touching your real notes.

## Start Here

- Open [[capture-inbox]] to see quick capture patterns.
- Open [[how-wikilinks-work]] to see how plain Markdown creates connections.
- Open [[search-playground]] and search for `nebula lattice`.
- Open [[tasks-and-review]] to see tasks aggregated from notes.
- Open [[graph-tour]] to see how links connect across the space.
- Open [[project-brief]] to see a nested project note.
- Open [[daily-note-demo]] to see how ordinary notes, tasks, and links can coexist.

## Try These

- [ ] Search for `meeting rhythm`
- [ ] Open the graph and find the strongest hub
- [ ] Open a note, then follow a wikilink instead of using the file tree
- [ ] Create one note in `projects/` and link it back to [[index]]
- [ ] Switch back to your own space when you are done
"#,
    ),
    (
        "capture-inbox.md",
        r#"---
title: Capture Inbox
created: 2026-04-19
tags: [demo, inbox]
---

# Capture Inbox

Use this note to understand MALD's quick-capture style.

- Short thoughts land here before they become proper notes.
- [[project-brief]] is where captured ideas become structured work.
- [[tasks-and-review]] is where open loops become visible again.

## Example Captures

- Remember the meeting rhythm change for next Tuesday.
- Nebula lattice is the phrase to test search quality.
- Ask whether [[daily-note-demo]] should become a reusable template.
- [ ] Turn one capture into a proper note
"#,
    ),
    (
        "how-wikilinks-work.md",
        r#"---
title: How Wikilinks Work
created: 2026-04-19
tags: [demo, wikilinks, guide]
---

# How Wikilinks Work

MALD uses plain Markdown files. You create note relationships by typing wikilinks directly into the note body.

## Example

```md
- [[capture-inbox]]
- [[project-brief]]
- [[daily-note-demo]]
```

That is enough for MALD to:

- show links in the graph
- compute backlinks
- keep everything readable as normal Markdown on disk

## Follow These Links

- [[capture-inbox]]
- [[project-brief]]
- [[daily-note-demo]]
- [[search-playground]]

If you inspect those files in an editor, the links are just Markdown text. MALD reads the structure from the notes themselves.
"#,
    ),
    (
        "search-playground.md",
        r#"---
title: Search Playground
created: 2026-04-19
tags: [demo, search]
---

# Search Playground

This note is intentionally full of unique phrases so search has something obvious to find.

## Unique phrases

- nebula lattice
- amber harbor review
- corridor memory sketch
- meeting rhythm reset
- shoreline draft memo

[[graph-tour]] links this note into the rest of the demo space.
[[how-wikilinks-work]] explains why these phrases are enough to test the app.
"#,
    ),
    (
        "graph-tour.md",
        r#"---
title: Graph Tour
created: 2026-04-19
tags: [demo, graph]
---

# Graph Tour

The graph becomes useful when notes point at each other on purpose.

- [[index]]
- [[capture-inbox]]
- [[search-playground]]
- [[tasks-and-review]]
- [[project-brief]]
- [[how-wikilinks-work]]
- [[daily-note-demo]]

If you open the graph in MALD, this note should act like a visible connector.
"#,
    ),
    (
        "workflows/tasks-and-review.md",
        r#"---
title: Tasks And Review
created: 2026-04-19
tags: [demo, tasks]
---

# Tasks And Review

MALD pulls tasks from regular markdown notes. Nothing special is required.

- [ ] Confirm the graph is readable
- [ ] Search for `amber harbor review`
- [ ] Create a new note inside this demo space
- [ ] Switch spaces and come back without fear
- [ ] Follow one backlink from [[how-wikilinks-work]]

See also [[index]], [[project-brief]], and [[daily-note-demo]].
"#,
    ),
    (
        "daily-note-demo.md",
        r#"---
title: Daily Note Demo
created: 2026-04-19
tags: [demo, daily, routine]
---

# Daily Note Demo

This note mixes journal-like writing, tasks, and references on purpose.

## Notes

- Client sync moved to Thursday afternoon.
- The phrase `shoreline draft memo` is here to test search.
- [[capture-inbox]] should receive loose notes before they become durable notes.

## Tasks

- [ ] Turn the client sync into a project note
- [ ] Link the final note back to [[project-brief]]

## Related

- [[index]]
- [[tasks-and-review]]
- [[graph-tour]]
"#,
    ),
    (
        "projects/project-brief.md",
        r#"---
title: Project Brief
created: 2026-04-19
tags: [demo, project]
---

# Project Brief

This note lives in a nested folder so you can test directory browsing.

## Why it exists

- to prove nested notes work
- to show backlinks and graph links across folders
- to make `--path` workflows feel real
- to show that [[how-wikilinks-work]] still works across folders

Related notes:

- [[capture-inbox]]
- [[tasks-and-review]]
- [[search-playground]]
- [[how-wikilinks-work]]
- [[daily-note-demo]]
"#,
    ),
];

pub fn ensure_demo_space(reset: bool) -> Result<DemoSpace> {
    seed_demo_space_at(&mald_home(), reset, true)
}

fn seed_demo_space_at(home: &Path, reset: bool, reindex: bool) -> Result<DemoSpace> {
    let kb_root = home.join("kb");
    ensure_directory(&kb_root)?;

    let kb_path = kb_root.join(DEMO_KB);
    if reset && kb_path.exists() {
        std::fs::remove_dir_all(&kb_path)?;
    }
    ensure_directory(&kb_path)?;

    for (relative_path, content) in DEMO_NOTES {
        let note_path = kb_path.join(relative_path);
        if let Some(parent) = note_path.parent() {
            ensure_directory(parent)?;
        }
        if reset || !note_path.exists() {
            std::fs::write(&note_path, content)?;
        }
    }

    if reindex {
        crate::daemon::indexer::fts_index_kb(&home.join("kb"))?;
    }

    Ok(DemoSpace {
        kb_name: DEMO_KB.into(),
        entry_note: kb_path.join("index.md"),
    })
}

pub fn activate_demo_space(reset: bool) -> Result<DemoSpace> {
    let demo = ensure_demo_space(reset)?;
    let config_path = mald_home().join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;
    config.set(
        "default_kb",
        serde_json::Value::String(demo.kb_name.clone()),
    )?;
    Ok(demo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ensure_demo_space_creates_entry_note() {
        let temp = TempDir::new().unwrap();
        let demo = seed_demo_space_at(temp.path(), true, false).unwrap();
        assert_eq!(demo.kb_name, "demo");
        assert!(demo.entry_note.exists());
        let content = std::fs::read_to_string(demo.entry_note).unwrap();
        assert!(content.contains("Demo Space"));
        let wikilinks = temp
            .path()
            .join("kb")
            .join("demo")
            .join("how-wikilinks-work.md");
        assert!(wikilinks.exists());
    }
}
