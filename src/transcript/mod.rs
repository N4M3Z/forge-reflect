use crate::config::Config;
use serde_json::Value;

pub struct TranscriptAnalysis {
    pub user_messages: usize,
    pub tool_using_turns: usize,
    pub has_memory_write: bool,
    pub insight_count: usize,
    pub insights_write_count: usize,
    /// List of insight topics extracted from ★ Insight blocks.
    pub insight_topics: Vec<String>,
    /// List of filenames written to the insights directory.
    pub insights_written: Vec<String>,
}

/// Analyze transcript for user messages, tool-using turns, memory writes, and insights.
pub fn analyze_transcript(transcript: &str, config: &Config) -> TranscriptAnalysis {
    let mut analysis = TranscriptAnalysis {
        user_messages: 0,
        tool_using_turns: 0,
        has_memory_write: false,
        insight_count: 0,
        insights_write_count: 0,
        insight_topics: Vec::new(),
        insights_written: Vec::new(),
    };

    // Regex to find ★ Insight blocks and capture the topic.
    // Matches "★ Insight: Topic" and "★ Insight Topic".
    let insight_re = regex::Regex::new(&format!(
        r"{}\s*:?\s*(.*)",
        regex::escape(&config.insight_marker)
    ))
    .expect("insight marker regex must compile");

    for line in transcript.lines() {
        let entry: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if is_user_entry(&entry) {
            analysis.user_messages += 1;
            continue;
        }

        if !is_assistant_entry(&entry) {
            continue;
        }

        let mut turn_has_tool_use = false;

        if let Some(content) = assistant_content(&entry) {
            for item in content {
                if let Some(text) = extract_text(item) {
                    for cap in insight_re.captures_iter(text) {
                        analysis.insight_count += 1;
                        let topic = cap[1].trim();
                        if !topic.is_empty() {
                            analysis.insight_topics.push(topic.to_string());
                        }
                    }
                }

                if !is_tool_use_item(item) {
                    continue;
                }
                turn_has_tool_use = true;

                let Some(tool_name) = extract_tool_name(item) else {
                    continue;
                };
                let is_write_tool = config
                    .write_tool_names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(tool_name));
                if !is_write_tool {
                    continue;
                }

                let Some(file_path) = extract_file_path(item) else {
                    continue;
                };

                if file_path.contains(config.insights_path()) {
                    analysis.insights_write_count += 1;
                    if let Some(filename) = std::path::Path::new(&file_path).file_name() {
                        analysis
                            .insights_written
                            .push(filename.to_string_lossy().into_owned());
                    }
                }

                for memory_path in &config.memory_paths {
                    if file_path.contains(memory_path.as_str()) {
                        analysis.has_memory_write = true;
                    }
                }
            }
        }

        if turn_has_tool_use {
            analysis.tool_using_turns += 1;
        }
    }

    analysis
}

fn is_user_entry(entry: &Value) -> bool {
    let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or("");
    let role = entry.get("role").and_then(Value::as_str).unwrap_or("");
    entry_type == "human" || entry_type == "user" || role == "user"
}

fn is_assistant_entry(entry: &Value) -> bool {
    let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or("");
    let role = entry.get("role").and_then(Value::as_str).unwrap_or("");
    entry_type == "assistant" || role == "assistant"
}

fn assistant_content(entry: &Value) -> Option<&Vec<Value>> {
    entry
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array)
        .or_else(|| entry.get("content").and_then(Value::as_array))
}

fn extract_text(item: &Value) -> Option<&str> {
    if let Some(t) = item.get("text").and_then(Value::as_str) {
        return Some(t);
    }
    item.get("content").and_then(Value::as_str)
}

fn is_tool_use_item(item: &Value) -> bool {
    let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
    if item_type == "tool_use" || item_type == "tool_call" || item_type == "function_call" {
        return true;
    }

    item.get("name").is_some()
        && (item.get("input").is_some()
            || item.get("tool_input").is_some()
            || item.get("arguments").is_some())
}

fn extract_tool_name(item: &Value) -> Option<&str> {
    item.get("name")
        .and_then(Value::as_str)
        .or_else(|| item.get("tool_name").and_then(Value::as_str))
        .or_else(|| item.get("tool").and_then(Value::as_str))
}

fn extract_file_path(item: &Value) -> Option<String> {
    let key_candidates = ["file_path", "path", "target_file", "target_path"];

    for key in &key_candidates {
        if let Some(path) = item.get(*key).and_then(Value::as_str) {
            return Some(path.to_string());
        }
    }

    let nested_candidates = ["input", "tool_input", "arguments", "params"];
    for nested in &nested_candidates {
        if let Some(obj) = item.get(*nested).and_then(Value::as_object) {
            for key in &key_candidates {
                if let Some(path) = obj.get(*key).and_then(Value::as_str) {
                    return Some(path.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests;
