# ModuKit — Agent-System Backlog

> Track scope: opencode plugins / config / skills / MCP only.
> The Stage-1 Rust mainline (toolchain, workspace scaffolding, crate layout) is a
> SEPARATE track and is NOT to be mixed in here — resume it only on explicit user
> instruction. (Maintainer correction, 2026-10-09: "focus on opencode plugins,
> config, skills, MCP; do not scatter attention.")
> Standing ruling: `.agents/rules/**` is an asset — slim via `instructions[]`
> mount/unmount ONLY, never delete directories or files.

## Done (anchor, so future sessions do not re-propose)

- [x] ecosystem-scan Full first run (local 15 + external 2 lanes) — report 2026-10-09
- [x] A / P0 bundle, committed `2909081`: AGENTS.md C1 gate drift fixed;
      openspec-* x3 flagged inoperative; instructions 26->16; plugins pinned
      (ponytail@5.1.0, context-mode@1.0.169, omo 5.1.27->5.1.28); PIT-4 recorded
- [x] doc-audit FULL first run 2026-10-10 (4 fresh-eyes auditors): 18 findings
      (0 CRIT / 5 HIGH / 9 MEDIUM / 4 LOW), ALL adjudicated one-by-one by user
      (A×16 / B×2); fix batch landed same day (see status.md History)
- [x] External REJECT verdicts recorded (ADR-MCP class dead; rust-analyzer-MCP
      dup of built-in LSP; serena/playwright no value at stage; npm spdx/osv
      trio provenance smell; anthropics skill-creator triple-overlap)

## Pending

> IDs renamed D->P 2026-10-10 to reserve D{n} for decisions.md; the two
> committed status.md History lines that say "D1" about the doc-audit item read
> as P1 retroactively (committed history is never rewritten).

| # | Item | Action | Status |
|---|---|---|---|
| B | skill-lint gate | `scripts/skill-lint.py` + negative-proved fixture; C0 compliance | DONE 2026-10-09 (red: fixture 3 FAIL exit 1; green: repo 10 skills 0 fail) |
| P1 | doc-audit re-scope | skill is still scoped to PolyOrch doc set — retarget to ModuKit canon (whitepaper / gate copies / docs/reference series) or mark deprecated | DONE 2026-10-09 (18 anchors re-targeted; stale AGENTS.md claims fixed; skill usable via /doc-audit) |
| P2 | github-MCP adopt? | v2.0.2 binary INSTALLED ~/.local/bin (sha256-verified); config entry added **enabled:false** pending PAT | HALF-DONE — user: create fine-grained read-only PAT, export GITHUB_PAT, flip enabled |
| P3 | arxiv-MCP adopt? | uv pinned in mise.toml (0.12.24 via it, bootstrap twins cover it); arxiv-mcp-server smoke-passed; config entry **enabled:true** | DONE (active after restart) |
| P4 | openspec-* disposition | 3 skills flagged inoperative; keep-on-disk (per rules-asset ruling) vs full removal of dirs | RESOLVED: KEEP (default per maintainer "mount, never delete" doctrine; veto = say the word) |
| P5 | affaan-m/everything-claude-code | profile lane timed out x3 — unprofiled gap from the scan | DONE 2026-10-14 (FULL rerun; profiled 278.8k stars MIT active; rust-patterns+rust-testing = P1 vendor candidates; star-growth flagged anomalous) |
| R | restart verification | after next opencode restart from activated shell: pinned versions resolve (ponytail@5.2.0, omo 5.1.33, superpowers ref-pinned @v7.0.0 since 2026-10-14; context-mode 1.0.169 unchanged), `lsp` row lights (mise node22), `.omo/omo.jsonc` schema recheck | pending restart |
## Not this track (cross-reference only — Stage-1 triggers)

`scripts/check.sh` seed (landed 2026-10-13 per D50) + doc-liveness + D-record mechanics (PORT_NOW bundle),
rust-analyzer component install, whitepaper reference corrections (Cargo workspace scaffolding STRUCK 2026-10-19 doc-audit: landed 2026-10-15)
(header-first vs metadata-first CLOSED as D63 2026-10-13 - struck at 2026-10-15 audit) — see
`docs/reference/00-overview.md` section 4 and `.agents/memorys/status.md`.

- O-M3S1 flake watchlist (M3S-1/G-cluster): 3 sightings total - hung_peer (plain parallel), crash_injection (plain parallel), one_driver_step (plain, audit build pass); every case ETXTBSY-or-timeout family on process_executor tests, all green on re-run. UPDATE 2026-10-19 (post-hardening): sighting #4 fired INSIDE a unique per-invocation
dir (mk-standin-902786-<ns>-hung) at execve - shared-path theory FALSIFIED, the
unique-dir fix is insufficient. New criterion: this is a real-bug track; probe =
strace/fanotify on the standin inode at spawn time, or switch spawn to
Command::spawn-failure retry-on-ETXTBSY (one bounded retry, documented).
CLOSED 2026-10-19: /proc-holders probe caught sightings 5-7 - holders_at_
failure=0, exec OK at +20ms on private fresh inodes: kernel async-close
window CONFIRMED, real-writer theory dead. Fix = bounded 3x50ms retry at
the production spawn door (executor_proc), 2 deterministic unit tests
(held-fd trick), kill-criterion met (3x gate + 1x concurrent-load run,
0 sightings). Upgrade condition noted in-code: a sighting surviving the
bound reopens the hunt.'s verdict: if a 4th appears OUTSIDE cargo-test parallel load, treat as real bug (fanotify probe), else schedule the unique-dir hardening already done as sufficient.
## Maintenance

- This file is loaded-adjacent: `status.md` (auto-mounted) carries the pointer.
- On any change here, keep the status.md pointer line truthful (drift = audit finding).

## Doc-audit 2026-10-19 dispositions (ruled one by one, clusters D+F)
- [x] P-REHANG rule-pack runnable -> LANDED 2026-10-19 (real run; receipt: 2 over-cap files caught -> new row below).
- [x] dangling-refs family CLOSED 2026-10-19 (M3S-1 sweep; unmounted language pool deliberately left - G10 doctrine).
- [x] P-NEW CLOSED 2026-10-19: both suites -> directory modules (same targets, verbatim bodies; 194/194, max file 359; cargo now prefixes module:: in test paths - cosmetic).
- P-ADD S2 row: user-side apiKey rotation (was history-prose only; status cites it ownerless).
- [x] P-DEFER (F-quad) CLOSED 2026-10-19: C1 scope +test-plugins/ (C0 red/green observed); book-to-skill claim corrected at its TRUE shape (package = book_to_skill/, 15 files, skill tree 22 - the "is 2" disposition figure was scripts/-only and would have mis-attributed); "145-char" -> 156; check.sh "five" -> 56.
