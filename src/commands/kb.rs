use anyhow::{bail, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    env,
    io::{stdout, IsTerminal},
    time::SystemTime,
};

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KbCandidate {
    pub name: String,
    pub reason: String,
    pub note_count: usize,
}

#[derive(Debug, Clone, Default)]
struct KbInsight {
    note_count: usize,
    last_modified: Option<SystemTime>,
}

pub async fn create(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::errors::bail_ctx(
            "Space name cannot be empty.",
            "Run `mald kb create <name>` with the words you want MALD to use for that space.",
        ));
    }

    let kb_path = mald_home().join("kb").join(name);
    if kb_path.exists() {
        bail!("Space '{name}' already exists");
    }
    ensure_directory(&kb_path)?;
    crate::commands::starter::seed_starter_space(&kb_path, name)?;

    // Create templates directory
    let templates = kb_path.join("templates");
    ensure_directory(&templates)?;
    std::fs::write(
        templates.join("default.md"),
        "---\ntitle: \ntags: []\ncreated: \n---\n\n# \n\n",
    )?;

    println!("Created space: {name}");
    println!("  Path: {}", kb_path.display());
    Ok(())
}

pub async fn list(json: bool) -> Result<()> {
    let kb_dir = mald_home().join("kb");
    if !kb_dir.exists() {
        if json {
            println!("[]");
        } else {
            println!("No spaces found. Run `mald init` first.");
        }
        return Ok(());
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let default_kb = config.typed().default_kb.clone();

    let mut kbs: Vec<serde_json::Value> = Vec::new();
    let mut count = 0;

    for entry in std::fs::read_dir(&kb_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let files = crate::fs::find_files(&entry.path(), "md")?;
            let is_default = name == default_kb;
            if json {
                kbs.push(serde_json::json!({
                    "name": name,
                    "notes": files.len(),
                    "path": entry.path().to_string_lossy(),
                    "default": is_default,
                }));
            } else {
                let marker = if is_default { " *" } else { "" };
                println!("  {} ({} files){}", name, files.len(), marker);
            }
            count += 1;
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&kbs)?);
    } else if count == 0 {
        println!("No spaces found. Create one with `mald kb create <name>`.");
    } else {
        println!();
        println!("Default space: {default_kb}");
        println!("Workspace: {}", mald_home().display());
        println!("Tip: run `mald launch` to choose a space and open MALD there.");
        println!("Tip: run `mald kb open` to pick one and open it in your editor.");
    }
    Ok(())
}

pub fn available_kbs() -> Result<Vec<String>> {
    let kb_root = mald_home().join("kb");
    Ok(crate::config::manager::list_kb_names(&kb_root))
}

pub fn set_default_kb_sync(name: &str) -> Result<()> {
    let kb_path = mald_home().join("kb").join(name);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Space `{name}` not found."),
            "Run `mald kb list` to see available spaces.",
        ));
    }

    let config_path = mald_home().join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;
    config.set("default_kb", serde_json::Value::String(name.to_string()))?;
    Ok(())
}

pub fn resolve_launch_target(query: Option<&str>) -> Result<Option<String>> {
    let candidates = ranked_kbs(query)?;
    if candidates.is_empty() {
        return Err(crate::errors::bail_ctx(
            "No spaces are available yet.",
            "Run `mald init` or `mald kb create <name>` first.",
        ));
    }

    let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) else {
        if stdout().is_terminal() {
            return run_kb_picker(&candidates, "");
        }
        return Ok(candidates
            .into_iter()
            .next()
            .map(|candidate| candidate.name));
    };

    if let Some(exact) = candidates
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(query))
    {
        return Ok(Some(exact.name.clone()));
    }

    let matches: Vec<String> = candidates
        .iter()
        .map(|candidate| candidate.name.clone())
        .collect();
    match matches.as_slice() {
        [] => Err(crate::errors::bail_ctx(
            format!("No space matches `{query}`."),
            format!(
                "Run `mald kb list` to see available spaces in {}.",
                mald_home().display()
            ),
        )),
        [single] => Ok(Some(single.clone())),
        _ if stdout().is_terminal() => run_kb_picker(&candidates, query),
        _ => Err(crate::errors::bail_ctx(
            format!("More than one space matches `{query}`."),
            format!(
                "Try a more specific name, or run `mald launch` and pick one interactively: {}",
                matches.join(", ")
            ),
        )),
    }
}

