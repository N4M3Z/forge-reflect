---
name: insight
description: Check for uncaptured insights — find ★ Insight blocks in this session that were not persisted as Memory/Learnings/ files.
---

# Insight Check

Manual equivalent of the insight Stop hook. Scans the current conversation for `★ Insight` blocks and verifies each one has a corresponding `Memory/Learnings/` file.

## Instructions

### Step 1: Load config

First, resolve the module root (handles both standalone and forge-core module paths):
```bash
MODULE="${CLAUDE_PLUGIN_ROOT}/Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="${CLAUDE_PLUGIN_ROOT}"
```

Read the config file to get the learnings path:

```bash
cat $MODULE/config.yaml
```

### Step 2: Scan conversation for insight blocks

Review the full conversation history and find every `★ Insight` block you output. Collect:
- The topic or title of each insight
- The key content (1-2 sentences)

### Step 3: Scan for learnings written this session

Check which `Memory/Learnings/` files were created or written during this session by reviewing your tool use history for Write/Edit calls targeting the learnings path.

### Step 4: Cross-reference

For each insight block found, determine whether a corresponding learning file was written. Match by content/topic, not exact title.

### Step 5: Report

If all insights are captured:
- Report: "All N insight(s) have corresponding learnings files."

If uncaptured insights exist:
- List each uncaptured insight with its topic and key content
- For each, propose a learning file title and ask: "Want me to capture these now?"
- If the user confirms, create the learning files following the Learning schema from CLAUDE.md
