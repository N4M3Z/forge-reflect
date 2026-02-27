---
name: MemoryInvariant
description: Create a memory invariant — a structural constraint that must always hold true. USE WHEN documenting a system constraint, writing to Invariants/, or promoting an imperative.
---

A structural constraint — violating it breaks the system, not just degrades quality. Invariants are rules. They can be used directly in the rule system (`rules/`) or as `Orchestration/Invariants/` files.

Read the template at the configured `template` path before creating the file. Replace Templater placeholders (`{{title}}`, `{{date:...}}`) with actual values.

Write to `rules/` using the title as filename. Tag with `type/memory/invariant`. The `invariant` field states what must always hold true (one sentence). The `domain` field scopes where it applies (code, process, identity, etc.).