pub async fn current() -> Result<()> {
    let (_config, _typed, kb_name, kb_path) = crate::config::resolve_kb(None)?;
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("No active space is available (expected `{kb_name}`)."),
            "Run `mald kb list` to inspect your workspace, or `mald kb create <name>` to make one.",
        ));
    }

    println!("Current space: {kb_name}");
    println!("Path: {}", kb_path.display());
    println!("Workspace: {}", mald_home().display());
    Ok(())
}

pub async fn use_kb(name: Option<&str>) -> Result<()> {
    let selected = if let Some(name) = name {
        resolve_launch_target(Some(name))?
    } else if stdout().is_terminal() {
        resolve_launch_target(None)?
    } else {
        return Err(crate::errors::bail_ctx(
            "Space name required in non-interactive mode.",
            "Run `mald kb use <name>`, or run `mald kb list` to inspect available spaces first.",
        ));
    };

    let Some(selected) = selected else {
        return Ok(());
    };
    let kb_path = mald_home().join("kb").join(&selected);
    set_default_kb_sync(&selected)?;
    println!("Default space set to: {selected}");
    println!("Path: {}", kb_path.display());
    Ok(())
}

pub async fn open(name: Option<&str>) -> Result<()> {
    let resolved = if let Some(name) = name {
        let Some(selected) = resolve_launch_target(Some(name))? else {
            return Ok(());
        };
        selected
    } else if stdout().is_terminal() {
        let Some(selected) = resolve_launch_target(None)? else {
            return Ok(());
        };
        selected
    } else {
        let (_, _, kb_name, _) = crate::config::resolve_kb(None)?;
        kb_name
    };
    let kb_path = mald_home().join("kb").join(&resolved);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Space `{resolved}` not found."),
            "Run `mald kb list` to see available spaces.",
        ));
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let editor = config.typed().editor.clone();

    crate::commands::launch::open_in_editor(&editor, kb_path.as_os_str())?;
    Ok(())
}

pub fn ranked_kbs(query: Option<&str>) -> Result<Vec<KbCandidate>> {
    let kbs = available_kbs()?;
    let default_kb = default_kb_name();
    let context_tokens = current_context_tokens();
    let query = query.map(str::trim).filter(|query| !query.is_empty());

    let mut candidates: Vec<(i64, i64, KbCandidate)> = kbs
        .iter()
        .filter_map(|name| {
            let insight = kb_insight(name);
            let (score, reason) = kb_match_score(
                name,
                query,
                default_kb.as_deref(),
                &context_tokens,
                &insight,
            )?;
            let recent = insight
                .last_modified
                .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or_default();

            Some((
                score,
                recent,
                KbCandidate {
                    name: name.clone(),
                    reason,
                    note_count: insight.note_count,
                },
            ))
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| b.2.note_count.cmp(&a.2.note_count))
            .then_with(|| a.2.name.len().cmp(&b.2.name.len()))
            .then_with(|| a.2.name.cmp(&b.2.name))
    });

    Ok(candidates
        .into_iter()
        .map(|(_, _, candidate)| candidate)
        .collect())
}

