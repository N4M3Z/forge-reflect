#!/usr/bin/env bash
# SessionStart hook: surface digest (overdue backlog, reminders, stale ideas, tabs, journal gaps).
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(builtin cd "$(dirname "$0")/.." && pwd)}}"
export FORGE_MODULE_ROOT="$MODULE_ROOT"

source "$MODULE_ROOT/bin/_build.sh"
ensure_built surface || exit 0

OUTPUT=$("$BIN_DIR/surface") || true

if [ -n "$OUTPUT" ]; then
  printf '%s\n' "$OUTPUT"
fi
