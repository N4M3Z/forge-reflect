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

    if !forge_reflect::in_data_dir(&input.cwd, &config) {
        eprintln!(
            "forge-reflect[insight]: cwd '{}' outside data dir, skipping",
            input.cwd
        );
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

    // Count-based heuristic: compares insight blocks output vs Insight files written.
    // Known limitation: matches by count, not content. A session that writes 2 insight
    // files about unrelated topics and outputs 2 insight blocks will pass even if they
    // don't correspond. Acceptable for a human-in-the-loop guardrail.
    let uncaptured = analysis
        .insight_count
        .saturating_sub(analysis.insights_write_count);

    if uncaptured > 0 {
        eprintln!(
            "forge-reflect[insight]: blocking — {} insight(s), {} Insight file(s) written",
            analysis.insight_count, analysis.insights_write_count
        );
        let base_reason = prompt::load_pattern(&input.cwd, &config.insight_pattern)
            .unwrap_or_else(|| config.uncaptured_insight_reason.clone());

        let reason = format!(
            "{base_reason} ({} insight(s) found, {} Insight file(s) written)",
            analysis.insight_count, analysis.insights_write_count
        );
        let output = serde_json::json!({
            "decision": "block",
            "reason": reason
        });
        println!("{output}");
    }

    ExitCode::SUCCESS
}