fn fuzzy_match_kbs(kbs: &[String], query: &str) -> Vec<String> {
    let query = Some(query);
    let default_kb = default_kb_name();
    let context_tokens = current_context_tokens();

    let mut candidates: Vec<(i64, i64, String)> = kbs
        .iter()
        .filter_map(|name| {
            let insight = kb_insight(name);
            let (score, _) = kb_match_score(
                name,
                query,
                default_kb.as_deref(),
                &context_tokens,
                &insight,
            )?;
            let recent = insight
                .last_modified
                .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or_default();
            Some((score, recent, name.clone()))
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.len().cmp(&b.2.len()))
            .then_with(|| a.2.cmp(&b.2))
    });

    candidates.into_iter().map(|(_, _, name)| name).collect()
}

fn run_kb_picker(candidates: &[KbCandidate], initial_filter: &str) -> Result<Option<String>> {
    struct PickerState {
        filter: String,
        filtered: Vec<KbCandidate>,
        selected: ListState,
    }

    impl PickerState {
        fn new(candidates: &[KbCandidate], initial_filter: &str) -> Self {
            let mut state = Self {
                filter: initial_filter.to_string(),
                filtered: filter_candidates(candidates, initial_filter),
                selected: ListState::default(),
            };
            if state.filtered.is_empty() {
                state.filtered = candidates.to_vec();
            }
            state.selected.select(Some(0));
            state
        }

        fn refresh(&mut self, candidates: &[KbCandidate]) {
            self.filtered = if self.filter.trim().is_empty() {
                candidates.to_vec()
            } else {
                filter_candidates(candidates, &self.filter)
            };
            let next = if self.filtered.is_empty() {
                None
            } else {
                Some(
                    self.selected
                        .selected()
                        .unwrap_or(0)
                        .min(self.filtered.len().saturating_sub(1)),
                )
            };
            self.selected.select(next);
        }
    }

    fn draw_picker(f: &mut Frame, state: &mut PickerState) {
        let area = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let input = Paragraph::new(state.filter.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Launch MALD into Space "),
        );
        f.render_widget(input, chunks[0]);
        f.set_cursor_position((chunks[0].x + state.filter.len() as u16 + 1, chunks[0].y + 1));

        let items: Vec<ListItem> = if state.filtered.is_empty() {
            vec![ListItem::new("  No matching spaces")]
        } else {
            state
                .filtered
                .iter()
                .map(|candidate| {
                    ListItem::new(format!(
                        "  {:<18}  {}  ·  {} notes",
                        candidate.name, candidate.reason, candidate.note_count
                    ))
                })
                .collect()
        };
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Spaces "))
            .highlight_symbol("> ")
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));
        f.render_stateful_widget(list, chunks[1], &mut state.selected);

        let footer_text = state
            .selected
            .selected()
            .and_then(|index| state.filtered.get(index))
            .map(|candidate| {
                format!(
                    "Recommended: {} · {} · Enter to launch · Esc to cancel",
                    candidate.name, candidate.reason
                )
            })
            .unwrap_or_else(|| {
                "Type to filter · Up/Down to select · Enter to launch · Esc to cancel".into()
            });
        let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut state = PickerState::new(candidates, initial_filter);

    let result = loop {
        terminal.draw(|f| draw_picker(f, &mut state))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => break Ok(None),
                    KeyCode::Enter => {
                        break Ok(state.selected.selected().and_then(|idx| {
                            state
                                .filtered
                                .get(idx)
                                .map(|candidate| candidate.name.clone())
                        }))
                    }
                    KeyCode::Up => {
                        if let Some(current) = state.selected.selected() {
                            state.selected.select(Some(current.saturating_sub(1)));
                        }
                    }
                    KeyCode::Down => {
                        let next = state
                            .selected
                            .selected()
                            .unwrap_or(0)
                            .saturating_add(1)
                            .min(state.filtered.len().saturating_sub(1));
                        state.selected.select(Some(next));
                    }
                    KeyCode::Backspace => {
                        state.filter.pop();
                        state.refresh(candidates);
                    }
                    KeyCode::Char(c) => {
                        state.filter.push(c);
                        state.refresh(candidates);
                    }
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}

