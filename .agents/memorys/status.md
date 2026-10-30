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

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-10-29: D1-D109 (D53 reserved) in `docs/architecture.md` + `docs/modules/00-11`; queue EMPTY, remainder trigger-gated. Stages 1-3 COMPLETE/ACCEPTED (D91; v1 STABLE D87; amendment D83); Stage-4 INFRA volume ACCEPTED/CLOSED (D95); wave-2 volumes ACCEPTED through D107; wave-3 parent-push hearing D108; VE-parity plan D109 (B0-B7 landed, B8 IN EXECUTION, revision 13; gap-closure #20-22, VE-new #23-26; host plan rev 7: H0a-H3 landed, H2b full 41, H4 closeout open). Full narrative: `history-archive.md` (2026-10-30 entry). Kernel v0 parked.

## Modules

| Module | What |
|---|---|
| modukit-core / -runtime / -transport / -codegen / -js | SHIPPED (workspace since 2026-10-15; test totals live in the plans of record — volatile counts unlisted per the 2026-10-10 audit; v1 frozen D87) |
| bindings/cpp + bindings/py + contracts C originals | SHIPPED (Stage-4 D93/D95 + python ctypes floor D100/D101) |
| modukit-compositor | planned — §12-3 demoted track; UI demand trigger stands (D89-K2) |

## Verification

| Check | Command | Result |
|---|---|---|
| cheap verifications | re-run by the doc-audit lead inline | pass (2026-10-23) |
| C1 English-artifacts check | command block in conventions.md C1 (scope grown with artifact classes: +M1 build files, +contracts/, +examples/, +bindings/ - each extension red/green proved) | pass (2026-10-16) |
| skill tree lint | `python3 scripts/skill-lint.py` (negative proof: fixture emits 3 FAIL, exit 1) | pass (2026-10-09) |
| numbering discipline | `bash scripts/check-numbering.sh` — derives the max D from decisions.md and asserts every D1-D<n> claim plus the AGENTS high-water matches; `--self-test` plants D110 and expects red | pass (2026-10-29; self-test red observed) |
| memory disposition (C5) | `bash scripts/check-memory.sh <base>..HEAD` (batch close only) | pass (2026-10-30; B7 red, B2 green observed) |
| local check gate (D50) | `bash scripts/check.sh` - 36 legs (13 demo + D104 + gate-rules/spec-trace), full list via `--list`, pixi per D65 | pass (2026-10-30) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md` 2026-10-14; red observed pre-move at 47924 bytes, green at 4144 | pass (2026-10-14) |
| gitignore hygiene | `git ls-files -ci --exclude-standard` empty; expected `git check-ignore` hits for out/build/target/agent layers | pass (2026-10-30; VE ownership order) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15: HEAD on a tmp branch -> RED, on master -> GREEN) | pass (2026-10-15) |
| doc-audit full | 2026-10-23: 45 raw -> 26 merged, swept (git-date/bat-hooks/C1-red/D83) | this entry IS the record |
| gate escalations live | check.sh ctest-stage4 leg (incremental rebuild THEN ctest; planted red observed); scripts/hooks/commit-msg via core.hooksPath (2 bad msgs blocked, good passed) | live (2026-10-21) |

## Repo Audit

- 2026-10-30 Quick (post-B8): 10/10 local checks; scripts syntax clean, no
  case/EOL findings; 36 gates, 41 skills, book 74. One MEDIUM fixed: www/book/en
  was stale in `modukit.sh`/`.bat` and `pixi.toml` (actual `build/book/en`);
  `gate-docs` 3c now asserts the front-door claim. B8 layer
  generated (18 rules / 41 skills); H3 restart probe pending. Full 2026-10-30
  record: HIGH fixed (shipping-pack-check, test-harness, README); gate-mcp O5 WARN.

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D109 in `.agents/memorys/decisions.md`)
- [ ] Agent-system track (plugins/config/skills/MCP ONLY — do not mix Stage-1): see `.agents/memorys/backlog.md`; next: P2 PAT (user) + R restart verification (+2026-10-14 bumps) + S2 user-side apiKey rotation; P3/P4/P5 done; ecosystem-scan FULL rerun 2026-10-14 — fix batches 1-3 adjudicated and landed (see history-archive.md later entry); residual Stage-1-gated items listed there

- [x] PolyOrch submodule live since M1; pin chain 3cbc155..f8f85db (seven standalone bumps), policy = follow dev-modukit tip; C2 bless cycles: first 2026-10-16, second 2026-10-23 (dev-into-main at f8f85db) (see Phase + history)
- [x] Stage-2 COMPLETE 2026-10-18: M1-M5 + freeze hearing (D82-D88); v1 STABLE per D87, v2 PLANNED, dictionary 19 names, hint() paid, coverage debt calendared (D84); Stage-3 scoping opens on maintainer word; push decision open
- [ ] K7 follow-through (OWNER: MAINTAINER): send the v1-STABLE note (draft: docs/reference/k7-v1-stable-note-draft.md); proposal stays ARMED (D89-K7)
- [x] Stage-3 COMPLETE 2026-10-19: K1-K7 (D89), freeze hearing (D82-D88), debt paid + hook lifted (D90), lanes closed, acceptance D91; UI tier stays demoted per K2; push decision remains maintainer-owned
- [x] Stage-4 volume ACCEPTED 2026-10-21 (D95): M1-M3 + coverage boundary run + §12 applied + wave-2 door landed; push tail + K7 remain maintainer calls
- [x] Wave-2 v2 bus: HEARING (D96 six-by-A) + **N1 door + N2 crossing both LANDED 2026-10-21** (five demo legs, wall transparent in the start window): docs/stage-5-plan.md
- [x] Clock lanes LANDED (T1+T2 2026-10-22, D99); debug-reader face
  LANDED + check.sh leg + core debt cleared (D97/D98). PYTHON VOLUME ACCEPTED
  2026-10-23 (D101; boundary green via pinned recipe). JS VOLUME LANDED 2026-10-23
  ACCEPTED (D102/D103; eleven-leg green; js region bin-like-exempt, proofs node-side).
  Benchmark volume ACCEPTED (D105); plugin-door volume ACCEPTED (D107). Live-attach armed.
- [ ] Media-server base doctrine landed (D68-D73, 2026-10-15; card `docs/modules/11-media-server-base.md`); MediaServo integration = declared intent only — K7=C (D89): v1-STABLE factual note pending maintainer send; wiring-hearing proposal ARMED, not opened
- [x] rules/common dangling-refs family CLOSED 2026-10-19 (M3S-1 sweep; doc-liveness leg in check.sh; unmounted pool kept per G10)
- [x] Cargo workspace landed 2026-10-15 (M2; kernel filled from M3 on)
- [x] License decided — MIT OR Apache-2.0 dual (D1, 2026-10-09); texts at repo root; workspace metadata = Stage-1 trigger
- [x] doc-audit re-scoped to ModuKit canon (commit 2aa7663); first full run 2026-10-10: 18 findings, all user-adjudicated
- [x] PORT_NOW bundle FULLY retired 2026-10-19 (seed + D-record + doc-liveness all in gates)
- [x] Benchmark volume ACCEPTED (D105) after a fresh boundary green + same-day doc-audit sweep.
- [ ] Benchmark volume: plan §7 debts = egress switches, Windows first-run, mirror slot (register 9/10).
- [ ] Maintainer calls pending (docs/reference/00-overview.md §4). Struck at 2026-10-15 doc-audit: A6 wording RATIFIED as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
