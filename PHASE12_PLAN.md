# Phase 12 — UI Pivot to "Terminal-First PKM"

Reference mockup: green-on-black terminal aesthetic, three-pane layout (vault tree | tabs/chat | context pane), mono everywhere, source-cited RAG with extracted concepts/tasks/questions.

## Mapping mockup → modules

| Mockup piece                              | Status         | Module                                           |
| ----------------------------------------- | -------------- | ------------------------------------------------ |
| Vault file tree (left)                    | exists         | `gui/widgets/file_tree.rs`                       |
| Tabs in main pane                         | exists         | `gui/widgets/tabs.rs` + `EditorTab`              |
| Ask MALD chat in main pane                | exists         | `gui/widgets/ai_chat.rs`                         |
| ⌘K palette                                | exists         | `gui/widgets/command_palette.rs`                 |
| Right pane (Context/Graph/Timeline tabs)  | new            | `gui/components/context_pane.rs`                 |
| Sources list w/ relevance scores          | backend exists | hook `RagResult.sources` → context_pane          |
| Related notes                             | backend exists | reuse graph/backlinks                            |
| Extracted concepts/tasks/questions        | new            | `ai/extract.rs` (background LLM pass)            |
| Mode pills (Smart / Deep / Focus)         | new            | message + `ai_chat` props                        |
| Source chips below answer                 | partial        | extend `ai_chat` answer footer                   |
| Quick Commands footer in left pane        | new            | `gui/components/sidebar_content.rs` extend       |
| Indexer % footer                          | new            | `gui/widgets/indexer_footer.rs` + daemon hook    |
| Timeline tab (right pane)                 | backlog        | journal index hook                               |
| Graph tab in right pane                   | exists         | rehouse `gui/canvas/graph.rs`                    |

## Mode semantics

- **Smart**: vault-only RAG. Default. Cheapest, deterministic.
- **Deep**: vault + web. Web fetched via reqwest, summarized, cited inline.
- **Focus: <scope>**: scope filter — path glob, tag, or KB. Applies on top of Smart/Deep.

## Phasing

1. **Cosmetic pass** — mono default font, GREEN primary, dense spacing. Days.
2. **Indexer footer** — bottom strip. Cheap. Trust signal.
3. **Right context pane scaffold** — shell w/ tabs, populate Sources + Related from existing RAG output.
4. **Mode pills** — wire Smart/Deep/Focus state into ai_chat message flow.
5. **Auto-extract** — second LLM pass after streaming completes; Extracted block in right pane.
6. **Timeline tab** — backlog, follow-up.

## Tradeoffs

- Iced text renderer: rich-text (numbered lists + inline bold) tedious. Use markdown_view widget for chat answers.
- Mono everywhere risks long-prose readability. Plan to test; fallback = sans for paragraph body, mono for code/paths.
- Three-pane <1280px = cramped. Auto-collapse right pane below threshold.
- Auto-extract = +1 LLM call per response. Background, cancellable, surfaces async into Extracted block.

## Acceptance

- Mono default font lands without breaking layout.
- Palette primary = GREEN; focus rings, active tabs, selected file all green.
- Indexer footer renders live `Indexed: N files (P%)`.
- Right context pane stub renders with dummy data; wire-up follows.
- All existing tests still pass (319).
