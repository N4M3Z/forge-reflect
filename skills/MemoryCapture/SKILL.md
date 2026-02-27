---
name: MemoryCapture
description: Spawn a capture agent for uncaptured insight candidates. USE WHEN precompact candidates file exists or insights need interactive triage.
---

# Memory Capture

This skill is a **launcher** — its only job is to read the candidates file, build the agent prompt, and spawn. It does no capture work itself.

## Instructions

### Step 1: Read candidates

Locate the candidates file:
- From argument, or glob `/tmp/forge-precompact-candidates-*.json` (most recent)

Read and parse the JSON. If no file found or `topics` is empty, report "No candidates" and stop.

### Step 2: Spawn

Use the **Task tool** with `subagent_type: "general-purpose"` and `description: "capture uncaptured insights"`.

The agent prompt MUST include:
1. The full candidates JSON (verbatim)
2. The following instructions:

---

For each topic in the candidates list, present an **AskUserQuestion**:

Question: "Capture '{topic}'?"
Options:
1. **Insight** — write to `insights_dir`
2. **Imperative** — write to `imperatives_dir`
3. **Idea** — write to `ideas_dir`
4. **Skip**
5. **Skip All** — stop immediately

For each capture, create a memory file in the target directory:
- Frontmatter: `title`, `tags` (include `type/memory/insight` or equivalent), `created` (today), `related`
- Body: brief description
- `## Log` section: `- [#] YYYY-MM-DD origin context #log/context`

After all topics (or Skip All), report: N captured, M skipped.

---

### Step 3: Cleanup

After the agent completes:
1. Delete the candidates file
2. Report the agent's summary
