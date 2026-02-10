use super::*;

fn cfg() -> Config {
    Config::default()
}

fn make_assistant_text(text: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                { "type": "text", "text": text }
            ]
        }
    })
    .to_string()
}

fn make_assistant_write(file_path: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "tool_use",
                    "name": "Write",
                    "input": { "file_path": file_path }
                }
            ]
        }
    })
    .to_string()
}

fn make_human() -> String {
    serde_json::json!({ "type": "human" }).to_string()
}

// ─── Insight counting ───

#[test]
fn test_counts_insight_markers() {
    let transcript = [
        make_human(),
        make_assistant_text(
            "Here is some analysis.\n\n\u{2605} Insight \u{2500}\nSomething learned\n\u{2500}",
        ),
        make_human(),
        make_assistant_text("Another \u{2605} Insight \u{2500}\nMore stuff\n\u{2500}"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 2);
    assert_eq!(analysis.insights_write_count, 0);
    assert_eq!(analysis.user_messages, 2);
}

#[test]
fn test_insight_with_matching_insight_file() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight \u{2500}\nKey finding\n\u{2500}"),
        make_assistant_write(
            "/Users/test/Data/Vaults/Personal/Orchestration/Memory/Insights/Key Finding.md",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insights_write_count, 1);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_no_insights_no_block() {
    let transcript = [
        make_human(),
        make_assistant_text("Just regular text, no insights."),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 0);
    assert_eq!(analysis.insights_write_count, 0);
}

// ─── Memory write classification ───

#[test]
fn test_imperatives_count_as_memory_not_insights() {
    let transcript = [
        make_human(),
        make_assistant_write(
            "/Users/test/Data/Vaults/Personal/Orchestration/Memory/Imperatives/Some Imperative.md",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insights_write_count, 0);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_custom_insight_marker() {
    let mut config = Config::default();
    config.insight_marker = "CUSTOM_MARKER".to_string();

    let transcript = [
        make_human(),
        make_assistant_text("Found a CUSTOM_MARKER here"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &config);
    assert_eq!(analysis.insight_count, 1);
}