fn filter_candidates(candidates: &[KbCandidate], query: &str) -> Vec<KbCandidate> {
    let matches = fuzzy_match_kbs(
        &candidates
            .iter()
            .map(|candidate| candidate.name.clone())
            .collect::<Vec<_>>(),
        query,
    );
    matches
        .into_iter()
        .filter_map(|name| {
            candidates
                .iter()
                .find(|candidate| candidate.name == name)
                .cloned()
        })
        .collect()
}

fn kb_match_score(
    name: &str,
    query: Option<&str>,
    default_kb: Option<&str>,
    context_tokens: &[String],
    insight: &KbInsight,
) -> Option<(i64, String)> {
    let lower = name.to_ascii_lowercase();
    let mut score = 0i64;
    let mut reason = "Available in this workspace";

    if let Some(query) = query {
        let query_lower = query.to_ascii_lowercase();
        let tokens: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .collect();
        let contains_all = if tokens.is_empty() {
            lower.contains(&query_lower)
        } else {
            tokens.iter().all(|token| lower.contains(token))
        };
        if !contains_all {
            return None;
        }

        if lower == query_lower {
            score += 500;
            reason = "Exact name match";
        } else if lower.starts_with(&query_lower) {
            score += 360;
            reason = "Starts with what you typed";
        } else {
            score += 220;
            reason = "Contains all typed words";
        }
    } else if default_kb.is_some_and(|default| default.eq_ignore_ascii_case(name)) {
        score += 220;
        reason = "Current default space";
    }

    if context_tokens
        .iter()
        .any(|token| lower.contains(token) || token.contains(&lower))
    {
        score += 180;
        if query.is_none() {
            reason = "Matches your current folder";
        }
    }

    if default_kb.is_some_and(|default| default.eq_ignore_ascii_case(name)) {
        score += 120;
    }

    score += recent_activity_bonus(insight.last_modified);
    score += (insight.note_count.min(24) as i64) * 2;

    Some((score, reason.to_string()))
}

fn recent_activity_bonus(last_modified: Option<SystemTime>) -> i64 {
    let Some(last_modified) = last_modified else {
        return 0;
    };
    let Ok(age) = SystemTime::now().duration_since(last_modified) else {
        return 0;
    };
    match age.as_secs() {
        0..=86_400 => 90,
        86_401..=259_200 => 60,
        259_201..=604_800 => 35,
        _ => 10,
    }
}

fn current_context_tokens() -> Vec<String> {
    let mut tokens = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        for component in cwd.components().rev().take(3) {
            let token = component.as_os_str().to_string_lossy().to_ascii_lowercase();
            if token.len() >= 2 {
                tokens.push(token);
            }
        }
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

fn default_kb_name() -> Option<String> {
    let config_path = mald_home().join("config").join("config.json");
    ConfigManager::load(&config_path)
        .ok()
        .map(|config| config.typed().default_kb)
}

fn kb_insight(name: &str) -> KbInsight {
    let kb_path = mald_home().join("kb").join(name);
    let Ok(files) = crate::fs::find_files(&kb_path, "md") else {
        return KbInsight::default();
    };

    let last_modified = files
        .iter()
        .filter_map(|path| path.metadata().ok()?.modified().ok())
        .max();

    KbInsight {
        note_count: files.len(),
        last_modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_kbs_prefers_exact_prefixes() {
        let kbs = vec!["work".into(), "work-notes".into(), "research".into()];
        let matches = fuzzy_match_kbs(&kbs, "work");
        assert_eq!(matches.first().map(String::as_str), Some("work"));
    }

    #[test]
    fn fuzzy_match_kbs_supports_multi_word_queries() {
        let kbs = vec![
            "client-acme-prod".into(),
            "client-acme-dev".into(),
            "research".into(),
        ];
        let matches = fuzzy_match_kbs(&kbs, "acme prod");
        assert_eq!(matches, vec!["client-acme-prod".to_string()]);
    }
}
