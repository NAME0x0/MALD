<p align="center">
  <img src="assets/favicon.svg" width="96" alt="MALD icon">
</p>

<h1 align="center">MALD</h1>

<p align="center">
  Local-first markdown knowledge workspace with a native desktop app, terminal UI, and optional local AI.
</p>

<p align="center">
  <a href="https://github.com/NAME0x0/MALD/releases/tag/v0.5.4"><img alt="Version" src="https://img.shields.io/badge/version-v0.5.4-black"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-black">
  <img alt="Surfaces" src="https://img.shields.io/badge/surfaces-GUI%20%7C%20TUI%20%7C%20CLI-black">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-black"></a>
</p>

MALD keeps your notes as plain markdown files on disk while adding fast search, backlinks, graph navigation, task extraction, local publishing, and grounded AI chat over your own spaces. It is designed for people who want one serious local workspace instead of one app for notes, another for tasks, another for search, and another for AI.

## Why MALD Exists

Most knowledge tools force one of two tradeoffs:

- they feel good but hide your files behind a hosted product
- they keep your files local but leave you with a pile of disconnected scripts and plugins

MALD is the middle path:

- local-first and plain-file based
- usable as a desktop app, a terminal UI, and a CLI
- knowledge-base aware instead of assuming one giant vault
- optional AI, not mandatory AI
- inspectable by default, not black-boxed

## Who MALD Is For

MALD is a good fit if you want:

- a serious markdown workspace that stays on your machine
- separate spaces for personal notes, client work, research, or study
- a native desktop app without giving up terminal workflows
- plain files, wikilinks, tasks, search, and graph views in one place
- local AI that works over your notes without becoming the whole product

MALD is probably not for you if you want:

- real-time multiplayer editing
- a cloud-first collaborative docs suite
- a block editor or database-style workspace
- a product that completely hides the file system

## What You Get

- Desktop app for notes, graph, search, tasks, AI chat, and workspace switching
- Terminal UI for keyboard-first workflows
- CLI for capture, automation, import/export, diagnostics, and scripting
- Markdown notes with wikilinks, tags, templates, and task extraction
- Multiple spaces inside one MALD workspace
- Optional local AI over your own notes with inspectable citations

## Install

### Windows

PowerShell install script:

```powershell
powershell -c "irm https://raw.githubusercontent.com/NAME0x0/MALD/main/install.ps1 | iex"
```

Scoop:

```powershell
scoop bucket add mald https://github.com/NAME0x0/scoop-mald
scoop install mald
```

Downloaded the standalone Windows EXE instead of using Scoop?

- Run `mald.exe` once.
- Then either:
  - run `mald setup path`
  - or open Settings in the GUI and click **Add MALD to PATH**

