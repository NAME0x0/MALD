use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

const DEBOUNCE_MS: u64 = 2000;

pub fn start_watching(dir: &Path, tx: Sender<PathBuf>) -> Result<RecommendedWatcher> {
    let dir = dir.to_path_buf();
    let mut last_event: Option<Instant> = None;

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    let now = Instant::now();
                    if last_event
                        .map(|t| now.duration_since(t) > Duration::from_millis(DEBOUNCE_MS))
                        .unwrap_or(true)
                    {
                        last_event = Some(now);
                        for path in event.paths {
                            if path.extension().is_some_and(|e| e == "md") {
                                let _ = tx.blocking_send(path);
                            }
                        }
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
