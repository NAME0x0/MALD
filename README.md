# MALD

MALD is a local-first markdown knowledge workspace with a native desktop app, a terminal UI, and optional local AI.

It keeps your notes as plain files on disk while adding fast search, backlinks, graph navigation, task extraction, local publishing, and cited RAG chat. There is no required cloud account, no Electron shell, and no hosted sync service sitting between you and your data.

## What MALD Is For

MALD is a good fit if you want:

- a serious markdown workspace that stays on your machine
- a native desktop experience without giving up CLI power
- local knowledge management for research, study, writing, or software projects
- plain files, knowledge bases, wikilinks, tasks, and inspectable AI answers
- one tool that works as a GUI, a TUI, and a command-line utility

MALD is probably not for you if you want:

- a cloud-first collaborative docs product
- real-time multiplayer editing
- a block editor or database-style workspace
- a product that hides the file system completely

## What You Get

- Desktop app for browsing notes, graph, search, tasks, AI chat, and settings
- Terminal UI for keyboard-first workflows
- Plain CLI for capture, automation, import/export, sync, diagnostics, and scripting
- Markdown notes with wikilinks, tags, templates, and task extraction
- Full-text search plus optional local AI over your own knowledge base
- Background daemon for indexing and health-aware workspace services

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

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/NAME0x0/MALD/main/install.sh | sh
```

### Build From Source

From the checked-out repository:

```bash
cargo build --release
```

Or install directly from Git:

```bash
cargo install --git https://github.com/NAME0x0/MALD
```

## Launch MALD

Release binaries ship with the desktop app enabled by default.

| Surface | Command | Best for |
| --- | --- | --- |
| Desktop app | `mald` | Full MALD workspace with graph, search, editor, tasks, and AI |
| Terminal UI | `mald hub` | Keyboard-first terminal workflow |
| Terminal UI | `mald --tui` | Same as `hub`, but explicit |
| Text dashboard | `mald status` | Quick non-interactive status view |
| Editor handoff | `mald open` | Open the active knowledge base in your configured editor |

## Use MALD In A Specific Directory

MALD works from a workspace directory called `MALD_HOME`.

By default, MALD stores its data in:

- Windows: `%USERPROFILE%\\.mald`
- macOS / Linux: `~/.mald`

If you want MALD to live inside a specific project or directory, point `MALD_HOME` there and initialize it once.

### PowerShell

```powershell
$env:MALD_HOME = "$PWD\.mald"
mald init
mald
```

### Bash / Zsh

```bash
export MALD_HOME="$PWD/.mald"
mald init
mald
```

If the workspace already exists, you can skip `mald init`.

For one-off launches, set `MALD_HOME` and start MALD in the same shell session:

```powershell
$env:MALD_HOME = "D:\Work\client-a\.mald"
mald
```

```bash
MALD_HOME="$HOME/work/client-a/.mald" mald
```

This is the cleanest way to keep separate MALD workspaces for different clients, repos, or domains.

## Quick Start

```bash
mald init
mald
mald new "Project Brief"
mald q "Follow up with design review"
mald search "brief"
mald open
```

`mald init` creates a ready-to-use workspace with:

- a default `personal` knowledge base
- config, templates, sessions, cache, and index directories
- an initial markdown note so the workspace is not empty

## Common Workflows

### Capture And Notes

```bash
mald new "Meeting Notes"
mald today
mald q "remember to update README"
mald edit meeting
```

### Search, Links, And Review

```bash
mald search "authentication"
mald backlinks auth-design
mald links auth-design
mald review
mald graph stats
```

### Import Existing Markdown

```bash
mald import ~/ObsidianVault
mald import ~/notes --kb research
```

### Serve Or Export

```bash
mald serve
mald export my-note
mald export --all --output-dir ./mald-export
```

## Optional Local AI

AI is optional. MALD remains useful without it.

When you want local AI features, MALD integrates with Ollama:

```bash
mald ai setup
mald ai index personal
mald ai chat "What did I write about authentication?"
```

AI responses are grounded in your local workspace, and MALD is designed around inspectable results rather than opaque chat output.

## Workspace Layout

```text
MALD_HOME/
├── kb/           Knowledge bases and markdown notes
├── config/       Configuration
├── index/        Search and retrieval data
├── sessions/     Chat and session history
├── templates/    Note templates
├── plugins/      Custom executable plugins
├── cache/        Local cache
└── logs/         Daemon and runtime logs
```

## Useful Commands

```bash
mald doctor
mald daemon status
mald kb list
mald tasks --json
mald search "query" --json
mald sync init
mald sync
```

## Philosophy

MALD is built on a simple premise:

- your knowledge base should stay in plain files
- local tools should feel fast and inspectable
- AI should be optional, grounded, and subordinate to the data
- the same workspace should be usable from a desktop app, a terminal UI, and scripts

## License

MIT
