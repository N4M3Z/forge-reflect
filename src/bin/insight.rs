use forge_reflect::config::Config;
use forge_reflect::transcript;
use std::fs;
use std::process::ExitCode;

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

    // Substantiality gate — mirrors reflect.rs thresholds.
    // Short sessions pass through without insight enforcement.
    if analysis.user_messages < config.user_msg_threshold
        || analysis.tool_using_turns < config.tool_turn_threshold
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
        let matched = analysis.insights_written.iter().any(|written| {
            let written_base = written
                .strip_suffix(".md")
                .unwrap_or(written)
                .to_lowercase();
            transcript::topic_matches_filename(&topic_lower, &written_base)
        });
        if !matched {
            uncaptured_topics.push(topic.as_str());
        }
    }

    // Insight markers without extracted topics (decorative format, no colon)
    let unnamed_insights = analysis
        .insight_count
        .saturating_sub(analysis.insight_topics.len());
    // Named topics that matched a written file
    let named_matched = analysis
        .insight_topics
        .len()
        .saturating_sub(uncaptured_topics.len());
    // Writes beyond those accounted for by named matches
    let surplus_writes = analysis.insights_write_count.saturating_sub(named_matched);
    let unnamed_uncaptured = unnamed_insights.saturating_sub(surplus_writes);

    // Final uncaptured count
    let total_uncaptured = uncaptured_topics.len() + unnamed_uncaptured;

    if total_uncaptured > 0 {
        let mut reason_detail = String::new();
        if !uncaptured_topics.is_empty() {
            reason_detail = format!(": {}", uncaptured_topics.join(", "));
        } else if unnamed_uncaptured > 0 {
            reason_detail = format!(" ({unnamed_uncaptured} unnamed)");
        }

        // Warn only — never block exit for uncaptured insights.
        eprintln!(
            "forge-reflect[insight]: warn - {total_uncaptured} uncaptured insight(s){reason_detail}"
        );
    }

    ExitCode::SUCCESS
}
