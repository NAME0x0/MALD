# MALD — User Journey & Surface Inventory

Audit basis. Every page/state listed with what must render in each region (top bar, activity bar, left sidebar, main pane, right context pane, status footer). Use this when reviewing screenshots or running QA passes — gap = bug.

---

## 1. High-level flow

```mermaid
flowchart TD
    Start([Launch `mald`]) --> Lock{Single-instance<br/>lock acquired?}
    Lock -- no --> Toast[Toast: GUI already running] --> ExitDup([Exit])
    Lock -- yes --> HasHome{`~/.mald/` exists?}
    HasHome -- no --> Wizard[First-run wizard<br/>CLI prompt: editor + KB]
    HasHome -- yes --> GuiBoot[GUI boot]
    Wizard --> GuiBoot

    GuiBoot --> Home[Home view<br/>Quick actions]
    Home -->|click note| Editor[Editor + tabs]
    Home -->|Ctrl+P| Palette[Command palette]
    Home -->|Ctrl+Shift+F| Search[Vault search]
    Home -->|click Ask MALD| AskMald[Ask MALD chat tab]

    Editor -->|click [[wikilink]]| Editor
    Editor -->|click backlink| Editor
    Editor -->|cmd-S| Saved[Saved → timeline refresh]
    Saved --> Editor
    Editor -->|click chat icon| AskMald

    AskMald -->|pick mode pill| Mode{Smart / Deep / Focus}
    Mode --> Send[Send → stream answer]
    Send --> Streaming[Streaming chunks +<br/>Meta event surfaces sources/related]
    Streaming -->|done| Extract[Background LLM extracts<br/>concepts/tasks/questions]
    Extract --> Context[Right pane Context tab populated]
    Context -->|click source / related| Editor

    Palette -->|run command| Editor
    Palette -->|switch KB| Reload[Reload tree + graph + tasks]
    Reload --> Home

    Search -->|select hit| Editor

    Editor -->|sidebar Graph| Graph[Graph canvas]
    Editor -->|sidebar Tasks| Tasks[Tasks list]
    Editor -->|sidebar Settings| Settings[Settings form]
    Editor -->|right-pane Timeline| Timeline[Recent edits]
    Editor -->|right-pane Backlinks| Backlinks
    Editor -->|right-pane Outline| Outline
```

---

## 2. Screen-by-screen surface inventory

Every row is a region that **must** render content. Empty cells = bug. Use as audit checklist.

### Common chrome (every screen)

| Region | Element | Notes |
|---|---|---|
| **Top bar** | MALD wordmark (left) | Brand green, mono |
| **Top bar** | Top search input (`Ctrl+Shift+F` opens modal) | Centered, focused state shows accent border |
| **Top bar** | Local model badge (`Local • Ollama (model)`) | Mockup target — currently unimplemented; daemon status indicator lives in status bar today |
| **Top bar** | Settings button | Switches activity to `Settings` |
| **Top bar** | Window controls (Win/Linux only) | Minimize / maximize / close |
| **Activity bar** (left rail, 52px) | Files / Search / Graph / Tasks / AI / Settings buttons | Active = green glyph + surface1 bg |
| **Status footer** | Daemon status dot (`●` green/red/grey) + Cursor position + index info | `gui/widgets/status_bar.rs` |
| **Sidebar footer** | Indexer `●` + `Indexed: N files (P%)` strip | Green dot when 100%, sub-text otherwise |
| **Keybindings** | Ctrl+P palette · Ctrl+Shift+F search · Ctrl+N new note · Ctrl+B sidebar · Ctrl+J terminal · Ctrl+Shift+B right pane | Set in `handle_key_press()` in `gui/app.rs` |

### 2.1 First-run wizard (CLI, no GUI yet)

| Region | Content |
|---|---|
| Whole terminal | Welcome banner, KB name prompt, editor pick (auto-detected list + free-form), final summary |

Acceptance: wizard never returns until either successful init or user-cancel; on success ~/.mald/ has `config/`, `kb/<name>/`, `index/`, `logs/`.

### 2.2 Home view (`ActiveView::Home`)

