---
name: Surface
description: Serendipity spark — resurfaces forgotten ideas and captured tabs for inspiration. Fires automatically on first forge-journals skill invocation via PostToolUse hook.
---

# Surface

Serendipity engine. Resurfaces forgotten or inspiring items — not tasks, not obligations, just things worth rediscovering.

## Sections

- **Stale ideas** — open ideas older than cutoff (default: 14 days), rotated daily
- **Rediscovery** — rotating selection from a mixed pool (captured tabs, open backlog items)

Task coverage (overdue, due-soon, yesterday's journal) lives in `/DailyPlan`.

## How it works

The `surface` binary reads the filesystem for ideas and tab archives. The PostToolUse hook (`hooks/PostToolUse.sh`) runs the binary on the first forge-journals skill invocation per session (DailyPlan, Log, Inbox, etc.) and injects the output via `additionalContext`. A PPID-scoped guard file ensures one-shot firing. The AI presentation prompt is configurable via `surface.post_tool_use_prompt` in `defaults.yaml`.

## Configuration

All parameters in `defaults.yaml` under `surface:` — archive paths, cutoff days, max items, presentation prompt.
