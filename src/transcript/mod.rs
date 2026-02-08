use crate::config::Config;

pub struct TranscriptAnalysis {
    pub user_messages: usize,
    pub tool_using_turns: usize,
    pub has_memory_write: bool,
    pub insight_count: usize,
    pub learnings_write_count: usize,
}

/// Analyze transcript for user messages, tool-using turns, memory writes, and insights.
pub fn analyze_transcript(transcript: &str, config: &Config) -> TranscriptAnalysis {
    let mut analysis = TranscriptAnalysis {
        user_messages: 0,
        tool_using_turns: 0,
        has_memory_write: false,
        insight_count: 0,
        learnings_write_count: 0,
    };

    for line in transcript.lines() {
        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if entry_type == "human" {
            analysis.user_messages += 1;
            continue;
        }

        if entry_type != "assistant" {
            continue;
        }

        let Some(content) = entry
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
        else {
            continue;
        };

        let mut turn_has_tool_use = false;

        for item in content {
            let Some(item_type) = item.get("type").and_then(|t| t.as_str()) else {
                continue;
            };

            // Count ★ Insight blocks in text content
            if item_type == "text" {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    analysis.insight_count += text.matches(config.insight_marker.as_str()).count();
                }
                continue;
            }

            if item_type != "tool_use" {
                continue;
            }

            turn_has_tool_use = true;

            // Hardcoded: these are Claude Code API tool names, not user-facing strings.
            let tool_name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if tool_name != "Edit" && tool_name != "Write" {
                continue;
            }

            let file_path = item
                .get("input")
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
                .unwrap_or("");

            if file_path.contains(config.learnings_path()) {
                analysis.learnings_write_count += 1;
            }

            for memory_path in &config.memory_paths {
                if file_path.contains(memory_path.as_str()) {
                    analysis.has_memory_write = true;
                }
            }
        }

        if turn_has_tool_use {
            analysis.tool_using_turns += 1;
        }
    }

    analysis
}

#[cfg(test)]
mod tests;
