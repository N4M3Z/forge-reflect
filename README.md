# forge-reflect

Self-learning memory management for AI coding sessions.

AI sessions generate a massive amount of implicit knowledge — architectural decisions, debugging breakthroughs, workflow preferences, project context. Without enforcement, all of it evaporates when the session ends. forge-reflect turns memory capture into a habit by making the AI actually learn from its own sessions.

## What it does

**Surface** (SessionStart) — Every session opens with a briefing: overdue backlog items, pending reminders, stale ideas, journal gaps, captured browser tabs. Everything that sank to the bottom gets surfaced for rediscovery.

**Reflect** (Stop) — When a substantial session ends (10+ tool turns, 4+ user messages), forge-reflect reads the session transcript and checks whether any insights or imperatives were captured. If nothing was written to memory, it blocks exit and forces reflection — either the AI captures what it learned, or the user decides nothing was worth keeping. No silent knowledge loss.

**Compact** (PreCompact) — Before the AI compresses its context window, forge-reflect injects a reflection prompt so captured knowledge survives the compression.

## What it looks like

You start a session. forge-reflect shows you what fell through the cracks:

```
📌 Surface ─────────────────────────────────────────
Stale ideas (12):
  • CLI tool for batch-renaming Obsidian attachments (since 2025-09-14)
  • Automated weekly review from journal entries (since 2025-11-02)
  • Plugin marketplace search command (since 2026-01-08)
Rediscovery:
  • Refactor TLP classification to support per-directory overrides
  • Investigate tree-sitter for markdown section extraction
  • Add --dry-run flag to forge-update
─────────────────────────────────────────────────────
```

You work for a while. You solve a tricky bug, discover an architectural pattern, figure out a debugging trick. Then you try to end the session:

```
✗ Session blocked — 2 insight(s) found, 0 Insight file(s) written

You output ★ Insight blocks during this session but didn't persist them.
Capture each insight as a Memory/Insights/ file before ending.
```

The AI writes the insight files. Now it can exit — and next session, those insights are loaded back into context. The AI actually remembers what it learned.

## Install

Requires Rust toolchain. Works as a **standalone Claude Code plugin** or as a **forge-core module**.

```bash
# Clone and build
git clone https://github.com/N4M3Z/forge-reflect.git
cd forge-reflect
cargo build --release
```

### Standalone plugin

The `.claude-plugin/plugin.json` registers all three hooks automatically. Add the module directory to your Claude Code plugins and it just works — lazy compilation builds binaries on first invocation if needed.

### As a forge-core module

Add `forge-reflect` to your module list in `defaults.yaml`. Hooks are dispatched by forge-core in order. No additional setup.

## The enforcement loop

```
Session starts
    │
    ▼
┌─────────────────────────────────────────┐
│  surface: here's what you forgot about  │
│  ─ overdue tasks, stale ideas, gaps     │
└─────────────────────────────────────────┘
    │
    ▼
  ... you work ...
    │
    ▼
Session ends
    │
    ▼
┌─────────────────────────────────────────┐
│  insight: did you capture ★ Insights?   │──── yes ──→ exit allowed
│  reflect: any memory writes at all?     │──── yes ──→ exit allowed
└─────────────────────────────────────────┘
    │ no
    ▼
  BLOCKED — reflect before you leave
    │
    ▼
  AI captures insights → Memory/Insights/
  AI captures rules    → Memory/Imperatives/
  AI captures ideas    → Memory/Ideas/
    │
    ▼
  exit allowed ✓
```

The captured files are loaded back into future sessions, creating a feedback loop: learn from sessions, surface what you learned, learn more.

## Memory types

| Type | Directory | What it captures |
|------|-----------|-----------------|
| **Insights** | `Memory/Insights/` | Architectural discoveries, debugging breakthroughs, codebase patterns |
| **Imperatives** | `Memory/Imperatives/` | Hard rules — "always do X", "never do Y" |
| **Ideas** | `Memory/Ideas/` | Speculative concepts that need investigation before becoming projects |

Each memory is a single markdown file. One file per insight, never accumulated lists.

## Skills

| Skill | Purpose |
|-------|---------|
| `/Surface` | Manually trigger the session briefing |
| `/InsightCheck` | Check for uncaptured `★ Insight` blocks in the current session |
| `/SessionReflect` | Interactive end-of-session reflection — capture decisions, insights, ideas |
| `/MemoryInsights` | Memory capture conventions and idea lifecycle reference |

## Configuration

All values have compiled defaults — zero config required. Create `config.yaml` (gitignored) to override:

| Setting | Default | What it controls |
|---------|---------|-----------------|
| `tool_turn_threshold` | 10 | Minimum tool turns for a session to count as "substantial" |
| `user_msg_threshold` | 4 | Minimum user messages for substantiality |
| `memory_paths` | `Memory/Insights/`, `Memory/Imperatives/` | Directories that count as memory writes |
| `insight_marker` | `★ Insight` | Pattern that marks insight blocks in output |

When used with forge-core, shared paths (backlog, journal, memory directories) are loaded from `defaults.yaml` automatically.

## Architecture

Three Rust binaries, one library crate. Binaries are thin wrappers — all logic lives in the library.

| Binary | Hook | Behaviour |
|--------|------|-----------|
| `surface` | SessionStart | Parses backlog, reminders, ideas, tabs, journal gaps; emits a digest |
| `insight` | Stop | Hard rule — blocks if `★ Insight` blocks exist without matching files |
| `reflect` | Stop / PreCompact | Soft heuristic — blocks if substantial session has zero memory writes |

All binaries read JSON from stdin, exit 0 always. Communication is via stdout: empty = allow, JSON = block or inject context. Errors go to stderr.

> `CLAUDE.md` and `AGENTS.md` are autogenerated by `/Init`. Do not edit directly — run `/Update` to regenerate.
