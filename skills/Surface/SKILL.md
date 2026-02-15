---
name: Surface
description: SessionStart spark — resurfaces forgotten ideas and captured tabs for serendipity and inspiration. Runs automatically at session start via hook.
---

# Surface

Serendipity engine. Resurfaces forgotten or inspiring items at session start — not tasks, not obligations, just things worth rediscovering.

## Sections

- **Stale ideas** — open ideas older than cutoff (default: 14 days), rotated daily
- **Rediscovery** — rotating selection from a mixed pool (captured tabs, open backlog items)

Task coverage (overdue, due-soon, yesterday's journal) lives in `/DailyPlan`.

## How it works

The `surface` binary reads the filesystem for ideas and tab archives. It outputs formatted text to stdout. The SessionStart hook script runs the binary and appends `Hook.md` as the AI instruction.

## Configuration

All parameters in `defaults.yaml` under `surface:` — archive paths, cutoff days, max items.
