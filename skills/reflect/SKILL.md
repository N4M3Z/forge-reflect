---
name: reflect
description: Interactive session reflection — capture decisions, learnings, ideas, and effort logs from the current session.
---

# Session Reflect

Interactive end-of-session (or mid-session) reflection. Reviews what happened in the conversation, proposes decisions/learnings/ideas to capture, asks the user to confirm or adjust, writes memory files, and logs effort to the daily journal.

## Instructions

Follow these phases IN ORDER. Every phase that needs user input MUST use AskUserQuestion with pre-guessed options. The user can always select "Other" for free-text input.

### Phase 1: Read Configuration

#### Step 1.1: Load config

First, resolve the module root (handles both standalone and forge-core module paths):
```bash
MODULE="${CLAUDE_PLUGIN_ROOT}/Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="${CLAUDE_PLUGIN_ROOT}"
```

Read the config file to get configurable paths:

```bash
cat $MODULE/config.yaml
```

Store the values:
- `memory_decisions_path` — directory for decision files
- `memory_learnings_path` — directory for learning files
- `memory_ideas_path` — directory for idea files
- `journal_daily_path` — daily journal path pattern (YYYY/MM/YYYY-MM-DD.md)
- `backlog_path` — persistent backlog file
- `safe_read_command` — command for reading AMBER-classified files

### Phase 2: Analyze the Session

#### Step 2.1: Review conversation history

Look back through the current conversation and identify:

1. **Decisions made** — choices about architecture, approach, tooling, process
2. **Learnings discovered** — factual findings, gotchas, patterns, things that worked or didn't
3. **Ideas surfaced** — future possibilities mentioned but not acted on
4. **Substantial work done** — anything worth an effort log entry
5. **Backlog items** — new tasks, follow-ups, or items to track
6. **Entities mentioned** — people, projects, organizations worth wikilink-ing

#### Step 2.2: Draft proposals

For each item identified, draft:
- **Decisions**: title, context, decision, rationale
- **Learnings**: title, origin, insight, actionable rule
- **Ideas**: title, spark, idea description
- **Effort entries**: project, duration tag, description
- **Backlog items**: description, priority

### Phase 3: Interactive Review

#### Step 3.1: Present decisions

If any decisions were identified, present them to the user via AskUserQuestion:

For each proposed decision (up to 4 per batch):
- Show the proposed title and one-line summary
- **Options**: "Capture it", "Adjust — let me refine", "Skip — not a decision"

For any the user wants to adjust, ask follow-up questions:
- "What would you change about the title or rationale?"
- "Is the status Active or should it supersede an existing decision?"

#### Step 3.2: Present learnings

If any learnings were identified, present them the same way:

For each proposed learning (up to 4 per batch):
- Show the proposed title and insight
- **Options**: "Capture it", "Adjust — let me refine", "Skip — already known"

For adjustments, ask:
- "What's the actionable takeaway? What should future-you remember?"

#### Step 3.3: Present ideas

If any ideas were surfaced, present them:

For each proposed idea (up to 4 per batch):
- Show the proposed title and description
- **Options**: "Capture it", "Adjust — let me refine", "Skip — not worth tracking"

#### Step 3.4: Present effort entries

If substantial work was done, propose effort log entries:

- Show the proposed entries grouped by project
- **Options**: "Log it", "Adjust duration/description", "Skip"
- Ask: "Is this a highlight advancing your active goals?" (for `#log/highlight` tagging)

#### Step 3.5: Present backlog items

If follow-ups or new tasks were identified:

For each proposed item (up to 4 per batch):
- Show the description
- **Options**: "Add to backlog — High", "Add to backlog — Medium", "Add to backlog — Low", "Skip"

### Phase 4: Write Memory Files

#### Step 4.1: Create decision files

For each confirmed decision, create a file at `<memory_decisions_path>/YYYY-MM-DD — Title.md` using the Decision schema from CLAUDE.md:

```yaml
title: Short descriptive title
aliases: []
tags:
keywords:
  - "[[Topic]]"
context: What prompted this decision
decision: What was decided
rationale: Why this approach was chosen
status: Active
superseded_by:
created: YYYY-MM-DD
updated: YYYY-MM-DD
```

Body = expanded detail where frontmatter fields are too brief.

#### Step 4.2: Create learning files

For each confirmed learning, create a file at `<memory_learnings_path>/YYYY-MM-DD — Title.md` using the Learning schema:

```yaml
title: Short descriptive title
aliases: []
tags:
keywords:
  - "[[Topic]]"
origin: Where the learning came from
insight: Concise factual finding
created: YYYY-MM-DD
updated: YYYY-MM-DD
```

Body = the rule (actionable takeaway, verbose enough for future-self).

#### Step 4.3: Create idea files

For each confirmed idea, create a file at `<memory_ideas_path>/YYYY-MM-DD — Title.md` using the Idea schema:

```yaml
title: Short descriptive title
aliases: []
tags:
keywords:
  - "[[Topic]]"
spark: What prompted this idea
idea: Concise description of the proposal
status: Open
adopted_as:
created: YYYY-MM-DD
updated: YYYY-MM-DD
```

Body = expanded detail, potential approaches, considerations.

### Phase 5: Update Daily Log & Backlog

#### Step 5.1: Append effort entries to daily log

Read today's daily log using `safe_read_command` via Bash. Insert new effort entries before `![[Daily.base]]`.

Follow the journal style:
- `#log/effort/short` (~30m), `#log/effort/mid` (~1h), `#log/effort/long` (~90m), `#log/effort/extended` (120+m)
- Use parent `#log/effort [[Project]]` to group related items
- Inline descriptions flow after the tag
- Notes as plain text children
- Wikilink liberally — people, projects, organizations, topics

For learnings: dual-write by adding a log entry that wikilinks to the learning file (full filename with date prefix).

#### Step 5.2: Update backlog

For each confirmed backlog item, read the backlog via `safe_read_command` and add under the appropriate priority heading:
- `- [ ] Description [priority:: level] [due:: YYYY-MM-DD]`
- Wikilink relevant projects and people

### Phase 6: Summary

#### Step 6.1: Present what was captured

Show the user a complete list of:
- Decision files created (with full filenames)
- Learning files created (with full filenames)
- Idea files created (with full filenames)
- Effort entries logged
- Backlog items added

#### Step 6.2: Confirm session can end

Tell the user: "Reflection complete. You can end the session now, or continue working."

!`"${CLAUDE_PLUGIN_ROOT}/hooks/skill-load.sh" 2>/dev/null`
!`"${CLAUDE_PLUGIN_ROOT}/Modules/forge-reflect/hooks/skill-load.sh" 2>/dev/null`
