#!/usr/bin/env bash
# Inject external steering content and user overrides into skill context.
# Called from SKILL.md via DCI (!`command`).
set -euo pipefail
MODULE_ROOT="$(builtin cd "$(dirname "$0")/.." && pwd)"
PROJECT_ROOT="${FORGE_ROOT:-$(builtin cd "$MODULE_ROOT/../.." && pwd)}"

# External steering (if forge-steering available)
STEER="$PROJECT_ROOT/Modules/forge-steering/bin/steer"
if [ -x "$STEER" ]; then "$STEER" "$MODULE_ROOT"; fi

# User overrides
USER_MD="$MODULE_ROOT/User.md"
if [ -f "$USER_MD" ]; then cat "$USER_MD"; fi
