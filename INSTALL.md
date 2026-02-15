# forge-reflect — Installation

> **For AI agents**: This guide covers installation of forge-reflect. Follow the steps for your deployment mode.

## As part of forge-core (submodule)

Already included as a submodule. Build with:

```bash
make install    # builds all modules including forge-reflect
```

Or build individually:

```bash
cargo build --release --manifest-path Modules/forge-reflect/Cargo.toml
```

Ensure the module is listed in `forge.yaml` (typically last — reflection runs after all other modules):

```yaml
modules:
  - forge-reflect    # SessionStart, Stop, PreCompact — reflection enforcement
```

## Standalone (Claude Code plugin)

```bash
claude plugin install forge-reflect
```

Or install from a local path during development:

```bash
claude plugin install /path/to/forge-reflect
```

## What gets installed

| Binary | Purpose |
|--------|---------|
| `surface` | SessionStart digest — overdue backlog, reminders, stale ideas, unchecked journal items, captured tabs |
| `insight` | Stop hook — detects uncaptured `★ Insight` blocks in the conversation transcript |
| `reflect` | Stop hook — soft heuristic for session substantiality (tool turns, user messages) |

## Configuration

### defaults.yaml

Ships with defaults for insight detection, reflection thresholds, surface digest preferences, and memory paths. See `defaults.yaml` for all fields.

Key settings:

```yaml
insight_marker: "★ Insight"
tool_turn_threshold: 10      # minimum tool turns before prompting reflection
user_msg_threshold: 4        # minimum user messages before prompting reflection

surface:
  ideas_cutoff_days: 14      # resurface ideas older than this
  due_soon_days: 3           # backlog items due within this window
  max_items: 5               # max items per surface section
```

### Module config override

Create `config.yaml` (gitignored) to override specific fields:

```yaml
# Custom memory paths
memory:
  imperatives: Orchestration/Memory/Imperatives
  insights: Orchestration/Memory/Insights
  ideas: Orchestration/Memory/Ideas

# Adjust reflection sensitivity
tool_turn_threshold: 15

# Surface digest preferences
surface:
  reminders_list: tasks
  max_items: 3
```

### Hook.md convention

forge-reflect skills include `Hook.md` files that the dispatcher appends to binary output. These keep AI prompts as editable markdown, separate from compiled code.

## Dependencies

| Dependency | Required | Purpose |
|-----------|----------|---------|
| Rust + cargo | Yes | Build the 3 binaries |
| ekctl | Optional | Reminders in surface digest (macOS only) |

## Verify

See [VERIFY.md](VERIFY.md) for the post-installation checklist.
