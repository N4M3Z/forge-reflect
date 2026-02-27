---
paths:
  - "**/forge-reflect/hooks/**"
---

Hook scripts MUST exit 0 always. Communication is via stdout JSON only — stderr is invisible to Claude Code.

- Empty stdout = allow
- `{"decision":"block","reason":"..."}` = block (Stop)
- `{"hookSpecificOutput":{"additionalContext":"..."}}` = AI context injection (PostToolUse)

Guard files use `$PPID` or `$SESSION_ID` scoping to prevent repeat firing within a session.
