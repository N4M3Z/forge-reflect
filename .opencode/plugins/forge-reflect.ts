import type { Plugin } from "@opencode-ai/plugin"
import path from "path"

/**
 * forge-reflect opencode plugin adapter.
 *
 * Bridges the Rust binaries (surface, insight, reflect) into opencode's
 * event-based plugin system. Covers three hook events:
 *
 * - session.created  -> surface digest (SessionStart equivalent)
 * - session.compacting -> reflect in PreCompact mode
 * - session.idle     -> insight + reflect in Stop mode (TUI toast, non-blocking)
 */
export const ForgeReflect: Plugin = async ({ $, directory, worktree }) => {
  const moduleRoot = worktree || directory
  const binDir = path.join(moduleRoot, "target", "release")
  const surface = path.join(binDir, "surface")
  const insight = path.join(binDir, "insight")
  const reflect = path.join(binDir, "reflect")

  // Ensure binaries are built; skip silently if cargo unavailable
  async function ensureBuilt(): Promise<boolean> {
    try {
      await $`test -x ${surface} && test -x ${insight} && test -x ${reflect}`
      return true
    } catch {
      // Binaries missing — try building
      try {
        await $`cargo build --release --manifest-path ${path.join(moduleRoot, "Cargo.toml")}`.quiet()
        return true
      } catch {
        return false
      }
    }
  }

  // Run a binary with optional stdin, return stdout or empty string
  async function runBinary(bin: string, stdin?: string): Promise<string> {
    try {
      if (stdin) {
        return await $`echo ${stdin} | FORGE_MODULE_ROOT=${moduleRoot} ${bin}`.text()
      }
      return await $`FORGE_MODULE_ROOT=${moduleRoot} ${bin}`.text()
    } catch {
      return ""
    }
  }

  const ready = await ensureBuilt()

  return {
    // SessionStart equivalent: run surface binary, show digest as TUI toast
    event: async ({ event }) => {
      if (event.type === "session.created" && ready) {
        const digest = await runBinary(surface)
        if (digest.trim()) {
          return {
            type: "tui.toast.show" as const,
            toast: { message: digest.trim(), level: "info" as const },
          }
        }
      }

      // Stop equivalent: run insight then reflect on session idle
      if (event.type === "session.idle" && ready) {
        const input = JSON.stringify({ cwd: directory, transcript_path: "" })

        // Insight check first (hard rule)
        const insightResult = await runBinary(insight, input)
        if (insightResult.trim()) {
          try {
            const parsed = JSON.parse(insightResult.trim())
            if (parsed.decision === "block") {
              return {
                type: "tui.toast.show" as const,
                toast: {
                  message: `forge-reflect: ${parsed.reason}`,
                  level: "warn" as const,
                },
              }
            }
          } catch {
            // Not valid JSON, ignore
          }
        }

        // Reflect check (soft heuristic)
        const reflectResult = await runBinary(reflect, input)
        if (reflectResult.trim()) {
          try {
            const parsed = JSON.parse(reflectResult.trim())
            if (parsed.decision === "block") {
              return {
                type: "tui.toast.show" as const,
                toast: {
                  message: `forge-reflect: ${parsed.reason}`,
                  level: "warn" as const,
                },
              }
            }
          } catch {
            // Not valid JSON, ignore
          }
        }
      }
    },

    // PreCompact equivalent: inject reflection context before compaction
    "experimental.session.compacting": async (input, output) => {
      if (!ready) return
      const payload = JSON.stringify({ cwd: directory, trigger: "auto" })
      const result = await runBinary(reflect, payload)
      if (result.trim()) {
        try {
          const parsed = JSON.parse(result.trim())
          if (parsed.additionalContext) {
            output.context.push(parsed.additionalContext)
          }
        } catch {
          // Not valid JSON, ignore
        }
      }
    },
  }
}
