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
    /// Tool names that are treated as file-writing operations in transcripts.
    pub write_tool_names: Vec<String>,

    // Substantiality thresholds
    pub tool_turn_threshold: usize,
    pub user_msg_threshold: usize,
    pub duration_threshold_minutes: u32,
    pub user_msg_floor: usize,

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

    // User content root (deserialized from YAML)
    pub user: UserConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub root: String,
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
            write_tool_names: vec![
                "Edit".to_string(),
                "Write".to_string(),
                "edit".to_string(),
                "write".to_string(),
                "safe-write".to_string(),
            ],
            tool_turn_threshold: 10,
            user_msg_threshold: 4,
            duration_threshold_minutes: 15,
            user_msg_floor: 2,
            reflection: "Orchestration/Skills/SessionReflect/SKILL.md".to_string(),
            insight_check: "Orchestration/Skills/InsightCheck/SKILL.md".to_string(),
            data_dir_suffix: "Data".to_string(),
            fallback_reason: "Substantial session with no insights captured. Create a file in \
                Memory/Insights/ or Memory/Imperatives/ before ending."
                .to_string(),
            precompact_prefix:
                "BEFORE COMPACTING - apply the reusability filter below to each insight. \
                Capture reusable patterns as Memory/Insights/ files, let one-off traces compact away. "
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
            user: UserConfig::default(),
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

    /// Load config from `{config,defaults}.yaml` in the module root directory.
    /// Discovery order: `FORGE_MODULE_ROOT` env → `CLAUDE_PLUGIN_ROOT` env →
    /// binary path self-discovery (target/release/ → 3 levels up).
    /// Returns compiled defaults if no module root is found.
    pub fn load() -> Self {
        let module_root = std::env::var("FORGE_MODULE_ROOT")
            .or_else(|_| std::env::var("CLAUDE_PLUGIN_ROOT"))
            .or_else(|_| {
                // Binary is at target/release/<name>, module root is 3 levels up
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent()?.parent()?.parent().map(Path::to_path_buf))
                    .filter(|p| p.join("config.yaml").exists() || p.join("defaults.yaml").exists())
                    .map(|p| p.to_string_lossy().into_owned())
                    .ok_or(std::env::VarError::NotPresent)
            });

        let mut config: Self = if let Ok(root) = module_root {
            let plugin_root = Path::new(&root);
            let defaults = forge_lib::sidecar::load_yaml_file(&plugin_root.join("defaults.yaml"))
                .unwrap_or(serde_yaml::Value::Null);
            let overlay = forge_lib::sidecar::load_yaml_file(&plugin_root.join("config.yaml"))
                .unwrap_or(serde_yaml::Value::Null);
            let merged = forge_lib::sidecar::merge_values(defaults, overlay);
            serde_yaml::from_value(merged).unwrap_or_else(|e| {
                eprintln!("forge-reflect: {e}, using defaults");
                Self::default()
            })
        } else {
            eprintln!("forge-reflect: module root not found, using defaults");
            Self::default()
        };

        // Resolve relative user.root against $HOME
        if !config.user.root.is_empty() && !Path::new(&config.user.root).is_absolute() {
            if let Ok(home) = std::env::var("HOME") {
                config.user.root = Path::new(&home)
                    .join(&config.user.root)
                    .to_string_lossy()
                    .into_owned();
            }
        }

        config
    }

    /// Resolve a user-content path against `user.root` (or cwd fallback).
    pub fn resolve_user_path(&self, cwd: &str, relative: &str) -> PathBuf {
        if Path::new(relative).is_absolute() {
            return PathBuf::from(relative);
        }
        if self.user.root.is_empty() {
            Path::new(cwd).join(relative)
        } else {
            Path::new(&self.user.root).join(relative)
        }
    }
}

#[cfg(test)]
mod tests;
