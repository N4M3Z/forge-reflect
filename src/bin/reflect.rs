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

    // PreCompact: always inject reflection prompt, regardless of directory scope.
    // This ensures compaction context includes reflection guidance everywhere.
    if input.trigger.is_some() {
        let reason = prompt::load_pattern(&input.cwd, &config.reflection_pattern)
            .unwrap_or_else(|| config.fallback_reason.clone());

        let output = serde_json::json!({
            "additionalContext": format!("{}{reason}", config.precompact_prefix)
        });
        println!("{output}");
        return ExitCode::SUCCESS;
    }

    // Stop hook guards
    if input.stop_hook_active {
        eprintln!("forge-reflect[reflect]: stop_hook_active, deferring");
        return ExitCode::SUCCESS;
    }

    if !forge_reflect::in_data_dir(&input.cwd, &config) {
        eprintln!(
            "forge-reflect[reflect]: cwd '{}' outside data dir, skipping",
            input.cwd
        );
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

    // Not substantial → allow stop
    if analysis.user_messages < config.user_msg_threshold
        || analysis.tool_using_turns < config.tool_turn_threshold
    {
        eprintln!(
            "forge-reflect[reflect]: session not substantial ({} msgs, {} tool turns), allowing",
            analysis.user_messages, analysis.tool_using_turns
        );
        return ExitCode::SUCCESS;
    }

    // Substantial + memory writes → allow stop
    if analysis.has_memory_write {
        return ExitCode::SUCCESS;
    }

    // Substantial + no memory writes → block and prompt reflection
    eprintln!("forge-reflect[reflect]: blocking — substantial session with no memory writes");
    let reason = prompt::load_pattern(&input.cwd, &config.reflection_pattern)
        .unwrap_or_else(|| config.fallback_reason.clone());

    let output = serde_json::json!({
        "decision": "block",
        "reason": reason
    });
    println!("{output}");

    ExitCode::SUCCESS
}
