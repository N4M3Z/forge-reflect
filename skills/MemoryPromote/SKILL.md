---
name: MemoryPromote
description: Promote a memory item (insight, imperative, or idea) into an actionable artifact — steering rule, skill, agent instruction, script, or auto-memory entry. USE WHEN the user selects a memory item for promotion, wants to operationalize a learning, or asks to turn an insight into a rule or skill.
argument-hint: "[MemoryTitle or path]"
---

# Memory Promote

Promote a memory artifact from `Memory/` into an actionable building block, then archive the source. This is the memory-to-artifact counterpart of `/Promote` (which handles skills).

## Instructions

Follow these phases IN ORDER. Use AskUserQuestion for decisions.

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

Store promotion paths (all user-root-relative, resolve via `FORGE_USER_ROOT`):
- `promote: steering:` — landing zone for steering rules
- `promote: skills:` — landing zone for skills
- `promote: agents:` — landing zone for agents
- `promote: scripts:` — landing zone for scripts
- `promote: archive:` — archive root for promoted memory
- `memory: insights:`, `memory: imperatives:`, `memory: ideas:` — source directories

### Phase 2: Read Source

Read the specified memory file. Determine the memory type from its parent directory:
- `Memory/Insights/` → insight
- `Memory/Imperatives/` → imperative
- `Memory/Ideas/` → idea

If the argument is a title (not a path), search all three memory directories for a matching filename.

Extract:
- Frontmatter fields (`description`, `comments`, `sources`, `related`)
- Body content
- `keywords:` for context

### Phase 3: Recommend Destination

Analyze the memory item's content and recommend a landing zone. Present recommendations to the user via AskUserQuestion.

Content-to-destination heuristics:
- **"Always/never do X"** behavioral rules, recurring failure patterns → **Steering rule** (`promote.steering`)
- **Multi-step workflows**, procedures, tool usage patterns → **Skill** (`promote.skills`)
- **Agent behavior** refinements, persona tweaks, tool restrictions → **Agent** (`promote.agents`)
- **Debugging patterns**, API quirks, tool gotchas, environment facts → **Auto-memory** (MEMORY.md)
- **Reusable automation** candidates, shell one-liners → **Script** (`promote.scripts`)
- **Vault structure** or format rules → **Convention** (module CONVENTIONS.md)

The user may choose a different destination than recommended, or promote to multiple destinations.

### Phase 4: Generate Artifact

Write the artifact to the chosen landing zone. Format depends on destination type:

#### Steering Rule → `$FORGE_USER_ROOT/<promote.steering>/<Name>.md`

Transform the memory content into Statement/Bad/Correct format:

```markdown
### <Rule Name>

**Statement:** <What should happen — derived from the memory item's core lesson>
**Bad:** <Concrete example of wrong behavior — derived from the failure/context that prompted the memory>
**Correct:** <Concrete example of right behavior — the actionable takeaway>
```

The file is a flat `.md` (no frontmatter required). forge-steering's DCI auto-loads it at session start.

#### Skill → `$FORGE_USER_ROOT/<promote.skills>/<Name>/SKILL.md`

Create a standard skill:
```yaml
---
name: SkillName
description: Brief description. USE WHEN trigger1, trigger2.
source_module: forge-reflect
---
```

Follow the body conventions from `/CreateSkill`: heading, instructions, constraints.

#### Agent → `$FORGE_USER_ROOT/<promote.agents>/<Name>.md`

Either edit an existing agent file or create a new one with `claude.*` frontmatter:
```yaml
---
claude.name: kebab-case-name
claude.model: sonnet
claude.description: What this agent does
---
```

#### Script → `$FORGE_USER_ROOT/<promote.scripts>/<name>.sh`

Generate a shell script following conventions: `set -euo pipefail`, `command` prefix for aliased commands (`cd`, `cp`, `mv`, `rm`).

#### Auto-memory → `~/.claude/projects/.../memory/MEMORY.md`

Append a concise entry to the appropriate section of MEMORY.md. Follow the existing format (topic headers, bullet points, actionable patterns).

#### Convention → Module `CONVENTIONS.md`

Propose an edit to the relevant module's CONVENTIONS.md. Show the proposed addition and confirm with the user before writing.

### Phase 5: Archive Source

After the artifact is successfully written:

1. Move the memory file from `Memory/<type>/` to `$FORGE_USER_ROOT/<promote.archive>/<type>/`:
   ```bash
   command mv "$source" "$archive_dir/"
   ```

2. Add lifecycle entries to `comments:`:
   ```yaml
   comments:
     - "Adopted: into [[ArtifactName]]"
     - "Archived: moved to Archives/Memory/ on YYYY-MM-DD"
   ```

3. Add a commit-pinned permalink to the file that captured the promoted concept to `sources:`:
   ```yaml
   sources:
     - https://github.com/N4M3Z/forge-text/blob/a1b2c3d/skills/Emojify/SKILL.md
   ```
   Use `git -C <module> rev-parse HEAD` to get the current commit SHA. Pinning to a commit ensures the link is a permanent snapshot.

4. Update the `updated:` date.

### Phase 6: Summary

Report:
- What artifact was generated (with path)
- Where the source was archived
- Suggested next steps:
  - Steering rules: "Rule will auto-load next session via DCI"
  - Skills: "Run `/Promote` when ready to push to a module"
  - Scripts: "Test in Scratch/, move to Hooks/ when proven"
  - Agents: "Run `Hooks/sync-agents.sh` to deploy"

## Constraints

- Only promote files from `Memory/{Insights,Imperatives,Ideas}/` — never from Archives or other locations
- Always archive the source after promotion — Memory/ should only contain unpromoted items
- Never delete the source — always move to Archives
- Add `Adopted:` and `Archived:` entries to `comments:` on the source file
- Multiple promotions from the same item are allowed (user can run MemoryPromote multiple times before archiving, or re-promote from Archives manually)

!`dispatch skill-load forge-reflect`
