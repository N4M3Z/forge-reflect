use super::*;

#[test]
fn test_default_values() {
    let config = Config::default();
    assert_eq!(config.insight_marker, "\u{2605} Insight");
    assert_eq!(config.memory_paths.len(), 2);
    assert_eq!(config.tool_turn_threshold, 10);
    assert_eq!(config.user_msg_threshold, 4);
    assert_eq!(
        config.reflection,
        "Orchestration/Skills/SessionReflect/SKILL.md"
    );
    assert_eq!(
        config.insight_check,
        "Orchestration/Skills/InsightCheck/SKILL.md"
    );
    assert_eq!(
        config.memory.imperatives,
        "Orchestration/Memory/Imperatives"
    );
    assert_eq!(config.memory.insights, "Orchestration/Memory/Insights");
    assert_eq!(config.memory.ideas, "Orchestration/Memory/Ideas");
    assert_eq!(
        config.journal.daily,
        "Resources/Journals/Daily/YYYY/MM/YYYY-MM-DD.md"
    );
    assert_eq!(config.backlog, "Orchestration/Backlog.md");
    assert_eq!(config.commands.safe_read, "Modules/forge-tlp/bin/safe-read");
}

#[test]
fn test_partial_yaml_uses_defaults_for_missing() {
    let yaml = "insight_marker: \"custom marker\"\ntool_turn_threshold: 20\n";
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.insight_marker, "custom marker");
    assert_eq!(config.tool_turn_threshold, 20);
    // Missing fields get defaults
    assert_eq!(config.user_msg_threshold, 4);
    assert_eq!(config.insights_path(), "Memory/Insights/");
    assert_eq!(
        config.memory.imperatives,
        "Orchestration/Memory/Imperatives"
    );
}

#[test]
fn test_empty_yaml_gives_defaults() {
    let config: Config = serde_yaml::from_str("{}").unwrap();
    assert_eq!(config.insight_marker, "\u{2605} Insight");
    assert_eq!(config.tool_turn_threshold, 10);
}

#[test]
fn test_load_without_env_var_gives_defaults() {
    // CLAUDE_PLUGIN_ROOT is not set in test environment
    std::env::remove_var("CLAUDE_PLUGIN_ROOT");
    let config = Config::load();
    assert_eq!(config.insight_marker, "\u{2605} Insight");
}

#[test]
fn test_nested_yaml_deserialization() {
    let yaml = r#"
memory:
  imperatives: custom/imperatives
  insights: custom/insights
  ideas: custom/ideas
journal:
  daily: custom/YYYY-MM-DD.md
commands:
  safe_read: custom/safe-read
backlog: custom/backlog.md
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.memory.imperatives, "custom/imperatives");
    assert_eq!(config.memory.insights, "custom/insights");
    assert_eq!(config.memory.ideas, "custom/ideas");
    assert_eq!(config.journal.daily, "custom/YYYY-MM-DD.md");
    assert_eq!(config.commands.safe_read, "custom/safe-read");
    assert_eq!(config.backlog, "custom/backlog.md");
    // Non-overridden fields keep defaults
    assert_eq!(config.insight_marker, "\u{2605} Insight");
}

#[test]
fn test_resolve_user_path() {
    let mut config = Config::default();
    config.user_root = "/home/user/vault".to_string();

    let result = config.resolve_user_path("/home/user/data", "Orchestration/Backlog.md");
    assert_eq!(
        result,
        std::path::PathBuf::from("/home/user/vault/Orchestration/Backlog.md")
    );
}

#[test]
fn test_resolve_user_path_falls_back_to_cwd() {
    let config = Config::default(); // user_root is empty

    let result = config.resolve_user_path("/home/user/data", "Orchestration/Backlog.md");
    assert_eq!(
        result,
        std::path::PathBuf::from("/home/user/data/Orchestration/Backlog.md")
    );
}
