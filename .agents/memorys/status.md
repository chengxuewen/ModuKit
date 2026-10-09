# ModuKit — Status

> Loaded into `instructions[]` on every turn. Keep it short — detail belongs in `decisions.md` / `pitfalls.md`.
> Reset from the ported PolyOrch scaffolding 2026-10-09; original PolyOrch records stay in `PolyOrch/.agents/memorys/`.

## Project

| Field | Value |
|---|---|
| Name | ModuKit |
| Path | `~/Documents/ModuKit` |
| Domain | Modular plugin host framework & polyglot extension toolkit (Rust core; in-process/multi-process plugins, service registry, C ABI bindings, optional UI compositor / SHM / WebRTC) |
| Language / stack | multi-language; `instructions[]` loads rust+typescript+python+web+golang+cpp rules (2026-10-09) |
| Git | no commits yet; config-only tree (`.agents/`, `.opencode/`, `.gitignore`) |

## Phase

Design baseline set 2026-10-09: `docs/whitepaper.md` v1.0 + root `README.md`. No code yet; implementation stage 1 (in-process plugin kernel) not started.

## Modules

| Module | What |
|---|---|
| modukit-core / runtime / compositor / transport | planned — namespace per whitepaper §2.3; no Cargo workspace yet |

## Verification

| Check | Command | Result |
|---|---|---|
| opencode config parses | `python3 -m json.tool .opencode/opencode.json > /dev/null` | pass (2026-10-09) |
| docs exist & README link resolves | `test -f docs/whitepaper.md && grep -q 'docs/whitepaper.md' README.md` | pass (2026-10-09) |
| C1 English-artifacts check | command block in `.agents/memorys/conventions.md` C1 (negative-proved: red on planted CJK, green after) | pass (2026-10-09) |
| memory files carry no ported content | `grep -ricE 'polyorch\|mediaservo' .agents/memorys/` -> hits only in the header port-notes | clean (2026-10-09) |

## Open Items

- [ ] `PolyOrch/` is a local reference checkout, git-ignored; decide whether it stays
- [ ] `rules/common` ported leftovers: `scripts/gate.sh` / `scan-hardcode.sh` refs (no scripts/ dir) and `~/.claude/agents/` list — inline or trim when Stage 1 tooling lands
- [ ] Cargo workspace scaffolding (`modukit-core` et al.) not created
- [ ] License undecided (README says TBD)
- [ ] `doc-audit` skill still targets PolyOrch docs (whitepaper, C1-C6, D1-D29, `docs/modules`); inoperative here until adapted (Plan B item, deferred)

## History

- 2026-10-09: agent/config scaffolding ported from PolyOrch (plan A). PolyOrch-era memory (status / conventions / decisions / pitfalls) reset to templates; `.gitignore` PolyOrch-specific entries dropped (`PolyOrch` ignore line kept for the local reference checkout); rules tree and skills otherwise kept as-is.

- 2026-10-09: whitepaper v1.0 landed as `docs/whitepaper.md` (user-provided, kept verbatim); root `README.md` generated from it; status Project/Phase/Modules filled.
- 2026-10-09: C1 recorded (all artifacts English; chat follows the user). README.md and docs/whitepaper.md regenerated in English (whitepaper content unchanged); functional-CJK exceptions in `.agents/skills/**` documented and excluded from the C1 scan.
- 2026-10-09: /init-deep generated `AGENTS.md` (root) + `.agents/AGENTS.md` hierarchy; C1 gate scope extended to cover both; two toothless ported references in rules/common recorded as open item.
- 2026-10-09: root `SKILL.md` skills registry added (pattern ported from `.refinfo/MediaServo/SKILL.md`, English per C1; content reflects ModuKit's actual 10 project skills — MediaServo-only entries like openspec-propose/explore/test-harness deliberately absent and noted). C1 gate scope extended to SKILL.md in both gate copies.
- 2026-10-09: plugin audit — superpowers/ponytail/context-mode already latest; upgraded oh-my-openagent 4.19.4 -> 5.1.27 in opencode.json (5.x fixes unbounded auto-continuation and opaque child-task errors observed this session). Pending: session restart to load; omo.jsonc schema recheck after restart.
- 2026-10-09: design doc `docs/codegraph-bootstrap.md` written + reviewed inline (3 perspectives, time-boxed; parallel team mode cancelled by user). Verdict: POSIX .sh stays the only repo launcher; the v1 .cjs cross-platform proposal was WITHDRAWN (win32 launcher is a codegraph.cmd shim, install.ps1:145; opencode guarantees neither node nor sh on native win32). Windows path = WSL or per-user global-config .ps1 twin, trigger-gated. Implementation triggers recorded in doc section 8.
- 2026-10-09: win32 discussion overturned doc v2's global-config remedy (R-6: project config overrides global, user caught it); implemented `.opencode/init-mcp-codegraph.bat` twin (cmd+powershell, pinned v1.6.2, CRLF-verified; install.ps1 has no CODEGRAPH_BIN_DIR, shim fixed at <INSTALL_DIR>\current\bin\codegraph.cmd). WSL now optional, not required. Doc at REVIEWED v3; first real-hardware run pending.
- 2026-10-09: LSP root-cause cleanup (user-approved): replaced 3 custom `lsp` wrapper entries with product built-ins `"lsp": true` (panel error was node-v10 ESM crash in init-lsp-wrap.mjs on every .md open via the self-invented remark entry); deleted dead `init-lsp-wrap.mjs` (`init-mcp-openspace.mjs` was already absent at cleanup time — recorded honestly); rust LSP trigger = Stage 1 rustup component; remark dropped (not built-in; CLI-first per opencode best practices). Bootstrap doc section 2 gained the MCP-vs-LSP asymmetry note.
- 2026-10-09: mise installed user-level (2026.10.7), `mise.toml` pins node 22.23.3 (system node v10 untouched; shims beat it via ~/.bashrc activation). Fixes OMO's bundled `lsp` MCP (-32000 was lsp-tools-mcp needing node>=20, unrelated to our own local-codegraph). C1 gate scope extended to mise.toml in both copies. Pending: restart opencode from an activated shell to light the lsp row; Stage-1 rust-analyzer trigger unchanged.
- 2026-10-09: bootstrap twins added (option C, user-approved): `bootstrap.sh` (live-verified idempotent on this box) + `bootstrap.bat` (GitHub-zip mise, .NET User-PATH API — no winget id gamble, no setx 1024 truncation; unverified until first real win run). mise itself deliberately NOT auto-installed per opencode session (PATH prerequisite, no config hook; silent rc-edit plugin rejected as invasive). README gained the Development status section; AGENTS.md structure/where-to-look/commands + both C1 gate copies synced.
