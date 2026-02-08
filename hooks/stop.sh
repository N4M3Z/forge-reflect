#!/usr/bin/env bash
# Stop hook: chain insight check + reflect gate.
# Dual-mode: works standalone (CLAUDE_PLUGIN_ROOT) or as forge-core module (FORGE_MODULE_ROOT).
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(builtin cd "$(dirname "$0")/.." && pwd)}}"
export CLAUDE_PLUGIN_ROOT="$MODULE_ROOT"  # So _build.sh finds Cargo.toml

source "$MODULE_ROOT/bin/_build.sh"
ensure_built insight || exit 0
ensure_built reflect || exit 0

INPUT=$(cat)

# Insight check first (hard rule, independent of substantiality)
RESULT=$(printf '%s\n' "$INPUT" | "$BIN_DIR/insight")
if [ -n "$RESULT" ]; then
    printf '%s\n' "$RESULT"
    exit 0
fi

# Reflect check (soft heuristic)
printf '%s\n' "$INPUT" | "$BIN_DIR/reflect"
