---
name: MemoryCapture
description: Interactive review and capture of uncaptured insights before context compaction. USE WHEN precompact candidates file exists or insights need interactive triage.
---

# Memory Capture

Spawns a foreground agent to interactively review uncaptured insight candidates. Each candidate is presented via AskUserQuestion with capture/skip options.

## Instructions

### Step 1: Locate candidates file

Find the candidates file:
- If an argument was provided, use it as the file path
- Otherwise, glob for `/tmp/forge-precompact-candidates-*.json`
- Use the most recent match if multiple exist

Read the JSON file. It contains:
```json
{
    "session_id": "...",
    "user_messages": 12,
    "tool_turns": 45,
    "duration_minutes": 30,
    "insights_dir": "/path/to/Memory/Insights",
    "imperatives_dir": "/path/to/Memory/Imperatives",
    "ideas_dir": "/path/to/Memory/Ideas",
    "topics": ["Topic A", "Topic B"]
}
```

If no candidates file is found or `topics` is empty, report "No uncaptured candidates found" and stop.

### Step 2: Spawn capture agent

Use the **Task tool** to spawn a foreground agent:
- `subagent_type`: `"general-purpose"`
- `description`: `"capture uncaptured insights"`

Provide the agent with:
- The full candidates JSON content
- The MemoryInsights schema (from `/MemoryInsights` skill or this summary):
  - One file per item in the appropriate directory
  - Frontmatter: `title`, `tags` (include type tag), `created`, `related`
  - Body: brief description of the insight/imperative/idea
  - `## Log` section with `- [#] YYYY-MM-DD origin context #log/context`

The agent prompt must include these instructions:

> For each topic in the candidates list, present an AskUserQuestion:
>
> Question: "Capture '{topic}'?"
> Options:
> 1. **Capture as Insight** — write to insights_dir
> 2. **Capture as Imperative** — write to imperatives_dir
> 3. **Capture as Idea** — write to ideas_dir
> 4. **Skip** — mark as reviewed, do not persist
> 5. **Skip All** — stop processing remaining topics
>
> For each "Capture" response, create the memory file in the appropriate directory using the MemoryInsights schema. Use today's date for `created` and the `## Log` entry.
>
> After processing all topics (or Skip All), report a summary: N captured, M skipped.

### Step 3: Cleanup

After the agent completes:
1. Delete the candidates file
2. Report the agent's summary to the user
