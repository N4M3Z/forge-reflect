# session-reflect

Enforces session reflection before compaction or exit. When a session has substantial work (4+ tool-using turns) but no memory writes to `Memory/Learnings/` or `Memory/Decisions/`, the plugin:

- **Stop hook**: Blocks session exit and prompts for reflection
- **PreCompact hook**: Injects a reflection prompt before context compaction

## Universal CLI Usage

The binary reads a JSON payload from stdin:

```bash
# Stop hook — blocks if substantial session has no memory writes
echo '{"cwd":"/path/to/workspace","transcript_path":"/path/to/transcript.jsonl"}' | ./target/release/session-reflect

# PreCompact hook — injects reflection context
echo '{"cwd":"/path/to/workspace","trigger":"auto"}' | ./target/release/session-reflect
```

### JSON input fields

| Field | Type | Description |
|-------|------|-------------|
| `cwd` | string | Working directory (must be under `~/Data`) |
| `transcript_path` | string | Path to session transcript (JSONL) |
| `stop_hook_active` | bool | True when invoked by the stop hook itself (prevents loops) |
| `trigger` | string? | `"manual"` or `"auto"` — presence indicates PreCompact mode |

### Output

- **Allow**: exits 0, no stdout
- **Block** (Stop): `{"decision":"block","reason":"..."}`
- **Inject** (PreCompact): `{"additionalContext":"..."}`

## Configuration

| Constant | Default | Description |
|----------|---------|-------------|
| `SUBSTANCE_THRESHOLD` | 4 | Minimum tool-using turns to consider a session substantial |
| `MEMORY_PATHS` | `Memory/Learnings/`, `Memory/Decisions/` | Paths that count as memory writes |

The reflection prompt is loaded from `Vaults/Personal/Orchestration/Patterns/Session Reflect.md` (relative to `cwd`). Falls back to a built-in message if the file doesn't exist.

## Claude Code

```bash
# Local testing
claude --plugin-dir ./Plugins/session-reflect

# Install from marketplace
/plugin install session-reflect@forge-plugins
```

## Building

Requires Rust toolchain:

```bash
cargo build --release
```

Binary outputs to `target/release/session-reflect`.

## Generic Integration

Wire the binary into any AI tool's pre-exit or pre-compaction hook. The tool must pipe a JSON payload to stdin with at minimum `cwd` and either `transcript_path` (for stop behavior) or `trigger` (for compaction behavior).
