# CodeGraph Bootstrap — Design Doc

**Status**: REVIEWED v3 — inline review + win32 twin implemented same-day (see §8 R-6)  
**Date**: 2026-10-09  
**Scope**: auto install → init → MCP serve for CodeGraph, on every opencode session start, cross-platform (Windows / macOS / Linux)  
**Implements**: the `local-codegraph` MCP wiring in `.opencode/opencode.json`

---

## 1. Problem

The repo's original CodeGraph wrapper (`.opencode/init-mcp-codegraph.mjs`, ported) ran under the system `node` — which on this machine is **v10 and dies on ESM syntax**. The MCP therefore never started: the binary was never installed, the project graph never built, and every session spawn crashed. The user-facing symptom was "I have to start codegraph manually."

## 2. Decision summary

The only correct hook for a serialized `check → install → init → serve` sequence is the MCP **`command` process itself**: opencode spawns it at session start, stdio is the MCP channel, and its exit is the session teardown. No other mechanism guarantees "install finished before serve starts".

- **Launchers**: two repo scripts, one per platform family — POSIX `.opencode/init-mcp-codegraph.sh` (live-tested) and Windows-native `.opencode/init-mcp-codegraph.bat` (cmd.exe + powershell.exe, guaranteed on every Windows; the release bundle vendors its own Node runtime, verified from install.ps1). WSL is optional, not required. opencode's `mcp.command` has no OS-conditional syntax and **project config overrides global config** (documented precedence), so a native-win32 user selects the twin with a LOCAL one-line edit of `opencode.json`: `"command": ["cmd", "/c", ".opencode\\init-mcp-codegraph.bat"]` (keep it out of commits via `git update-index --skip-worktree` or stash hygiene). The v1 `.cjs` single-launcher proposal stays withdrawn (R-1/R-4).

- **Asymmetry note** (2026-10-09, LSP cleanup): the launcher pattern applies to MCP only. LSP servers have first-class product support — `"lsp": true` enables opencode's built-ins including auto-download (official docs, "How It Works" + built-in table) — so the ported custom wrappers (`init-lsp-wrap.mjs`, three hand-rolled entries, and the dead pixi-based `init-mcp-openspace.mjs`) were **deleted, not fixed**; the node-v10 ESM crash class they belonged to cannot recur there. `rust` LSP lights up at Stage 1 via `rustup component add rust-analyzer` (+ `rust-toolchain.toml`, the MediaServo pattern); `remark` was dropped outright (not a built-in; markdown diagnostics go through CLI tools per the docs' best-practices stance).
- **Installer delegation**: never hand-roll download logic. Call the official installer for each platform, with env overrides so everything lands inside the repo:
  - `darwin|linux`: `curl install.sh | CODEGRAPH_VERSION=… CODEGRAPH_INSTALL_DIR=… CODEGRAPH_BIN_DIR=… sh`
  - `win32` (in `.opencode/init-mcp-codegraph.bat`): run `install.ps1` via `powershell.exe -NoProfile -ExecutionPolicy Bypass` with `$env:CODEGRAPH_VERSION` / `$env:CODEGRAPH_INSTALL_DIR` (install.ps1 env contract: only these two — it does NOT support `CODEGRAPH_BIN_DIR`). The launcher it creates is fixed at `<INSTALL_DIR>\current\bin\codegraph.cmd` (install.ps1:130/145) — a batch shim, so the twin invokes it by absolute path with `call`; the installer's user-PATH registry edit is irrelevant to our spawn. CRLF line endings are mandatory for the `.bat` itself.
- **State machine**: idempotent, no network when clean — see the full flow in *Runtime sequence* below.

### Runtime sequence (both launchers share this flow)

