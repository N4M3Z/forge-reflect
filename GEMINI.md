# GEMINI.md

Instructional context for the `forge-reflect` module within the Forge Framework.

## Project Overview

**forge-reflect** is a session lifecycle module that forms part of the **Behaviour** layer in the Forge Framework's three-layer architecture (Identity / Behaviour / Knowledge). Its primary purpose is to maintain session quality by surfacing actionable items at the start and enforcing memory capture (insights/imperatives) at the end.

### Architecture & Components
The module is implemented in Rust and integrates with AI coding tools (like Claude Code) via hooks and skills.

-   **Binaries (Rust):**
    -   `surface`: Generates a digest of stale ideas, captured tabs, and backlog items at session start.
    -   `insight`: A hard-rule validator that blocks session exit if uncaptured insights (`★ Insight`) are detected in the transcript.
    -   `reflect`: A heuristic tool that prompts for reflection or blocks exit if a "substantial" session has no memory writes.
-   **Hooks (Bash):**
    -   `session-start.sh`: Runs `surface` and appends instructions.
    -   `stop.sh`: Invokes `insight` and `reflect` to gate session termination.
    -   `pre-compact.sh`: Injects reflection prompts before context compaction.
-   **Skills (Markdown):** Define specific AI behaviors and conventions for `Surface`, `InsightCheck`, `SessionReflect`, and `MemoryInsights`.

## Building and Running

### Prerequisites
-   Rust toolchain (cargo)
-   `forge-core` (located at `../../Core`)

### Key Commands
-   **Build all binaries:** `cargo build --release`
-   **Run tests:** `cargo test`
-   **Lazy Build:** The hook scripts use `bin/_build.sh` to automatically compile binaries on their first invocation.
-   **Manual Execution:** Binaries typically expect a JSON payload via stdin.
    ```bash
    echo '{"cwd":"/path/to/workspace","transcript_path":"/path/to/transcript.jsonl"}' | ./target/release/insight
    ```

## Development Conventions

### Configuration
-   **Precedence:** `config.yaml` (gitignored user overrides) > `defaults.yaml` (project-level shared) > module `defaults.yaml` > Compiled Defaults.
-   **Shared Paths:** Many paths (backlog, journal, memory) are shared across Forge modules via the `shared:` section in the root `defaults.yaml`.

### Logic & Safety
-   **Substantiality:** A session is considered "substantial" based on `tool_turn_threshold` (default: 10) and `user_msg_threshold` (default: 4).
-   **Data Scope:** The `in_data_dir` check ensures the module only operates on paths within the user's designated Data directory (usually `~/Data`).
-   **Transcript Analysis:** The `insight` binary parses JSONL transcripts to detect tool calls to memory-writing functions or the presence of the `★ Insight` marker.

### File Locations
-   **Source:** `src/` (Lib and Binaries)
-   **Hooks:** `hooks/`
-   **Skills:** `skills/`
-   **Metadata:** `module.yaml`
