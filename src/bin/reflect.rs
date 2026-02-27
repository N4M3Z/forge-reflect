use forge_reflect::config::Config;
use forge_reflect::prompt;
use forge_reflect::transcript;
use std::fmt::Write;
use std::fs;
use std::process::ExitCode;

/// Hook contract: exit 0 always. Block/allow is communicated via JSON on stdout.
/// Empty stdout = allow. `{"decision":"block","reason":"..."}` = block.
fn main() -> ExitCode {
    let config = Config::load();

    let Some(input) = forge_reflect::read_hook_input() else {
        return ExitCode::SUCCESS;
    };

    let cwd = if input.cwd.is_empty() {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        input.cwd
    };

    // PreCompact: inject reflection prompt with reusability filter + uncaptured topics.
    // Runs everywhere (no directory scope check).
    if input.trigger.is_some() {
        // Load reflection skill (includes reusability filter section)
        let skill_path = config.resolve_user_path(&cwd, &config.reflection);
        let reason =
            prompt::load_pattern_abs(&skill_path).unwrap_or_else(|| config.fallback_reason.clone());

        // Read transcript + compute uncaptured topics (graceful fallback)
        let mut topics_section = String::new();
        if !input.transcript_path.is_empty() {
            if let Ok(transcript) = fs::read_to_string(&input.transcript_path) {
                let analysis = transcript::analyze_transcript(&transcript, &config);
                let uncaptured = compute_uncaptured_topics(&analysis);
                if !uncaptured.is_empty() {
                    let capped: Vec<_> = uncaptured.iter().take(5).copied().collect();
                    let _ = write!(
                        topics_section,
                        "\n\nUncaptured topics from this session: {}.",
                        capped.join(", ")
                    );
                }
                eprintln!(
                    "forge-reflect[reflect]: precompact - {} msgs, {} tool turns, {} min, {} uncaptured",
                    analysis.user_messages,
                    analysis.tool_using_turns,
                    analysis.session_duration_minutes,
                    uncaptured.len()
                );
            }
        }

        let output = serde_json::json!({
            "additionalContext": format!("{}{reason}{topics_section}", config.precompact_prefix)
        });
        println!("{output}");
        return ExitCode::SUCCESS;
    }

    // Stop hook guards
    if input.stop_hook_active {
        eprintln!("forge-reflect[reflect]: stop_hook_active, deferring");
        return ExitCode::SUCCESS;
    }

    if !forge_reflect::in_data_dir(&cwd, &config) {
        eprintln!("forge-reflect[reflect]: cwd '{cwd}' outside data dir, skipping");
        return ExitCode::SUCCESS;
    }

    let Ok(transcript) = fs::read_to_string(&input.transcript_path) else {
        eprintln!(
            "forge-reflect[reflect]: transcript unreadable at '{}', skipping",
            input.transcript_path
        );
        return ExitCode::SUCCESS;
    };

    let analysis = transcript::analyze_transcript(&transcript, &config);

    // Not substantial (duration + floor) -> allow stop
    let below_duration =
        analysis.session_duration_minutes < u64::from(config.duration_threshold_minutes);
    let below_floor = analysis.user_messages < config.user_msg_floor;
    if below_duration || below_floor {
        eprintln!(
            "forge-reflect[reflect]: session not substantial ({} min, {} msgs), allowing",
            analysis.session_duration_minutes, analysis.user_messages
        );
        return ExitCode::SUCCESS;
    }

    // Substantial + memory writes -> allow stop
    if analysis.has_memory_write {
        return ExitCode::SUCCESS;
    }

    // Substantial + no memory writes -> block (force reflection)
    if config.reflect_blocking.unwrap_or(true) {
        eprintln!(
            "forge-reflect[reflect]: blocking \u{2014} substantial session ({} min, {} msgs) with no memory writes",
            analysis.session_duration_minutes, analysis.user_messages
        );
        let skill_path = config.resolve_user_path(&cwd, &config.reflection);
        let reason =
            prompt::load_pattern_abs(&skill_path).unwrap_or_else(|| config.fallback_reason.clone());
        let output = serde_json::json!({
            "decision": "block",
            "reason": reason
        });
        println!("{output}");
    } else {
        eprintln!(
            "forge-reflect[reflect]: warn \u{2014} substantial session ({} min, {} msgs) with no memory writes",
            analysis.session_duration_minutes, analysis.user_messages
        );
    }

    ExitCode::SUCCESS
}

/// Compute insight topics that were mentioned but not written to files.
fn compute_uncaptured_topics(analysis: &transcript::TranscriptAnalysis) -> Vec<&str> {
    analysis
        .insight_topics
        .iter()
        .filter_map(|topic| {
            let topic_lower = topic.to_lowercase();
            let matched = analysis.insights_written.iter().any(|written| {
                let base = written
                    .strip_suffix(".md")
                    .unwrap_or(written)
                    .to_lowercase();
                transcript::topic_matches_filename(&topic_lower, &base)
            });
            if matched {
                None
            } else {
                Some(topic.as_str())
            }
        })
        .collect()
}
