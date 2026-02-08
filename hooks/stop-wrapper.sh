#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../bin/_build.sh"
ensure_built insight || exit 0
ensure_built reflect || exit 0

export CLAUDE_PLUGIN_ROOT="$PLUGIN_ROOT"

INPUT=$(cat)

# Insight check first (hard rule, independent of substantiality)
RESULT=$(printf '%s\n' "$INPUT" | "$BIN_DIR/insight")
if [ -n "$RESULT" ]; then
    printf '%s\n' "$RESULT"
    exit 0
fi

# Reflect check (soft heuristic)
printf '%s\n' "$INPUT" | "$BIN_DIR/reflect"
