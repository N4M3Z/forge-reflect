---
name: MemoryImperative
description: Create a memory imperative — a rule or decision with scope, rationale, and exceptions. USE WHEN recording a decision, establishing a convention, writing to Memory/Imperatives/.
---

A rule or decision — scope, rationale, and exceptions documented so it can be followed consistently. Proven imperatives can promote to invariants via `/MemoryPromote`.

Read the template at the configured `template` path before creating the file. Replace Templater placeholders (`{{title}}`, `{{date:...}}`) with actual values.

Write to `memory/imperatives/` using the title as filename. Tag with `type/memory/imperative`. The `context` field records what prompted the decision. The `rationale` field explains why this approach was chosen.
