---
name: InsightCheck
description: Check for uncaptured insights — find ★ Insight blocks in this session that were not persisted as Memory/Insights/ files.
---

# Insight Check

Manual equivalent of the insight Stop hook. Scans the current conversation for `★ Insight` blocks and verifies each one has a corresponding `Memory/Insights/` file.

## Instructions

### Step 1: Load config

First, resolve the module root (handles both standalone and forge-core module paths):
```bash
MODULE="${CLAUDE_PLUGIN_ROOT}/Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="${CLAUDE_PLUGIN_ROOT}"
```

Read the config file to get the insights path:

```bash
cat $MODULE/config.yaml
```

### Step 2: Scan conversation for insight blocks

Review the full conversation history and find every `★ Insight` block you output. Collect:
- The topic or title of each insight
- The key content (1-2 sentences)

### Step 3: Scan for insight files written this session

Check which `Memory/Insights/` files were created or written during this session by reviewing your tool use history for Write/Edit calls targeting the insights path.

### Step 4: Cross-reference

For each insight block found, determine whether a corresponding insight file was written. Match by content/topic, not exact title.

### Step 5: Report

If all insights are captured:
- Report: "All N insight(s) have corresponding insight files."

If uncaptured insights exist:
- List each uncaptured insight with its topic and key content
- For each, propose an insight file title and ask: "Want me to capture these now?"
- If the user confirms, create the insight files following the Insight schema from CLAUDE.md

!`"${CLAUDE_PLUGIN_ROOT}/hooks/skill-load.sh" 2>/dev/null`
!`"${CLAUDE_PLUGIN_ROOT}/Modules/forge-reflect/hooks/skill-load.sh" 2>/dev/null`
