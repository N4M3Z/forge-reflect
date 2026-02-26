---
name: MemoryInsights
description: Memory capture conventions — imperatives, insights, ideas, insight persistence, and idea lifecycle. USE WHEN capturing insights, recording imperatives, creating ideas, or managing the idea lifecycle.
---

## Memory & Insights

- Create individual files in `Memory/Imperatives/`, `Memory/Insights/`, and `Memory/Ideas/` — one file per item, never accumulate lists in a single file.
- Memory files (imperatives, insights, ideas) link back to their originating daily note in the body text.
- Every ★ Insight block you output MUST also be captured as a `Memory/Insights/` file. No ephemeral insights — if it's worth saying, it's worth persisting.
- Forked patterns track origin via `source:` frontmatter.
- Never discard plans or research. Promote well-specced Ideas via `/MemoryPromote` or the Idea Lifecycle.

### Unified Frontmatter

All memory types use one universal schema. Type lives in `tags:`, not in field structure. Templates in `Templates/`:

| Type | Template | Tag | Icon | Collection |
|------|----------|-----|------|------------|
| Insight | `Templates/Insight.md` | `type/memory/insight` | LiLightbulb | `[[Insights]]` |
| Imperative | `Templates/Imperative.md` | `type/memory/imperative` | LiShield | `[[Imperatives]]` |
| Idea | `Templates/Idea.md` | `type/memory/idea` | LiSparkles | `[[Ideas]]` |

### Log Section

Every memory file has a `## Log` section — the lifecycle ledger. Each entry is a `- [#]` task with a `#log/*` tag, queryable via Obsidian Tasks across the entire vault. Format: `- [#] YYYY-MM-DD description #log/tag`

Tags are hierarchical — searching `#log/context` also matches `#log/context/origin`, `#log/context/rationale`, and `#log/context/background`.

**Context tags** (provenance — what created this memory):

| Tag | Seeded in | Meaning |
|-----|-----------|---------|
| `#log/context` | All types | General context (supertype) |
| `#log/context/origin` | Ideas | What sparked the idea |
| `#log/context/rationale` | Imperatives | Why this approach was chosen |
| `#log/context/background` | Insights, Imperatives | Background situation or prior state |

**Decision tags** (lifecycle — what happened to it):

| Tag | Used by | Meaning |
|-----|---------|---------|
| `#log/decision` | All types | A decision point was reached (supertype) |
| `#log/decision/adopted` | All types | Promoted to artifact — `into [[ArtifactName]]` |
| `#log/decision/dismissed` | Ideas | Investigated and rejected |
| `#log/decision/superseded` | All types | Replaced by a successor |

External URLs go in `sources:` frontmatter, not in the log.

### Idea Lifecycle

Ideas track status via directory location and `## Log` entries — no dedicated `status:` field needed.

| State | Location | Log entry | Action |
|-------|----------|-----------|--------|
| **Open** | `Memory/Ideas/` | `#log/context/origin` | No action needed |
| **Exploring** | `Memory/Ideas/` | (none) | Accumulate findings in body |
| **Adopted** | `Archives/Memory/Ideas/` | `#log/decision/adopted` | Promoted via `/MemoryPromote` |
| **Dismissed** | `Archives/Memory/Ideas/` | `#log/decision/dismissed` | Body explains why |

**Promotion trigger**: An Idea is "well-specced" when it has a clear goal, researched alternatives, an architecture or approach, and phased deliverables. Use `/MemoryPromote` to operationalize it.

Items evolve across types (idea → insight → imperative). Use `related:` to link predecessors/successors.

### Imperative → Invariant Promotion

Some imperatives harden into **structural invariants** — properties that must hold for the system to function correctly (violation = broken system, not just bad practice). Invariants are not a 4th memory type; they're a promotion destination for imperatives that have proven durable.

Invariants are standalone Obsidian notes in `Orchestration/Invariants/`, wikilinked from the relevant ARCHITECTURE.md or CONVENTIONS.md `## Invariants` section. This gives them full vault citizenship (discoverable, linkable, frontmatter-searchable) while anchoring them in the architectural docs that enforce them.

**When an imperative is an invariant candidate:**
- It describes a structural constraint, not a behavioral preference
- Violating it breaks the system (not just degrades quality)
- It's testable — a linter, schema check, or CI step could enforce it

**Promotion path:** Imperative → `/MemoryPromote` → invariant note in `Orchestration/Invariants/` + wikilink in ARCHITECTURE.md or CONVENTIONS.md `## Invariants`. The imperative gets archived with a `#log/decision/adopted` entry in `## Log`.

### Memory Promotion

Promoted memory items move to `Archives/Memory/<type>/` with `#log/decision/adopted` entries in `## Log`. Use `/MemoryHarvest` to find candidates, `/MemoryPromote` to execute.

!`dispatch skill-load forge-reflect`
