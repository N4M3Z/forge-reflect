use super::*;
use chrono::NaiveDate;

fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

// --- parse_backlog ---

#[test]
fn backlog_overdue_items() {
    let content = "- [ ] Fix deploy script [priority:: high] [due:: 2026-02-10]\n\
                   - [ ] Write docs [due:: 2026-02-20]\n\
                   - [x] Completed task [due:: 2026-02-01]\n";
    let result = parse_backlog(content, date("2026-02-12"), 3).unwrap();
    assert!(result.contains("Overdue:"));
    assert!(result.contains("Fix deploy script"));
    assert!(!result.contains("Completed task"));
}

#[test]
fn backlog_due_soon_items() {
    let content = "- [ ] Review PR [due:: 2026-02-14]\n\
                   - [ ] Far future task [due:: 2026-06-01]\n";
    let result = parse_backlog(content, date("2026-02-12"), 3).unwrap();
    assert!(result.contains("Due soon:"));
    assert!(result.contains("Review PR"));
    assert!(!result.contains("Far future"));
}

#[test]
fn backlog_empty_returns_none() {
    let content = "- [x] Done [due:: 2026-02-01]\n\
                   - [ ] No due date task\n";
    assert!(parse_backlog(content, date("2026-02-12"), 3).is_none());
}

#[test]
fn backlog_strips_metadata() {
    let content = "- [ ] Deploy v2 [priority:: high] [due:: 2026-02-10]\n";
    let result = parse_backlog(content, date("2026-02-12"), 3).unwrap();
    assert!(result.contains("Deploy v2"));
    assert!(!result.contains("[priority::"));
    assert!(!result.contains("[due::"));
}

// --- format_reminders ---

#[test]
fn reminders_formats_relative_dates() {
    let json = r#"{"count": 2, "reminders": [
        {"title": "Depart", "dueDate": "2026-02-12T10:00:00+01:00"},
        {"title": "Follow up", "dueDate": "2026-02-13T09:00:00+01:00"}
    ]}"#;
    let result = format_reminders(json, date("2026-02-12")).unwrap();
    assert!(result.contains("Depart (today)"));
    assert!(result.contains("Follow up (tomorrow)"));
    assert!(result.contains("Reminders (2):"));
}

#[test]
fn reminders_overdue_label() {
    let json = r#"{"count": 1, "reminders": [
        {"title": "Late task", "dueDate": "2026-02-10T09:00:00+01:00"}
    ]}"#;
    let result = format_reminders(json, date("2026-02-12")).unwrap();
    assert!(result.contains("2d overdue"));
}

#[test]
fn reminders_zero_count_returns_none() {
    let json = r#"{"count": 0, "reminders": []}"#;
    assert!(format_reminders(json, date("2026-02-12")).is_none());
}

#[test]
fn reminders_no_due_date() {
    let json = r#"{"count": 1, "reminders": [{"title": "Undated task"}]}"#;
    let result = format_reminders(json, date("2026-02-12")).unwrap();
    assert!(result.contains("Undated task"));
    // No date parenthetical on the item line itself
    assert!(result.contains("\u{2022} Undated task\n"));
}

// --- parse_ideas ---

#[test]
fn ideas_stale_items_with_rotation() {
    let entries = vec![
        (
            "Alpha idea".to_string(),
            "Open".to_string(),
            "2026-01-01".to_string(),
        ),
        (
            "Beta idea".to_string(),
            "Open".to_string(),
            "2026-01-05".to_string(),
        ),
        (
            "Gamma idea".to_string(),
            "Adopted".to_string(),
            "2026-01-01".to_string(),
        ),
        (
            "Delta idea".to_string(),
            "Open".to_string(),
            "2026-01-10".to_string(),
        ),
    ];
    let result = parse_ideas(&entries, date("2026-01-20"), 0, 2).unwrap();
    assert!(result.contains("Stale ideas (3):"));
    // Only 2 shown (max)
    let bullet_count = result.matches('\u{2022}').count();
    assert_eq!(bullet_count, 2);
}

#[test]
fn ideas_none_stale_returns_none() {
    let entries = vec![(
        "Recent".to_string(),
        "Open".to_string(),
        "2026-02-10".to_string(),
    )];
    // Cutoff is 2026-02-01 — entry created after cutoff, so not stale
    assert!(parse_ideas(&entries, date("2026-02-01"), 0, 3).is_none());
}

#[test]
fn ideas_skips_non_open() {
    let entries = vec![
        (
            "Adopted".to_string(),
            "Adopted".to_string(),
            "2026-01-01".to_string(),
        ),
        (
            "Dismissed".to_string(),
            "Dismissed".to_string(),
            "2026-01-01".to_string(),
        ),
    ];
    assert!(parse_ideas(&entries, date("2026-02-12"), 0, 3).is_none());
}

// --- extract_tab_titles ---

#[test]
fn extract_tabs_gets_titles() {
    let content = "## Research\n\
                   - [Archfey Wiki](https://example.com/archfey)\n\
                   - [RAG with LM Studio](https://example.com/rag)\n\
                   ## Shopping\n\
                   - [Eight Sleep Pod](https://example.com/sleep)\n";
    let titles = extract_tab_titles(content);
    assert_eq!(titles.len(), 3);
    assert_eq!(titles[0], "Archfey Wiki");
    assert_eq!(titles[1], "RAG with LM Studio");
    assert_eq!(titles[2], "Eight Sleep Pod");
}

#[test]
fn extract_tabs_empty_content() {
    let titles = extract_tab_titles("## Empty section\nNo links here\n");
    assert!(titles.is_empty());
}

