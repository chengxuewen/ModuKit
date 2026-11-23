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
| local check gate (D50) | `bash scripts/check.sh` — leg set lives in `--list` / CONSTRAINTS.md (13 demo legs; count not frozen here), pixi per D65 | pass (2026-11-23) |
| status.md size discipline | `wc -c .agents/memorys/status.md` < 8000 — History offloaded to `history-archive.md`; red observed pre-move at 47924 bytes | pass (2026-11-04 offload) |
| gitignore hygiene | `git ls-files -ci --exclude-standard` empty; expected `git check-ignore` hits for out/build/target/agent layers | pass (2026-10-30; VE ownership order) |
| C2 branch discipline | command block in conventions.md C2 (negative proof observed 2026-10-15) | pass (2026-10-15) |
| doc-audit full | 2026-10-23: 45 raw -> 26 merged, swept; this row is the record | pass (2026-10-23) |
| gate escalations live | check.sh ctest-stage4 leg; commit-msg via core.hooksPath | live (2026-10-21) |
| pre-commit hook legs | `sh scripts/hooks/pre-commit` — agents sync + re-stage + gate-agents + check-doc-refs | live (2026-11-05; doc-refs leg C0-proven, PIT-37) |

## Repo Audit

- Active remainders: codegraph 1.6.2 advisory, gate-mcp O5 WARN, CodeBuddy IDE restart visibility, MediaServo donor gitlink drift (refinfo-sync FAILs it until resolved upstream-side). Next scan re-checks this list first; last recorded scan: Full 2026-11-04.
- Settled rows (2026-11-04 Quick; 11-05 HIGH cleanup; 11-06 P2 #31-33, #31 LANDED, EMBED-02, D65 re-check; hearings #34-37 / #38-43 / #44-47 executed) moved verbatim to `history-archive.md` (2026-11-23 entry).
- 2026-11-23 MediaServo scan @ f589e8f4 (refinfo FAIL drift stands; 210-commit delta over the 9d22cba baseline): donor findings = mcp dual-list drift, /home/ literal in scripts/agents/mcp-servers.json, 3 naming outliers. Pattern cards 8: adopt = tracked-intersect-ignored leak gate, shadcn execution trio, smoke hardening trio (TMPDIR pin, dual-face env control, bootstrap assertions), localized .gitignore discipline; consider = STRUCTURE generator (P2, ship with the donor's Full#2 fixes), 2D collector script, fail-closed pure-function template, three-tier config ladder; hold = per-recipient roster trimming (no consumer). Ported-echo: donor landed aggregation batch-2 (dispatcher chain into scripts/agents/), 10+ agent test files, FOLDER hierarchy plan; abs-path gate / selftests runner / e2e naming unchanged. Dismissal log: "ModuKit has no leak gate" corrected - credential guard (b6ab6fb) is name-form complementary, the tracked-intersect-ignored check is still missing and stays the P1 adopt.
- 2026-11-23 scan follow-up (directed continuation, all four adopt cards landed same day): leak gate `scripts/check-leaks.sh` as check.sh leg 41 (injected self-test: planted force-add red, clean green); demo TMPDIR pins x5 into out/tmp-scratch (five legs re-PASS); shadcn execution trio + ignore-locality section appended to development-workflow.md. Consider cards stay parked in the table above for the next round.

## Open Items

- [ ] Architecture docs awaiting maintainer review (`docs/architecture.md` + `docs/modules/00-11`; ledger D1-D110 in `.agents/memorys/decisions.md`)
- [ ] Agent-system track: see `.agents/memorys/backlog.md`; CodeBuddy restart visibility pending after skill/rule-count regeneration
- [ ] K7 follow-through (OWNER: MAINTAINER): send the v1-STABLE note (draft: `docs/reference/k7-v1-stable-note-draft.md`); proposal stays ARMED (D89-K7)
- [ ] Media-server base doctrine landed (D68-D73); MediaServo integration = declared intent only; wiring-hearing proposal ARMED, not opened
- [ ] Benchmark volume: plan §7 debts = egress switches, Windows first-run, mirror slot (register 9/10)
- [ ] Maintainer calls pending (`docs/reference/00-overview.md` §4); A6 wording ratified as D63; §5 fork wording adopted by D25

## History

> Full log: `.agents/memorys/history-archive.md` — append new entries THERE (top); this file is loaded every turn, keep it lean.