| Region | Element |
|---|---|
| Sidebar | File tree of default KB, expandable dirs, modified-dot indicator |
| Sidebar footer | Indexer strip + (planned) Quick Commands list |
| Main pane | Stats grid: note count, link count, open/done tasks, orphan count, modified count, connected ratio, graph hub, active focus; recent files (6); open tasks (5); spaces (KB switcher). Action buttons: New note, Ask MALD, Search, Index vault, Doctor |
| Right pane | Hidden by default; toggle via `Ctrl+Shift+B` |

### 2.3 Editor view (`ActiveView::Editor`)

| Region | Element |
|---|---|
| Sidebar | File tree (mode = Files); current file highlighted in green |
| Tab bar | One pill per open tab; modified = yellow dot; close × on hover |
| Main pane | Source text editor (mono); split with markdown preview pane via grid handle |
| Wikilink popover | Active when typing `[[`; arrow keys navigate, Enter selects |
| Right pane (when open) | Tabs: Context / Backlinks / AIChat / Outline / Timeline (see 2.7) |
| Status | Cursor `Ln C`, byte count, daemon status |

Acceptance: tab close with unsaved → modal w/ Save / Discard / Cancel.

### 2.4 Ask MALD chat (right pane `AIChat` or main-pane chat tab)

| Region | Element |
|---|---|
| Header | "Ask MALD" + close × |
| Chat scroll area | Per-message bubble: role label (You/AI/System) + body; AI bubble shows source pills below answer once Meta event lands |
| Streaming row | Spinner + "Thinking…" while streaming |
| Mode pills | Smart / Deep / Focus: General; active = green outline + green text |
| Input row | Multi-line input + send (`⌘↵`); disabled while streaming |

Acceptance: clicking source pill opens cited note in editor; Deep mode injects web context; Focus mode filters retrieval to active note's directory.

### 2.5 Search modal (`cmd-F`)

| Region | Element |
|---|---|
| Overlay | Modal w/ search input |
| Results list | Title + snippet + path; up/down keyboard nav; Enter opens |
| Empty | "No matches" hint + tip about FTS |

### 2.6 Command palette (`cmd-K`)

| Region | Element |
|---|---|
| Overlay | Modal w/ palette input |
| Results | Filtered fuzzy-matched commands; group separators by category |
| Hint | `↵ run · ↑↓ navigate · Esc close` |

### 2.7 Right pane (feature panel)

Tab order: **Context** (default), **Backlinks**, **AIChat**, **Outline**, **Timeline**.

#### 2.7.1 Context tab

| Section | Content |
|---|---|
| Sources | List of cited chunks: `filename:Lstart-end` + relevance score (green text) — click → open |
| Related Notes | Wikilink-derived neighbours of cited sources — click → open |
| Extracted | "Extracting…" while async pass running; Key Concepts / Tasks / Questions bullets when ready |
| Model & Retrieval | Model · Embedding · Index (HNSW/FTS) · Retrieved (N chunks in M ms) · Mode · Confidence (avg score %) |

Acceptance: empty state before first chat shows placeholder + sub-text; never blank section labels.

#### 2.7.2 Backlinks tab

| Section | Content |
|---|---|
| List | Notes linking to current file: link source name (lavender) + surrounding context line (sub-text); click → open |
| Empty | "No backlinks" + sub-text |

#### 2.7.3 AIChat tab — same layout as 2.4

#### 2.7.4 Outline tab

| Section | Content |
|---|---|
| List | Active file's headings, indented by level; click → scroll editor to that line |
| Empty | "No outline" + sub-text |

#### 2.7.5 Timeline tab

| Section | Content |
|---|---|
| List | 20 most-recently modified `.md` across all KBs: filename (text) + relative time (green: `3m ago`, `2d ago`, …); secondary line `<kb> · <full path>` (sub-text); click → open |
| Empty | "No recent edits" + sub-text |

Acceptance: refreshes on `EditorSaved`. Relative time formatter supports just-now / minutes / hours / days / months / years.

### 2.8 Graph view (`ActiveView::Graph`)

