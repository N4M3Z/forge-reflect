# forge-reflect

Session reflection enforcement. Surfaces digest at session start, enforces memory
capture (insights, imperatives) at session stop. Standalone Rust crate with a path
dependency on `forge-core` (`../../Core`).

## Build & Test

```bash
cargo build --release --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml                          # all tests
cargo test --manifest-path Cargo.toml test_name                # single test by name
cargo clippy --manifest-path Cargo.toml -- -D warnings         # lint
cargo fmt --manifest-path Cargo.toml --check                   # format check
```

Always run clippy and fmt check before considering work complete.

### Binaries

| Binary    | Source                | Purpose                                              |
|-----------|-----------------------|------------------------------------------------------|
| `surface` | `src/bin/surface.rs`  | SessionStart digest — resurfaces ideas, tabs, insights |
| `insight` | `src/bin/insight.rs`  | Hard rule — blocks exit if uncaptured insight blocks  |
| `reflect` | `src/bin/reflect.rs`  | Soft heuristic — blocks exit if substantial session lacks memory writes |

All binaries return `ExitCode` from `main()` and always exit 0 (hook contract).
Errors are logged to stderr via `eprintln!`; structured output goes to stdout as JSON
(`{"decision":"block","reason":"..."}`) or plain text.

## Architecture

```
src/
├── lib.rs                    # Library root — HookInput, read_hook_input(), in_data_dir()
├── bin/
│   ├── insight.rs            # Binary: uncaptured insight detection
│   ├── reflect.rs            # Binary: session substantiality heuristic
│   └── surface.rs            # Binary: session-start digest
├── config/
│   ├── mod.rs                # Config struct, Default impl, YAML loading, path resolution
│   └── tests.rs              # Config tests
├── surface/
│   ├── mod.rs                # Pure parsing: backlog, reminders, ideas, tabs, journal
│   └── tests.rs              # Surface parsing tests
├── transcript/
│   ├── mod.rs                # JSONL transcript analysis (messages, tool turns, memory writes)
│   └── tests.rs              # Transcript analysis tests
└── prompt/
    ├── mod.rs                # Pattern file loading with frontmatter/H1 stripping
    └── tests.rs              # Prompt tests
```

### Module Pattern

Every module uses `mod.rs` + sibling `tests.rs`. Each `mod.rs` ends with:
```rust
#[cfg(test)]
mod tests;
```
Exception: `lib.rs` has inline `#[cfg(test)] mod tests { ... }`.

## Code Style

### Rust Edition & Linting

- **Edition 2021**. `unsafe` code is **forbidden** (`unsafe_code = "forbid"`).
- **Clippy**: `all` + `pedantic` at warn level. Four pedantic lints are allowed:
  `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`.
- **Formatting**: Default `rustfmt` settings (no `rustfmt.toml`). 4-space indentation.

### Imports

- Selective imports only — no globs (`use x::*`) except `use super::*;` in test files.
- Multi-item braces: `use chrono::{Datelike, Local, NaiveDate};`
- No blank-line separation between import groups. Rough order: external crates, crate-local, std.
- Binaries import from the library crate: `use forge_reflect::config::Config;`
- Modules import internally via `crate::`: `use crate::config::Config;`

### Error Handling

- **No `anyhow`, `thiserror`, or custom error enums.** Keep it that way.
- **`Option<T>`** is the dominant return type for fallible functions (not `Result`).
- Convert `Result` to `Option` via `.ok()?` for chained propagation.
- Use `let Ok(x) = expr else { return None; };` or `else { continue; }` for early exits.
- Use `.expect("valid regex")` only for compile-time-constant regex patterns.
- Use `.unwrap_or_default()` / `.unwrap_or("...")` for safe fallbacks.
- Binaries: `eprintln!` for diagnostics, always exit 0. Never panic in binaries.

### Naming

