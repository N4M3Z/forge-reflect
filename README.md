# forge-reflect

Enforces session reflection before compaction or exit. When a session has substantial work (10+ tool-using turns, 4+ user messages) but no memory writes to `Memory/Learnings/` or `Memory/Decisions/`, the module:

- **Stop hook**: Blocks session exit and prompts for reflection
- **PreCompact hook**: Injects a reflection prompt before context compaction

## Layout

```
forge-reflect/
├── .claude-plugin/plugin.json         # Claude Code plugin registration
├── bin/_build.sh                      # Lazy Rust compilation helper
├── hooks/
│   ├── hooks.json                     # Hook registration (Stop, PreCompact)
│   ├── stop.sh                        # Stop hook — chains insight + reflect
│   ├── pre-compact.sh                 # PreCompact hook — injects reflection
│   └── skill-load.sh                  # DCI injector for steering + User.md
├── skills/
│   ├── InsightCheck/SKILL.md          # /insight skill
│   └── SessionReflect/SKILL.md        # /reflect skill
├── src/                               # Rust source
│   ├── lib.rs                         # Library crate (shared logic)
│   ├── config/mod.rs                  # Configuration with compiled defaults
│   ├── transcript/mod.rs              # JSONL transcript analysis
│   ├── prompt/mod.rs                  # Pattern file loading
│   └── bin/
│       ├── insight.rs                 # Insight binary (hard rule)
│       └── reflect.rs                 # Reflect binary (soft heuristic)
├── module.yaml                        # Module metadata (events, metadata)
├── Cargo.toml                         # Rust package manifest
└── config.yaml                        # User config override (gitignored)
```

## Universal CLI Usage

The binaries read a JSON payload from stdin:

```bash
# Stop hook — blocks if substantial session has no memory writes
echo '{"cwd":"/path/to/workspace","transcript_path":"/path/to/transcript.jsonl"}' | ./target/release/insight
echo '{"cwd":"/path/to/workspace","transcript_path":"/path/to/transcript.jsonl"}' | ./target/release/reflect

# PreCompact hook — injects reflection context
echo '{"cwd":"/path/to/workspace","trigger":"auto"}' | ./target/release/reflect
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

All values have compiled defaults — `config.yaml` is optional, gitignored, and used for user overrides only.

| Setting | Default | Description |
|---------|---------|-------------|
| `tool_turn_threshold` | 10 | Minimum tool-using turns to consider a session substantial |
| `user_msg_threshold` | 4 | Minimum user messages for substantiality |
| `memory_paths` | `Memory/Learnings/`, `Memory/Decisions/` | Paths that count as memory writes |
| `insight_marker` | `★ Insight` | Marker to detect insight blocks |

The reflection prompt is loaded from `Vaults/Personal/Orchestration/Patterns/Session Reflect.md` (relative to `cwd`). Falls back to a built-in message if the file doesn't exist.

## Building

Requires Rust toolchain:

```bash
cargo build --release
```

Binaries output to `target/release/insight` and `target/release/reflect`. Hook scripts use lazy compilation via `bin/_build.sh` — binaries are built on first invocation if missing.

## User Extensions

- **Steering**: External steering rules injected via `forge-steering` when skills are invoked
- **User overrides**: Create `User.md` in the module root for custom skill context (gitignored)
