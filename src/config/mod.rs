use serde::Deserialize;
use std::fs;
use std::path::Path;

/// All configurable values for forge-reflect. Loaded from `config.yaml`
/// in the plugin root directory. Falls back to compiled defaults if the
/// file is missing or unreadable.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    // Transcript analysis
    pub insight_marker: String,
    /// Path fragments for substring matching in transcript `tool_use` entries.
    /// First element is the learnings path (used for insight counting).
    pub memory_paths: Vec<String>,

    // Substantiality thresholds
    pub tool_turn_threshold: usize,
    pub user_msg_threshold: usize,

    // Pattern file paths (relative to cwd)
    pub reflection_pattern: String,
    pub insight_pattern: String,

    // Data directory suffix for scope check
    pub data_dir_suffix: String,

    // Hook message strings
    pub fallback_reason: String,
    pub precompact_prefix: String,
    pub uncaptured_insight_reason: String,

    // Skill-facing paths — full vault-relative paths for /reflect and /insight skills.
    // Not used by Rust binaries; present so config.yaml is a single source of truth
    // for both Rust hooks and skill prompts.
    pub memory_decisions_path: String,
    pub memory_learnings_path: String,
    pub memory_ideas_path: String,
    pub journal_daily_path: String,
    pub backlog_path: String,
    pub safe_read_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            insight_marker: "\u{2605} Insight".to_string(),
            memory_paths: vec![
                "Memory/Learnings/".to_string(),
                "Memory/Decisions/".to_string(),
            ],
            tool_turn_threshold: 10,
            user_msg_threshold: 4,
            reflection_pattern: "Vaults/Personal/Orchestration/Patterns/Session Reflect.md"
                .to_string(),
            insight_pattern: "Vaults/Personal/Orchestration/Patterns/Insight Check.md".to_string(),
            data_dir_suffix: "Data".to_string(),
            fallback_reason: "Substantial session with no learnings captured. Create a file in \
                Memory/Learnings/ or Memory/Decisions/ before ending."
                .to_string(),
            precompact_prefix:
                "BEFORE COMPACTING \u{2014} capture session learnings and decisions now. "
                    .to_string(),
            uncaptured_insight_reason:
                "Uncaptured insights detected. Rule 12: every \u{2605} Insight block \
                MUST be persisted as a Memory/Learnings/ file before ending."
                    .to_string(),
            memory_decisions_path: "Vaults/Personal/Orchestration/Memory/Decisions".to_string(),
            memory_learnings_path: "Vaults/Personal/Orchestration/Memory/Learnings".to_string(),
            memory_ideas_path: "Vaults/Personal/Orchestration/Memory/Ideas".to_string(),
            journal_daily_path: "Vaults/Personal/Resources/Journals/Daily/YYYY/MM/YYYY-MM-DD.md"
                .to_string(),
            backlog_path: "Vaults/Personal/Orchestration/Backlog.md".to_string(),
            safe_read_command: "Modules/forge-tlp/bin/safe-read".to_string(),
        }
    }
}

impl Config {
    /// Learnings path fragment, derived from first element of `memory_paths`.
    /// Used for counting learning file writes in transcript analysis.
    pub fn learnings_path(&self) -> &str {
        self.memory_paths
            .first()
            .map_or("Memory/Learnings/", |s| s.as_str())
    }

    /// Load config from `$CLAUDE_PLUGIN_ROOT/config.yaml`.
    /// Returns defaults if the env var is unset or the file is unreadable.
    /// All fallback paths are logged to stderr for debugging.
    pub fn load() -> Self {
        let Some(root) = std::env::var("CLAUDE_PLUGIN_ROOT").ok() else {
            eprintln!("forge-reflect: CLAUDE_PLUGIN_ROOT not set, using defaults");
            return Self::default();
        };

        let path = Path::new(&root).join("config.yaml");
        let Ok(content) = fs::read_to_string(&path) else {
            eprintln!(
                "forge-reflect: config unreadable at {}, using defaults",
                path.display()
            );
            return Self::default();
        };

        match serde_yaml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("forge-reflect: config parse error: {e}, using defaults");
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests;
