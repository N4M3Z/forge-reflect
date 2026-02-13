#!/usr/bin/env bash
# SessionStart hook: surface digest (overdue backlog, reminders, stale ideas, tabs, journal gaps).
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(builtin cd "$(dirname "$0")/.." && pwd)}}"
export CLAUDE_PLUGIN_ROOT="$MODULE_ROOT"  # So _build.sh finds Cargo.toml

source "$MODULE_ROOT/bin/_build.sh"
ensure_built surface || exit 0

exec "$BIN_DIR/surface"
