use chrono::{Datelike, Local, NaiveDate};
use forge_reflect::config::Config;
use forge_reflect::surface;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = Config::load();
    let input = forge_reflect::read_hook_input().unwrap_or_default();

    let cwd = if input.cwd.is_empty() {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        input.cwd
    };

    let today = Local::now().date_naive();
    let day_of_year = today.ordinal();

    let mut sections: Vec<String> = Vec::new();

    // Inspiration pools only — task coverage (overdue, due-soon, yesterday)
    // lives in /DailyPlan. Surface is for serendipity and rediscovery.

    // --- Stale ideas ---
    if let Some(s) = ideas_section(&config, &cwd, today, day_of_year) {
        sections.push(s);
    }

    // --- Rediscovery pool (tabs + backlog, mixed) ---
    if let Some(s) = rediscovery_section(&config, &cwd, day_of_year) {
        sections.push(s);
    }

    if sections.is_empty() {
        return ExitCode::SUCCESS;
    }

    println!("\u{1f4cc} Surface \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
    for s in &sections {
        print!("{s}");
    }
    println!("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");

    ExitCode::SUCCESS
}

/// Resolve a user-content path (relative to vault root).
fn resolve_user(config: &Config, cwd: &str, relative: &str) -> std::path::PathBuf {
    config.resolve_user_path(cwd, relative)
}

/// Ideas section: stale open ideas.
fn ideas_section(config: &Config, cwd: &str, today: NaiveDate, day_of_year: u32) -> Option<String> {
    let ideas_dir = resolve_user(config, cwd, &config.memory.ideas);
    if !ideas_dir.is_dir() {
        return None;
    }

    let cutoff = today - chrono::Duration::days(i64::from(config.surface.ideas_cutoff_days));
    let mut entries = Vec::new();

    let Ok(read_dir) = fs::read_dir(&ideas_dir) else {
        return None;
    };

    for entry in read_dir {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let title = extract_frontmatter_value(&content, "title").unwrap_or_default();
        let status = extract_frontmatter_value(&content, "status").unwrap_or_default();
        let created = extract_frontmatter_value(&content, "created").unwrap_or_default();

        if !title.is_empty() {
            entries.push((title, status, created));
        }
    }

    surface::parse_ideas(&entries, cutoff, day_of_year, config.surface.max_items)
}

/// Rediscovery section: rotate through a mixed pool of tabs + backlog items.
fn rediscovery_section(config: &Config, cwd: &str, day_of_year: u32) -> Option<String> {
    let mut pool: Vec<String> = Vec::new();

    // Collect tab titles from most recent archive
    let archive_dir = resolve_user(config, cwd, &config.surface.archive_dir);
    if archive_dir.is_dir() {
        let mut candidates: Vec<_> = fs::read_dir(&archive_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(&config.surface.archive_prefix)
            })
            .collect();

        candidates.sort_by_key(|e| e.file_name().to_string_lossy().into_owned());
        if let Some(latest) = candidates.last() {
            if let Ok(content) = fs::read_to_string(latest.path()) {
                pool.extend(surface::extract_tab_titles(&content));
            }
        }
    }

    // Collect open backlog items
    let backlog_path = resolve_user(config, cwd, &config.backlog);
    if let Ok(content) = fs::read_to_string(&backlog_path) {
        pool.extend(surface::extract_backlog_titles(&content));
    }

    surface::format_rotating_pool(&pool, config.surface.max_items, day_of_year as usize)
}

/// Extract a simple YAML frontmatter value (single-line, between --- fences).
fn extract_frontmatter_value(content: &str, key: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let prefix = format!("{key}:");

    for line in content.lines() {
        if line.trim() == "---" {
            if in_frontmatter {
                return None; // Passed end of frontmatter
            }
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter && line.starts_with(&prefix) {
            let val = line[prefix.len()..].trim().to_string();
            // Strip quotes
            let val = val.trim_matches('"').trim_matches('\'').to_string();
            return Some(val);
        }
    }
    None
}
