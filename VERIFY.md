# forge-reflect — Verification

> **For AI agents**: Complete this checklist after installation. Every check must pass before declaring the module installed.

## Quick check

```bash
cargo test --manifest-path Modules/forge-reflect/Cargo.toml
```

Expected: 50 tests pass (config, surface, transcript, prompt modules).

## Binaries available

```bash
command -v surface    # or: Modules/forge-reflect/bin/surface --help
command -v insight    # or: Modules/forge-reflect/bin/insight --help
command -v reflect    # or: Modules/forge-reflect/bin/reflect --help
```

## Manual checks

### surface (SessionStart digest)

```bash
FORGE_ROOT=. FORGE_USER_ROOT=Vaults/Personal \
  Modules/forge-reflect/bin/surface
# Should emit digest sections (backlog, ideas, journal gaps)
# Empty sections are omitted — some output depends on vault content
```

### insight (uncaptured insight detection)

```bash
echo "No insights here" | Modules/forge-reflect/bin/insight
echo $?   # should be 0 (no uncaptured insights → allow)

echo "★ Insight: something learned" | Modules/forge-reflect/bin/insight
echo $?   # should be 2 (uncaptured insight → block with prompt)
```

### reflect (session substantiality)

```bash
echo '{"tool_turns":2,"user_messages":1}' | Modules/forge-reflect/bin/reflect
echo $?   # should be 0 (below threshold → no reflection needed)
```

### Skill discovery

```bash
ls Modules/forge-reflect/skills/*/SKILL.md
# Should list: InsightCheck, SessionReflect, MemoryInsights, Surface
```

## Expected results

- All 3 binaries compile and are available in PATH (or via bin/ wrappers)
- `surface` generates digest from vault content (graceful with missing data)
- `insight` detects `★ Insight` markers and blocks if uncaptured
- `reflect` applies substantiality heuristic based on tool/message counts
- All 50 tests pass
