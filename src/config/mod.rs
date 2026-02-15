use serde::Deserialize;
use std::path::{Path, PathBuf};

/// All configurable values for forge-reflect. Loaded from `config.yaml`
/// (or `defaults.yaml`) in the plugin root directory. Falls back to compiled
/// defaults if the file is missing or unreadable.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    // Transcript analysis
    pub insight_marker: String,
    /// Path fragments for substring matching in transcript `tool_use` entries.
    /// First element is the insights path (used for insight counting).
    pub memory_paths: Vec<String>,

    // Substantiality thresholds
    pub tool_turn_threshold: usize,
    pub user_msg_threshold: usize,

    // Skill file paths (user-root-relative)
    pub reflection: String,
    pub insight_check: String,

    // Data directory suffix for scope check
    pub data_dir_suffix: String,

    // Hook message strings
    pub fallback_reason: String,
    pub precompact_prefix: String,
    pub uncaptured_insight_reason: String,

    // Nested groups
    pub memory: MemoryConfig,
    pub journal: JournalConfig,
    pub commands: CommandsConfig,

    // Flat
    pub backlog: String,

    // Surfacing
    pub surface: SurfaceConfig,

    // Resolved at load time (not deserialized from YAML)
    #[serde(skip)]
    pub user_root: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub imperatives: String,
    pub insights: String,
    pub ideas: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct JournalConfig {
    pub daily: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CommandsConfig {
    pub safe_read: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SurfaceConfig {
    pub archive_dir: String,
    pub archive_prefix: String,
    pub reminders_list: String,
    pub ideas_cutoff_days: u32,
    pub due_soon_days: u32,
    pub max_items: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            insight_marker: "\u{2605} Insight".to_string(),
            memory_paths: vec![
                "Memory/Insights/".to_string(),
                "Memory/Imperatives/".to_string(),
            ],
            tool_turn_threshold: 10,
            user_msg_threshold: 4,
            reflection: "Orchestration/Skills/SessionReflect/SKILL.md".to_string(),
            insight_check: "Orchestration/Skills/InsightCheck/SKILL.md".to_string(),
            data_dir_suffix: "Data".to_string(),
            fallback_reason: "Substantial session with no insights captured. Create a file in \
                Memory/Insights/ or Memory/Imperatives/ before ending."
                .to_string(),
            precompact_prefix:
                "BEFORE COMPACTING \u{2014} capture session insights and imperatives now. "
                    .to_string(),
            uncaptured_insight_reason:
                "Uncaptured insights detected. Rule 12: every \u{2605} Insight block \
                MUST be persisted as a Memory/Insights/ file before ending."
                    .to_string(),
            memory: MemoryConfig::default(),
            journal: JournalConfig::default(),
            commands: CommandsConfig::default(),
            backlog: "Orchestration/Backlog.md".to_string(),
            surface: SurfaceConfig::default(),
            user_root: String::new(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            imperatives: "Orchestration/Memory/Imperatives".to_string(),
            insights: "Orchestration/Memory/Insights".to_string(),
            ideas: "Orchestration/Memory/Ideas".to_string(),
        }
    }
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            daily: "Resources/Journals/Daily/YYYY/MM/YYYY-MM-DD.md".to_string(),
        }
    }
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            safe_read: "Modules/forge-tlp/bin/safe-read".to_string(),
        }
    }
}

impl Default for SurfaceConfig {
    fn default() -> Self {
        Self {
            archive_dir: "Resources/Archives".to_string(),
            archive_prefix: "Safari Tab Snapshot".to_string(),
            reminders_list: "work".to_string(),
            ideas_cutoff_days: 14,
            due_soon_days: 3,
            max_items: 5,
        }
    }
}

impl Config {
    /// Insights path fragment, derived from first element of `memory_paths`.
    /// Used for counting insight file writes in transcript analysis.
    pub fn insights_path(&self) -> &str {
        self.memory_paths
            .first()
            .map_or("Memory/Insights/", |s| s.as_str())
    }

    /// Load config from `$CLAUDE_PLUGIN_ROOT/{config,defaults}.yaml`.
    /// Returns defaults if the env var is unset or the file is unreadable.
    /// Always resolves user_root (via env var or forge.yaml discovery).
    pub fn load() -> Self {
        let mut config: Self = match std::env::var("CLAUDE_PLUGIN_ROOT") {
            Ok(root) => {
                let plugin_root = Path::new(&root);
                forge_core::yaml::load_config(plugin_root).unwrap_or_else(|e| {
                    eprintln!("forge-reflect: {e}, using defaults");
                    Self::default()
                })
            }
            Err(_) => {
                eprintln!("forge-reflect: CLAUDE_PLUGIN_ROOT not set, using defaults");
                Self::default()
            }
        };

        // Always resolve user root — works with env var, caller args, or discovery
        config.user_root = forge_core::yaml::user_root("", "");

        // Overlay project-level shared config (forge.yaml) onto compiled defaults.
        // Module config.yaml overrides (already loaded above) take precedence.
        let project = forge_core::project::ProjectConfig::load();
        config.apply_shared(&project);
        config
    }

    /// Apply project-level shared values where this config still has compiled defaults.
    fn apply_shared(&mut self, project: &forge_core::project::ProjectConfig) {
        use forge_core::project::apply_if_default;
        let defaults = Self::default();
        apply_if_default(
            &mut self.commands.safe_read,
            &defaults.commands.safe_read,
            &project.commands.safe_read,
        );
        apply_if_default(
            &mut self.journal.daily,
            &defaults.journal.daily,
            &project.journal.daily,
        );
        apply_if_default(&mut self.backlog, &defaults.backlog, &project.backlog);
        apply_if_default(
            &mut self.memory.imperatives,
            &defaults.memory.imperatives,
            &project.memory.imperatives,
        );
        apply_if_default(
            &mut self.memory.insights,
            &defaults.memory.insights,
            &project.memory.insights,
        );
        apply_if_default(
            &mut self.memory.ideas,
            &defaults.memory.ideas,
            &project.memory.ideas,
        );
    }

    /// Resolve a user-content path against user_root (or cwd fallback).
    pub fn resolve_user_path(&self, cwd: &str, relative: &str) -> PathBuf {
        forge_core::yaml::resolve_path(&self.user_root, cwd, relative)
    }
}

#[cfg(test)]
mod tests;
