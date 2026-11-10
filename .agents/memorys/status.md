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

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-11-03: D1-D110 (D53 reserved). Stages 1-3 COMPLETE/ACCEPTED (Stage-3 UI tier stays demoted, D89-K2); Stage-4 INFRA volume ACCEPTED/CLOSED (D95); wave-2 volumes ACCEPTED through D107; wave-3 parent-push ACCEPTED/CLOSED (D108 batches 0-2 + D110 acceptance); VE-parity plan D109: B0-B7 landed, B8 COMPLETE (`ve-parity-plan` rev 17 / `agent-host-parity-plan` rev 9; live counts gate-verified: 43 skills / 20 rules / 75 book chapters). Full narrative: `history-archive.md` (2026-11-05 entry). Kernel v0 parked.

## Modules

| Module | What |
|---|---|
| modukit-core / -runtime / -transport / -codegen / -js | SHIPPED (workspace since 2026-10-15; test totals live in the plans of record — volatile counts unlisted per the 2026-10-10 audit; v1 frozen D87) |
| bindings/cpp + bindings/py + bindings/js + contracts C originals | SHIPPED (Stage-4 D93/D95 + python ctypes floor D100/D101 + node plugin door D106/D107) |
| modukit-compositor | planned — §12-3 demoted track; UI demand trigger stands (D89-K2) |

## Verification

| Check | Command | Result |
|---|---|---|
| C1 English-artifacts check | `gate-i18n` in `check.sh` (scope grown with artifact classes; each extension red/green proved) | pass (2026-11-04) |
| skill tree lint | `python3 scripts/agents/skill-lint.py` | pass (2026-11-04) |
| numbering discipline | `bash scripts/check-numbering.sh` — derives max D, asserts AGENTS/status claims; `--self-test` red observed | pass (2026-11-04) |
| memory disposition (C5) | `bash scripts/check-memory.sh <base>..HEAD` (batch close only) | pass (2026-10-30; B7 red, B2 green observed) |
| local check gate (D50) | `bash scripts/check.sh` — 36 legs (13 demo + D104 + gate-rules/spec-trace), full list via `--list`, pixi per D65 | pass (2026-11-04; 123.10 s) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md`; red observed pre-move at 47924 bytes | pass (2026-11-04 offload) |
| gitignore hygiene | `git ls-files -ci --exclude-standard` empty; expected `git check-ignore` hits for out/build/target/agent layers | pass (2026-10-30; VE ownership order) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15) | pass (2026-10-15) |
| doc-audit full | 2026-10-23: 45 raw -> 26 merged, swept; this row is the record | pass (2026-10-23) |
| gate escalations live | check.sh ctest-stage4 leg; commit-msg via core.hooksPath | live (2026-10-21) |
| pre-commit hook legs | `sh scripts/hooks/pre-commit` — agents sync + re-stage + gate-agents + check-doc-refs | live (2026-11-05; doc-refs leg C0-proven, PIT-37) |

## Repo Audit

- 2026-11-04 Quick: 36 gates green, 42 skills, 42 CodeBuddy skill links, 20 generated rules, book 75. Count drift fixed; `verify-codebuddy-host.py` 59 checks / 0 findings. Remaining LOW: codegraph 1.6.2 advisory, gate-mcp O5 WARN, CodeBuddy IDE restart visibility pending.
- 2026-11-05 offload: audit/batch/lesson/hook/flaky rows moved verbatim to `history-archive.md` (2026-11-05 entry). Active remainders: codegraph advisory, gate-mcp O5 WARN, CodeBuddy restart visibility; MediaServo donor clone carries gitlink drift (refinfo-sync FAILs it until resolved upstream-side).
- 2026-11-05 HIGH cleanup (repo-audit trio closed): llms.txt joined the doc-refs scan set (136 candidates, planted dead link reddened); book-check wired as check.sh leg 5b (37 legs, tamper-red observed); thirteen ctest twins minted under label demo-twin - census asserts the names in the fast gate, `ctest -L demo-twin` runs them (sabotage-lever red + clean green observed); census regex widened for ctest's padded numbering.
- 2026-11-06 P2 adjudications #31-33 (walked one by one, A/A/A; #31 corrected to EXCLUDE remote_control_host_computer as a downstream donor, not upstream): donor-sync script to port with per-donor SHA evidence; gate tiering declined (the serial status quo IS the tiering); wire-changelog pairing declined (decisions.md is the changelog; an incident-triggered fallback leg recorded). ve-parity rev 17.
- 2026-11-06 #31 LANDED: `scripts/sync-refinfo.sh` (8 tracked donors, self-test clean-SYNC/dirty-FAIL/HELD observed, live run 7 SYNC / 1 FAIL / 4 HELD); pixi task `refinfo-sync`; repo-audit Relay section wired.
- 2026-11-06 EMBED-02 landed (doc-audit open item closed): the FFI panic wall is a traced SDD clause with its guard test in embed_codec (spec-trace 6 pairs aligned; clause-without-test RED observed mid-batch).
- 2026-11-06 D65 anchor re-check landed (last doc-audit open item): drift measured (:20 -> :18 at donor HEAD 2f88608), append-only note added to the Reference line per D71 precedent; documentation-and-adrs gained the Reference-anchor-drift section so the discipline is written down, not folklore.

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D110 in `.agents/memorys/decisions.md`)
- [ ] Agent-system track: see `.agents/memorys/backlog.md`; CodeBuddy restart visibility pending after skill/rule-count regeneration
- [ ] K7 follow-through (OWNER: MAINTAINER): send the v1-STABLE note (draft: `docs/reference/k7-v1-stable-note-draft.md`); proposal stays ARMED (D89-K7)
- [ ] Media-server base doctrine landed (D68-D73); MediaServo integration = declared intent only; wiring-hearing proposal ARMED, not opened
- [ ] Benchmark volume: plan §7 debts = egress switches, Windows first-run, mirror slot (register 9/10)
- [x] MediaServo governance-face hearing #34-37 ruled 2026-11-09 (ve-parity-plan §2.7); gate-i18n self-test leg landed in check.sh same day (backlog P6 DONE); skip-credential / CHANGELOG / P-mode deferrals stay trigger-armed
- [x] MediaServo engineering-face hearing #38-43 ruled 2026-11-10 (ve-parity-plan §2.8, donor pinned 9d22cba) and executed same day (agents-engineering-plan B1-B6; backlog P12-P16 all DONE): scripts/agents/ aggregation complete (21 files), dispatcher tests in gate-agents (10), abs-path gate + selftests runner legs in check.sh (--list 40), IDE folder grouping machine-verified; e2e naming discarded
- [ ] Maintainer calls pending (`docs/reference/00-overview.md` §4); A6 wording ratified as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
