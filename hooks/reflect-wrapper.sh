#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../bin/_build.sh"
ensure_built reflect || exit 0

export CLAUDE_PLUGIN_ROOT="$PLUGIN_ROOT"

exec "$BIN_DIR/reflect"
