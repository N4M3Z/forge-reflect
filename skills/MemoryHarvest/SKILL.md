---
name: MemoryHarvest
description: Scan memory for promotion candidates — insights, imperatives, and ideas that could become steering rules, skills, agents, or scripts. USE WHEN reviewing accumulated memory for actionable patterns, operationalizing learnings, or asking what memory items are ready to promote.
---

# Memory Harvest

Scan all memory directories for items that could be promoted into actionable artifacts. Everything in `Memory/` is an unpromoted candidate (promoted items live in `Archives/Memory/`).

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
- `memory: insights:`, `memory: imperatives:`, `memory: ideas:` — source directories

### Phase 2: Scan Memory

Use Glob to list all `.md` files in the three memory directories (exclude folder notes and index files).

Count totals per type for the user's awareness.

### Phase 3: Read Candidates

Read the frontmatter (first ~20 lines) of each candidate. Extract:
- `title:` — display name
- `keywords:` — for clustering
- `status:` — for imperatives (Active/Superseded) and ideas (Open/Exploring/Adopted/Dismissed)
- `insight:` or `decision:` or `idea:` — the core content for recommendation

For large sets, prioritize:
1. **Imperatives with `status: Active`** — behavioral rules ready to codify
2. **Ideas with `status: Adopted` or `Exploring`** — well-specced and ready to act on
3. **Insights** — sort by age if many, oldest first (longest unacted-on)

### Phase 4: Group and Recommend

Group candidates by:
1. **Keyword clusters** — items sharing the same `keywords:` wikilinks (e.g., multiple items about `[[Bash Scripting]]` suggest a consolidated steering rule or skill)
2. **Destination affinity** — group by recommended landing zone:
   - Behavioral rules → Steering
   - Procedures → Skills
   - Agent tweaks → Agents
   - Tool gotchas → Auto-memory

Present grouped candidates to the user with:
- The item title and one-line summary (from `insight:`/`decision:`/`idea:`)
- Recommended destination type
- Keyword cluster context (if applicable)

### Phase 5: User Selection

Ask the user which items to promote via AskUserQuestion. Options per item or group:
- **Promote** — proceed to `/MemoryPromote`
- **Skip** — leave in Memory for now
- **Dismiss** — (ideas only) mark as `status: Dismissed`

For selected items, invoke `/MemoryPromote` for each.

## Constraints

- Only scan `Memory/{Insights,Imperatives,Ideas}/` — never scan Archives
- Do not modify memory files during harvest — modifications happen in `/MemoryPromote`
- If no candidates found, report that Memory is clean and suggest running `/SessionReflect` to capture new learnings
- For very large sets (50+ candidates), present a summary first and let the user filter by type or keyword before showing details

!`dispatch skill-load forge-reflect`
