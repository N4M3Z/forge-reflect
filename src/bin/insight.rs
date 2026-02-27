use forge_reflect::config::Config;
use forge_reflect::transcript;
use std::fs;
use std::process::ExitCode;

fn emit_uncaptured(
    config: &Config,
    advisory_mode: bool,
    total: usize,
    uncaptured_topics: &[&str],
    unnamed: usize,
) {
    let mut reason_detail = String::new();
    if !uncaptured_topics.is_empty() {
        reason_detail = format!(": {}", uncaptured_topics.join(", "));
    } else if unnamed > 0 {
        reason_detail = format!(" ({unnamed} unnamed)");
    }

    if advisory_mode {
        let topics_display = if uncaptured_topics.is_empty() {
            format!("{unnamed} unnamed")
        } else {
            uncaptured_topics.join(", ")
        };
        let msg = config
            .insight_advisory_prompt
            .replace("{count}", &total.to_string())
            .replace("{topics}", &topics_display);
        let output = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": msg
            }
        });
        println!("{output}");
    } else if config.insight_blocking.unwrap_or(true) {
        eprintln!(
            "forge-reflect[insight]: blocking \u{2014} {total} uncaptured insight(s){reason_detail}"
        );
        let output = serde_json::json!({
            "decision": "block",
            "reason": format!(
                "{} ({total} uncaptured{reason_detail})",
                config.uncaptured_insight_reason
            )
        });
        println!("{output}");
    } else {
        eprintln!(
            "forge-reflect[insight]: warn \u{2014} {total} uncaptured insight(s){reason_detail}"
        );
    }
}

/// Hook contract: exit 0 always. Block/allow is communicated via JSON on stdout.
/// Empty stdout = allow. `{"decision":"block","reason":"..."}` = block.
fn main() -> ExitCode {
    let config = Config::load();

    let Some(input) = forge_reflect::read_hook_input() else {
        return ExitCode::SUCCESS;
    };

    if input.stop_hook_active {
        eprintln!("forge-reflect[insight]: stop_hook_active, deferring");
        return ExitCode::SUCCESS;
    }

    let cwd = if input.cwd.is_empty() {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        input.cwd
    };

    if !forge_reflect::in_data_dir(&cwd, &config) {
        eprintln!("forge-reflect[insight]: cwd '{cwd}' outside data dir, skipping");
        return ExitCode::SUCCESS;
    }

    let Ok(transcript) = fs::read_to_string(&input.transcript_path) else {
        eprintln!(
            "forge-reflect[insight]: transcript unreadable at '{}', skipping",
            input.transcript_path
        );
        return ExitCode::SUCCESS;
    };

    let analysis = transcript::analyze_transcript(&transcript, &config);

    let advisory_mode = std::env::var("FORGE_INSIGHT_ADVISORY").unwrap_or_default() == "1";

    // Substantiality gate — mirrors reflect.rs thresholds.
    // In advisory mode, skip the gate — nudging is low-cost, we want early detection.
    if !advisory_mode
        && (analysis.user_messages < config.user_msg_threshold
            || analysis.tool_using_turns < config.tool_turn_threshold)
    {
        eprintln!(
            "forge-reflect[insight]: session not substantial ({} msgs, {} tool turns), allowing",
            analysis.user_messages, analysis.tool_using_turns
        );
        return ExitCode::SUCCESS;
    }

    let mut uncaptured_topics = Vec::new();
    for topic in &analysis.insight_topics {
        let topic_lower = topic.to_lowercase();
        // Check if captured (written to Memory/Insights/)
        let captured = analysis.insights_written.iter().any(|written| {
            let written_base = written
                .strip_suffix(".md")
                .unwrap_or(written)
                .to_lowercase();
            transcript::topic_matches_filename(&topic_lower, &written_base)
        });
        // Check if explicitly skipped via ☆ Insight marker
        let skipped = analysis
            .skipped_topics
            .iter()
            .any(|s| transcript::topic_matches_filename(&topic_lower, s));
        // Check if explicitly captured via ✓ Insight marker
        let explicitly_captured = analysis
            .captured_topics
            .iter()
            .any(|c| transcript::topic_matches_filename(&topic_lower, c));
        if !captured && !skipped && !explicitly_captured {
            uncaptured_topics.push(topic.as_str());
        }
    }

    let unnamed_insights = analysis
        .insight_count
        .saturating_sub(analysis.insight_topics.len());
    let named_matched = analysis
        .insight_topics
        .len()
        .saturating_sub(uncaptured_topics.len());
    let surplus_writes = analysis.insights_write_count.saturating_sub(named_matched);
    let unnamed_uncaptured = unnamed_insights.saturating_sub(surplus_writes);
    let total_uncaptured = uncaptured_topics.len() + unnamed_uncaptured;

    if total_uncaptured > 0 {
        emit_uncaptured(
            &config,
            advisory_mode,
            total_uncaptured,
            &uncaptured_topics,
            unnamed_uncaptured,
        );
    }

    ExitCode::SUCCESS
}
