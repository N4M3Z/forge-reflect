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

fn make_codex_user() -> String {
    serde_json::json!({
        "role": "user",
        "content": [{ "type": "input_text", "text": "hi" }]
    })
    .to_string()
}

fn make_codex_assistant_write(tool_name: &str, file_path: &str) -> String {
    serde_json::json!({
        "role": "assistant",
        "content": [
            { "type": "text", "text": "★ Insight: Codex Topic" },
            {
                "type": "tool_call",
                "name": tool_name,
                "input": { "path": file_path }
            }
        ]
    })
    .to_string()
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
        make_assistant_text("More analysis:\n\u{2605} Insight \u{2500}\nMore stuff\n\u{2500}"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 2);
    assert_eq!(analysis.insights_write_count, 0);
    assert_eq!(analysis.user_messages, 2);
}

#[test]
fn test_mid_line_insight_marker_not_matched() {
    let transcript = [
        make_human(),
        make_assistant_text("The marker is \u{2605} Insight and the regex matches it"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 0);
    assert!(analysis.insight_topics.is_empty());
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

    let transcript = [make_human(), make_assistant_text("CUSTOM_MARKER here")].join("\n");

    let analysis = analyze_transcript(&transcript, &config);
    assert_eq!(analysis.insight_count, 1);
}

#[test]
fn test_topic_extraction() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Topic A"),
        make_assistant_text("\u{2605} Insight Topic B"),
        make_assistant_text("\u{2605} Insight\nTopic C"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 3);
    assert_eq!(
        analysis.insight_topics,
        vec!["Topic A", "Topic B", "Topic C"]
    );
}

#[test]
fn test_insights_written_filenames() {
    let transcript = [
        make_human(),
        make_assistant_write("Memory/Insights/Topic A.md"),
        make_assistant_write("/abs/path/to/Memory/Insights/Topic B.md"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insights_write_count, 2);
    assert_eq!(analysis.insights_written, vec!["Topic A.md", "Topic B.md"]);
}

#[test]
fn test_codex_role_schema_counts_user_and_write() {
    let transcript = [
        make_codex_user(),
        make_codex_assistant_write(
            "write",
            "/Users/test/Data/Vaults/Personal/Orchestration/Memory/Insights/Codex Topic.md",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.user_messages, 1);
    assert_eq!(analysis.tool_using_turns, 1);
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insights_write_count, 1);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_safe_write_name_is_treated_as_write() {
    let transcript = [
        make_human(),
        make_codex_assistant_write(
            "safe-write",
            "/Users/test/Data/Vaults/Personal/Orchestration/Memory/Imperatives/Test.md",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.tool_using_turns, 1);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_non_write_tool_does_not_count_memory_write() {
    let transcript = [
        make_human(),
        serde_json::json!({
            "role": "assistant",
            "content": [{
                "type": "tool_call",
                "name": "Read",
                "input": { "path": "/Users/test/Data/Vaults/Personal/Orchestration/Memory/Insights/Foo.md" }
            }]
        })
        .to_string(),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.tool_using_turns, 1);
    assert_eq!(analysis.insights_write_count, 0);
    assert!(!analysis.has_memory_write);
}

// ─── Decorative border filtering ───

#[test]
fn test_decorative_border_not_extracted_as_topic() {
    let transcript = [
        make_human(),
        make_assistant_text(
            "\u{2605} Insight \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\nActual content\n\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Marker still counts as an insight
    assert_eq!(analysis.insight_count, 1);
    // But the decorative border is NOT pushed as a topic
    assert!(analysis.insight_topics.is_empty());
}

#[test]
fn test_real_topic_after_colon_not_filtered() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: forge-module alignment"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["forge-module alignment"]);
}

#[test]
fn test_is_decorative() {
    assert!(is_decorative("\u{2500}\u{2500}\u{2500}"));
    assert!(is_decorative("---"));
    assert!(is_decorative("___"));
    assert!(is_decorative("```"));
    assert!(!is_decorative("Real topic"));
    assert!(!is_decorative("\u{2500} mixed text"));
    assert!(!is_decorative(""));
}
