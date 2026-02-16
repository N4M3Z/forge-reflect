// Library crate — modules are re-exported for use by binaries in src/bin/.
pub mod config;
pub mod prompt;
pub mod surface;
pub mod transcript;

use clap::Parser;
use serde::Deserialize;
use std::io::{IsTerminal, Read};

/// JSON payload or CLI arguments from AI coding tool hook events.
#[derive(Parser, Deserialize, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct HookInput {
    /// True when a Stop hook previously blocked in this cycle.
    #[serde(default)]
    #[arg(long)]
    pub stop_hook_active: bool,

    /// Current working directory.
    #[serde(default)]
    #[arg(long, default_value = "")]
    pub cwd: String,

    /// Path to session transcript JSONL (Stop hooks only).
    #[serde(default)]
    #[arg(long, default_value = "")]
    pub transcript_path: String,

    /// Present in `PreCompact` hooks: "manual" or "auto".
    #[serde(default)]
    #[arg(long)]
    pub trigger: Option<String>,
}

/// Read and parse hook input from CLI args or stdin.
/// CLI arguments take precedence. If no CLI args (other than binary name) are present,
/// it falls back to reading JSON from stdin.
pub fn read_hook_input() -> Option<HookInput> {
    // Try CLI first
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        return Some(HookInput::parse());
    }

    // Fallback to stdin
    let mut buf = String::new();

    // Use a small timeout or non-blocking read if we wanted to be super safe,
    // but hooks usually have stdin piped.
    // For manual use without args, this might hang.
    // We can check if stdin is a terminal.
    if std::io::stdin().is_terminal() {
        return Some(HookInput::default());
    }

    if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
        eprintln!("forge-reflect: failed to read stdin: {e}");
        return None;
    }

    if buf.trim().is_empty() {
        return Some(HookInput::default());
    }

    match serde_json::from_str(&buf) {
        Ok(input) => Some(input),
        Err(e) => {
            eprintln!("forge-reflect: failed to parse hook input JSON: {e}");
            None
        }
    }
}

/// Check if cwd is inside `$HOME/<suffix>` (or `$HOME/<suffix>/...`).
/// Returns false if HOME is unset. Trailing `/` prevents prefix collisions
/// (e.g., `~/DataBackup` won't match when suffix is `Data`).
pub fn in_data_dir(cwd: &str, config: &config::Config) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        eprintln!("forge-reflect: HOME not set, cannot determine data dir scope");
        return false;
    }
    let prefix = format!("{home}/{}", config.data_dir_suffix);
    cwd == prefix || cwd.starts_with(&format!("{prefix}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with_suffix(suffix: &str) -> config::Config {
        let mut c = config::Config::default();
        c.data_dir_suffix = suffix.to_string();
        c
    }

    #[test]
    fn test_exact_match() {
        std::env::set_var("HOME", "/Users/test");
        assert!(in_data_dir("/Users/test/Data", &cfg_with_suffix("Data")));
    }

    #[test]
    fn test_subdirectory() {
        std::env::set_var("HOME", "/Users/test");
        assert!(in_data_dir(
            "/Users/test/Data/Plugins",
            &cfg_with_suffix("Data")
        ));
    }

    #[test]
    fn test_prefix_collision_rejected() {
        std::env::set_var("HOME", "/Users/test");
        assert!(!in_data_dir(
            "/Users/test/DataBackup",
            &cfg_with_suffix("Data")
        ));
    }

    #[test]
    fn test_different_path() {
        std::env::set_var("HOME", "/Users/test");
        assert!(!in_data_dir(
            "/Users/test/Projects",
            &cfg_with_suffix("Data")
        ));
    }

    #[test]
    fn test_empty_home_returns_false() {
        std::env::set_var("HOME", "");
        assert!(!in_data_dir("/Data", &cfg_with_suffix("Data")));
    }
}
