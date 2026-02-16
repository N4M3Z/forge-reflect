//! Surfacing — pure parsing functions for `SessionStart` digest.
//!
//! All functions take string content and return `Option<String>` (None = nothing to show).
//! No I/O — the binary handles subprocess calls and file reads.

use chrono::NaiveDate;
use regex::Regex;
use std::fmt::Write;

/// Parsed backlog item.
struct BacklogItem {
    description: String,
    priority: String,
    due: Option<NaiveDate>,
}

/// Parse Backlog.md content. Returns overdue + due-soon sections.
pub fn parse_backlog(content: &str, today: NaiveDate, horizon_days: u32) -> Option<String> {
    let soon = today + chrono::Duration::days(i64::from(horizon_days));
    let date_re = Regex::new(r"\[due::\s*(\d{4}-\d{2}-\d{2})\]").expect("valid regex");
    let priority_re = Regex::new(r"\[priority::\s*(\w+)\]").expect("valid regex");

    let mut overdue = Vec::new();
    let mut due_soon = Vec::new();

    for line in content.lines() {
        // Only open tasks: "- [ ] ..."
        if !line.starts_with("- [ ] ") {
            continue;
        }

        let due = date_re.captures(line).and_then(|c| {
            c.get(1)
                .and_then(|m| NaiveDate::parse_from_str(m.as_str(), "%Y-%m-%d").ok())
        });

        let priority = priority_re
            .captures(line)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_else(|| "medium".to_string());

        // Strip markdown task prefix and metadata
        let desc = line
            .trim_start_matches("- [ ] ")
            .replace(date_re.find(line).map_or("", |m| m.as_str()), "")
            .replace(priority_re.find(line).map_or("", |m| m.as_str()), "")
            .trim()
            .to_string();

        let item = BacklogItem {
            description: desc,
            priority,
            due,
        };

        if let Some(d) = item.due {
            if d < today {
                overdue.push(item);
            } else if d <= soon {
                due_soon.push(item);
            }
        }
    }

    if overdue.is_empty() && due_soon.is_empty() {
        return None;
    }

    let mut output = String::new();
    if !overdue.is_empty() {
        output.push_str("Overdue:\n");
        for item in &overdue {
            let due_str = item.due.map_or(String::new(), |d| format!(", due {d}"));
            let _ = writeln!(
                output,
                "  \u{2022} {} [{}{due_str}]",
                item.description, item.priority
            );
        }
    }
    if !due_soon.is_empty() {
        output.push_str("Due soon:\n");
        for item in &due_soon {
            let due_str = item.due.map_or(String::new(), |d| format!("due {d}"));
            let _ = writeln!(output, "  \u{2022} {} [{due_str}]", item.description);
        }
    }

    Some(output)
}

/// Parse ekctl reminders JSON output. Returns formatted reminder list.
pub fn format_reminders(json: &str, today: NaiveDate) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(json).ok()?;
    let count = parsed.get("count")?.as_u64()?;
    if count == 0 {
        return None;
    }

    let reminders = parsed.get("reminders")?.as_array()?;
    let mut output = format!("Reminders ({count}):\n");

    for r in reminders.iter().take(5) {
        let title = r
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Untitled");
        let when = r.get("dueDate").and_then(|d| d.as_str()).and_then(|d| {
            // Parse ISO date, compute relative label
            let date = NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()?;
            let diff = (date - today).num_days();
            Some(match diff {
                0 => "today".to_string(),
                1 => "tomorrow".to_string(),
                d if d < 0 => format!("{}d overdue", d.abs()),
                _ => date.format("%a %d %b").to_string(),
            })
        });

        if let Some(w) = when {
            let _ = writeln!(output, "  \u{2022} {title} ({w})");
        } else {
            let _ = writeln!(output, "  \u{2022} {title}");
        }
    }

    Some(output)
}

/// Parse Ideas directory entries. Returns stale (open, older than cutoff) ideas with rotation.
///
/// `entries`: Vec of (filename, status, `created_date_str`) tuples.
/// `cutoff`: date before which an idea is considered stale.
/// `day_of_year`: used for rotating which ideas to show.
pub fn parse_ideas(
    entries: &[(String, String, String)],
    cutoff: NaiveDate,
    day_of_year: u32,
    max: usize,
) -> Option<String> {
    let mut stale: Vec<(&str, &str)> = Vec::new();

    for (title, status, created) in entries {
        if status != "Open" {
            continue;
        }
        let Ok(date) = NaiveDate::parse_from_str(created, "%Y-%m-%d") else {
            continue;
        };
        if date < cutoff {
            stale.push((title.as_str(), created.as_str()));
        }
    }

    if stale.is_empty() {
        return None;
    }

    stale.sort_by_key(|(t, _)| t.to_lowercase());
    let count = stale.len();
    let offset = day_of_year as usize % count;

    let mut output = format!("Stale ideas ({count}):\n");
    for (shown, i) in (0..count).enumerate() {
        if shown >= max {
            break;
        }
        let idx = (offset + i) % count;
        let (title, created) = stale[idx];
        let _ = writeln!(output, "  \u{2022} {title} (since {created})");
    }

    Some(output)
}

