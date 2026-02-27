Query ContextKeeper MCP (`fetch_context`, `search_archive`) at these trigger points:

1. **Post-compaction** — recover implementation details the summary compressed away
2. **Session resume** — check prior sessions on the same topic before starting work
3. **Before non-trivial work** — see if a similar problem was solved before

Use `fetch_context` with a task-specific query, scope to `project`.
