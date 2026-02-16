use forge_reflect::config::Config;
use forge_reflect::prompt;
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

    let mut uncaptured_topics = Vec::new();
    for topic in &analysis.insight_topics {
        let topic_lower = topic.to_lowercase();
        let mut matched = false;
        for written in &analysis.insights_written {
            let written_base = written
                .strip_suffix(".md")
                .unwrap_or(written)
                .to_lowercase();
            // Match if topic is in filename or vice versa, or if they share significant overlap
            if written_base.contains(&topic_lower) || topic_lower.contains(&written_base) {
                matched = true;
                break;
            }
        }
        if !matched {
            uncaptured_topics.push(topic.as_str());
        }
    }

    // Handle cases where markers exist but no topics were extracted
    let unnamed_uncaptured = analysis
        .insight_count
        .saturating_sub(analysis.insight_topics.len())
        .saturating_sub(
            analysis.insights_write_count.saturating_sub(
                analysis
                    .insight_topics
                    .len()
                    .saturating_sub(uncaptured_topics.len()),
            ),
        );

    // Final uncaptured count
    let total_uncaptured = uncaptured_topics.len() + unnamed_uncaptured;

    if total_uncaptured > 0 {
        let mut reason_detail = String::new();
        if !uncaptured_topics.is_empty() {
            reason_detail = format!(": {}", uncaptured_topics.join(", "));
        } else if unnamed_uncaptured > 0 {
            reason_detail = format!(" ({unnamed_uncaptured} unnamed)");
        }

        eprintln!(
            "forge-reflect[insight]: blocking — {total_uncaptured} uncaptured insight(s){reason_detail}"
        );
        let skill_path = config.resolve_user_path(&cwd, &config.insight_check);
        let base_reason = prompt::load_pattern_abs(&skill_path)
            .unwrap_or_else(|| config.uncaptured_insight_reason.clone());

        let reason = format!(
            "{} ({} uncaptured insight(s){})",
            base_reason.trim(),
            total_uncaptured,
            reason_detail
        );
        let output = serde_json::json!({
            "decision": "block",
            "reason": reason
        });
        println!("{output}");
    }

    ExitCode::SUCCESS
}