/// Extract titles from a captured tabs archive (markdown links).
///
/// Returns a list of link titles from `- [Title](URL)` lines.
pub fn extract_tab_titles(content: &str) -> Vec<String> {
    let link_re = Regex::new(r"^- \[([^\]]+)\]\(").expect("valid regex");
    let mut titles = Vec::new();

    for line in content.lines() {
        if let Some(caps) = link_re.captures(line) {
            if let Some(title) = caps.get(1) {
                titles.push(title.as_str().to_string());
            }
        }
    }

    titles
}

/// Extract descriptions from open backlog tasks.
///
/// Parses `- [ ] description [priority:: x] [due:: y]` lines, strips metadata.
pub fn extract_backlog_titles(content: &str) -> Vec<String> {
    let date_re = Regex::new(r"\[due::\s*\d{4}-\d{2}-\d{2}\]").expect("valid regex");
    let priority_re = Regex::new(r"\[priority::\s*\w+\]").expect("valid regex");
    let mut titles = Vec::new();

    for line in content.lines() {
        if !line.starts_with("- [ ] ") {
            continue;
        }

        let desc = line
            .trim_start_matches("- [ ] ")
            .replace(date_re.find(line).map_or("", |m| m.as_str()), "")
            .replace(priority_re.find(line).map_or("", |m| m.as_str()), "")
            .trim()
            .to_string();

        if !desc.is_empty() {
            titles.push(desc);
        }
    }

    titles
}

/// Format a rotating selection from a pool of items.
///
/// Source-agnostic: takes any list of strings and rotates based on offset.
/// Returns None if the pool is empty.
pub fn format_rotating_pool(items: &[String], max: usize, offset: usize) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let count = items.len();
    let start = offset % count;
    let mut output = String::from("Rediscovery:\n");

    for i in 0..max.min(count) {
        let idx = (start + i) % count;
        let _ = writeln!(output, "  \u{2022} {}", items[idx]);
    }

    Some(output)
}

/// Parse captured tabs archive. Returns rotating selection of markdown links.
///
/// Extracts `- [Title](URL)` lines from archive content.
/// `offset`: rotation offset (e.g. day-of-year).
#[deprecated(note = "Use extract_tab_titles + format_rotating_pool instead")]
pub fn parse_captured_tabs(content: &str, max: usize, offset: usize) -> Option<String> {
    let titles = extract_tab_titles(content);
    if titles.is_empty() {
        return None;
    }

    let count = titles.len();
    let start = offset % count;
    let mut output = String::from("Captured tabs:\n");

    for i in 0..max.min(count) {
        let idx = (start + i) % count;
        let _ = writeln!(output, "  \u{2022} {}", titles[idx]);
    }

    Some(output)
}

/// Parse journal content for unchecked items in Daily plan and Daily review sections.
pub fn parse_journal_gaps(content: &str) -> Option<String> {
    let mut unchecked = Vec::new();
    let mut in_plan = false;
    let mut in_review = false;

    for line in content.lines() {
        if line.starts_with("## Daily plan") {
            in_plan = true;
            in_review = false;
            continue;
        }
        if line.starts_with("## Daily review") {
            in_review = true;
            in_plan = false;
            continue;
        }
        if line.starts_with("## ") || line.starts_with("![[") {
            in_plan = false;
            in_review = false;
            continue;
        }

        if (in_plan || in_review) && line.starts_with("- [ ] ") {
            let item = line.trim_start_matches("- [ ] ").trim();
            // Skip task metadata markers (log markers, due dates alone)
            if !item.starts_with("#log/daily/") {
                unchecked.push(item.to_string());
            }
        }
    }

    if unchecked.is_empty() {
        return None;
    }

    let mut output = String::from("Yesterday:\n");
    for item in &unchecked {
        let _ = writeln!(output, "  \u{2022} {item}");
    }
    Some(output)
}

#[cfg(test)]
mod tests;