```
opencode session start
  spawn mcp.local-codegraph.command      (stdio = the MCP channel; its exit = session teardown)
  launcher = init-mcp-codegraph.sh  [Linux/macOS]  |  init-mcp-codegraph.bat  [Windows native,
                                       selected by a LOCAL one-line project-config edit - see R-6]

  1. CHECK BINARY
       sh :  .opencode/bin/codegraph exists?                        yes -> step 3
       bat :  .opencode\codegraph\current\bin\codegraph.cmd exists?  yes -> step 3
       missing v
  2. ONE-TIME INSTALL (pinned version, ~55 MB, official installer only)
       sh :  curl install.sh  |  sh   with CODEGRAPH_VERSION / INSTALL_DIR / BIN_DIR
       bat :  powershell -NoProfile -ExecutionPolicy Bypass  with
              $env:CODEGRAPH_VERSION + $env:CODEGRAPH_INSTALL_DIR,  irm install.ps1 | iex
       still missing afterwards -> launcher exits non-zero -> opencode marks the MCP failed
       (visible, no silent half-state) -> next session retries
  3. ENSURE PROJECT GRAPH
       .codegraph/ missing -> codegraph init   (empty repo = instant;
       init failure = warn + continue, watcher heals it later)
  4. SERVE
       sh :  exec codegraph serve --mcp
       bat :  call codegraph.cmd serve --mcp
       the serve process embeds the file watcher (2 s debounce incremental sync) and
       connect-time catch-up -> the graph never goes stale; zero manual maintenance
```

### File layout: committed vs per-machine

```
COMMITTED (this repo)                       PER-MACHINE (all gitignored)
|-- .opencode/opencode.json                 |-- .opencode/codegraph/   store: self-contained
|-- .opencode/init-mcp-codegraph.sh         |        CLI bundle (~286 MB unpacked)
|-- .opencode/init-mcp-codegraph.bat        |-- .opencode/bin/         POSIX launcher links
`-- docs/codegraph-bootstrap.md (this)      |-- .codegraph/            project symbol graph
                                            `-- (default ~/.local/bin and %LOCALAPPDATA%
                                                 are redirected in-repo by the env vars;
                                                 ps1 still writes its bin dir into the user
                                                 PATH registry entry - cosmetic here, our
                                                 spawn is always by absolute path)
```

## 3. Platform matrix

| Platform | Path | Status |
|---|---|---|
| Linux | POSIX `sh` launcher | ✅ implemented + live-tested (install 1.6.2 → init → serve verified on this box 2026-10-09) |
| macOS | same branch (`darwin`) | ⚠️ code path identical, unverified on Apple hardware |
| Windows native | `.opencode/init-mcp-codegraph.bat` twin, selected by a LOCAL one-line project-config edit (global config cannot override project — precedence verified) | ✅ written; first real-hardware run pending — closes R-2 |
| Windows via WSL | opencode officially recommends WSL — appears as Linux to the launcher | ✅ by delegation |

## 4. Version & supply-chain policy

- **Pinned**: `CODEGRAPH_VERSION` constant (`v1.6.2` = current latest at design time). Upgrade = one deliberate edit + commit; never floating latest in a repo.
- Release assets are per-OS bundles (50–63 MB, self-contained — no system Node dependency for CodeGraph itself).
- **Gap (accepted, documented)**: official install.sh performs no checksum verification (`grep sha256` = 0). Mitigations, in order of cost: (a) pin + review-the-diff on bump; (b) follow-up: wrapper downloads `SHA256SUMS` (shipped with every release) and verifies before executing the installer. B is a design-approved TODO, not a blocker.
- Repo footprint: `.opencode/codegraph/` (store) + `.opencode/bin/` (links) — both gitignored; `.codegraph/` (project graph) gitignored; nothing outside the repo except the one-time `~/.local/bin` link (POSIX) which we override to stay in-repo.

## 5. Failure semantics & observability

| Failure | Behavior |
|---|---|
| No network at first-ever session | install fails → launcher exits non-zero → opencode marks the MCP as failed for the session (visible, no silent half-state). Next session retries. |
| `init` fails | log warning, still `exec serve` — server runs with empty/stale graph; watcher + connect-time catch-up heal it later. |
| Installer PATH-shadowing | installers ship their own PATH sanity check (`Find-FirstCodegraph` in ps1; PATH walk in sh); our launcher never relies on PATH — it invokes the absolute `.opencode/bin` path. |
| Manual verification | `codegraph status` (CLI) prints pending-sync sections; `ls .opencode/bin && ls .codegraph`. |

## 6. Rejected alternatives

