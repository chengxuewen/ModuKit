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

| # | Item | Action | Status |
|---|---|---|---|

> IDs renamed D->P 2026-10-10 to reserve D{n} for decisions.md; the two
> committed status.md History lines that say "D1" about the doc-audit item read
> as P1 retroactively (committed history is never rewritten).

| B | skill-lint gate | `scripts/skill-lint.py` + negative-proved fixture; C0 compliance | DONE 2026-10-09 (red: fixture 3 FAIL exit 1; green: repo 10 skills 0 fail) |
| P1 | doc-audit re-scope | skill is still scoped to PolyOrch doc set — retarget to ModuKit canon (whitepaper / gate copies / docs/reference series) or mark deprecated | DONE 2026-10-09 (18 anchors re-targeted; stale AGENTS.md claims fixed; skill usable via /doc-audit) |
| P2 | github-MCP adopt? | official server, ★33k MIT; needs Go binary install (no docker on box) + fine-grained read-only PAT | awaiting user |
| P3 | arxiv-MCP adopt? | ★3.2k Apache, uvx install — `uv/uvx` absent on box (mise can pin it) | awaiting user |
| P4 | openspec-* disposition | 3 skills flagged inoperative; keep-on-disk (per rules-asset ruling) vs full removal of dirs | awaiting user |
| P5 | affaan-m/everything-claude-code | profile lane timed out x3 — unprofiled gap from the scan | optional rerun |
| R | restart verification | after next opencode restart from activated shell: pinned versions resolve, omo 5.1.28 fetched, `lsp` row lights (mise node22), `.omo/omo.jsonc` schema recheck | pending restart |

## Not this track (cross-reference only — Stage-1 triggers)

`scripts/gate.sh` seed + doc-liveness + D-record mechanics (PORT_NOW bundle),
rust-analyzer component install, Cargo workspace scaffolding, header-first vs
metadata-first contract, whitepaper reference corrections — see
`docs/reference/00-overview.md` section 4 and `.agents/memorys/status.md`.

## Maintenance

- This file is loaded-adjacent: `status.md` (auto-mounted) carries the pointer.
- On any change here, keep the status.md pointer line truthful (drift = audit finding).
