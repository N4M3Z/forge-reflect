---
name: ContextKeeper
description: Query past session context via c0ntextkeeper MCP tools — search archived problems, solutions, implementations, and patterns. USE WHEN starting a non-trivial task, looking for prior solutions, searching past sessions, checking what was done before, or debugging a recurring issue.
---

# ContextKeeper

Retrieve context from past sessions using c0ntextkeeper's MCP tools. Archiving is automatic (hooks handle it). This skill is about **retrieval** — querying the archive before diving into work.

## When to Query

Query the archive at the start of any non-trivial task. Ask yourself: "Have I seen this problem before?" If maybe, query first.

Good retrieval moments:
- Starting a task that touches unfamiliar code
- Debugging an issue that might have occurred before
- Resuming work after a compaction or new session
- Building on something done in a previous session
- Looking for patterns in how a problem was solved

## MCP Tools

Three retrieval tools, each for a different need:

| Tool | Purpose | Use when |
|------|---------|----------|
| `fetch_context` | Semantic search — finds relevant problems, solutions, implementations | You have a task description and want related prior work |
| `search_archive` | Filtered search — date ranges, file patterns, sort options | You know roughly when or where something happened |
| `get_patterns` | Recurring patterns — common solutions and approaches | You want to see what keeps coming up across sessions |

### fetch_context

Best for: "Have I solved something like this before?"

```
query: "RTK token savings shell commands"
scope: "project"          # project (default), session, or global
limit: 5                  # max results (default 5, max 100)
minRelevance: 0.5         # 0-1 threshold (default 0.5)
```

Returns: problems, solutions, implementations ranked by relevance.

### search_archive

Best for: "What did I do last week with the config system?"

```
query: "config deep merge"
sortBy: "date"            # relevance (default), date, frequency
limit: 10                 # max results (default 10, max 100)
filePattern: "*.rs"       # glob filter
dateRange:
  from: "2026-02-20"
  to: "2026-02-26"
```

### get_patterns

Best for: "What patterns keep recurring in this project?"

```
type: "all"               # code, command, architecture, or all
limit: 10                 # max patterns (default 10, max 50)
minFrequency: 2           # minimum occurrences (default 2)
```

Returns: recurring code patterns, common commands, architectural decisions with frequency counts.

## CLI Reference

The `c0ntextkeeper` CLI (installed at `/opt/homebrew/bin/c0ntextkeeper`) handles administration and diagnostics. Use it for maintenance, not retrieval — prefer MCP tools during sessions.

### Diagnostics

| Command | Purpose |
|---------|---------|
| `c0ntextkeeper status` | Version, storage, hook counts, archive stats |
| `c0ntextkeeper doctor` | Health diagnostics (`--fix` to auto-repair) |
| `c0ntextkeeper validate` | Config and storage integrity (`--strict`) |
| `c0ntextkeeper stats` | Storage statistics (`--json`, `--project <path>`) |

### Hook Management

| Command | Purpose |
|---------|---------|
| `c0ntextkeeper hooks list` | All hooks (PreCompact, SessionStart, Stop, etc.) |
| `c0ntextkeeper hooks health` | Hook data health check |
| `c0ntextkeeper hooks enable <hook>` | Enable a hook |
| `c0ntextkeeper hooks disable <hook>` | Disable a hook |
| `c0ntextkeeper hooks test <hook>` | Test with sample data |
| `c0ntextkeeper hooks stats` | Hook activity statistics |

### Storage Maintenance

| Command | Purpose |
|---------|---------|
| `c0ntextkeeper cleanup` | Remove old archives |
| `c0ntextkeeper rebuild-index` | Rebuild project indexes |
| `c0ntextkeeper archive <path>` | Archive a transcript file |

### Context Auto-Load

| Command | Purpose |
|---------|---------|
| `c0ntextkeeper context preview` | Preview what auto-loads |
| `c0ntextkeeper context test` | Test context loading |
| `c0ntextkeeper context configure` | Configure auto-load settings |

## Constraints

- Always prefer MCP tools over CLI for in-session retrieval
- Archive storage lives at `~/.c0ntextkeeper/` — do not modify directly
- Archiving is handled by hooks — do not manually archive during sessions
- CLI commands that modify hooks require a Claude Code restart to take effect
