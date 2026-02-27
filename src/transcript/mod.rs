use crate::config::Config;
use serde_json::Value;
use std::collections::HashSet;

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
    /// Session duration in minutes (last timestamp - first timestamp). 0 if unavailable.
    pub session_duration_minutes: u64,
    /// Topics explicitly marked as reviewed/skipped via ☆ Insight skip markers.
    pub skipped_topics: Vec<String>,
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
        session_duration_minutes: 0,
        skipped_topics: Vec::new(),
    };

    let mut first_timestamp: Option<chrono::DateTime<chrono::FixedOffset>> = None;
    let mut last_timestamp: Option<chrono::DateTime<chrono::FixedOffset>> = None;

    // Regex to find ★ Insight blocks and capture the topic.
    // Anchored to line-start ((?m)^) so prose ABOUT insights doesn't match.
    // Matches "★ Insight: Topic" and "★ Insight Topic" at start of line.
    let insight_re = regex::Regex::new(&format!(
        r"(?m)^\s*`*\s*{}\s*:?\s*(.*)",
        regex::escape(&config.insight_marker)
    ))
    .expect("insight marker regex must compile");

    let skip_re = regex::Regex::new(&format!(
        r"(?m)^\s*`*\s*{}\s*:?\s*(.*)",
        regex::escape(&config.insight_skip_marker)
    ))
    .expect("insight skip marker regex must compile");

    for line in transcript.lines() {
        let entry: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(ts_str) = entry.get("timestamp").and_then(Value::as_str) {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                if first_timestamp.is_none() {
                    first_timestamp = Some(ts);
                }
                last_timestamp = Some(ts);
            }
        }

        if is_user_entry(&entry) {
            analysis.user_messages += 1;
            // Reset insight tracking on compaction boundary — insights from
            // the previous session were already reviewed/captured there.
            if is_compaction_boundary(&entry) {
                analysis.insight_count = 0;
                analysis.insight_topics.clear();
                analysis.insights_write_count = 0;
                analysis.insights_written.clear();
                analysis.skipped_topics.clear();
            }
            continue;
        }

        if !is_assistant_entry(&entry) {
            continue;
        }

        let mut turn_has_tool_use = false;

        if let Some(content) = assistant_content(&entry) {
            for item in content {
                if let Some(text) = extract_text(item) {
                    scan_text_markers(text, &insight_re, &skip_re, &mut analysis);
                }

                if !is_tool_use_item(item) {
                    continue;
                }
                turn_has_tool_use = true;

                let Some(tool_name) = extract_tool_name(item) else {
                    continue;
                };

                // SessionReflect resets insight tracking — pre-reflection
                // insights were reviewed during reflection, so only
                // post-reflection insights should be checked for capture.
                if tool_name == "Skill" {
                    if let Some(skill) = extract_skill_name(item) {
                        if skill == "SessionReflect" {
                            analysis.insight_count = 0;
                            analysis.insight_topics.clear();
                            analysis.insights_write_count = 0;
                            analysis.insights_written.clear();
                            analysis.skipped_topics.clear();
                        }
                    }
                }

                let is_write_tool = config
                    .write_tool_names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(tool_name));

                let file_path = if is_write_tool {
                    extract_file_path(item)
                } else if tool_name == "Bash" {
                    extract_bash_command(item).and_then(extract_safe_write_path)
                } else {
                    None
                };

                let Some(file_path) = file_path else {
                    continue;
                };

                check_memory_paths(&mut analysis, &file_path, config);
            }
        }

        if turn_has_tool_use {
            analysis.tool_using_turns += 1;
        }
    }

    if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
        let duration = last.signed_duration_since(first);
        analysis.session_duration_minutes = u64::try_from(duration.num_minutes()).unwrap_or(0);
    }

    analysis
}

