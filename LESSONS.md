# Lessons Learned

## Design & Architecture
- **2026-02-04:** The `MaldApp` struct became a God Object because new features were added directly to the main state instead of being composited. Future features must be independent components.
- **2026-02-04:** "Magic numbers" in layout code (e.g. `padding(10)`) significantly degrade the "premium feel" by breaking subtle accumulation of visual rhythm. Strict token usage is required.
- **2026-02-04:** Structural alignment hack (e.g. adding spacer views to fake borders) are fragile and create layout shift. Better to style the container border directly or use overlays.
- **2026-02-04:** "Silence" in design means removing redundant indicators. If a button changes color, it doesn't need a border *and* a background change *and* a spacer. One signal is enough.

## Performance & Systems Audit (2026-02-05)

### HNSW Index Hot Path
- **Clone storm in insert:** The HNSW neighbor pruning loop was cloning full 768-dim vectors (`neighbor.vector.clone()`) inside tight loops. Fixed by using slice references for distance computation—zero-copy path now.
- **SIMD cosine distance (2026-02-05):** Replaced scalar `cosine_distance` with 8-wide SIMD using `wide::f32x8`. Processes 8 floats per cycle, with scalar fallback for remainder. Expected 4-8x speedup for 768-dim vectors (96 SIMD iterations vs 768 scalar ops).

### Chunker Optimization (2026-02-05)
- **Iterator over collect:** Original chunker collected all lines into `Vec<&str>` upfront. Replaced with byte-offset index approach that slices directly from the source string—eliminates intermediate allocation for large documents.

### SQLite Query Performance (2026-02-05)
- **Prepared statement caching:** Converted frequent queries (`get_chunk`, `needs_reindex`, `fts_search`, `fts_search_since`) from `prepare()` to `prepare_cached()`. rusqlite maintains an internal LRU cache of compiled statements, avoiding repeated SQLite parsing overhead on repeated calls.

### Graph Data Sharing (2026-02-05)
- **Arc wrapping:** `GraphData` now wraps `nodes` and `edges` in `Arc<Vec<_>>`. Cloning graph data for multiple panes/views now costs O(1) atomic increment instead of O(n) deep copy. Essential for responsive pane splits with large knowledge bases.

### Daemon Reliability  
- **blocking_send in watcher:** File watcher used `tx.blocking_send()` which silently dropped events when channel was full. Replaced with `try_send()` + explicit `tracing::warn!` for observability.
- **mald_home panic:** `dirs::home_dir().expect()` crashed in CI containers. Added `try_mald_home() -> Option<PathBuf>` for graceful degradation.

### Code Quality
- **LayoutState missing Default:** Clippy warned about `new()` without `Default` impl. Added `impl Default for LayoutState`.
- **Format string drift:** Scattered `format!("{}", var)` instead of `format!("{var}")`. Standardized to inlined format args.

### Observability & Hardening (2026-02-05)

#### Structured Daemon Telemetry
- **HealthMetrics struct:** Added `HealthMetrics { uptime_secs, version, index_file_count, index_last_updated, note_count, tag_count }` for daemon introspection. IPC `status` command now returns structured JSON health data.
- **DAEMON_START tracking:** `OnceLock<Instant>` captures daemon startup time for uptime calculation—avoids global mutable state.

#### Silent Error Elimination
- **let _ = patterns:** Replaced silent `let _ = tx.send()` and `let _ = process_file()` with explicit `if let Err(e)` + `tracing::warn!`/`tracing::debug!`. Silent failures mask production issues.
- **FTS row iteration:** Added `tracing::debug!` for row deserialization errors in FTS search. Corrupted rows no longer silently disappear.

#### Version Compatibility
- **check_version utility:** Added `check_version(&mmap) -> Result<u32>` to validate mmap index version before read. Returns actionable error messages: "Please upgrade MALD" for newer versions, "Try `mald reindex`" for outdated.
- **current_version export:** Added `current_version() -> u32` for external callers to query expected index format version.