- **Functions**: `snake_case` — `parse_backlog`, `format_reminders`, `analyze_transcript`
- **Structs**: `PascalCase` — `Config`, `HookInput`, `TranscriptAnalysis`
- **Nested config structs**: `{Feature}Config` suffix — `MemoryConfig`, `SurfaceConfig`
- **Fields**: `snake_case` — `stop_hook_active`, `insight_marker`, `ideas_cutoff_days`
- **Modules**: Short `snake_case` single words — `config`, `surface`, `transcript`, `prompt`
- **Constants**: None defined. Constant-like values live in `Default` impls.
- **Test functions**: Prefer `test_` prefix; descriptive `snake_case` also accepted.

### Types & Signatures

- **String parameters**: Always `&str`. Struct fields always owned `String`.
- **Config parameters**: `&Config`. Path parameters: `&Path`.
- **Collection params**: Slice references — `items: &[String]`, `entries: &[(String, String, String)]`.
- **Return types**: `Option<String>` for "might produce output", `Vec<String>` for collections,
  `ExitCode` for binary `main()`, `bool` for predicates.
- **No explicit lifetimes** — all elided. No custom generics or traits.
- **Derive macros**: `#[derive(Debug, Deserialize)]` on structs. `Default` is hand-implemented.
- **Serde**: `#[serde(default)]` on all config structs for partial YAML. `#[serde(skip)]` to
  exclude computed fields.

### Visibility

- All library module functions and structs are `pub` with `pub` fields.
- Binary-local helper functions are private (no `pub`).
- Internal-only structs (e.g., `BacklogItem`) are private.

### Documentation

- `///` doc comments on all public functions and structs.
- `//!` module-level docs at the top of `mod.rs` files.
- Inline `//` comments for private config fields and implementation notes.
- No doc comments on test functions or test helpers.

### String Handling

- Params: `&str`. Fields: `String`. Returns: `Option<String>` or `Vec<String>`.
- Build multi-line output with `String::new()` + `.push_str()`.
- Use `format!()` with inline interpolation (`{variable}`, not positional).
- Convert with `.to_string()` or `.into()` (the latter mainly in tests).
- Unicode escapes for special characters: `\u{2605}` (star), `\u{2022}` (bullet).

## Testing

- ~41 unit tests across 5 test locations. No integration tests directory.
- **Test files** use `use super::*;` as sole import from parent, plus needed externals.
- **Helpers** at the top of each test file: short factory functions (`cfg()`, `date()`,
  `make_assistant_text()`). Use `serde_json::json!` for JSON fixtures.
- **Assertions**: `assert!(result.contains("..."))` for string checks, `assert_eq!` for
  exact values, `assert!(x.is_none())` / `assert!(x.is_empty())` for absence.
- **Section separators**: `// --- function_name ---` comments group related tests.
- **Inline fixtures**: YAML/JSON defined as string literals inside tests, not external files.
- **Deprecated functions**: Test with `#[allow(deprecated)]` on the test function.
- **Dev dependencies**: `serde_yaml` (config tests), `tempfile` (filesystem tests),
  `assert_cmd` + `predicates` (available for binary integration tests).

## Hooks

| Event        | Script               | Purpose                                        |
|--------------|----------------------|------------------------------------------------|
| SessionStart | `session-start.sh`   | Run `surface` binary, append Hook.md           |
| Stop         | `stop.sh`            | Chain `insight` (hard) then `reflect` (soft)   |
| PreCompact   | `pre-compact.sh`     | Run `reflect` binary for context compaction    |

Hook scripts source `bin/_build.sh` for lazy compilation. The `ensure_built` function
checks if the release binary exists and builds it on demand.

## Dependencies

| Crate         | Purpose                    |
|---------------|----------------------------|
| `forge-core`  | Path dep (`../../Core`) — shared YAML/config utilities |
| `chrono`      | Date/time handling         |
| `regex`       | Pattern matching           |
| `serde`       | Serialization (with derive)|
| `serde_json`  | JSON parsing/construction  |
