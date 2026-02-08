#!/usr/bin/env bash
# PreCompact hook: inject reflection prompt before context compaction.
# Dual-mode: works standalone (CLAUDE_PLUGIN_ROOT) or as forge-core module (FORGE_MODULE_ROOT).
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(builtin cd "$(dirname "$0")/.." && pwd)}}"
export CLAUDE_PLUGIN_ROOT="$MODULE_ROOT"  # So _build.sh finds Cargo.toml

source "$MODULE_ROOT/bin/_build.sh"
ensure_built reflect || exit 0

exec "$BIN_DIR/reflect"
