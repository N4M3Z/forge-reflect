---
name: MemoryInsights
description: Memory capture conventions — imperatives, insights, ideas, insight persistence, and idea lifecycle. USE WHEN capturing insights, recording imperatives, creating ideas, or managing the idea lifecycle.
---

## Memory & Insights

- Create individual files in `Memory/Imperatives/`, `Memory/Insights/`, and `Memory/Ideas/` — one file per item, never accumulate lists in a single file.
- Memory files (imperatives, insights, ideas) link back to their originating daily note in the body text.
- Every ★ Insight block you output MUST also be captured as a `Memory/Insights/` file. No ephemeral insights — if it's worth saying, it's worth persisting.
- Forked patterns track origin via `source:` frontmatter.
- Never discard plans or research. Promote well-specced Ideas to `Projects/` per the Idea Lifecycle. See [[Well-specced Ideas promote to Projects with full research preserved]].

### Idea Lifecycle

| Status | Meaning | Action |
|--------|---------|--------|
| **Open** | Captured, not yet investigated | No action needed |
| **Exploring** | Active research underway | Accumulate findings in body |
| **Adopted** | Promoted to Project or Imperative | Set `adopted_as:` wikilink |
| **Dismissed** | Investigated and rejected | Body explains why |

**Promotion trigger**: An Idea is "well-specced" when it has a clear goal, researched alternatives, an architecture or approach, and phased deliverables.