That installs the current binary into `%LOCALAPPDATA%\mald` and makes `mald` work in new Command Prompt and PowerShell windows.

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/NAME0x0/MALD/main/install.sh | sh
```

### Build From Source

If you want a local release build only:

```bash
cargo build --release --features gui
```

If you want `mald` available on your `PATH` from a local checkout:

```bash
cargo install --path . --features gui
```

If you want to install directly from Git:

```bash
cargo install --git https://github.com/NAME0x0/MALD
```

## The Core Model

MALD uses two layers:

- **Workspace**: the root MALD directory (`MALD_HOME`)
- **Spaces**: separate note collections inside `MALD_HOME/kb/`

Example:

```text
MALD_HOME/
├── kb/
│   ├── personal/
│   ├── work/
│   └── research/
├── config/
├── index/
├── sessions/
├── templates/
├── cache/
└── logs/
```

One workspace can hold multiple spaces. MALD always has a **working space**. That working space is the default target for:

- `mald`
- `mald gui`
- `mald new`
- `mald today`
- AI chat and indexing commands that rely on the active space

## Launch MALD

These are the main entrypoints:

| Surface | Command | Use it when |
| --- | --- | --- |
| Desktop app | `mald` | You want the full GUI and the default entrypoint |
| Desktop app | `mald gui` | You want to force the GUI explicitly |
| Space picker + GUI | `mald launch` | You want to choose a space with arrow keys and enter |
| Fuzzy space launch | `mald launch client acme` | You know part of the space name but not the exact string |
| Terminal UI | `mald tui` | You want the keyboard-first TUI |
| Text dashboard | `mald status` | You want a fast non-interactive overview |
| Editor handoff | `mald open` | You want the current space opened in your configured editor |

### Fastest Way To Start In The Right Space

If you want MALD to ask which space to use:

```bash
mald launch
```

What happens:

1. MALD ranks your spaces based on what you typed and your current context.
2. You move with the arrow keys.
3. Press `Enter`.
4. MALD makes that space the working space and launches the desktop app there.

If you know enough of the name already:

```bash
mald launch research
mald launch client acme
mald launch prod notes
```

MALD will fuzzy-match the space name. If the match is unique, it launches directly. If there is more than one plausible match and your terminal is interactive, MALD opens the picker so you can choose.

## Check Or Change The Working Space

From the CLI:

```bash
mald kb list
mald kb current
mald kb use work
mald kb use client acme
mald kb open
mald kb open client acme
```

What these are for:

- `mald kb list`: inspect everything MALD knows about in the current workspace
- `mald kb current`: confirm what MALD will use by default
- `mald kb use work`: set the working space without launching the app
- `mald kb use client acme`: fuzzy-match a multiword space name without quoting it
- `mald kb open`: pick a space and open it in your external editor
- `mald kb open client acme`: fuzzy-open the right space in your editor from a few words

From the GUI:

- use the **Working Space** section on the home screen
- or press `Ctrl+P` and search for `Switch Space`

## Use MALD In A Specific Directory

MALD stores everything inside `MALD_HOME`.

Defaults:

- Windows: `%USERPROFILE%\\.mald`
- macOS / Linux: `~/.mald`

If you want a separate workspace for a project, repo, or client, point `MALD_HOME` at a directory and initialize there.

### PowerShell

```powershell
$env:MALD_HOME = "$PWD\.mald"
mald init
mald launch
```

### Bash / Zsh

```bash
export MALD_HOME="$PWD/.mald"
mald init
mald launch
```

For a one-off session:

```powershell
$env:MALD_HOME = "D:\Work\client-a\.mald"
mald launch
```

```bash
MALD_HOME="$HOME/work/client-a/.mald" mald launch
```

This is the cleanest way to keep separate MALD workspaces for different domains without mixing them together.

## Quick Start

```bash
mald init
mald launch
mald kb current
mald new "Project Brief"
mald new "Incident Review" --path projects/incidents
mald q "Follow up with design review"
mald search "brief"
```

`mald init` creates:

- config, templates, sessions, cache, and index directories
- at least one usable space
- a starter note so the workspace is not empty

## Workflow Examples

### 1. Personal Notes And Daily Capture

```bash
mald launch
mald today
mald q "Book dentist appointment"
mald new "Weekly Review"
mald tasks
```

Use this when MALD is acting as your main local note and task layer.

### 2. Client Or Project Workspace

```powershell
$env:MALD_HOME = "D:\Clients\acme\.mald"
mald init
mald kb create delivery
mald kb create research
mald launch
```

Use this when you want one MALD workspace per client, and several spaces inside that client workspace.

### 3. Research Or Study Space

```bash
mald kb create research
mald kb use research
mald import ~/papers --kb research
mald new "Transformer Notes" --kb research --path nlp/transformers
mald backlinks transformer-notes
mald graph stats
```

Use this when you want a clean boundary around reading notes, source material, and study questions.

### 4. GUI-First Space Switching

```bash
mald
```

Then inside the app:

- use the **Working space** bar at the top of the app
- switch space from the home screen
- or press `Ctrl+P`
- type `Switch Space`
- hit `Enter` on the space you want

Use this when you stay in the desktop app most of the time and do not want to remember space commands.

### 5. Terminal-Only Session

```bash
mald tui
```

Then inside the TUI:

- press `n` to create a note with space and folder selection
- press `s` to switch spaces with ranking + fuzzy match
- press `/` to search without leaving the TUI
- press `d` on Home to open the safe demo space

Use this when you want a terminal workflow but still want MALD to understand notes, links, and tasks.

### 6. Safe Demo Workspace

```bash
mald
```

Then inside the app:

- click **Try demo space**
- or press `Ctrl+P`
- type `demo`
- open the sample notes and inspect graph, search, and tasks without touching your real notes

Use this when you want to learn MALD safely before you start organizing real material.

## Command Guide

This is the user-facing command surface. Hidden internal commands are omitted on purpose.

### Launch And Navigation

| Command | What it does | Use it when |
| --- | --- | --- |
| `mald` | Opens the desktop app | You want the default MALD experience |
| `mald gui` | Explicitly opens the desktop app | You want to force the GUI from a script, alias, or habit |
| `mald launch [space words]` | Picks or fuzzy-resolves a space, then launches MALD there | You want the right space without typing the full exact name |
| `mald init` | Creates the MALD workspace structure | You are setting up a new workspace |
| `mald setup` | Runs the guided onboarding wizard | You want editor, space, and baseline setup handled for you |
| `mald setup editor [editor words]` | Picks or auto-detects an editor like VS Code or Neovim | You want MALD to handle editor setup without path knowledge |
| `mald setup path` | Installs the current MALD binary into a stable location and adds it to PATH on Windows | You downloaded the standalone EXE or `mald` is not found in new terminals |
| `mald status` | Prints a non-interactive text dashboard | You want a quick overview in the terminal |
| `mald tui` | Opens the terminal UI | You want MALD without the desktop app |
| `mald help-topic <topic>` | Shows focused help for one area | You need help on AI, sync, search, templates, graph, or tasks |
| `mald doctor` | Runs diagnostics | Something feels broken or unclear and you want MALD to inspect itself |
| `mald update` | Checks for updates and self-update paths | You want to see whether MALD is behind |

### Notes And Capture

| Command | What it does | Use it when |
| --- | --- | --- |
| `mald new "Title"` | Creates a new note in the working space | You want a clean new note quickly |
| `mald new "Title" --kb work --path projects/api` | Creates a note in a specific space subdirectory | You want nested note placement without manually navigating folders |
| `mald today` | Opens or creates today’s daily note | You use MALD as a daily journal or capture surface |
| `mald capture ...` / `mald q ...` | Appends quick text to today’s note | You want a fast capture path with minimal friction |
| `mald find <query>` | Fuzzy-finds and opens a note | You know roughly what the note is called |
| `mald edit <query>` | Fuzzy-finds and opens a note in your editor | You want to jump straight into external editing |
| `mald rename <old> <new>` | Renames a note and updates wikilinks | You are cleaning up note structure without breaking links |
| `mald open` | Opens the working space in your configured editor | You want folder-level editing outside MALD |
| `mald info <note>` | Shows metadata for a note | You want note-level details without opening the file |
| `mald template ...` | Lists, creates, edits, or uses note templates | You create repeatable note types often |

### Knowledge Bases, Search, And Review

| Command | What it does | Use it when |
| --- | --- | --- |
| `mald kb list` | Lists spaces in the current workspace | You want to see what exists before switching |
| `mald kb current` | Shows the working space | You want to verify what MALD will target next |
| `mald kb use [name]` | Sets the working space, with interactive picker support when no name is given | You want to change the default space without launching the GUI |
| `mald kb open [name]` | Opens a space in your editor, with picker support when interactive | You want the files, not the app |
| `mald space ...` | Alias for `mald kb ...` | You prefer user-facing wording over the shorter internal alias |
| `mald search "query"` | Full-text search across notes | You know the content you want, not the file name |
| `mald links <note>` | Shows outgoing links from a note | You want to inspect what a note points to |
| `mald backlinks <note>` | Shows incoming links to a note | You want to see what references a note |
| `mald orphans` | Finds notes with no incoming links | You are cleaning up isolated notes |
| `mald tags` | Lists tags or filters notes by tag | You use tag-driven navigation |
| `mald tasks` | Aggregates tasks from notes | You want actionable items without leaving markdown |
| `mald review` | Surfaces recent activity, stale notes, orphans, and broken links | You want a maintenance or weekly review pass |
| `mald graph ...` | Runs graph analysis commands | You want structural insight into the current space |
| `mald fix-links` | Detects and optionally fixes broken wikilinks | You renamed or moved notes and want to repair link structure |
| `mald reindex` | Rebuilds the search index | Search feels stale or you imported a lot of content |

### Preview, Import, Export, And Automation

| Command | What it does | Use it when |
| --- | --- | --- |
| `mald preview <note>` | Renders a note in the terminal | You want to inspect formatting quickly without opening the GUI |
| `mald run <note>` | Lists or executes code blocks from a note | You keep runnable snippets in markdown and want controlled execution |
| `mald import <folder>` | Imports markdown into a space | You are bringing in Obsidian, Logseq, or loose notes |
| `mald export ...` | Exports notes as HTML or markdown | You want a portable output or publishing handoff |
| `mald serve` | Serves the active space as a local site | You want a lightweight local web view |
| `mald sync ...` | Handles git-based versioning and sync | You want history, backup, or machine-to-machine sync |

### AI And System Control

| Command | What it does | Use it when |
| --- | --- | --- |
| `mald ai setup` | Installs or configures local AI prerequisites | You want Ollama-backed AI features |
| `mald ai chat "..."` | Chats over your active space with citations | You want grounded answers from your notes |
| `mald ai index <kb>` | Builds embeddings for a space | You want RAG quality over a specific space |
| `mald config get/set ...` | Reads or writes MALD config values | You want to change editor, AI model, shell, or hooks directly |
| `mald daemon status` | Checks background daemon state | You want to inspect indexing/runtime support services |

## Optional Local AI

AI is optional. MALD remains useful without it.

When you want local AI features, MALD integrates with Ollama:

```bash
mald ai setup
mald ai index personal
mald ai chat "What did I write about authentication?"
```

MALD is designed so AI stays grounded in your files and subordinate to your workspace instead of replacing it.

## Troubleshooting The Most Common Intent

| If you want to... | Use |
| --- | --- |
| open MALD and choose a space first | `mald launch` |
| open MALD in a space when you only remember part of the name | `mald launch client acme` |
| verify which space MALD is currently using | `mald kb current` |
| switch the default space without opening the GUI | `mald kb use work` |
| switch the default space with only part of a multiword name | `mald kb use client acme` |
| switch the default space with a picker instead of typing a name | `mald kb use` |
| open a space directly in your external editor | `mald kb open` |
| pick VS Code or Neovim without typing a full path | `mald setup editor` |
| fix the `mald` command in new Windows terminals | `mald setup path` |
| use MALD in a project-local directory | set `MALD_HOME`, then run `mald launch` |
| get contextual help after a bad command | read MALD’s `Did you mean ...` hint, then run `mald --help` if needed |

## Philosophy

MALD is built on a few non-negotiable ideas:

- your notes should stay in plain files
- local tools should feel fast, native, and inspectable
- AI should be optional, grounded, and subordinate to the notes
- multiple spaces should feel like a first-class workflow, not a workaround
- the same workspace should be usable from the GUI, the TUI, and scripts

## License

MIT
