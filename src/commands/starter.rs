use anyhow::Result;
use std::path::Path;

use crate::fs::ensure_directory;

const STARTER_NOTES: &[(&str, &str)] = &[
    (
        "index.md",
        r#"---
title: __SPACE_NAME__
created: 2026-04-20
tags: [mald, guide, starter]
---

# __SPACE_NAME__

This starter space is here to make MALD feel obvious on day one.

## Start Here

- [[inbox]] keeps quick thoughts lightweight.
- [[how-wikilinks-work]] shows how plain Markdown links become connected notes.
- [[projects/first-project]] proves folders, backlinks, tasks, and graph links all work together.
- [[search-and-review]] gives you phrases to test search, tasks, and the graph without inventing your own examples first.

## Core Workflow

- Open MALD with `mald`
- Create a new note with `mald new "Title"`
- Capture a thought with `mald q a quick idea`
- Search everything with `mald search`
- Open the terminal UI with `mald tui`

## Try These

- [ ] Open [[how-wikilinks-work]] and follow one link
- [ ] Search for `amber harbor review`
- [ ] Create one note in `projects/`
- [ ] Add one `[[wikilink]]` to connect your own notes
"#,
    ),
    (
        "inbox.md",
        r#"---
title: Inbox
created: 2026-04-20
tags: [starter, inbox]
---

# Inbox

Use this note for thoughts that arrive faster than structure.

- Remember to compare the graph before and after adding one more link.
- Amber harbor review is a good phrase to test search quality.
- [ ] Turn one loose thought into a proper note

Captured ideas often grow into [[first-project]] or point back to [[how-wikilinks-work]] when you want to connect them properly.
"#,
    ),
    (
        "how-wikilinks-work.md",
        r#"---
title: How Wikilinks Work
created: 2026-04-20
tags: [starter, wikilinks, guide]
---

# How Wikilinks Work

MALD reads ordinary Markdown files. You create relationships just by typing wikilinks directly into the note text.

## The syntax

Write links like this inside a Markdown file:

```md
- [[inbox]]
- [[first-project]]
- [[search-and-review]]
```

That is enough for MALD to understand that this note connects to those notes.

## What MALD gives you back

- The graph can show the connection
- Backlinks can show which notes point here
- Search still works because these are plain Markdown files

## Follow the links

- [[inbox]]
- [[first-project]]
- [[search-and-review]]

If you open one of those notes, it should be obvious that the connection came from simple Markdown, not hidden metadata.
"#,
    ),
    (
        "search-and-review.md",
        r#"---
title: Search And Review
created: 2026-04-20
tags: [starter, search, review]
---

# Search And Review

This note exists to make MALD easy to test.

## Search phrases

- amber harbor review
- meeting rhythm reset
- corridor memory sketch
- nebula lattice

## Review prompts

- [ ] Search for `meeting rhythm`
- [ ] Open the graph and look for the strongest hub
- [ ] Check which notes link back to [[how-wikilinks-work]]

Related notes:

- [[index]]
- [[inbox]]
- [[first-project]]
"#,
    ),
    (
        "projects/first-project.md",
        r#"---
title: First Project
created: 2026-04-20
tags: [starter, project]
---

# First Project

This note lives in a folder on purpose. It teaches you that spaces can stay organized without losing link-based navigation.

## Why it matters

- Folders help you browse
- Wikilinks help you think
- Backlinks help you recover context

## Next actions

- [ ] Add one real project note beside this one
- [ ] Link it back to [[index]]
- [ ] Search for `nebula lattice`

See also:

- [[index]]
- [[inbox]]
- [[how-wikilinks-work]]
- [[search-and-review]]
"#,
    ),
];

pub fn seed_starter_space(kb_path: &Path, kb_name: &str) -> Result<()> {
    ensure_directory(kb_path)?;

    for (relative_path, template) in STARTER_NOTES {
        let note_path = kb_path.join(relative_path);
        if note_path.exists() {
            continue;
        }
        if let Some(parent) = note_path.parent() {
            ensure_directory(parent)?;
        }
        let content = template.replace("__SPACE_NAME__", kb_name);
        std::fs::write(note_path, content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn starter_space_seeds_wikilink_guide_and_project_note() {
        let temp = TempDir::new().unwrap();
        let kb_path = temp.path().join("personal");
        seed_starter_space(&kb_path, "personal").unwrap();

        let index = kb_path.join("index.md");
        let wikilinks = kb_path.join("how-wikilinks-work.md");
        let project = kb_path.join("projects").join("first-project.md");

        assert!(index.exists());
        assert!(wikilinks.exists());
        assert!(project.exists());

        let index_content = std::fs::read_to_string(index).unwrap();
        assert!(index_content.contains("[[how-wikilinks-work]]"));

        let wikilink_content = std::fs::read_to_string(wikilinks).unwrap();
        assert!(wikilink_content.contains("[[inbox]]"));
        assert!(wikilink_content.contains("plain Markdown files"));
    }
}
