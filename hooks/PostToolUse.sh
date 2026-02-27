#!/usr/bin/env bash
# PostToolUse hook: dual-purpose
# Phase 1: One-shot surface digest on first journals skill invocation
# Phase 2: Periodic insight advisory every FORGE_ADVISORY_INTERVAL tool calls (default 25)
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(command cd "$(dirname "$0")/.." && pwd)}}"
export FORGE_MODULE_ROOT="$MODULE_ROOT"

source "$MODULE_ROOT/bin/_build.sh"

# Read stdin ONCE (dispatch pipes Claude Code JSON)
INPUT=$(cat)

# --- Session identity ---
# Extract session_id from hook JSON (stable across all hook invocations in a session).
# $PPID won't work — dispatch is a new process each time, so PPID changes every call.
SESSION_ID=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('session_id', ''))
" 2>/dev/null) || true
: "${SESSION_ID:=$PPID}"

# --- Phase 1: Surface digest (one-shot, journals skills only) ---
SURFACE_GUARD="/tmp/forge-surface-shown-$SESSION_ID"
if [ ! -f "$SURFACE_GUARD" ]; then
    SKILL=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('tool_input', {}).get('skill', ''))
" 2>/dev/null) || true

    case "${SKILL:-}" in
        DailyPlan|DailyReview|Log|Inbox|BacklogJournals|JournalStructure|Timesheet|WeeklyReview)
            ensure_built surface || true
            if [ -x "$BIN_DIR/surface" ]; then
                OUTPUT=$("$BIN_DIR/surface") || true
                if [ -n "$OUTPUT" ]; then
                    DEFAULT_PROMPT="Display this surface digest VERBATIM to the user. Do not summarize, reframe, or skip any items."
                    PROMPT=""
                    for cfg in "$MODULE_ROOT/config.yaml" "$MODULE_ROOT/defaults.yaml"; do
                        [ -f "$cfg" ] || continue
                        if command -v yaml >/dev/null 2>&1; then
                            PROMPT=$(yaml nested "$cfg" surface post_tool_use_prompt 2>/dev/null) || true
                            [ -n "$PROMPT" ] && break
                        fi
                        if command -v yq >/dev/null 2>&1; then
                            PROMPT=$(yq '.surface.post_tool_use_prompt // ""' "$cfg" 2>/dev/null) || true
                            [ -n "$PROMPT" ] && break
                        fi
                    done
                    : "${PROMPT:=$DEFAULT_PROMPT}"
                    CONTEXT=$(printf '%s\n\n%s' "$PROMPT" "$OUTPUT")
                    ESCAPED=$(printf '%s' "$CONTEXT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))" 2>/dev/null) || true
                    if [ -n "$ESCAPED" ]; then
                        printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$ESCAPED"
                        touch "$SURFACE_GUARD"
                        exit 0  # Only one JSON object per invocation
                    fi
                fi
            fi
            touch "$SURFACE_GUARD"
            ;;
    esac
fi

# --- Phase 2: Periodic insight enforcement ---
# Triggers: (1) every N tool calls, (2) one-shot at 20 min, (3) high insight count.
INTERVAL="${FORGE_ADVISORY_INTERVAL:-25}"
INSIGHT_THRESHOLD="${FORGE_INSIGHT_THRESHOLD:-10}"
DURATION_THRESHOLD="${FORGE_DURATION_THRESHOLD:-1200}"  # 20 minutes in seconds

COUNTER_FILE="/tmp/forge-insight-count-$SESSION_ID"
START_FILE="/tmp/forge-session-start-$SESSION_ID"
DURATION_GUARD="/tmp/forge-duration-fired-$SESSION_ID"

# Track session start time
if [ ! -f "$START_FILE" ]; then
    date +%s > "$START_FILE"
fi

COUNT=0
if [ -f "$COUNTER_FILE" ]; then
    COUNT=$(cat "$COUNTER_FILE" 2>/dev/null) || true
fi
COUNT=$((COUNT + 1))
printf '%s' "$COUNT" > "$COUNTER_FILE"

# Extract transcript_path once (shared by all triggers)
TRANSCRIPT_PATH=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('transcript_path', ''))
" 2>/dev/null) || true

SHOULD_CHECK=0

# Trigger 1: Every INTERVAL tool calls (default 25)
if [ "$COUNT" -ge "$INTERVAL" ]; then
    SHOULD_CHECK=1
    printf '0' > "$COUNTER_FILE"
fi

# Trigger 2: Session duration exceeded (one-shot at 20 min)
if [ "$SHOULD_CHECK" -eq 0 ] && [ ! -f "$DURATION_GUARD" ]; then
    START_TIME=$(cat "$START_FILE" 2>/dev/null) || true
    if [ -n "$START_TIME" ]; then
        NOW=$(date +%s)
        ELAPSED=$((NOW - START_TIME))
        if [ "$ELAPSED" -ge "$DURATION_THRESHOLD" ]; then
            SHOULD_CHECK=1
            touch "$DURATION_GUARD"
        fi
    fi
fi

# Trigger 3: High insight count (quick grep, checked every 10 calls)
if [ "$SHOULD_CHECK" -eq 0 ] && [ $((COUNT % 10)) -eq 0 ]; then
    if [ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ]; then
        INSIGHT_COUNT=$(grep -c '★ Insight' "$TRANSCRIPT_PATH" 2>/dev/null) || true
        if [ "${INSIGHT_COUNT:-0}" -ge "$INSIGHT_THRESHOLD" ]; then
            SHOULD_CHECK=1
        fi
    fi
fi

# Run advisory check
if [ "$SHOULD_CHECK" -eq 1 ] && [ -n "$TRANSCRIPT_PATH" ]; then
    CWD_VAL=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
print(json.load(sys.stdin).get('cwd', ''))
" 2>/dev/null) || true

    ensure_built insight || exit 0
    if [ -x "$BIN_DIR/insight" ]; then
        export FORGE_INSIGHT_ADVISORY=1
        printf '{"transcript_path":"%s","cwd":"%s"}' "$TRANSCRIPT_PATH" "${CWD_VAL:-}" \
            | "$BIN_DIR/insight" || true
    fi
fi