/// Scan a text block for ★ Insight markers and ☆ Insight skip markers.
fn scan_text_markers(
    text: &str,
    insight_re: &regex::Regex,
    skip_re: &regex::Regex,
    analysis: &mut TranscriptAnalysis,
) {
    for cap in insight_re.captures_iter(text) {
        let topic = cap[1].trim().trim_end_matches('`').trim();
        analysis.insight_count += 1;
        if !topic.is_empty() && !is_decorative(topic) && topic.split_whitespace().count() >= 2 {
            analysis.insight_topics.push(topic.to_string());
        } else {
            // Same-line text is decorative or empty — try next line
            let match_end = cap.get(0).map_or(0, |m| m.end());
            if let Some(next_topic) = extract_next_line_topic(&text[match_end..]) {
                analysis.insight_topics.push(next_topic);
            }
        }
    }
    // Scan for ☆ Insight skip markers (reviewed but intentionally not captured)
    for cap in skip_re.captures_iter(text) {
        let topic = cap[1].trim().trim_end_matches('`').trim();
        if !topic.is_empty() && !is_decorative(topic) && topic.split_whitespace().count() >= 2 {
            analysis.skipped_topics.push(topic.to_lowercase());
        }
    }
}

/// Detect session continuation after context compaction.
/// Claude Code injects a user message with the compaction summary when resuming.
fn is_compaction_boundary(entry: &Value) -> bool {
    let candidates = [
        entry
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_array),
        entry.get("content").and_then(Value::as_array),
    ];
    for arr in candidates.into_iter().flatten() {
        for item in arr {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                if text.contains("continued from a previous conversation") {
                    return true;
                }
            }
        }
    }
    false
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

fn extract_bash_command(item: &Value) -> Option<&str> {
    item.get("input")
        .and_then(|i| i.get("command"))
        .and_then(Value::as_str)
}

fn extract_safe_write_path(command: &str) -> Option<String> {
    let re = regex::Regex::new(r#"safe-write\s+(?:write|edit|insert)\s+"?([^"\n]+)"?"#)
        .expect("safe-write regex must compile");
    re.captures(command)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn check_memory_paths(analysis: &mut TranscriptAnalysis, file_path: &str, config: &Config) {
    if file_path.contains(config.insights_path()) {
        analysis.insights_write_count += 1;
        if let Some(filename) = std::path::Path::new(file_path).file_name() {
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

/// Match insight topic to written filename using word-token overlap.
/// Splits both on non-alphanumeric boundaries, requires at least one
/// shared token of length >= 4 characters. Case-insensitive.
pub fn topic_matches_filename(topic: &str, filename: &str) -> bool {
    let topic_tokens: HashSet<&str> = topic
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 4)
        .collect();
    let file_tokens: HashSet<&str> = filename
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 4)
        .collect();
    topic_tokens.intersection(&file_tokens).next().is_some()
}

/// When `★ Insight` line has decorative border text, extract topic from the next
/// non-empty, non-decorative line. Strips markdown bold markers and trailing punctuation.
fn extract_next_line_topic(remaining: &str) -> Option<String> {
    for line in remaining.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || is_decorative(trimmed) {
            continue;
        }
        let cleaned = trimmed
            .trim_start_matches('*')
            .trim_end_matches('*')
            .trim_end_matches(':')
            .trim();
        if cleaned.split_whitespace().count() >= 2 {
            return Some(cleaned.to_string());
        }
        break;
    }
    None
}

/// Extract the `skill` field from a `Skill` `tool_use` input.
fn extract_skill_name(item: &Value) -> Option<&str> {
    let nested = ["input", "tool_input", "arguments"];
    for key in &nested {
        if let Some(skill) = item
            .get(*key)
            .and_then(|i| i.get("skill"))
            .and_then(Value::as_str)
        {
            return Some(skill);
        }
    }
    None
}

/// Returns true if the string is purely decorative box-drawing or border characters.
/// Filters out insight "topics" like `─────────────` from formatted headers.
fn is_decorative(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| matches!(c, '─' | '━' | '═' | '╌' | '╍' | '-' | '_' | '`'))
}

#[cfg(test)]
mod tests;
