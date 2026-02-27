Every `★ Insight` block in a session MUST be either:
- **Captured** — written as a `Memory/Insights/` file
- **Skipped** — marked with `☆ Insight: Topic Name` in your response

The Stop hook scans the transcript for both markers. Uncaptured insights block session exit.

When skipping an insight during `/SessionReflect`, emit the skip marker on its own line so the hook can parse it.
