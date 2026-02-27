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

fn make_assistant_bash(command: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "tool_use",
                    "name": "Bash",
                    "input": { "command": command }
                }
            ]
        }
    })
    .to_string()
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
    // Decorative markers (★ Insight ─) — counted, topics extracted via next-line fallback.
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
    // ★ Insight ─ is decorative — next-line fallback extracts topic.
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

// ─── Backtick-wrapped insight markers ───

#[test]
fn test_backtick_wrapped_insight_marker() {
    let transcript = [
        make_human(),
        make_assistant_text(
            "`\u{2605} Insight \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}`\nSomething learned\n`\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}`",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Backtick-wrapped decorative border → next-line fallback extracts the topic.
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["Something learned"]);
}

#[test]
fn test_backtick_insight_with_topic() {
    let transcript = [
        make_human(),
        make_assistant_text("`\u{2605} Insight: Important Topic`"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["Important Topic"]);
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

// ─── Bash-tool safe-write detection ───

#[test]
fn test_bash_safe_write_detected_as_memory_write() {
    let transcript = [
        make_human(),
        make_assistant_bash(
            "cat <<'EOF' | safe-write write \"/Users/test/Data/Vaults/Personal/Orchestration/Memory/Imperatives/Test.md\"\ncontent\nEOF\n",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.tool_using_turns, 1);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_bash_safe_write_to_insights_counted() {
    let transcript = [
        make_human(),
        make_assistant_bash(
            "cat <<'EOF' | safe-write write \"Memory/Insights/Important Finding.md\"\ncontent\nEOF\n",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insights_write_count, 1);
    assert_eq!(analysis.insights_written, vec!["Important Finding.md"]);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_bash_safe_write_edit_detected() {
    let transcript = [
        make_human(),
        make_assistant_bash(
            "safe-write edit \"Memory/Insights/Topic.md\" --old \"foo\" --new \"bar\"",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insights_write_count, 1);
    assert!(analysis.has_memory_write);
}

#[test]
fn test_bash_non_safe_write_ignored() {
    let transcript = [make_human(), make_assistant_bash("ls -la /tmp")].join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.tool_using_turns, 1);
    assert!(!analysis.has_memory_write);
    assert_eq!(analysis.insights_write_count, 0);
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
    // Decorative border on same line → next-line fallback extracts "Actual content".
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["Actual content"]);
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

#[test]
fn test_decorative_insight_counted_but_no_topic() {
    // Full backtick-boxed format from explanatory output style.
    // The insight IS counted (insight_count incremented) but no topic is extracted.
    let transcript = [
        make_human(),
        make_assistant_text(
            "`\u{2605} Insight \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}`\n\
             Key educational point here\n\
             `\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}`",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    // Next-line fallback extracts the topic from the line after the decorative border.
    assert_eq!(analysis.insight_topics, vec!["Key educational point here"]);
}

// ─── Topic-to-filename matching ───

#[test]
fn test_topic_matches_filename_exact_words() {
    assert!(topic_matches_filename("yaml deep merge", "yaml-deep-merge"));
    assert!(topic_matches_filename(
        "rust compiler performance",
        "rust-compiler-performance"
    ));
}

#[test]
fn test_topic_matches_filename_partial_overlap() {
    assert!(topic_matches_filename(
        "yaml deep merge arrays",
        "yaml-deep-merge"
    ));
    assert!(topic_matches_filename(
        "forge module alignment",
        "module-alignment-patterns"
    ));
}

#[test]
fn test_topic_matches_filename_no_match() {
    assert!(!topic_matches_filename(
        "rust compiler performance",
        "yaml-deep-merge"
    ));
    assert!(!topic_matches_filename(
        "session reflection",
        "compiler-optimizations"
    ));
}

#[test]
fn test_topic_matches_filename_short_tokens_ignored() {
    // Tokens shorter than 4 chars are ignored — "the", "a", "is" don't count
    assert!(!topic_matches_filename("the fix is in", "the fix is out"));
    // But "fix" alone (3 chars) doesn't count — need a 4+ char match
    assert!(!topic_matches_filename("fix bug", "fix error"));
}

#[test]
fn test_topic_matches_filename_case_insensitive_via_lowered_input() {
    // Callers pass lowercased strings
    assert!(topic_matches_filename("yaml deep merge", "yaml-deep-merge"));
}

// ─── Single-word topic filtering ───

#[test]
fn test_single_word_topic_filtered() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: refactor"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert!(analysis.insight_topics.is_empty());
}

#[test]
fn test_multi_word_topic_kept() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: forge module alignment"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["forge module alignment"]);
}

// --- Session duration ---

fn make_timestamped_human(ts: &str) -> String {
    serde_json::json!({ "type": "human", "timestamp": ts }).to_string()
}

fn make_timestamped_assistant(ts: &str, text: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "timestamp": ts,
        "message": {
            "content": [{ "type": "text", "text": text }]
        }
    })
    .to_string()
}

#[test]
fn test_session_duration_from_timestamps() {
    let transcript = [
        make_timestamped_human("2026-02-26T10:00:00+01:00"),
        make_timestamped_assistant("2026-02-26T10:05:00+01:00", "hello"),
        make_timestamped_human("2026-02-26T10:20:00+01:00"),
        make_timestamped_assistant("2026-02-26T10:30:00+01:00", "done"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.session_duration_minutes, 30);
    assert_eq!(analysis.user_messages, 2);
}

#[test]
fn test_session_duration_zero_without_timestamps() {
    let transcript = [make_human(), make_assistant_text("no timestamps here")].join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.session_duration_minutes, 0);
}

#[test]
fn test_session_duration_single_timestamp() {
    let transcript = [make_timestamped_human("2026-02-26T10:00:00+01:00")].join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.session_duration_minutes, 0);
}

#[test]
fn test_session_duration_ignores_bad_timestamps() {
    let transcript = [
        serde_json::json!({ "type": "human", "timestamp": "not-a-date" }).to_string(),
        make_timestamped_human("2026-02-26T10:00:00+01:00"),
        make_timestamped_assistant("2026-02-26T10:45:00+01:00", "done"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.session_duration_minutes, 45);
}

// ─── SessionReflect insight reset ───

fn make_skill_invoke(skill: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "tool_use",
                    "name": "Skill",
                    "input": { "skill": skill }
                }
            ]
        }
    })
    .to_string()
}

