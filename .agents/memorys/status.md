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

Design baseline whitepaper v1.0 (2026-10-09). Architecture adjudicated through 2026-11-03: D1-D110 (D53 reserved). Stages 1-3 COMPLETE/ACCEPTED; Stage-4 INFRA volume ACCEPTED/CLOSED (D95); wave-2 volumes ACCEPTED through D107; wave-3 parent-push ACCEPTED/CLOSED (D108 batches 0-2 + D110 acceptance); VE-parity plan D109: B0-B7 landed, B8 COMPLETE (`agent-host-parity-plan` rev 9, live counts 42 skills / 20 rules / 75 book chapters). Full narrative: `history-archive.md` (2026-11-04 entry). Kernel v0 parked.

## Modules

| Module | What |
|---|---|
| modukit-core / -runtime / -transport / -codegen / -js | SHIPPED (workspace since 2026-10-15; test totals live in the plans of record — volatile counts unlisted per the 2026-10-10 audit; v1 frozen D87) |
| bindings/cpp + bindings/py + contracts C originals | SHIPPED (Stage-4 D93/D95 + python ctypes floor D100/D101) |
| modukit-compositor | planned — §12-3 demoted track; UI demand trigger stands (D89-K2) |

## Verification

| Check | Command | Result |
|---|---|---|
| C1 English-artifacts check | `gate-i18n` in `check.sh` (scope grown with artifact classes; each extension red/green proved) | pass (2026-11-04) |
| skill tree lint | `python3 scripts/skill-lint.py` | pass (2026-11-04) |
| numbering discipline | `bash scripts/check-numbering.sh` — derives max D, asserts AGENTS/status claims; `--self-test` red observed | pass (2026-11-04) |
| memory disposition (C5) | `bash scripts/check-memory.sh <base>..HEAD` (batch close only) | pass (2026-10-30; B7 red, B2 green observed) |
| local check gate (D50) | `bash scripts/check.sh` — 36 legs (13 demo + D104 + gate-rules/spec-trace), full list via `--list`, pixi per D65 | pass (2026-11-04; 123.10 s) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md`; red observed pre-move at 47924 bytes | pass (2026-11-04 offload) |
| gitignore hygiene | `git ls-files -ci --exclude-standard` empty; expected `git check-ignore` hits for out/build/target/agent layers | pass (2026-10-30; VE ownership order) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15) | pass (2026-10-15) |
| doc-audit full | 2026-10-23: 45 raw -> 26 merged, swept; this row is the record | pass (2026-10-23) |
| gate escalations live | check.sh ctest-stage4 leg; commit-msg via core.hooksPath | live (2026-10-21) |

## Repo Audit

- 2026-11-04 Quick: 36 gates green, 42 skills, 42 CodeBuddy skill links, 20 generated rules, book 75. Count drift fixed; `verify-codebuddy-host.py` 59 checks / 0 findings. Remaining LOW: codegraph 1.6.2 advisory, gate-mcp O5 WARN, CodeBuddy IDE restart visibility pending.
- 2026-11-04 Full doc-audit: README/AGENTS/architecture/VE-plan synced for wave-3 D110, plugin-door D107 acceptance, B8 completion, and adjudications #1-#26.

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D110 in `.agents/memorys/decisions.md`)
- [ ] Agent-system track: see `.agents/memorys/backlog.md`; CodeBuddy restart visibility pending after skill/rule-count regeneration
- [ ] K7 follow-through (OWNER: MAINTAINER): send the v1-STABLE note (draft: `docs/reference/k7-v1-stable-note-draft.md`); proposal stays ARMED (D89-K7)
- [ ] Media-server base doctrine landed (D68-D73); MediaServo integration = declared intent only; wiring-hearing proposal ARMED, not opened
- [ ] Benchmark volume: plan §7 debts = egress switches, Windows first-run, mirror slot (register 9/10)
- [ ] Maintainer calls pending (`docs/reference/00-overview.md` §4); A6 wording ratified as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
