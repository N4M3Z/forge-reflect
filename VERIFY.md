# forge-reflect — Verification

> **For AI agents**: Complete this checklist after installation. Every check must pass before declaring the module installed.

## Quick check

```bash
cargo test --manifest-path Modules/forge-reflect/Cargo.toml
cargo clippy --manifest-path Modules/forge-reflect/Cargo.toml -- -D warnings
cargo fmt --manifest-path Modules/forge-reflect/Cargo.toml --check
```

Expected: 53 tests pass (config, surface, transcript, prompt modules).
Clippy and fmt must also pass clean.

## Binaries available

```bash
# Check availability and help
surface --help
insight --help
reflect --help
```

## Manual checks

### surface (SessionStart digest)

```bash
# Can be run without args (uses CWD) or with --cwd
surface --cwd .
```

### insight (uncaptured insight detection)

```bash
# Topic-based matching (blocks if "My Topic" not in Memory/Insights/My Topic.md)
echo '{"type":"assistant","message":{"content":[{"type":"text","text":"★ Insight: My Topic"}]}}' | insight --cwd . --transcript-path /dev/stdin
```

### reflect (session substantiality)

```bash
echo '{"type":"human"}' | reflect --cwd . --transcript-path /dev/stdin
echo $?   # should be 0 (below threshold → no reflection needed)
```

### Skill discovery

```bash
ls Modules/forge-reflect/skills/*/SKILL.md
# Should list: InsightCheck, SessionReflect, MemoryInsights, Surface
```

## OpenCode plugin

If using opencode, verify the plugin adapter loads:

```bash
ls .opencode/plugins/forge-reflect.ts
# Should exist
```

The plugin auto-builds binaries on first use. To verify manually:

```bash
cargo build --release --manifest-path Cargo.toml
# Then start opencode in the repo — surface digest should appear as a toast
```

## Expected results

- All 3 binaries compile and are available in PATH (or via bin/ wrappers)
- `surface` generates digest from vault content (graceful with missing data)
- `insight` detects `★ Insight` markers and blocks if uncaptured
- `reflect` applies substantiality heuristic based on tool/message counts
- All 53 tests pass, clippy clean, fmt clean
