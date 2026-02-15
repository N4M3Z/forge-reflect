# forge-reflect

Session lifecycle module: surfaces actionable items at session start, enforces memory capture at session end.

- **SessionStart hook**: Runs the `surface` binary to produce a digest (overdue backlog, reminders, stale ideas, journal gaps, captured tabs), then appends `skills/Surface/Hook.md` as the AI instruction
- **Stop hook**: Blocks session exit and prompts for reflection when a substantial session has no memory writes
- **PreCompact hook**: Injects a reflection prompt before context compaction

## Layer

**Behaviour** — part of forge-core's three-layer architecture (Identity / Behaviour / Knowledge). Enforced via `Stop` and `PreCompact` hooks.

## Layout

```
forge-reflect/
├── .claude-plugin/plugin.json         # Claude Code plugin registration
├── bin/_build.sh                      # Lazy Rust compilation helper
├── hooks/
│   ├── hooks.json                     # Hook registration (standalone plugin use)
│   ├── session-start.sh               # SessionStart hook — runs surface + appends Hook.md
│   ├── stop.sh                        # Stop hook — chains insight + reflect
│   ├── pre-compact.sh                 # PreCompact hook — injects reflection
│   └── skill-load.sh                  # DCI injector for steering + User.md
├── skills/
│   ├── InsightCheck/SKILL.md          # /InsightCheck — detect uncaptured insights
│   ├── SessionReflect/SKILL.md        # /SessionReflect — end-of-session reflection
│   ├── MemoryInsights/SKILL.md        # /MemoryInsights — memory capture conventions
│   └── Surface/
│       ├── SKILL.md                   # Surface skill definition
│       └── Hook.md                    # AI instruction appended after surface output
├── src/                               # Rust source
│   ├── lib.rs                         # Library crate (shared logic)
│   ├── config/mod.rs                  # Configuration with compiled defaults
│   ├── surface/mod.rs                 # Pure parsing functions for surface digest
│   ├── transcript/mod.rs              # JSONL transcript analysis
│   ├── prompt/mod.rs                  # Pattern file loading
│   └── bin/
│       ├── surface.rs                 # Surface binary (SessionStart digest)
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

### Module-specific settings

| Setting | Default | Description |
|---------|---------|-------------|
| `tool_turn_threshold` | 10 | Minimum tool-using turns to consider a session substantial |
| `user_msg_threshold` | 4 | Minimum user messages for substantiality |
| `memory_paths` | `Memory/Insights/`, `Memory/Imperatives/` | Paths that count as memory writes |
| `insight_marker` | `★ Insight` | Marker to detect insight blocks |
| `reflection` | `Orchestration/Skills/SessionReflect/SKILL.md` | Reflection prompt (user-root-relative) |
| `insight_check` | `Orchestration/Skills/InsightCheck/SKILL.md` | Insight check prompt |
| `data_dir_suffix` | `Data` | Directory suffix for scope check |

### Shared settings (from defaults.yaml)

These paths are shared with other modules. The authoritative source is `defaults.yaml`'s `shared:` section (see forge-core README). They also appear in module `defaults.yaml` for standalone use.

| Setting | Default | Description |
|---------|---------|-------------|
| `backlog` | `Orchestration/Backlog.md` | Backlog file (user-root-relative) |
| `journal.daily` | `Resources/Journals/Daily/YYYY/MM/YYYY-MM-DD.md` | Daily journal pattern |
| `commands.safe_read` | `Modules/forge-tlp/bin/safe-read` | Safe-read binary |
| `memory.imperatives` | `Orchestration/Memory/Imperatives` | Imperatives directory |
| `memory.insights` | `Orchestration/Memory/Insights` | Insights directory |
| `memory.ideas` | `Orchestration/Memory/Ideas` | Ideas directory |

### Config precedence

```
    Highest ──► 1. config.yaml           user override (gitignored)
                │                        e.g. backlog: My/Custom/Backlog.md
                │
                2. defaults.yaml shared: project-level source of truth
                │                        loaded via ProjectConfig::load()
                │
                3. defaults.yaml         module-shipped defaults
                │                        (standalone use, identical to compiled)
                │
    Lowest  ──► 4. Compiled Default      no files needed
```

How it works: `Config::load()` deserializes `config.yaml` (or `defaults.yaml`), then calls `apply_shared()` which overlays project `defaults.yaml shared:` values onto any field that still equals its compiled default. A `config.yaml` override changes the value away from the compiled default, so `apply_shared()` never touches it.

```
Config::load()
     │
     ├── Deserialize config.yaml or defaults.yaml
     │
     ├── Resolve user_root (FORGE_USER_ROOT → defaults.yaml → cwd)
     │
     └── apply_shared(ProjectConfig)
              │
              for each shared field:
              ├── value == compiled default?
              │     YES → replace with defaults.yaml shared: value
              │     NO  → keep (was overridden by config.yaml)
              └── defaults.yaml value empty?
                    YES → keep compiled default
                    NO  → apply
```

This means forge-reflect works in three contexts:
- **Via dispatch** — env vars set, shared config loaded from defaults.yaml
- **As Claude Code plugin** — discovers defaults.yaml via `CLAUDE_PROJECT_ROOT`
- **Standalone CLI** — discovers defaults.yaml from cwd, falls back to module defaults.yaml

## Building

Requires Rust toolchain:

```bash
cargo build --release
```

Binaries output to `target/release/{surface,insight,reflect}`. Hook scripts use lazy compilation via `bin/_build.sh` — binaries are built on first invocation if missing.

## Skills

| Skill | Purpose |
|-------|---------|
| `Surface` | SessionStart digest — overdue backlog, reminders, stale ideas, journal gaps, captured tabs. `Hook.md` provides the AI instruction. |
| `InsightCheck` | Detect uncaptured insights in the session — hard rule, blocks exit |
| `SessionReflect` | End-of-session reflection prompt — soft heuristic |
| `MemoryInsights` | Memory capture conventions — schemas for Insights, Imperatives, and Ideas files; idea lifecycle |

## User Extensions

- **Steering**: External steering rules injected via `forge-steering` when skills are invoked
- **User overrides**: Create `User.md` in the module root for custom skill context (gitignored)