#[test]
fn test_session_reflect_resets_insight_tracking() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Pre-reflection Topic"),
        make_skill_invoke("SessionReflect"),
        make_assistant_text("\u{2605} Insight: Post-reflection Topic"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Pre-reflection insight wiped — only post-reflection counted
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["Post-reflection Topic"]);
}

#[test]
fn test_session_reflect_resets_writes_and_skips() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Old Topic"),
        make_assistant_write("Memory/Insights/Old Topic.md"),
        make_assistant_text("\u{2606} Insight: Skipped Thing"),
        make_skill_invoke("SessionReflect"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Everything reset — nothing to check
    assert_eq!(analysis.insight_count, 0);
    assert!(analysis.insight_topics.is_empty());
    assert_eq!(analysis.insights_write_count, 0);
    assert!(analysis.insights_written.is_empty());
    assert!(analysis.skipped_topics.is_empty());
}

#[test]
fn test_compaction_boundary_resets_insight_tracking() {
    let compaction_msg = serde_json::json!({
        "type": "human",
        "message": {
            "content": [{
                "type": "text",
                "text": "This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation."
            }]
        }
    })
    .to_string();

    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Old Session Finding"),
        make_assistant_write("Memory/Insights/Old Session Finding.md"),
        compaction_msg,
        make_human(),
        make_assistant_text("New session work, no insights here"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Pre-compaction insight and write both wiped
    assert_eq!(analysis.insight_count, 0);
    assert!(analysis.insight_topics.is_empty());
    assert_eq!(analysis.insights_write_count, 0);
    assert!(analysis.insights_written.is_empty());
}

#[test]
fn test_compaction_boundary_preserves_post_compaction_insights() {
    let compaction_msg = serde_json::json!({
        "type": "human",
        "message": {
            "content": [{
                "type": "text",
                "text": "This session is being continued from a previous conversation that ran out of context."
            }]
        }
    })
    .to_string();

    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Old Finding"),
        compaction_msg,
        make_human(),
        make_assistant_text("\u{2605} Insight: New Finding"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Only post-compaction insight survives
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["New Finding"]);
}

// ─── Captured marker (✓ Insight) ───

#[test]
fn test_captured_marker_extracts_topic() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Session boundary awareness"),
        make_assistant_text(
            "\u{2713} Insight: Session boundary awareness \u{2192} Transcript scanners need session boundary awareness.md",
        ),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.captured_topics, vec!["session boundary awareness"]);
}

#[test]
fn test_captured_marker_without_arrow() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: forge module alignment"),
        make_assistant_text("\u{2713} Insight: forge module alignment"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.captured_topics, vec!["forge module alignment"]);
}

#[test]
fn test_captured_marker_resets_with_session_reflect() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2713} Insight: Old captured topic"),
        make_skill_invoke("SessionReflect"),
        make_assistant_text("\u{2605} Insight: New uncaptured topic"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    assert!(analysis.captured_topics.is_empty());
    assert_eq!(analysis.insight_count, 1);
    assert_eq!(analysis.insight_topics, vec!["New uncaptured topic"]);
}

#[test]
fn test_session_reflect_preserves_session_wide_counters() {
    let transcript = [
        make_human(),
        make_assistant_text("\u{2605} Insight: Pre Topic"),
        make_assistant_write("Memory/Imperatives/Some Rule.md"),
        make_skill_invoke("SessionReflect"),
        make_human(),
        make_assistant_text("Continuing work"),
    ]
    .join("\n");

    let analysis = analyze_transcript(&transcript, &cfg());
    // Session-wide counters preserved
    assert_eq!(analysis.user_messages, 2);
    assert_eq!(analysis.tool_using_turns, 2);
    assert!(analysis.has_memory_write);
}
