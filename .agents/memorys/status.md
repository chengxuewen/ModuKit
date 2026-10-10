# ModuKit — Status

> Loaded into `instructions[]` on every turn. Keep it short — detail belongs in `decisions.md` / `pitfalls.md`.
> Reset from the ported PolyOrch scaffolding 2026-10-09; original PolyOrch records stay in `PolyOrch/.agents/memorys/`.

## Project

| Field | Value |
|---|---|
| Name | ModuKit |
| Path | `~/Documents/ModuKit` |
| Domain | Modular plugin host framework & polyglot extension toolkit (Rust core; in-process/multi-process plugins, service registry, C ABI bindings, optional UI compositor / SHM / WebRTC) |
| Language / stack | multi-language target; `instructions[]` = common + rust only (G10 slim 2026-10-09; ts/python/web/golang/cpp unmounted) |
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
| skill tree lint | `python3 scripts/skill-lint.py` (negative proof: fixture emits 3 FAIL, exit 1) | pass (2026-10-09) |
| memory files carry no ported content | `grep -ricE 'polyorch\|mediaservo' .agents/memorys/` -> hits only in the header port-notes | clean (2026-10-09) |

## Open Items

- [ ] Agent-system track (plugins/config/skills/MCP ONLY — do not mix Stage-1): see `.agents/memorys/backlog.md`; next: doc-audit re-scope, 4 user decisions (github-MCP/arxiv-MCP/openspec-*/affaan-m), restart verification

- [ ] `PolyOrch/` is a local reference checkout, git-ignored; decide whether it stays
- [ ] `rules/common` ported leftovers: `scripts/gate.sh` / `scan-hardcode.sh` refs (no scripts/ dir) and `~/.claude/agents/` list — inline or trim when Stage 1 tooling lands
- [ ] Cargo workspace scaffolding (`modukit-core` et al.) not created
- [x] License decided — MIT OR Apache-2.0 dual (D1, 2026-10-09); texts at repo root; workspace metadata = Stage-1 trigger
- [ ] `doc-audit` skill still targets PolyOrch docs (whitepaper, C1-C6, D1-D29, `docs/modules`); inoperative here until adapted (Plan B item, deferred)
- [ ] Maintainer calls pending (docs/reference/00-overview.md §4): §5 iceoryx2/Zenoh fork wording; ratify A6 generated-only binding wording as D-record; adopt tribunal PORT_NOW discipline bundle (gate.sh seed + doc-liveness + D-record mechanics) when Stage-1 tooling lands

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

- 2026-10-09: docs/reference series delivered (user-specified location; earlier docs/reference/visiaengine mis-scope pulled and superseded): 8 external-project profiles + 00-overview decision matrix, 1,280 lines total, C1 green. Method: 4-lane research team (auto-closed) -> hyperplan tribunal (5 adversarial lenses adjudicated a 30-item VisiaEngine port-candidate list; closed early on user pivot; verdicts distilled into visiaengine.md §8 + cross-cutting rules) -> 7-member profile team (fast-model balance deaths recovered via auto-retry + premium-channel substitutes; see PIT-2). Key outputs: Stage-2 fork converges on composition (iceoryx2 local plane + zenoh carrier, decision sheet 00-overview §2, NOT yet a D-entry); I420 confirmed as own design term (smuggled-residue suspicion withdrawn); A6 hand-written-bindings REJECT wording drafted for ratification. Task store proved unwritable mid-run (PIT-3) — files are truth.

- 2026-10-09: remote-round profiles added (user pivot beyond .refinfo): 9 GitHub projects profiled from blobless clones under ~/.cache/modukit-research/ (rutis, zellij, CLAP, uniffi-rs, wasmtime+Extism survey, ipc-channel, smithay, sysplugin-existence-audit, webrtc-rust-state audit) + whitepaper-citation audit (7 named-but-unvendored refs: 1 strong rutis, 1 toy vnrit, 1 misfire rustbridge, 4 unresolvable). docs/reference/ now 20 files (17 profiles + audits + index + 00-overview), all C1-verified; README + 00-overview carry both rounds, Stage-2 fork sheet gained a third option (typed control plane via ipc-channel shape) and Stage-4 decision axis (sans-I/O dual vs libwebrtc-FFI vs vendor; getstream-rtc = naming+category error). Team modukit-ref-remote survived a premium-family Bad Gateway storm via 5 recovery substitutes reusing on-disk clones (PIT-2 recovery recipe held: files over reports, never trust member completion notices); all teams closed and deleted. New maintainer canon questions queued (00-overview §4): header-first vs metadata-first contract (uniffi finding), whitepaper reference corrections list, Stage-4 engine tiering.

- 2026-10-09: license decided (user "landed" approval) — D1 recorded: MIT OR Apache-2.0 dual; LICENSE-MIT + LICENSE-APACHE at root (Apache text from family-standard full text, 202 lines verified); README License section + status open item closed; both C1 gate copies extended to scan the license files; DCO required from first external contribution (relicensing-space protection). Cargo workspace `license` metadata = recorded Stage-1 trigger.

- 2026-10-09: ecosystem-scan (Full, first run): 15-item local audit + 2-way external scan. Landed same session — R1 AGENTS.md C1 gate copy drift fixed (was missing crates/ dynamic scope); R2 openspec-* x3 flagged inoperative (CLI absent); R4 G10 executed (ts/python/web/golang/cpp rule files unmounted: instructions 26->16; 7 never-mounted dirs (csharp/dart/java/kotlin/perl/php/swift) initially deleted then RESTORED 35/35 per maintainer veto — rule files stay on disk, mount-only slimming); plugins pinned (ponytail@5.1.0, context-mode@1.0.169, omo 5.1.27->5.1.28; superpowers git-ref left floating by design); PIT-4 recorded. External verdicts: ADR-MCP class dead, rust-analyzer-MCP dup with built-in LSP, serena/playwright-MCP no-value-at-stage, npm spdx/osv trio provenance smell — all REJECT; github-MCP / arxiv-MCP / skill-lint+evals pattern = conditional P2 pending; Rust FFI/C-ABI skill = community gap (nothing to adopt, will be ours to write). Uncovered honestly: affaan-m/everything-claude-code timed out x3, not profiled.

- 2026-10-09: A bundle committed (`2909081`). Backlog planning doc added at `.agents/memorys/backlog.md` (opencode-plugin/config/skill/MCP track only; Stage-1 is a separate track resumable on explicit instruction; rules-asset "mount, never delete" ruling recorded). B landed same session: `scripts/skill-lint.py` (7 checks over .agents/skills + registry sync), C0 negative-proved via /tmp fixture (3 FAIL, exit 1) then repo green (10 skills, 0 fail); C1 gate scope extended to scripts/ in both copies; skill-lint registered in AGENTS.md COMMANDS.

- 2026-10-09: D1 landed (user invoked /doc-audit right after plan doc): doc-audit SKILL.md re-scoped to ModuKit canon — 18 anchor replacements (decision/consistency/agent/gap/phase dimensions re-targeted to whitepaper + docs/reference series + gate copies; PolyOrch xmake/C2-C6/D2-D29 targets removed; skill-lint wired as the mechanical first pass). Rescope process itself surfaced+fixed two stale claims in AGENTS.md (NOTES "License TBD" post-D1; header "Commit: none yet" post-commits) — exactly the finding-class dimension 4 hunts. C1 + skill-lint green on all edits.
