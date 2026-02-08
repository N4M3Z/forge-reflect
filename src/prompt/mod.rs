use std::fs;
use std::path::Path;

/// Load a pattern file by relative path, stripping frontmatter and H1.
pub fn load_pattern(cwd: &str, relative_path: &str) -> Option<String> {
    let pattern_path = Path::new(cwd).join(relative_path);

    let content = fs::read_to_string(pattern_path).ok()?;
    let stripped = strip_frontmatter_and_h1(&content);

    if stripped.trim().is_empty() {
        None
    } else {
        Some(stripped.trim().to_string())
    }
}

/// Remove YAML frontmatter (between first --- pair) and the first H1 line.
pub fn strip_frontmatter_and_h1(content: &str) -> String {
    let mut lines = content.lines();
    let mut result = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_done = false;
    let mut h1_removed = false;

    if let Some(first) = lines.next() {
        if first.trim() == "---" {
            in_frontmatter = true;
        } else {
            if first.starts_with("# ") {
                h1_removed = true;
            } else {
                result.push(first);
            }
            frontmatter_done = true;
        }
    }

    for line in lines {
        if in_frontmatter {
            if line.trim() == "---" {
                in_frontmatter = false;
                frontmatter_done = true;
            }
            continue;
        }

        if frontmatter_done && !h1_removed && line.starts_with("# ") {
            h1_removed = true;
            continue;
        }

        result.push(line);
    }

    result.join("\n")
}

#[cfg(test)]
mod tests;
