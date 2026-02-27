---
name: SessionReflect
description: Interactive session reflection — capture decisions, insights, ideas, and effort logs from the current session.
---

# Session Reflect

Interactive end-of-session (or mid-session) reflection. Reviews what happened in the conversation, proposes imperatives/insights/ideas to capture, asks the user to confirm or adjust, writes memory files, and logs effort to the daily journal.

## Reusability Filter

For EACH insight, apply the REUSABILITY test — will I encounter this again?

If YES: (1) strip project-specific names, paths, line numbers,
(2) extract the transferable principle or reusable pattern,
(3) phrase it so it helps in ANY future project.

If NO (one-off debug trace, specific fix): skip it.

Example: "forge-lib deep merge fails on arrays" becomes
"YAML deep merge libraries typically replace arrays, not append —
test array merge behavior explicitly."

## Instructions

Follow these phases IN ORDER. Every phase that needs user input MUST use AskUserQuestion with pre-guessed options. The user can always select "Other" for free-text input.
If AskUserQuestion is unavailable in the current runtime, emulate the same flow in plain chat with numbered options and identical capture/adjust/skip gates.

### Phase 1: Read Configuration

#### Step 1.1: Load config

Resolve the module root:
```bash
MODULE="Modules/forge-reflect"
[ -d "$MODULE" ] || MODULE="."
```

Read the config file to get configurable paths:

```bash
cat $MODULE/defaults.yaml
```

Store the values (all content paths are user-root-relative, resolve via `FORGE_USER_ROOT`):
- `memory: imperatives:` — directory for imperative files
- `memory: insights:` — directory for insight files
- `memory: ideas:` — directory for idea files
- `journal: daily:` — daily journal path pattern (YYYY/MM/YYYY-MM-DD.md)
- `backlog:` — persistent backlog file
- `commands: safe_read:` — command for reading AMBER-classified files (project-relative)

### Phase 2: Analyze the Session

#### Step 2.1: Review conversation history

Look back through the current conversation and identify:

1. **Imperatives identified** — choices about architecture, approach, tooling, process
2. **Insights discovered** — factual findings, gotchas, patterns, things that worked or didn't
3. **Ideas surfaced** — future possibilities mentioned but not acted on
4. **Substantial work done** — anything worth an effort log entry
5. **Backlog items** — new tasks, follow-ups, or items to track
6. **Entities mentioned** — people, projects, organizations worth wikilink-ing

#### Step 2.2: Draft proposals

For each item identified, draft:
- **Imperatives**: title, description (the rule), context, rationale
- **Insights**: title, description (the finding), context (origin)
- **Ideas**: title, description (the proposal), spark
- **Effort entries**: project, duration tag, description
- **Backlog items**: description, priority

### Phase 3: Interactive Review

#### Step 3.1: Present imperatives

If any imperatives were identified, present them to the user via AskUserQuestion:

For each proposed imperative (up to 4 per batch):
- Show the proposed title and one-line summary
- **Options**: "Capture it", "Adjust — let me refine", "Skip — not an imperative"

For any the user wants to adjust, ask follow-up questions:
- "What would you change about the title or rationale?"
- "Is the status Active or should it supersede an existing imperative?"

#### Step 3.2: Present insights

If any insights were identified, present them the same way:

For each proposed insight (up to 4 per batch):
- Show the proposed title and insight
- Apply the REUSABILITY test (see filter above) — skip one-off debug traces
- **Options**: "Capture it", "Adjust — let me refine", "Skip — not reusable"

For adjustments, ask:
- "What's the transferable principle? What should any project learn from this?"

**Skip marker**: When the user chooses "Skip" for an insight, emit a line in your response:

☆ Insight: Topic Name

This marks the topic as reviewed-and-skipped in the transcript so the Stop hook won't block on it later. Use the exact topic title from the proposal.

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

#### Step 4.1: Create imperative files

For each confirmed imperative, create a file at `<memory_imperatives_path>/Title.md`:

Use `Templates/Imperative.md` as the base. Fill in frontmatter, then complete the `## Log` entries:

```markdown
## Log

- [#] YYYY-MM-DD what prompted this imperative #log/context/background
- [#] YYYY-MM-DD why this approach was chosen #log/context/rationale
```

Body = expanded detail — the actionable rule, examples, or reasoning that doesn't fit in `description:`.

#### Step 4.2: Create insight files

For each confirmed insight, create a file at `<memory_insights_path>/Title.md`:

Use `Templates/Insight.md` as the base. Fill in frontmatter, then complete the `## Log` entry:

```markdown
## Log

- [#] YYYY-MM-DD where the learning came from #log/context/background
```

Body = the actionable takeaway — verbose enough for future-self to act on without re-reading the original session.

#### Step 4.3: Create idea files

For each confirmed idea, create a file at `<memory_ideas_path>/Title.md`:

Use `Templates/Idea.md` as the base. Fill in frontmatter, then complete the `## Log` entry:

```markdown
## Log

- [#] YYYY-MM-DD what prompted this idea #log/context/origin
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

For insights: dual-write by adding a log entry that wikilinks to the insight file (full filename).

#### Step 5.2: Update backlog

For each confirmed backlog item, read the backlog via `safe_read_command` and add under the appropriate priority heading:
- `- [ ] Description [priority:: level] [due:: YYYY-MM-DD]`
- Wikilink relevant projects and people

### Phase 6: Summary

#### Step 6.1: Present what was captured

Show the user a complete list of:
- Imperative files created (with full filenames)
- Insight files created (with full filenames)
- Idea files created (with full filenames)
- Effort entries logged
- Backlog items added

#### Step 6.2: Confirm session can end

Tell the user: "Reflection complete. You can end the session now, or continue working."

!`dispatch skill-load forge-reflect`
