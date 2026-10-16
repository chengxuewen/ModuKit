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

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-10-16: D1-D80 (D53 reserved) in `docs/architecture.md` + `docs/modules/00-11`, all walked one by one; queue EMPTY, remainder trigger-gated. **STAGE-1 COMPLETE 2026-10-16**: M1-M8 + M8.5 lanes + M8.6 shelves all landed (108 tests; demo PASS falsifiable; coverage 78.6% vs 60 soft; ctest 6/6; shelves: crates/ engine, test-plugins/ fixtures, examples/ demos, plugins/ reserved - rulings F+B) - per-milestone evidence in `docs/stage-1-plan.md`, narrative in history-archive. C2 (corrected): ModuKit lands directly on master; PolyOrch upstream = dev-modukit branch; PIN POLICY: gitlink tracks the dev-modukit tip, fresh standalone bumps (pin `5945f2a` = CMP0174 guards, first full bless cycle merge->main-tests->push->bump; IDE pinned to pixi cmake via .vscode, PIT-7). **STAGE-2 KICKOFF ADJUDICATED 2026-10-16**: plan of record `docs/stage-2-plan.md` (Momus OKAY; call sheet A1-A7 user-adjudicated one by one -> D74-D80); execution gate CLOSED until maintainer go. Kernel v0 parked.

## Modules

| Module | What |
|---|---|
| modukit-core / runtime / compositor / transport | planned — namespace per whitepaper §2.3; no Cargo workspace yet |

## Verification

| Check | Command | Result |
|---|---|---|
| opencode config parses | `python3 -m json.tool .opencode/opencode.json > /dev/null` | pass (2026-10-09) |
| docs exist & README link resolves | `test -f docs/whitepaper.md && grep -q 'docs/whitepaper.md' README.md` | pass (2026-10-09) |
| C1 English-artifacts check | command block in conventions.md C1 (scope grown with artifact classes: +M1 build files, +contracts/, +examples/ - each extension red/green proved) | pass (2026-10-16) |
| skill tree lint | `python3 scripts/skill-lint.py` (negative proof: fixture emits 3 FAIL, exit 1) | pass (2026-10-09) |
| numbering discipline | `grep -rnE '^## (C[4-9][0-9]?|D(8[1-9]|9[0-9]|[0-9]{3,}))' .agents/memorys/` must be empty — allows D1-D80 / C0-C3 (widened 2026-10-16 when D74-D80 minted; negative proof same day: planted `## D81:` -> red, removed -> green) | pass (2026-10-16) |
| local check gate (D50) | `bash scripts/check.sh` - fast checks, cargo legs pixi-routed per D65, skip-when-absent | pass (2026-10-15) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md` 2026-10-14; red observed pre-move at 47924 bytes, green at 4144 | pass (2026-10-14) |
| M1-M8.6 milestone gates | check.sh + demo-stage1.sh PASS + 108 tests + clippy all-targets + coverage 78.6% + cmake 7 edges + ctest 6/6 + codemodel FOLDER exact; first-reds & falsifiability probes per plan row; orchestrator re-ran independently | pass (2026-10-16) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15: HEAD on a tmp branch -> RED, on master -> GREEN) | pass (2026-10-15) |

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D80 in `.agents/memorys/decisions.md`); Stage-1 plan of record `docs/stage-1-plan.md` complete; C2 branch discipline governs the PolyOrch submodule upstream only
- [ ] Agent-system track (plugins/config/skills/MCP ONLY — do not mix Stage-1): see `.agents/memorys/backlog.md`; next: P2 PAT (user) + R restart verification (now also carries 2026-10-14 plugin bumps) + S2 user-side apiKey rotation; P3/P4/P5 done; ecosystem-scan FULL rerun 2026-10-14 — fix batches 1-3 adjudicated and landed (see history-archive.md later entry); residual Stage-1-gated items listed there

- [x] PolyOrch submodule live since M1; pin chain 3cbc155..5945f2a (five standalone bumps), policy = follow dev-modukit tip; first full C2 bless cycle closed 2026-10-16 (see Phase + history)
- [ ] Stage-2 kickoff closed 2026-10-16: A1-A7 adjudicated one by one (D74-D80), verdicts in `docs/stage-2-plan.md`; S2-M1+M2+M3(a,b) LANDED 2026-10-16 (133 tests; D17 test-2 closed via wire dial; D80 deferral; bus clause ESCALATED to D77 hearing as IOU-5); S2-M4 awaits A3-gated go; push decision open (ahead 12)
- [ ] Media-server base doctrine landed (D68-D73, 2026-10-15; card `docs/modules/11-media-server-base.md`); MediaServo integration = declared intent only — wiring hearing when kernel API stabilizes (D73)
- [ ] `rules/common` dangling refs family remainder (full list in history 2026-10-14/15 entries; resolved branches struck) — inline or trim when Stage-2 tooling lands
- [x] Cargo workspace landed 2026-10-15 (M2; kernel filled from M3 on)
- [x] License decided — MIT OR Apache-2.0 dual (D1, 2026-10-09); texts at repo root; workspace metadata = Stage-1 trigger
- [x] doc-audit re-scoped to ModuKit canon (commit 2aa7663); first full run 2026-10-10: 18 findings, all user-adjudicated
- [ ] Maintainer calls pending (docs/reference/00-overview.md §4, among others): adopt tribunal PORT_NOW discipline bundle (check.sh seed landed per D50 2026-10-13; doc-liveness + D-record mechanics pending) when Stage-1 tooling lands. Struck at 2026-10-15 doc-audit: A6 wording RATIFIED as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