| Region | Element |
|---|---|
| Sidebar | KB selector + node count badge |
| Main pane | Force-directed graph canvas; pan + zoom; hover = highlight neighbours |
| Controls | Charge / link distance / link strength / center sliders + reset; orphan toggle |
| Right pane | (none specific — share Editor's right pane) |

### 2.9 Tasks view (`ActiveView::Tasks`)

| Region | Element |
|---|---|
| Sidebar | Mode + open/done counts badge |
| Main pane | List or kanban: `[ ]` open / `[x]` done; click row → open source note |
| Top toolbar | Toggle list ↔ kanban |

### 2.10 Settings view (`ActivityMode::Settings`)

| Region | Element |
|---|---|
| Sidebar form | Editor command, default KB, AI model, Ollama URL, embedding model, shell, daemon auto-start; Save / Reset; "Add to PATH" button (Windows-only); Demo space launcher |
| Main pane | Status messages + previewed effects |

### 2.11 Terminal panel (toggle from status bar / `ctrl-`backtick`)

| Region | Element |
|---|---|
| Header | Title `Terminal` + close × |
| Body | PTY output (mono); strips ANSI for safety |
| Input | Single-line input bound to running shell |

---

## 3. Audit checklist

Run per release. Check every box per build.

- [ ] Top bar: wordmark + search input + window controls render in every view
- [ ] Activity bar shows correct active glyph per view; hover state visible
- [ ] Sidebar header badge updates per mode (file count, search count, graph node count, task open/done, AI message count)
- [ ] Sidebar footer indexer dot + count updates within 5s of init and after every reindex
- [ ] Status bar daemon dot reflects health within 1 tick (green=running, red=stopped, dim=unknown)
- [ ] Tab bar shows modified dot; close × visible on hover; unsaved-close modal blocks
- [ ] Right pane tabs: **Context** is the default tab on first open; tab switching does not flicker; close × hides pane
- [ ] Context tab: Sources / Related / Extracted / Model & Retrieval — section headers always rendered, even when empty (placeholder body required)
- [ ] Mode pills: keyboard reachable; active pill rendered with green outline + green text; mode persists across sends
- [ ] Sources list: clicking row opens cited note in editor at correct line range when range is known
- [ ] Timeline: refreshes after every successful `EditorSaved`; relative time formatter handles secs / minutes / hours / days / months / years
- [ ] Backlinks: hits when current note appears in another note as `[[wikilink]]`
- [ ] Outline: matches headings in current note; jump-to-line works
- [ ] Graph: pan/zoom/reset all responsive; orphans toggle hides degree-0 nodes
- [ ] Settings: dirty flag turns Save active; Reset reverts; daemon auto-start toggle persists
- [ ] Terminal: opening doesn't stall main thread; ANSI stripped; close button kills PTY

---

## 4. Cross-cutting motion / accessibility checklist

- [ ] Sidebar / right-pane / terminal expand-collapse uses velocity-aware animation
- [ ] Modal fade in/out completes within 100–150ms; no layout shift after
- [ ] Focus ring visible on every interactive element when keyboard-focused
- [ ] Mono font is the default everywhere (text editor, chat, sidebar, footer, palette)
- [ ] Brand accent (green) is the only "primary" colour; blue restricted to wikilinks/lavender contexts
- [ ] Empty states never show blank rectangles — always a label + sub-text + suggested action

---

## 5. Audit findings — 2026-05-01 (against USER_JOURNEY v1)

| # | Finding | Status |
|---|---|---|
| 1 | `feature_panel_content` initialised to `Backlinks`, not `Context` — contradicts journey spec | **Fixed** in `gui/app.rs` |
| 2 | Top bar in mockup includes a `Local • Ollama (model)` badge; current implementation only shows daemon health in the **status footer** via `widgets/status_bar.rs`. Spec updated to reflect today's reality + flag the mockup gap | Documented |
| 3 | Original spec listed `cmd-K` for palette and `cmd-F` for search; actual bindings are `Ctrl+P` and `Ctrl+Shift+F`. Spec updated | Documented |
| 4 | Sidebar footer in spec described "Quick Commands list" as already-shipped; today only the indexer strip lives there. Spec marked Quick Commands as planned | Documented |
| 5 | Home view renders a much richer dashboard than the spec described (stats grid + recent + tasks + spaces). Spec rewritten to match | Documented |
| 6 | Context tab section labels (Sources / Related / Extracted / Model & Retrieval) all render even with no data, matching the spec | Verified in `gui/components/feature_panel.rs::view_context` |
| 7 | Timeline tab refresh on `EditorSaved` is wired via `Message::TimelineRefresh` after save | Verified in `gui/app.rs::Message::EditorSaved` |
| 8 | Mode pills active state uses `colors::ACCENT` border + accent label; matches spec | Verified in `gui/components/feature_panel.rs::mode_pill_row` |
