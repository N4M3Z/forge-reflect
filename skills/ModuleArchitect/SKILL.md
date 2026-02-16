---
name: ModuleArchitect
description: General guidance for designing, building, and validating Forge modules.
---

# Module Architect

Expert guide for creating robust Forge Framework modules. Focuses on the "Three-Layer Concern" architecture and ensures modules are portable across AI coding tools.

## Module Structure

Every Forge module must follow this standard layout:
- `module.yaml` — Metadata and event registration.
- `defaults.yaml` — Default configuration with shared path support.
- `bin/` — Entry points or build scripts (e.g., `_build.sh`).
- `hooks/` — Bash scripts triggered by Forge events.
- `skills/` — AI instructions and steering content.
- `src/` — Source code (typically Rust) for module logic.
- `VERIFY.md` — Post-installation checklist for AI agents.

## Core Mandates

1. **Separation of Concerns**: Keep parsing logic "pure" (no I/O) in library modules. Let binaries handle the environment and file reads.
2. **Path Resolution**: Use `forge-core` utilities to resolve user paths relative to the vault root or CWD.
3. **Lazy Compilation**: Use `bin/_build.sh` to compile binaries on first hook invocation, ensuring low overhead.
4. **Validation Driven**: Always provide a `VERIFY.md` that allows an AI agent to confirm the module is functional without manual intervention.

## Architectural Patterns

- **Identity Layer**: Does this module store user-specific knowledge?
- **Behaviour Layer**: Does it change how the AI responds or gates session flow?
- **Knowledge Layer**: Does it provide new tools or skills?

## Validation Flow

1. **Unit Tests**: Run `cargo test` (or equivalent).
2. **Binary Availability**: Check if binaries respond to `--help` or `--version`.
3. **Hook Dry-run**: Simulate hook inputs (JSON via stdin or CLI args) and check the output JSON for correct decisions.