## Design Audit Patterns (2026-02-04)
- **Hierarchy First:** The eye should land on the primary action, then supporting content. If stat cards use H1 size, they compete with page headings. Use size + color together to establish hierarchy.
- **Active State Clarity:** Tabs need visible active indicators (2px bottom border in accent color). A 8% luminosity difference between BASE and SURFACE0 is not enough.
- **Non-Interactive Styling:** Decorative elements (badges, indicators) should not have hover/press states. This creates false affordance.
- **Typography Tokens:** Every hardcoded size creates drift. Use `type_scale::*` everywhere. When in doubt, use fewer sizes with color differentiation.
- **Double-Wrapping:** Nested containers add structural complexity and potential misalignment. Simplify to single containers where possible.
- **Animation Curves:** `ease_in_out_quad` feels smoother for panel toggles than `ease_out_cubic`. The quad curve has gentler acceleration.
- **Empty States:** Empty icons should use SUBTEXT0 (brighter) not SURFACE2 (too muted). Empty states are designed states, not absences.
- **Error Feedback:** Silent failures erode trust. Toast notifications provide essential feedback for errors, warnings, and success states.

## Design Audit V2 Patterns (2026-02-04)
- **Border as Indicator:** Full-border active indicators create visual weight that competes with content. A single bottom-edge accent (2px) is the quieter, more professional VSCode pattern.
- **Token Reference Rule:** Even when a hardcoded value matches a token numerically (e.g., `14` vs `icon_size::SECONDARY`), always reference the token. This ensures future system-wide changes propagate correctly.
- **Scroll Affordance:** Completely invisible scrollbars fail the "no thinking required" test. A subtle 30% opacity rail background signals scrollability without visual noise.
- **Fixed Dimensions for Pattern Elements:** Empty state circles should have fixed 64x64 dimensions, not padding-based sizing. Different icon sizes would otherwise create inconsistent circle sizes.
- **Vertical Rhythm Groups:** Use XL (24px) between major sections, LG (16px) between related groups, SM (8px) for tight coupling. Inconsistent spacing makes pages feel assembled rather than designed.
- **Component Width Unification:** Duplicate implementations with different widths (500px vs 600px) guarantee drift. Single source, single token.
- **Supporting vs Dominating:** Stat card values should use H3 (16px), not H2 (20px), when appearing on a page with DISPLAY (32px) heading. Values support context; they don't compete for attention.

## Systems Hardening Phase 1 (2026-02-05)

### HNSW Index Performance
- **Heap-based result tracking:** Replaced `Vec<Candidate>` with `BinaryHeap<MaxCandidate>` in `search_layer()`. Old approach sorted results after every neighbor expansion — O(ef × k × log k). New approach uses max-heap to efficiently evict worst candidates — O(ef × k × log ef).
- **O(1) length computation:** Added `deleted_count: usize` field to `HnswIndex`. `len()` now returns `nodes.len() - deleted_count` instead of iterating all nodes. `delete()` increments the counter atomically.
- **Worst-distance lookup:** Replaced `results.iter().fold(f32::NEG_INFINITY, f32::max)` with `results.peek()` — O(1) instead of O(n) on every iteration.

### Ollama Client Fault Tolerance
- **Timeout configuration:** Added `CONNECT_TIMEOUT` (5s), `REQUEST_TIMEOUT` (120s), and `HEALTH_TIMEOUT` (2s) constants. Client builder now sets default timeouts.
- **Health check fast-fail:** `is_running()` uses dedicated client with 2s timeout for quick failure detection.
- **Streaming without timeout:** Streaming methods (`chat_streaming`, `pull_model_stream`) use clients with only connect timeout, no total timeout (streams can be arbitrarily long).
- **Error context propagation:** All HTTP calls now include `.context()` with actionable error messages ("is Ollama running?").

### Daemon Server Hardening
- **Connection limiting:** Added `Semaphore::new(MAX_CONNECTIONS)` (100 concurrent) to both TCP and Unix socket servers. Prevents FD exhaustion under attack or misbehaving clients.
- **Permit-based lifecycle:** Connections acquire permit before spawning handler. Permit auto-releases when handler completes via RAII.

