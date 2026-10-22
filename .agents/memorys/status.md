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

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-10-21: D1-D99 (D53 reserved) in `docs/architecture.md` + `docs/modules/00-11`, all walked one by one; queue EMPTY, remainder trigger-gated. **STAGE-1 COMPLETE 2026-10-16**: M1-M8 + M8.5 lanes + M8.6 shelves all landed (then: 108 tests; demo PASS falsifiable; coverage 78.6% vs 60 soft; ctest 6/6; shelves: crates/ engine, test-plugins/ fixtures, examples/ demos, plugins/ reserved - rulings F+B) - per-milestone evidence in `docs/stage-1-plan.md`, narrative in history-archive. C2 + PIN POLICY live in conventions.md (pin `5945f2a`; IDE pixi-cmake via .vscode, PIT-7). **STAGE-2 ACCEPTED AND CLOSED 2026-10-18**; **STAGE-3 (stability/docs re-scope) ACCEPTED 2026-10-19 (D91)**: plan `docs/stage-2-plan.md` (Momus OKAY; kickoff A1-A7 -> D74-D81; M1-M5 landed incl. wire dial, vendor-shm, modukit-host, dual demo + ORPHAN-PROOF); freeze hearing 8 rulings -> D82-D88: **v1 surfaces STABLE (D87)**, v2 bus/clock verbs PLANNED (D82), dictionary 14->19 + hint() paid (D85/D86), coverage debt calendared to the Stage-3 kickoff gate (D84), ABI amendment policy live (D83). **STAGE-3 CLOSED 2026-10-19 (D91)**; **STAGE-4 INFRA volume: C/C++ bindings M1+M2 CLOSED 2026-10-20, ruled in as D93; volume ACCEPTED AND CLOSED 2026-10-21 (D95)** — plan `docs/stage-4-plan.md`; wave-2 OPENED 2026-10-21 (D94, rulings A/A/A): **M3 sup service-call door LANDED same day** - cpp x exe fidelity now gated (FIDELITY stanza + sabotage hook); §12 amendment APPLIED at D95. Wave-2 since: v2 bus LANDED (D96 N1+N2) + debug-reader MINIMAL FACE LANDED (D97) + CLOSEOUT LANDED (D98) + CLOCK HEARING DRAWN (D99: v3 door / two verbs / bell-on-bus / wall asymmetry named; lanes spec'd, not open). Kernel v0 parked.

## Modules

| Module | What |
|---|---|
| modukit-core / -runtime / -transport / -codegen | SHIPPED (workspace since 2026-10-15; test totals live in the plans of record — volatile counts unlisted per the 2026-10-10 audit; v1 frozen D87) |
| bindings/cpp + contracts C originals | SHIPPED (Stage-4 M1/M2, D93+M3: HostEmbedV1 24 verbs two rungs, C++ emitters + fixtures, ORPHAN-PROOF + byte-fidelity demos; volume CLOSED (D95)) |
| modukit-compositor | planned — §12-3 demoted track; UI demand trigger stands (D89-K2) |

## Verification

| Check | Command | Result |
|---|---|---|
| opencode config parses | `python3 -m json.tool .opencode/opencode.json > /dev/null` | pass (2026-10-09) |
| docs exist & README link resolves | `test -f docs/whitepaper.md && grep -q 'docs/whitepaper.md' README.md` | pass (2026-10-09) |
| C1 English-artifacts check | command block in conventions.md C1 (scope grown with artifact classes: +M1 build files, +contracts/, +examples/ - each extension red/green proved) | pass (2026-10-16) |
| skill tree lint | `python3 scripts/skill-lint.py` (negative proof: fixture emits 3 FAIL, exit 1) | pass (2026-10-09) |
| numbering discipline | `grep -rnE '^## (C[4-9][0-9]?|D[0-9]{3,})' .agents/memorys/` must be empty — allows D1-D99 / C0-C3 (widened same-day for each mint D93-D99, red observed each time) | pass (2026-10-21) |
| local check gate (D50) | `bash scripts/check.sh` - fast checks incl SEVEN demo legs (wave2-reader + T2 clock red-proved), pixi-routed per D65 | pass (2026-10-22) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md` 2026-10-14; red observed pre-move at 47924 bytes, green at 4144 | pass (2026-10-14) |
| M1-M8.6 milestone gates | per `docs/stage-1-plan.md` evidence rows; orchestrator re-ran independently | pass (2026-10-16) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15: HEAD on a tmp branch -> RED, on master -> GREEN) | pass (2026-10-15) |
| doc-audit dated-gate re-run | cheap verification rows re-run 2026-10-21 (config/whitelink/C1/skill-lint/numbering/size) | all pass (2026-10-21) |
| gate escalations live | check.sh ctest-stage4 leg (incremental rebuild THEN ctest; planted red observed); scripts/hooks/commit-msg via core.hooksPath (2 bad msgs blocked, good passed) | live (2026-10-21) |

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D99 in `.agents/memorys/decisions.md`)
- [ ] Agent-system track (plugins/config/skills/MCP ONLY — do not mix Stage-1): see `.agents/memorys/backlog.md`; next: P2 PAT (user) + R restart verification (+2026-10-14 bumps) + S2 user-side apiKey rotation; P3/P4/P5 done; ecosystem-scan FULL rerun 2026-10-14 — fix batches 1-3 adjudicated and landed (see history-archive.md later entry); residual Stage-1-gated items listed there

- [x] PolyOrch submodule live since M1; pin chain 3cbc155..5945f2a (five standalone bumps), policy = follow dev-modukit tip; first full C2 bless cycle closed 2026-10-16 (see Phase + history)
- [x] Stage-2 COMPLETE 2026-10-18: M1-M5 + freeze hearing (D82-D88); v1 STABLE per D87, v2 PLANNED, dictionary 19 names, hint() paid, coverage debt calendared (D84); Stage-3 scoping opens on maintainer word; push decision open
- [ ] K7 follow-through (OWNER: MAINTAINER): send the v1-STABLE note (draft: docs/reference/k7-v1-stable-note-draft.md); proposal stays ARMED (D89-K7)
- [x] Stage-3 COMPLETE 2026-10-19: K1-K7 (D89), freeze hearing (D82-D88), debt paid + hook lifted (D90), lanes closed, acceptance D91; UI tier stays demoted per K2; push decision remains maintainer-owned
- [x] Stage-4 volume ACCEPTED 2026-10-21 (D95): M1-M3 + coverage boundary run + §12 applied + wave-2 door landed; push tail + K7 remain maintainer calls
- [x] Wave-2 v2 bus: HEARING (D96 six-by-A) + **N1 door + N2 crossing both LANDED 2026-10-21** (five demo legs, wall transparent in the start window): docs/stage-5-plan.md
- [ ] Still armed: Python / live-attach depth / JS; debug-reader face
  LANDED 2026-10-21; promoted to check.sh leg + core debt cleared (D98).
  CLOCK drawn 2026-10-21 (D99 B,A,A,A,A,A); T1/T2 await go order
- [ ] Media-server base doctrine landed (D68-D73, 2026-10-15; card `docs/modules/11-media-server-base.md`); MediaServo integration = declared intent only — K7=C (D89): v1-STABLE factual note pending maintainer send; wiring-hearing proposal ARMED, not opened
- [x] rules/common dangling-refs family CLOSED 2026-10-19 (M3S-1 sweep; doc-liveness leg in check.sh; unmounted pool kept per G10)
- [x] Cargo workspace landed 2026-10-15 (M2; kernel filled from M3 on)
- [x] License decided — MIT OR Apache-2.0 dual (D1, 2026-10-09); texts at repo root; workspace metadata = Stage-1 trigger
- [x] doc-audit re-scoped to ModuKit canon (commit 2aa7663); first full run 2026-10-10: 18 findings, all user-adjudicated
- [x] PORT_NOW bundle FULLY retired 2026-10-19 (seed + D-record + doc-liveness all in gates)
- [ ] Maintainer calls pending (docs/reference/00-overview.md §4). Struck at 2026-10-15 doc-audit: A6 wording RATIFIED as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
