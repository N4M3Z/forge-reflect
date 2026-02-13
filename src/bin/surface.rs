use chrono::{Datelike, Local, NaiveDate};
use forge_reflect::config::Config;
use forge_reflect::surface;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let config = Config::load();
    let today = Local::now().date_naive();
    let day_of_year = today.ordinal();

    // FORGE_ROOT = project root (set by dispatch.sh)
    let forge_root = std::env::var("FORGE_ROOT").unwrap_or_default();

    let mut sections: Vec<String> = Vec::new();

    // --- Yesterday's journal gaps ---
    if let Some(s) = journal_section(&config, today, &forge_root) {
        sections.push(s);
    }

    // --- Backlog: overdue + due soon ---
    if let Some(s) = backlog_section(&config, today, &forge_root) {
        sections.push(s);
    }

    // --- Reminders ---
    if let Some(s) = reminders_section(&config, today) {
        sections.push(s);
    }

    // --- Stale ideas ---
    if let Some(s) = ideas_section(&config, today, day_of_year) {
        sections.push(s);
    }

    // --- Captured tabs ---
    if let Some(s) = tabs_section(&config, day_of_year) {
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
    println!("Print a concise daily briefing from the above — highlight the most urgent items, \
              skip anything the user likely already knows. 2-4 bullet points max.");

    ExitCode::SUCCESS
}

/// Resolve safe-read binary path. Commands are project-root-relative (FORGE_ROOT).
fn resolve_safe_read(config: &Config, forge_root: &str) -> PathBuf {
    if forge_root.is_empty() {
        // Fallback: try relative to cwd
        PathBuf::from(&config.commands.safe_read)
    } else {
        Path::new(forge_root).join(&config.commands.safe_read)
    }
}

/// Run safe-read to read an AMBER file. Returns content or None.
fn safe_read(config: &Config, forge_root: &str, target_path: &Path) -> Option<String> {
    let safe_read_path = resolve_safe_read(config, forge_root);

    // Clear CLAUDE_PLUGIN_ROOT so the safe-read wrapper resolves its own
    // module root via dirname fallback (not the caller's plugin root).
    Command::new("bash")
        .arg(safe_read_path.to_str()?)
        .arg(target_path.to_str()?)
        .env_remove("CLAUDE_PLUGIN_ROOT")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Resolve a user-content path (relative to vault root).
fn resolve_user(config: &Config, relative: &str) -> PathBuf {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    config.resolve_user_path(&cwd, relative)
}

/// Backlog section: overdue + due-soon items.
fn backlog_section(config: &Config, today: NaiveDate, forge_root: &str) -> Option<String> {
    let backlog_path = resolve_user(config, &config.backlog);
    let content = safe_read(config, forge_root, &backlog_path)?;
    surface::parse_backlog(&content, today, config.surface.due_soon_days)
}

/// Reminders section via ekctl.
fn reminders_section(config: &Config, today: NaiveDate) -> Option<String> {
    let output = Command::new("ekctl")
        .args([
            "list",
            "reminders",
            "--list",
            &config.surface.reminders_list,
            "--completed",
            "false",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let json = String::from_utf8_lossy(&output.stdout);
    surface::format_reminders(&json, today)
}

/// Ideas section: stale open ideas.
fn ideas_section(config: &Config, today: NaiveDate, day_of_year: u32) -> Option<String> {
    let ideas_dir = resolve_user(config, &config.memory.ideas);
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
        if path.extension().map_or(true, |e| e != "md") {
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

/// Captured tabs section: rotate through archived tabs.
fn tabs_section(config: &Config, day_of_year: u32) -> Option<String> {
    let archive_dir = resolve_user(config, &config.surface.archive_dir);
    if !archive_dir.is_dir() {
        return None;
    }

    // Find most recent archive file matching prefix
    let mut candidates: Vec<_> = fs::read_dir(&archive_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&config.surface.archive_prefix)
        })
        .collect();

    candidates.sort_by_key(|e| e.file_name().to_string_lossy().into_owned());
    let latest = candidates.last()?;

    let content = fs::read_to_string(latest.path()).ok()?;
    surface::parse_captured_tabs(&content, config.surface.max_items, day_of_year as usize)
}

/// Journal section: check yesterday's journal for unchecked items.
fn journal_section(config: &Config, today: NaiveDate, forge_root: &str) -> Option<String> {
    let yesterday = today - chrono::Duration::days(1);

    // Resolve journal path by substituting date placeholders
    let daily_pattern = &config.journal.daily;
    let yesterday_path = resolve_date_pattern(daily_pattern, yesterday);

    let full_path = resolve_user(config, &yesterday_path);
    if !full_path.exists() {
        return Some(format!(
            "Yesterday:\n  \u{2022} No journal for {yesterday}\n"
        ));
    }

    // Read via safe-read (AMBER file)
    let content = safe_read(config, forge_root, &full_path)?;
    surface::parse_journal_gaps(&content)
}

/// Replace YYYY, MM, DD placeholders in a path pattern with date values.
fn resolve_date_pattern(pattern: &str, date: NaiveDate) -> String {
    // Replace YYYY-MM-DD first (longest match), then individual components
    pattern
        .replace("YYYY-MM-DD", &date.format("%Y-%m-%d").to_string())
        .replace("YYYY", &date.format("%Y").to_string())
        .replace("MM", &date.format("%m").to_string())
        .replace("DD", &date.format("%d").to_string())
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