### Code Execution Safety
- **Explicit execution flag:** `mald run` now requires `--allow-exec` flag to actually execute code blocks. Without it, shows preview of what would be executed.
- **Two-stage confirmation:** Even with `--allow-exec`, user must confirm Y/n before execution proceeds.
- **Preview mode:** Default behavior shows first 10 lines of each code block without executing, with guidance on how to enable execution.

## Systems Hardening Phase 2 (2026-02-05)

### SQLite Index Optimization
- **Chunk foreign key index:** Added `CREATE INDEX IF NOT EXISTS idx_chunks_doc_id ON chunks(doc_id)` for O(log n) lookups in `delete_doc_chunks()` and chunk-document joins. Previously required full table scan.
- **Note:** `documents.path` already has implicit index via UNIQUE constraint.

### Daemon Parallel Indexing
- **Bounded concurrency:** File watcher now spawns parallel tasks for each changed file with `MAX_CONCURRENT_INDEX = 4` semaphore. Previously processed files sequentially, causing backlog during burst saves.
- **Async-friendly structure:** Config path wrapped in `Arc` for sharing across tasks. Each task acquires semaphore permit before processing.

### Streaming Directory Walker
- **Stack-based iteration:** Replaced recursive collect-all-then-iterate with lazy `WalkDir` iterator using explicit stack. Memory usage now proportional to directory depth, not total file count.
- **Iterator pattern:** Implements `Iterator<Item = Result<DirEntry, io::Error>>` for standard Rust iteration semantics.

### Dead Code Identified
- **`graph.rs::search_content()`:** Function exists but is never called. Linear scan implementation would be inefficient but is moot. Consider removal in future cleanup.

## Systems Hardening Phase 3: Observability & Resource Safety (2026-02-05)

### Daemon Health Telemetry
- **Index failure tracking:** Added `AtomicU64` counter for vector indexing failures. `record_index_failure()` increments on embedding/HNSW errors. Counter exposed via daemon `status` IPC command.
- **Health query via IPC:** New `query_health()` async function in `commands/daemon.rs` connects to daemon (TCP on Windows, Unix socket elsewhere), authenticates, and retrieves `HealthMetrics` including uptime, version, FTS status, vector status, document count, and failure count.
- **Doctor integration:** `mald doctor` now displays daemon runtime health when daemon is running: version, uptime, index failure count. Warns if daemon is running but not responding to IPC.

### Degraded Mode Logging (ai/chat.rs)
- **FTS-only mode logging:** When HNSW index doesn't exist, logs `info!("running in degraded mode: vector index not found")` to make search mode visible.
- **Fallback logging:** When vector search returns empty and FTS provides results, logs `debug!("FTS fallback returned results after vector search miss")`.
- **Embedding failure handling:** If embedding generation fails (Ollama unavailable), logs warning with error and falls back to FTS gracefully.
- **Total degradation warning:** If neither vector nor FTS index exists, logs `warn!("no search index available")`.

### RAII Resource Cleanup (commands/run.rs)
- **tempfile crate for Rust blocks:** Replaced manual temp directory management with `tempfile::TempDir`. Automatic cleanup on drop regardless of early returns or errors.
- **Moved from dev-dependencies:** `tempfile` promoted to regular dependency for production use.

## Design Audit V3 Patterns (2026-02-05)

### Category Errors in Token Usage
- **Semantic separation matters:** Using `spacing::MD` for text size (activity_bar.rs:63) is a category error — even when the numeric value (12.0) is identical to `type_scale::UI`. If spacing tokens are adjusted for layout reasons, typography would break. Always use tokens from the correct semantic category.

### Local Constants Are Drift Vectors
- **Local `const` creates hidden coupling:** Constants like `TAB_CLOSE_SIZE`, `PANEL_HEADER_HEIGHT`, and `SEARCH_INPUT_WIDTH` were duplicating design system values locally. When a component defines its own constants, future components may choose different values, creating inconsistency.
- **Solution:** All component-specific dimensions belong in `theme::layout` module as the single source of truth.

### Platform-Specific Elements
- **Emojis are not icons:** Using `text("📁")` renders differently across Windows, macOS, and Linux. Font fallbacks cause inconsistent sizing and styling. Always use the icon component system which guarantees visual consistency.

