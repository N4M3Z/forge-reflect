# forge-reflect

Session reflection enforcement. Surfaces digest at session start, enforces memory capture (insights, imperatives) at session stop. Rust crate (`forge-reflect`).

## Build & Test

```bash
cargo build --release --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml                          # all tests
cargo test --manifest-path Cargo.toml test_name                # single test by name
cargo clippy --manifest-path Cargo.toml -- -D warnings         # lint
cargo fmt --manifest-path Cargo.toml --check                   # format check
```

### Binaries

| Binary | Purpose |
|--------|---------|
| `surface` | Session digest — resurfaces ideas, tabs, recent insights |
| `insight` | Capture insight from session transcript |
| `reflect` | Full session reflection — decisions, insights, effort |

## Code Style

- **Edition 2021**, `unsafe` forbidden, clippy pedantic enabled
- **Error handling**: `Result<T, String>` — no `anyhow`/`thiserror`
- **Module pattern**: `mod.rs` + sibling `tests.rs` (config, surface, transcript, prompt)
- **Dependencies**: `forge-core` (path dep), `chrono`, `regex`, `serde`, `serde_json`

## Hooks

| Event | Script | Purpose |
|-------|--------|---------|
| SessionStart | `session-start.sh` | Run `surface` binary, append Hook.md |
| Stop | `stop.sh` | Enforce reflection before session ends |
| PreCompact | `pre-compact.sh` | Enforce reflection before context compaction |