| Option | Why rejected |
|---|---|
| opencode plugin (`session.created`) installs, config declares `command: ["codegraph", …]` | opencode's plugin API (docs fetched 2026-10-09) exposes event/tool/shell/compaction hooks — **no MCP-registration or config-rewrite hook**; and async install racing a sync spawn is undefined behavior on the first session. |
| Keep `.mjs` with ESM | system node v10 cannot parse it (measured crash at line 4). Also rejected for the twin itself: Windows users should not need Node installed — cmd/powershell are guaranteed present. |
| Two MCP entries (`sh` + `powershell`) both enabled | no OS-conditionals in opencode config → guaranteed one failing server per session everywhere. |
| Global install (official `codegraph install --target=opencode`) | writes MCP config + an AGENTS.md section into user/project files — we own those documents; per-machine manual step contradicts "automatic at session start". |
| `bun` as launcher runtime | opencode embeds Bun but does not guarantee a `bun` CLI on PATH. |
| single `.cjs` launcher branching on `process.platform` (v1 proposal, overturned in review) | swaps "`sh` exists" for "node exists" — opencode guarantees neither; win32 spawn targets a `.cmd` shim needing shell-spawn special cases; resolved 2026-10-09 when a real native-win32 user materialized — solved with the .bat twin instead. |

## 7. Current state vs target

| Item | Status |
|---|---|
| `.opencode/init-mcp-codegraph.sh` (POSIX-only) | ✅ on disk, live-verified on Linux |
| opencode.json `mcp.local-codegraph.command` | ✅ wired to the launcher |
| Install artifacts (`v1.6.2`, project-local) | ✅ present, gitignored |
| ~~Target: single .cjs launcher~~ | ❌ withdrawn by review R-1/R-4 |
| `.opencode/init-mcp-codegraph.bat` (win32 twin) | ✅ on disk, CRLF-verified; first run on real hardware pending |
| SHA256 verification (TODO B) | ⏳ approved follow-up |

## 8. Review log

Inline three-perspective review (supply chain / cross-platform / simplicity), 2026-10-09; claims re-verified against live install.sh / install.ps1 / opencode docs.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| R-1 | HIGH | v1 claimed the win32 launcher is `codegraph(.exe)` spawnable cross-platform from a shared script; install.ps1:145 proves it is a `codegraph.cmd` PATH shim needing `cmd /c`, and opencode guarantees neither system node nor `sh` on native win32 | v2 withdraws the `.cjs` design; native-win moves to per-user global-config twin |
| R-2 | MEDIUM | v1 marked win32 "designed from source" while nothing was ever run on Windows; combined with R-1 the native branch rested entirely on unverified assumptions | Downgraded to trigger-gated; supported Windows path = WSL (official) |
| R-3 | LOW | Confirmed: install.sh contains no checksum/verify step (full-text scan for sha/sign/attest empty); `curl \| sh` at first session remains an accepted, pinned, documented risk; SHA256SUMS ships with releases and is usable | TODO B kept (verify SHA256SUMS before exec) — not a blocker |
| R-4 | MEDIUM | Repo has ZERO source code; any launcher redesign is speculative infrastructure. The live-tested POSIX `.sh` already serves every actual user today | `.cjs` delta cancelled; §2/§6/§7 rewritten to "implement on trigger" |
| R-5 | LOW | §6 plugin-race rejection holds: opencode plugin docs (fetched 2026-10-09) expose event/tool/shell.env/compaction/custom-tool hooks only — no MCP-registration or config-rewrite hook | No change |
| R-6 | HIGH (v2 self-error, caught by user) | v2's win32 remedy said "override in global config" — **wrong direction**: opencode precedence is project-over-global, so a global override is silently beaten by the project's `sh` command. Corrected: local edit of the project file (+skip-worktree), plus the `.bat` twin implemented now that a real win32 user exists; install.ps1 lacks `CODEGRAPH_BIN_DIR` and pins its shim at `<INSTALL_DIR>\current\bin\codegraph.cmd` (130/145) — twin accounts for both | Fixed in v3 |

**Implementation triggers (remaining work):**
1. ~~native-win32 twin~~ DONE same-day: `.bat` written (R-6 below); awaiting first real-hardware session to close R-2
2. Stage 1 crates land → first real `codegraph init` index; confirm MCP tools return graph data
3. follow-up B → SHA256SUMS verification in the launcher install path