### Hardcoded Values Hide in Plain Sight
- **`.padding(4)` escaped multiple audits:** Numeric literals blend into code flow and are easy to miss. A grep for `padding\(\d` helps catch these stragglers. The design system rule is absolute: no raw pixel values in widget code.

### Framework Limitations Shape Implementation
- **Iced per-side borders:** Iced's `Border` struct only supports uniform borders (single width/color/radius). To achieve per-side effects like VSCode's left-edge activity indicator, use composition: a small colored container adjacent to the button rather than a border property.
- **Iced asymmetric padding:** `Padding` only accepts `[vertical, horizontal]` arrays or uniform values, not 4-sided `[top, right, bottom, left]`. Use Space elements or nested containers for asymmetric layouts.
- **Property transitions:** Iced doesn't support CSS-like property transitions (e.g., animating border color on focus). State changes are instant unless integrated with the animation system.

### Designed Empty States
- **Tab bar hint text:** An empty tab bar that just shows a blank colored strip feels unfinished. Adding subtle hint text ("Open a file to start") in muted color makes empty states feel intentional rather than broken.

## Motion Architecture Audit Phase 1 (2026-02-05)

### Disney Principle: "Does it move like it has weight and intent?"
- **Velocity-aware animations:** Fixed 200ms panel animations felt wrong—small collapses were slow, large ones felt rushed. Duration now scales with distance (pixels ÷ velocity), clamped to 120-350ms for natural feel.
- **Asymmetric timing:** Enter animations should be slightly slower than exits. Toast enter: 200ms, exit: 150ms. Users wait for entrance, but exits shouldn't block.

### Easing Function Selection
- **ease_out_quint for panels:** The "5" power creates snappy start with gentle settle—premium feel for reveals.
- **ease_in_quad for exits:** Accelerating out prevents "rubber band" feel on dismissals.
- **ease_out_back for micro-feedback:** Subtle overshoot (1.7 coefficient) makes button presses feel springy.

### Toast Animation Pattern
- **Enter:** Slide up 20px + fade in. Uses ease_out_quint for confident arrival.
- **Exit:** Fade out + slight slide down (8px). Uses ease_in_quad for quick departure.
- **Auto-dismiss:** 4s display time, then triggers exit animation (never instant removal).
- **Opacity threading:** All toast colors (background, border, text, shadow) multiply by animation opacity for cohesive fade.

### Animation State Machine
- **ToastPhase enum:** Entering → Visible → Exiting. Clear state prevents animation conflicts.
- **tick() returns removal signal:** Animation completion in Exiting phase returns true, allowing clean `retain_mut(|t| !t.tick())` pattern.
- **has_active_animation includes toasts:** Animation subscription checks all animation sources to ensure 60fps tick runs when needed.

### Framework Constraints
- **Iced padding limitation:** Only supports `[vertical, horizontal]` or uniform padding. Slide animations use `Space::new().height()` instead of top-only padding.
- **No transform support:** Iced lacks CSS-like transforms. Scale animations would require custom canvas rendering.

## Motion Architecture Audit Phase 2 (2026-02-06)

### Modal Fade Pattern
- **Deferred close:** When closing a modal, the animation starts but visibility stays true. On animation completion in AnimationTick, a `ModalKind` enum tells the handler which visibility flag to clear. This prevents the modal from vanishing before the fade finishes.
- **Shared animation slot:** Only one modal can be open at a time. A single `modal_animation: Option<ModalAnimation>` is cleaner than per-modal animation state.
- **Overlay opacity only:** Since Iced can't animate scale or translate on arbitrary containers, only the overlay background alpha is animated. This is sufficient for perceived smoothness; the dark overlay fade does most of the visual work.

### Canvas Hover Animation
- **State tracking in update, rendering in draw:** The `canvas::Program` trait splits mutation (`update` with `&mut State`) from rendering (`draw` with `&State`). Hover state changes (which node, when) are tracked in `update`, and time-based interpolation is computed in `draw`.
- **Per-node intensity function:** `hover_intensity(node_idx) -> f32` returns 0.0-1.0 for both entering and exiting nodes. Entering uses ease_out_quad (quick response), exiting uses ease_in_quad (gentle fade). Both use 150ms.
- **Interpolate everything:** Radius, color, glow opacity, and glow radius all interpolate as functions of hover_intensity. This creates a cohesive transition rather than binary state jumps.
- **Cursor-driven redraws:** Canvas redraws on cursor move events. Since hover animations happen during active cursor movement, no additional tick subscription is needed for the animation to look smooth.

