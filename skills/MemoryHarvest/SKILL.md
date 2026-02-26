---
name: MemoryHarvest
description: Mine conversation exports for new memory items — scan ChatGPT exports, Claude Code transcripts, or other conversation logs for insights, imperatives, and ideas worth capturing. USE WHEN harvesting learnings from chat history, extracting patterns from ChatGPT, mining conversation exports.
argument-hint: "[path to export or transcript]"
---

# Memory Harvest

Scan conversation history for patterns, decisions, and learnings worth capturing as memory items. Sources include ChatGPT exports (JSON/markdown in `Inbox/`), Claude Code session transcripts, and other conversation logs.

## Instructions

### Phase 1: Read Configuration

Resolve the module root:
```bash
MODULE="Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="."
```

Read config:
```bash
cat $MODULE/defaults.yaml
```

Store memory paths (user-root-relative, resolve via `FORGE_USER_ROOT`):
- `memory: insights:`, `memory: imperatives:`, `memory: ideas:` — target directories for captured items

### Phase 2: Locate Source

Identify the conversation source:
- **ChatGPT export**: JSON or markdown file in `Inbox/` or user-specified path
- **Claude Code transcript**: JSONL file in `~/.claude/projects/` (use `/Sessions` to find specific sessions)
- **Other**: any structured conversation log the user provides

Read the source content. For large files, scan in chunks and summarize.

### Phase 3: Extract Candidates

Scan the conversation for:
- **Insights** — recurring patterns, discovered conventions, tool behaviors, surprising findings
- **Imperatives** — behavioral rules derived from failures ("always X", "never Y")
- **Ideas** — proposals, future possibilities, unfinished threads worth tracking

For each candidate, draft:
- `title:` — concise, noun-phrase
- `description:` — one-line summary
- `## Log` entries with appropriate `#log/*` tags
- Relevant `keywords:` wikilinks
- Body content with enough context for future-self to act without re-reading the original conversation

### Phase 4: Present Candidates

Show each extracted candidate with its proposed content. Present via `AskUserQuestion` in batches (up to 4 per question):
- **Capture as Insight** — create in `Memory/Insights/`
- **Capture as Imperative** — create in `Memory/Imperatives/`
- **Capture as Idea** — create in `Memory/Ideas/`
- **Skip** — not worth capturing

### Phase 5: Create Memory Items

For approved candidates, create files from the type templates (`Templates/*.md`, validated by `Templates/.mdschema`):
- Insights: `Templates/Insight.md` — `## Log` seeded with `#log/context/background`
- Imperatives: `Templates/Imperative.md` — `## Log` seeded with `#log/context/background` + `#log/context/rationale`
- Ideas: `Templates/Idea.md` — `## Log` seeded with `#log/context/origin`

All types share: `cssclasses: [memory]`, `upstream: [[Memory]]`, `created:` and `updated:` timestamps.

## Constraints

- Never modify the source conversation — it's read-only input
- Deduplicate against existing memory — check if a similar insight/imperative/idea already exists before creating
- Use `safe-read` for AMBER vault files, regular Read for everything else
- If the source is very large (>1000 lines), summarize sections first and let the user direct which sections to mine

!`dispatch skill-load forge-reflect`
