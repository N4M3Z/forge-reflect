#!/usr/bin/env bash
# PostToolUse hook: one-shot surface digest on first journals skill invocation.
# Injects via additionalContext so AI presents it to user.
# Prompt is configurable via surface.post_tool_use_prompt in defaults.yaml.
set -euo pipefail

GUARD="/tmp/forge-surface-shown-$PPID"
[ -f "$GUARD" ] && exit 0

# Read stdin (dispatch pipes Claude Code JSON)
INPUT=$(cat)

# Filter: only forge-journals skills
SKILL=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('tool_input', {}).get('skill', ''))
" 2>/dev/null) || exit 0

case "$SKILL" in
    DailyPlan|DailyReview|Log|Inbox|BacklogJournals|JournalStructure|Timesheet|WeeklyReview) ;;
    *) exit 0 ;;
esac

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(command cd "$(dirname "$0")/.." && pwd)}}"
export FORGE_MODULE_ROOT="$MODULE_ROOT"

source "$MODULE_ROOT/bin/_build.sh"
ensure_built surface || exit 0

OUTPUT=$("$BIN_DIR/surface") || true
[ -z "$OUTPUT" ] && { touch "$GUARD"; exit 0; }

# Read prompt from config: yaml (forge-lib) → yq → hardcoded default
DEFAULT_PROMPT="Display this surface digest VERBATIM to the user. Do not summarize, reframe, or skip any items."
PROMPT=""

# Try config.yaml first (user override), then defaults.yaml
for cfg in "$MODULE_ROOT/config.yaml" "$MODULE_ROOT/defaults.yaml"; do
    [ -f "$cfg" ] || continue

    # yaml nested (forge-core binary)
    if command -v yaml >/dev/null 2>&1; then
        PROMPT=$(yaml nested "$cfg" surface post_tool_use_prompt 2>/dev/null) || true
        [ -n "$PROMPT" ] && break
    fi

    # yq fallback
    if command -v yq >/dev/null 2>&1; then
        PROMPT=$(yq '.surface.post_tool_use_prompt // ""' "$cfg" 2>/dev/null) || true
        [ -n "$PROMPT" ] && break
    fi
done
: "${PROMPT:=$DEFAULT_PROMPT}"

# Build additionalContext: prompt + digest
CONTEXT=$(printf '%s\n\n%s' "$PROMPT" "$OUTPUT")
ESCAPED=$(printf '%s' "$CONTEXT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))" 2>/dev/null) || exit 0

printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$ESCAPED"

touch "$GUARD"