## Motion Architecture Audit Phase 3 (2026-02-06)

### Activity Bar Pulse Pattern
- **Inverted ease for flash:** The pulse uses `1.0 - ease_out_quint(t)` which starts bright and fades quickly. Regular easing goes 0→1; inverting it creates a "flash" that's intense at the start and gently settles to nothing.
- **Width animation on indicator:** The 2px left edge indicator briefly widens to 3px during the pulse, creating a physical "impact" feel. Subtle but perceptible.
- **300ms duration:** Longer than hover (150ms) because the pulse should be noticeable even in peripheral vision. Too short and users miss it; too long and it feels like a loading indicator.

### Graph Simulation Restart
- **Differential alpha bumps:** Zoom gets alpha=0.1 (more visible repositioning), pan gets alpha=0.05 (subtle settling). Different interactions warrant different intensities of node movement.
- **Guard clause pattern:** `if alpha < threshold { alpha = threshold }` prevents accumulation during rapid interactions. Without this, repeated scrolls would keep raising alpha and cause jittery behavior.

### Command Palette Keyboard Navigation
- **Wrapping navigation:** Up from first item wraps to last, down from last wraps to first. Standard UI pattern users expect.
- **Enter triggers existing select path:** `CommandPaletteSubmit` dispatches to `CommandPaletteSelect(self.palette_selected)`, reusing the existing selection/close/execute logic.
- **iced_anim handles visual crossfade:** Since the button widget uses iced_anim, the style transition between selected/unselected states is already animated. No custom crossfade needed.

## Systems Architecture Audit Phase 1: Stability & Integrity (2026-02-06)

### Unbounded Collection Prevention
- **Ring buffer pattern for terminal:** `drain(..excess)` when `len() > MAX` is simpler than a true ring buffer (VecDeque) and avoids API changes. For terminal lines where order matters and append is dominant, this is the right trade-off.
- **Oldest-evicted toasts:** When toast count exceeds limit, `remove(0)` evicts the oldest. This ensures the most recent error is always visible — users care about "what just happened", not the first error from 10 minutes ago.
- **Chat message cap:** The RAG system (`chat.rs:195`) already limits context to last 10 messages for LLM input. The GUI cap of 500 is a safety net for display memory, not a semantic limit.

### Generation Counter Pattern for Async Races
- **Problem:** Iced's `Task::perform()` spawns async work that returns a `Message`. If user types "r", "ru", "rust" quickly, three search tasks run concurrently. The "r" task may complete last and overwrite the correct "rust" results.
- **Solution:** Monotonic `u64` generation counter. Increment before dispatch, embed in the `Message` variant. On receipt, compare against current generation — discard if stale. Cost: 8 bytes per message variant, zero runtime overhead.
- **Applied to:** search, backlinks, graph, tasks — all async data fetches that are re-triggered by user input.

### Debounce vs. Generation Counter
- **Debounce reduces work:** Prevents spawning N tasks in rapid succession (saves CPU/IO).
- **Generation counter ensures correctness:** Even with debounce, two tasks can race. The counter is the safety net.
- **Both are needed:** Debounce is an optimization; generation counter is a correctness guarantee. 150ms debounce + generation counter = fast and correct.

