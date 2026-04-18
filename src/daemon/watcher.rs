use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

const DEBOUNCE_MS: u64 = 2000;

pub fn start_watching(dir: &Path, tx: Sender<PathBuf>) -> Result<RecommendedWatcher> {
    let dir = dir.to_path_buf();
    let mut last_events: HashMap<PathBuf, Instant> = HashMap::new();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    let now = Instant::now();
                    let debounce = Duration::from_millis(DEBOUNCE_MS);
                    for path in event.paths {
                        if path.extension().is_some_and(|e| e == "md") {
                            let should_send = last_events
                                .get(&path)
                                .map(|t| now.duration_since(*t) > debounce)
                                .unwrap_or(true);
                            if should_send {
                                last_events.insert(path.clone(), now);
                                if let Err(e) = tx.try_send(path.clone()) {
                                    tracing::warn!(
                                        path = %path.display(),
                                        error = %e,
                                        "watcher channel full, event dropped"
                                    );
                                }
                            }
                        }
                    }
                    // Prune stale entries to prevent unbounded growth
                    if last_events.len() > 1000 {
                        last_events.retain(|_, t| now.duration_since(*t) < debounce * 5);
                    }
                }
                _ => {}
            }
        }
    })?;

    watcher.watch(&dir, RecursiveMode::Recursive)?;
    tracing::info!("Watching {} for changes", dir.display());

    Ok(watcher)
}
