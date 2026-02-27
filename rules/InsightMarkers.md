Every `★ Insight` block in a session MUST be either:
- **Captured** — written as a `Memory/Insights/` file, then marked with `✓ Insight: Topic Name → Filename.md`
- **Skipped** — marked with `☆ Insight: Topic Name` in your response

The Stop hook scans the transcript for all three markers (`★`, `☆`, `✓`). Uncaptured insights block session exit.

After writing an insight file, emit the captured marker on its own line so the hook can link the topic to the file. When skipping, emit the skip marker on its own line.