### Atomic Config Writes
- **Write-to-temp + rename:** `std::fs::write` is not atomic — a crash mid-write leaves a truncated file. Writing to `.json.tmp` first, then `rename()` is atomic on same-filesystem. The old file is either fully old or fully new, never partial.
- **Fallback for cross-device:** `rename()` fails across filesystems. Fallback to direct write + cleanup handles edge cases (shouldn't happen for same-directory rename, but defense in depth).

### Async File I/O in GUI
- **Problem:** `std::fs::read_to_string()` in `update()` blocks the entire GUI event loop. For large files or NFS mounts, this causes visible freezes.
- **Solution:** Move file read to `Task::perform()` with `tokio::fs::read_to_string()`. Returns `EditorFileLoaded(path, Result<String, String>)`. On error, shows toast instead of silently opening empty tab.
- **Pattern:** All file I/O in GUI should go through `Task::perform()`. The `update()` function should never call `std::fs::*` directly.

## Systems Architecture Audit Phase 2: Performance & Scale (2026-02-06)

### Shared KB Snapshot
- **Problem:** `load_graph()`, `load_tasks()`, and `load_backlinks()` each independently read the same KB directories and parse the same files. Three full filesystem traversals for one user action.
- **Solution:** `load_kb_files()` reads all KBs once into `Vec<KbFile>` structs (path, name, kb_name, content, links). Each consumer function filters and processes from this shared snapshot.
- **Trade-off:** Slightly higher peak memory (all files in memory at once) vs. 3x fewer disk reads. For typical KBs (<10K notes), the memory cost is negligible.

### Duplicate Work Elimination
- **all_links() called twice:** In `generate_graph_data()`, `doc.all_links()` was called once for edges and once for node construction. Each call re-parses wikilinks via regex. Storing the result in a local variable eliminates the redundant parse.
- **Pattern:** Any method that performs parsing/computation should be called once and stored, especially inside loops over document collections.

### Doctor Health Check Timeout
- **Problem:** `check_ollama()` used `reqwest::Client::new()` with no timeout. If Ollama is installed but unresponsive (port bound, hanging process), the doctor command hangs indefinitely.
- **Solution:** `reqwest::Client::builder().timeout(Duration::from_secs(2))` ensures the health check fails fast. 2s is generous for a local HTTP roundtrip.

### Per-File Watcher Debounce
- **Problem:** Global debounce (`last_event: Option<Instant>`) suppresses events for *all* files when *any* file triggers. Saving file A within 2s of file B means B's event is silently dropped.
- **Solution:** `HashMap<PathBuf, Instant>` tracks last event per file. Each path is debounced independently.
- **Memory safety:** Pruning guard (`if len > 1000 { retain entries < 10s old }`) prevents the HashMap from growing unbounded in large repositories with many active files.

## Systems Architecture Audit Phase 3: Hardening & Observability (2026-02-06)

### Config Key Input Sanitization
- **Attack surface:** `ConfigManager::set("..evil", value)` could create unexpected JSON paths. Empty segments from double dots or leading/trailing dots produce malformed nested structures.
- **Validation rules:** Non-empty, no leading/trailing dots, no double dots, no control characters, max 128 chars. Applied to both `set()` (returns error) and `get()` (returns None silently).
- **Pattern:** Validate at system boundary (the config API), not at every internal call site. This is the single entry point for user-supplied keys from CLI `mald config set`.

### Bounds-Checked Binary Deserialization
- **Problem:** `read_u32(data, &mut pos)` indexed directly with `data[*pos..*pos+4].try_into().unwrap()`. A truncated or corrupt index file causes a panic (index out of bounds or unwrap failure) instead of a recoverable error.
- **Solution:** Each read function checks `end > data.len()` before slicing, returning a descriptive error with the offset and file size. This converts all panics to `Result::Err` for graceful handling.
- **Coverage:** All read functions (read_u32, read_i64, read_u64, read_f32) plus inline bounds checks for deleted flags and vector slices.

### Header Sanity Validation
- **Problem:** A corrupt header with `count = 4_000_000_000` would attempt to allocate ~150GB of memory before any data reads could detect the corruption.
- **Validation before allocation:**
  - `dim`: Must be 1-4096 (largest production embeddings are 3072-dim)
  - `max_layer`: Must be ≤64 (theoretical max for billions of nodes)
  - `count`: Must be plausible given file size (count × min_node_size ≤ file_size)
  - `vectors_offset`, `graph_offset`: Must be within file bounds
- **Fail-fast principle:** Catching impossible values in the header prevents cascading failures in the deserialization loop. Error messages include "Index may be corrupted" to guide users toward `mald reindex`.
