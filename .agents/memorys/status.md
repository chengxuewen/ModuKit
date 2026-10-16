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
| Git | master; history via `git log` (volatile counts deliberately not recorded here — 2026-10-10 audit fixed the rot class); origin=gitee, push only on explicit instruction |

## Phase

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-10-15: decisions D1-D73 (D53 reserved) in `docs/architecture.md` + `docs/modules/00-11`, walked one by one (build-orchestration batch D64-D67: PolyOrch adopted, votes A/A/A/A; media-base batch D68-D73: control-plane P1 + frame clause, votes A/A''/C/C/C/C, integration stays unwired by choice); adjudication queue EMPTY — everything left is trigger-gated. **Stage-1 RESUMED 2026-10-15 by maintainer order**: M1 build trio + M2 cargo scaffold + M3 kernel vertical slice + M4 contract codegen v0 + M5 bus + seam + **M6 observability floors landed** (clock/events/logger/edge-ledger/counters/introspect-trio/testhost; 88 tests; ambiguities adjudicated in plan rows); all ModuKit work lands directly on master per the corrected C2 (branch discipline governs the PolyOrch upstream only: develop on dev-modukit -> merge -> test on main); PIN POLICY since 2026-10-15: the gitlink follows the dev-modukit TIP, kept fresh via standalone bump commits - pin now a620933. Plan of record: `docs/stage-1-plan.md`. M7 (bundle install protocol) next. Kernel v0 experiment parked (backup outside repo).

## Modules

| Module | What |
|---|---|
| modukit-core / runtime / compositor / transport | planned — namespace per whitepaper §2.3; no Cargo workspace yet |

## Verification

| Check | Command | Result |
|---|---|---|
| opencode config parses | `python3 -m json.tool .opencode/opencode.json > /dev/null` | pass (2026-10-09) |
| docs exist & README link resolves | `test -f docs/whitepaper.md && grep -q 'docs/whitepaper.md' README.md` | pass (2026-10-09) |
| C1 English-artifacts check | command block in `.agents/memorys/conventions.md` C1 (negative-proved: red on planted CJK, green after; scope extended 2026-10-15 with M1 artifact classes pixi.toml/CMakeLists.txt/cmake/ - new scope proven red on planted probe, green after removal) | pass (2026-10-15) |
| skill tree lint | `python3 scripts/skill-lint.py` (negative proof: fixture emits 3 FAIL, exit 1) | pass (2026-10-09) |
| numbering discipline (ported ids absent) | `grep -rnE '^## (C[4-9][0-9]?|D(7[4-9]|[89][0-9]|[0-9]{3,}))' .agents/memorys/` must be empty — allows modukit-native high-water D1-D73 / C0-C3 (2026-10-15; C boundary moved with the C2 then C3 entries, D boundary with the D68-D73 batch), flags PolyOrch-era shapes; negative proofs observed: piped `## D74:` and `## C4:` -> red, `## C3:` passes, real tree -> green | pass (2026-10-15) |
| local check gate (D50) | `bash scripts/check.sh` - five fast checks; cargo legs route through pixi per D65 (first real cargo run 2026-10-15: observed RED on the empty mise shim -> GREEN after pixi routing; skip-when-absent kept) | pass (2026-10-15) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md` 2026-10-14; red observed pre-move at 47924 bytes, green at 4144 | pass (2026-10-14) |
| M1-M5 milestone gates green | `bash scripts/check.sh` (6 checks) + `pixi run cargo test --workspace` (88 tests incl. M6 floors) + clippy -D warnings all-targets; per-milestone first-red evidence lives in `docs/stage-1-plan.md` rows + feat commit messages; orchestrator re-ran all gates independently after each executor agent (2026-10-15/16) | pass (2026-10-16) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15: HEAD on a tmp branch -> RED, on master -> GREEN) | pass (2026-10-15) |

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D73 in `.agents/memorys/decisions.md`); Stage-1 in resumption since 2026-10-15 per plan of record `docs/stage-1-plan.md` (work lands directly on master; C2 branch discipline governs the PolyOrch submodule upstream only)
- [ ] Agent-system track (plugins/config/skills/MCP ONLY — do not mix Stage-1): see `.agents/memorys/backlog.md`; next: P2 PAT (user) + R restart verification (now also carries 2026-10-14 plugin bumps) + S2 user-side apiKey rotation; P3/P4/P5 done; ecosystem-scan FULL rerun 2026-10-14 — fix batches 1-3 adjudicated and landed (see history-archive.md later entry); residual Stage-1-gated items listed there

- [x] PolyOrch submodule landed 2026-10-15 (M1 per D64-D67; gitlink bumped same day 3cbc155 -> 76490a5 - maintainer pulled PolyOrch main and created the `dev-modukit` upstream branch, both at 76490a5 and pushed; no branch= field in .gitmodules; root reference checkout retired into the submodule, `.gitignore` entry lifted with an out/ build-root rule in its place)
- [ ] Media-server base doctrine landed (D68-D73, 2026-10-15; card `docs/modules/11-media-server-base.md`); MediaServo integration = declared intent only — wiring hearing when kernel API stabilizes (D73)
- [ ] `rules/common` dangling refs family remainder: `~/.claude/agents/`, `~/.claude/settings.json` / `~/.claude.json`, and `See skill: python-patterns` / `golang-testing` pointers — inline or trim when Stage 1 tooling lands (rust-patterns half RESOLVED 2026-10-14 by vendoring ECC rust-patterns; the family's gate.sh/scan-hardcode.sh entries retired 2026-10-13: scripts/check.sh landed per D50). Added by 2026-10-15 doc-audit: `rules/rust/hooks.md` ~/.claude/settings.json mention + `rules/rust/security.md` "See skill: security-review" pointer
- [x] Cargo workspace scaffolding landed 2026-10-15 (M2 on branch stage1/m2-cargo-scaffold: workspace resolver 3 / edition 2024, modukit-core + example/bad fixture stubs, Cargo.lock committed per constraints.md, CMake version single-source wired with observed red->green) — kernel content starts at M3
- [x] License decided — MIT OR Apache-2.0 dual (D1, 2026-10-09); texts at repo root; workspace metadata = Stage-1 trigger
- [x] doc-audit re-scoped to ModuKit canon (commit 2aa7663); first full run 2026-10-10: 18 findings, all user-adjudicated
- [ ] Maintainer calls pending (docs/reference/00-overview.md §4, among others): adopt tribunal PORT_NOW discipline bundle (check.sh seed landed per D50 2026-10-13; doc-liveness + D-record mechanics pending) when Stage-1 tooling lands. Struck at 2026-10-15 doc-audit: A6 wording RATIFIED as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
