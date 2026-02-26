---
name: MemoryReview
description: Review and maintain memory files — one-by-one migration, archive review, quality cleanup, and promotion candidate scanning. USE WHEN reviewing accumulated memory, migrating memory schema, reviewing archived memories, finding promotion candidates.
argument-hint: "[archive|migrate|scan] [insights|imperatives|ideas]"
---

# Memory Review

Quality maintenance for the memory system. Three modes: **scan** for promotion candidates, **archive** review for archived items, and **migrate** for one-by-one schema + content review of active items.

## Instructions

Resolve the module root and read `defaults.yaml`:
```bash
MODULE="Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="."
cat $MODULE/defaults.yaml
```

Store memory paths (user-root-relative, resolve via `FORGE_USER_ROOT`):
- `memory: insights:`, `memory: imperatives:`, `memory: ideas:` — active directories
- `promote: archive:` — archive root

Route to the appropriate mode based on the user's request.

---

## Promotion Scan

Scan memory for items ready to be promoted into actionable artifacts.

1. Use Glob to list all `.md` files in the three memory directories (exclude folder notes and index files). Count totals per type.
2. Read each candidate — extract `title:`, `description:`, `keywords:`, and `## Log` entries. Prioritize imperatives first, then well-specced ideas, then oldest insights.
3. Group candidates by keyword clusters and destination affinity (behavioral rules → Steering, procedures → Skills, agent tweaks → Agents, tool gotchas → Auto-memory, structural constraints → Invariants).
4. Present grouped candidates to the user via `AskUserQuestion` with title, summary, and recommended destination.
5. For items the user chooses to promote, invoke `/MemoryPromote` for each. For dismissed ideas, add `- [#] YYYY-MM-DD reason #log/decision/dismissed` to `## Log`.

---

## Archive Review

Deep research on archived items before presenting improvement recommendations.

1. Read all `.md` files in `Archives/Memory/<type>/`. Read full content, not just frontmatter.
2. For each item, research before recommending:
   - **Cross-reference the vault** — find related active notes, steering rules, skills encoding the same principle. Check if `sources:` artifacts still exist and have evolved.
   - **Check conversation history** — use `mcp__c0ntextkeeper__fetch_context` and `mcp__c0ntextkeeper__search_archive` to find follow-up decisions or reinforcements.
   - **Analyze** for redundancy, generality (tool-specific → universal), staleness, evolution, or consolidation opportunities.
3. Always show the **full current content** of each item before presenting recommendations. The user must see what exists before deciding what to change. Present items one at a time via `AskUserQuestion` with a specific, evidence-backed recommendation. Options: generalize, merge, update, delete, or looks good.
4. For approved changes, show a **unified diff** of proposed changes. Present via `AskUserQuestion` for final approval — no file writes without explicit sign-off.
5. Apply only approved changes. Update `updated:` timestamps. Use `safe-write write` only for files with `#tlp/red` blocks or redacted secrets; use the Write tool for everything else.

---

## Migration

Walk through active memory files one at a time, combining schema migration with content review.

1. List all `.md` files in `Memory/<type>/` (excluding collection index files like `Ideas.md`). Show the full list so the user knows the scope.
2. For each file, process in order:
   - **Show** the full file content (frontmatter + body)
   - **Research** — cross-reference the vault, check c0ntextkeeper, find related notes for the `related:` field
   - **Recommend** via `AskUserQuestion` — schema migration (see Legacy Field Migration table), content improvement (generalize, reframe, clean up), type conversion (see Type Conversion table), or archive if adopted/superseded
   - **Diff** — show a unified diff of proposed changes
   - **Apply** — only after user approves

### Type Conversion

When converting between memory types, update all type-specific fields:

| Field | Insight | Imperative | Idea |
|-------|---------|------------|------|
| tags | `type/memory/insight` | `type/memory/imperative` | `type/memory/idea` |
| collection | `[[Insights]]` | `[[Imperatives]]` | `[[Ideas]]` |
| icon | LiLightbulb | LiShield | LiSparkles |
| log tags | `#log/context/background` | `#log/context/background`, `#log/context/rationale` | `#log/context/origin` |
| directory | `Memory/Insights/` | `Memory/Imperatives/` | `Memory/Ideas/` |

### Canonical Schema

**Frontmatter** (all types share one structure):

`title`, `aliases`, `tags`, `keywords`, `description`, `collection`, `icon`, `image`, `cssclasses`, `created`, `updated`, `related`, `upstream`, `sources`

**Body**: HTML comment (type-specific writing prompt) + `## Log` section with `- [#]` tasks using `#log/*` tags.

**Log tags** — hierarchical, queryable via Obsidian Tasks:

| Tag | Meaning |
|-----|---------|
| `#log/context/origin` | What sparked this (Ideas) |
| `#log/context/background` | Background situation (Insights, Imperatives) |
| `#log/context/rationale` | Why this approach (Imperatives) |
| `#log/decision/adopted` | Promoted to artifact |
| `#log/decision/dismissed` | Investigated and rejected |
| `#log/decision/superseded` | Replaced by a successor |

---

## Constraints

- **Promotion scan**: `Memory/{Insights,Imperatives,Ideas}/` only. Do not modify files — hand off to `/MemoryPromote`.
- **Archive review**: `Archives/Memory/<type>/` only. May modify files directly.
- **Migration**: `Memory/<type>/` one file at a time. Always modifies files.
- If no candidates found (scan mode), report Memory is clean and suggest `/SessionReflect`.
- For large sets (50+ items), present a summary first and let the user filter.

!`dispatch skill-load forge-reflect`
