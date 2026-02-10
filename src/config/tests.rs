use super::*;

#[test]
fn test_default_values() {
    let config = Config::default();
    assert_eq!(config.insight_marker, "\u{2605} Insight");
    assert_eq!(config.memory_paths.len(), 2);
    assert_eq!(config.tool_turn_threshold, 10);
    assert_eq!(config.user_msg_threshold, 4);
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