// --- extract_backlog_titles ---

#[test]
fn extract_backlog_open_tasks() {
    let content = "## High\n\
                   - [ ] Fix deploy script [priority:: high] [due:: 2026-02-10]\n\
                   - [x] Completed task [due:: 2026-02-01]\n\
                   ## Medium\n\
                   - [ ] Write docs\n\
                   - [ ] Build Spark — ambient serendipity [priority:: medium]\n";
    let titles = extract_backlog_titles(content);
    assert_eq!(titles.len(), 3);
    assert!(titles[0].contains("Fix deploy script"));
    assert!(!titles[0].contains("[priority::"));
    assert!(!titles[0].contains("[due::"));
    assert_eq!(titles[1], "Write docs");
    assert!(titles[2].contains("Build Spark"));
}

#[test]
fn extract_backlog_skips_completed() {
    let content = "- [x] Done task [due:: 2026-02-01]\n- [ ] Open task\n";
    let titles = extract_backlog_titles(content);
    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0], "Open task");
}

#[test]
fn extract_backlog_empty() {
    let content = "- [x] All done\n";
    let titles = extract_backlog_titles(content);
    assert!(titles.is_empty());
}

// --- format_rotating_pool ---

#[test]
fn pool_formats_with_rotation() {
    let items: Vec<String> = vec!["Alpha".into(), "Beta".into(), "Gamma".into()];
    let result = format_rotating_pool(&items, 2, 0).unwrap();
    assert!(result.contains("Rediscovery:"));
    assert!(result.contains("Alpha"));
    assert!(result.contains("Beta"));
    assert!(!result.contains("Gamma"));
}

#[test]
fn pool_rotation_offset() {
    let items: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    let r0 = format_rotating_pool(&items, 1, 0).unwrap();
    let r1 = format_rotating_pool(&items, 1, 1).unwrap();
    assert!(r0.contains('A'));
    assert!(r1.contains('B'));
}

#[test]
fn pool_wraps_around() {
    let items: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    let result = format_rotating_pool(&items, 2, 2).unwrap();
    // Starts at C, wraps to A
    assert!(result.contains('C'));
    assert!(result.contains('A'));
}

#[test]
fn pool_empty_returns_none() {
    let items: Vec<String> = vec![];
    assert!(format_rotating_pool(&items, 5, 0).is_none());
}

#[test]
fn pool_max_exceeds_count() {
    let items: Vec<String> = vec!["Only".into()];
    let result = format_rotating_pool(&items, 5, 0).unwrap();
    let bullet_count = result.matches('\u{2022}').count();
    assert_eq!(bullet_count, 1);
}

// --- parse_captured_tabs (deprecated, backward compat) ---

#[allow(deprecated)]
#[test]
fn tabs_legacy_extracts_titles() {
    let content = "- [Archfey Wiki](https://example.com/archfey)\n\
                   - [RAG with LM Studio](https://example.com/rag)\n";
    let result = parse_captured_tabs(content, 2, 0).unwrap();
    assert!(result.contains("Captured tabs:"));
    assert!(result.contains("Archfey Wiki"));
}

#[allow(deprecated)]
#[test]
fn tabs_legacy_empty_returns_none() {
    let content = "## Empty section\nNo links here\n";
    assert!(parse_captured_tabs(content, 3, 0).is_none());
}

// --- parse_journal_gaps ---

#[test]
fn journal_unchecked_plan_items() {
    let content = "## Daily plan\n\
                   - [#] #log/daily/plan [due:: 2026-02-11]\n\
                   - [ ] Review PR #42\n\
                   - [x] Team standup\n\
                   ## Daily log\n\
                   Some log\n";
    let result = parse_journal_gaps(content).unwrap();
    assert!(result.contains("Review PR #42"));
    assert!(!result.contains("Team standup"));
}

#[test]
fn journal_unchecked_review_habits() {
    let content = "## Daily review\n\
                   - [ ] [[Sleep 7 hours a day]] #log/daily [due:: 2026-02-11]\n\
                   - [x] [[30 minutes outside]] #log/daily [due:: 2026-02-11]\n\
                   - [ ] [[150 grams of protein]] #log/daily [due:: 2026-02-11]\n";
    let result = parse_journal_gaps(content).unwrap();
    assert!(result.contains("Sleep 7 hours"));
    assert!(result.contains("150 grams of protein"));
    assert!(!result.contains("30 minutes outside"));
}

#[test]
fn journal_all_checked_returns_none() {
    let content = "## Daily plan\n\
                   - [x] Done task\n\
                   ## Daily review\n\
                   - [x] [[Habit]] #log/daily [due:: 2026-02-11]\n";
    assert!(parse_journal_gaps(content).is_none());
}

#[test]
fn journal_skips_log_markers() {
    let content = "## Daily plan\n\
                   - [ ] #log/daily/plan [due:: 2026-02-11]\n\
                   - [ ] Actual task\n";
    let result = parse_journal_gaps(content).unwrap();
    assert!(!result.contains("#log/daily/plan"));
    assert!(result.contains("Actual task"));
}

#[test]
fn journal_stops_at_template_embed() {
    let content = "## Daily review\n\
                   - [ ] [[Habit one]] #log/daily [due:: 2026-02-11]\n\
                   ![[Daily.base]]\n\
                   - [ ] This should not appear\n";
    let result = parse_journal_gaps(content).unwrap();
    assert!(result.contains("Habit one"));
    assert!(!result.contains("should not appear"));
}
