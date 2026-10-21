# Wave-2 closeout plan (reader-leg promotion + coverage census)

> STATUS: ADJUDICATED 2026-10-21 — P1-P5 walked one by one (reply order A, A, C,
> A, A); Momus review OKAY (§9). No code lands until the maintainer orders
> execution; the D98 mint itself is part of the execution round (§5).
> This file deliberately decides only the two named closeout items — no new doors.

## 1. Background

- D97 landed the debug-reader minimal face: bin `modukit-debug-reader`
  (`crates/modukit-runtime/src/debug_reader.rs`), prints `debug-reader: PASS`,
  sabotage lever `MODUKIT_DEBUG_READER_SABOTAGE=1` (red path), four integration
  tests in `crates/modukit-runtime/tests/debug_reader.rs`. Its own re-review
  trigger says the PASS-line gate may be promoted into `scripts/check.sh` only
  once a planted red has been observed there (C0). Today check.sh runs FIVE demo
  legs (stage1, stage2, stage4-m1, stage4-m2, stage5-n2) — the reader face has
  no gate of its own beyond its cargo tests.
- PIT-23 (2026-10-21): an instrumented coverage rerun can silently execute fewer
  suites than the tree has — a false-HIGH percent. Corrected canon measured on
  the then-tree (51 suites): core 66.95 / transport 80.78 / codegen 84.25,
  region metric per D84a, census attached. The N1/N2/debug-reader landings moved
  the tree again; the canon is now two landings stale. status.md at the D97
  landing records "coverage debt stands"; the D84 anti-second-delay hook belongs
  to stage-boundary gates, none of which is currently at hand.

## 2. Scope

Two closeout items, neither opens a door:

1. **Reader leg**: promote the `debug-reader: PASS` line to a sixth demo leg in
   `scripts/check.sh`, with the house PASS/SKIP/RED grammar and a C0-observed
   planted red.
2. **Coverage census**: re-measure workspace coverage on the CURRENT tree with a
   cold clean instrumented run + suite-execution census, record the honest canon
   in the ledger, and re-name (or clear) core's debt — measurement and bookkeeping
   only, no test-invention.

## 3. Non-scope

- Timer hearing, Python bindings, JS surface, live-attach, `modukit-cli` — all
  stay armed per stage-4 §6 / D97; this plan does not touch them.
- Any ABI / wire / contract / frozen-surface (D87/D83) change: zero.
- Force-clearing core below 80 by manufacturing tests (D84 ruling-6 anti-gaming
  clause: punishing the metric with fake tests is refused).
- `pixi run vendor-test` stays out of the fast gate (unchanged).

## 4. Proposed rulings

**Adjudication results (user-adjudicated one by one, 2026-10-21): P1=A, P2=A,
P3=C, P4=A, P5=A.** Plain-language notes: P2-A was justified on top of the
original options by the fact that the sabotage lever already has a cargo test
running in every gate pass (double auto-probe buys the same insurance twice);
P3-C = measure + record + copy the existing D84 boundary repayment clause next
to the new number, no invented deadline and no same-round test cramming.

| # | Item | Verdict | One-line note |
|---|---|---|---|
| P1 | Leg form | A | wrapper script (n2 template) + sixth check.sh leg |
| P2 | Red demonstration | A | one observed red + three evidence rows; lever self-tested by cargo |
| P3 | Coverage handling | C | honest canon now, repayment calendared at the next boundary gate |
| P4 | Run hygiene | A | cold clean instrumented run + suite census for the cited number |
| P5 | Ledger form | A | mint D98 + doc sweep (incl. README stale "All four" row) |

### P1 — Leg form

A. Wrapper script `scripts/demo-wave2-reader.sh` following the demo-stage5-n2.sh
   template (40 lines: pixi-routed cargo build of the bin, run, `grep -qx` the
   PASS line, SKIP-when-no-toolchain), plus a check.sh leg block.
B. Inline block in check.sh (build + run + grep, no wrapper).
C. Wrapper plus a ctest bridge entry in the CMake graph.

Recommendation: **A**. The house grammar is one script per demo leg (all five
predecessors), the sabotage lever is documented in the script header the way
n2 documents its own, and B would make check.sh the odd script carrying demo
logic. C is a second surface with no consumer.

### P2 — Red demonstration mechanics

A. Observe red once by hand (`MODUKIT_DEBUG_READER_SABOTAGE=1
   bash scripts/demo-wave2-reader.sh` must exit 1 with no PASS line; the leg in
   check.sh must print its FAIL note), record both rows, restore green.
B. Auto-run the sabotage probe inside every gate pass.

Recommendation: **A**. Every predecessor documents the lever rather than
double-running it (n2 header, stage2 hook); the sabotage path already has a
cargo test; B doubles the leg's cost for evidence that exists once and stays true.

### P3 — Coverage outcome handling

A. Measure + census + record the honest canon; debt column updated; nothing else.
B. Also clear core to >= 80 in this round by adding targeted tests.
C. Measure + record + explicitly calendar the repayment at the next stage /
   acceptance boundary, where the D84 anti-second-delay hook fires as already ruled.

Recommendation: **C** (A plus the calendar row). The 80-hard line is not relaxed;
its bite point is the boundary (D84b). B invents a deadline the ledger does not
have and pressures the exact fake-test class D84 ruling 6 refuses.

### P4 — Run hygiene for any cited number

A. Cold clean instrumented run (`cargo llvm-cov` with a clean instrumentation
   cache) for the number this plan cites; census = `grep -c "Running" <covlog>`
   vs the suite inventory from a redirected `cargo test --workspace` log
   (test-targets minus lib-bins must equal executed). Mismatch -> rerun, never
   re-cite the warm number.
B. Warm run + census check, rerun only on mismatch.

Recommendation: **A** for this acceptance record (PIT-23 is exactly the warm-run
class; the rerun cost is minutes, once). B remains the everyday habit.

### P5 — Ledger form

A. Mint one decision record (D98) covering both items' rulings and evidence,
   widen the numbering gate, sync: status.md verification row + phase line
   (FIVE -> SIX demo legs), AGENTS.md COMMANDS block, stage-4-plan §5 debt row,
   wave-2-debug-reader-plan §9 sibling pointer.
B. Doc-row sync only, no new D record.

Recommendation: **A**. The promotion closes a named D97 re-review trigger and
retires the PIT-23 stale-canon state — adjudicated content of the same class as
D96/D97, so it belongs in the ledger, not only in prose.

## 5. Implementation outline (post-ruling; order is parallel-safe after step 1)

1. Numbering gate widened for D98 (red observed, per the same-day-widen precedent).
2. Write `scripts/demo-wave2-reader.sh` (n2 template: `set -uo pipefail`, pixi
   routing per D65, SKIP exit 0 when no toolchain, build `-p modukit-runtime
   --bin modukit-debug-reader`, run, `grep -qx 'debug-reader: PASS'`,
   header documents the sabotage lever + expected RED).
3. check.sh: append the leg block with the stage4/n2 PASS/SKIP/FAIL grammar.
4. C0 cycle: leg green -> sabotage run red (script + check.sh note) -> green
   restored; three rows recorded in the plan and the D98 reference.
5. Coverage run per P4-A, both logs redirected to files (PIT-9); census line
   written next to every percentage.
6. Ledger sweep per P5-A (grep the whole docs set for the "FIVE demo legs"
   claim and the five-leg enumeration so no doc keeps the old count; the Momus
   review additionally flagged README.md's stale "All four are also wired as
   standing legs" row - same drift class, fixed in the same sweep.)
7. Commits (C3 shape): `feat(check): reader leg` (script + gate + red-proof
   header notes), `docs(cov): honest census canon post-N2 tree`, memory batch
   with D98 last.

**Execution outcome (2026-10-21, same day):** steps landed in order - the
numbering-gate D98 probe was observed RED under the old bound then restored;
wrapper + leg written; GREEN -> sabotage RED (exit 1, no PASS line, check.sh
takes its FAIL branch) -> restored GREEN, three-row evidence in the script
header; full gate green with `ok demo-wave2-reader` (six legs); the cold
instrumented run executed 50/50 suites (census matches the gate log):
core 84.15 / transport 80.78 / codegen 84.25, region metric D84a - the P3-C
calendar clause went unused: the named core debt is CLEARED on this tree
(66.95 was the stale pre-N2 short-run reading). runtime 12.02 + bare
fixture regions ride the D84 boundary, recorded as boundary not gap.

## 6. Done-when

- `bash scripts/check.sh` prints `ok demo-wave2-reader` and the leg has been
  observed RED once through its own sabotage lever (evidence rows recorded).
- A coverage canon for the current tree exists with a suite census that matches
  the inventory; core's debt is re-named (or cleared) in stage-4 §5 + status.md.
- No doc still claims five demo legs; C1 + C3 clean; frozen surfaces untouched
  (the diff is one script + one check.sh block + docs).

## 7. Risks

- Cold cov run cost (minutes, one-off) — accepted; it is the honest-price side
  of PIT-23.
- If the census mismatches repeatedly the item stops and becomes a tooling
  finding, not a silently re-cited number.
- check.sh runtime grows by one cargo build + one short bin run; the gate stays
  skip-when-absent (no toolchain -> SKIP green).
- "FIVE demo legs" phrasing lives in more than one file; a missed row is a
  doc-liveness finding the next gate leg will catch.

## 8. Review checklist for the critic

- Consistency with D84 (gate, boundary exclusions, anti-gaming), D87/D83
  (zero frozen-surface touch), D97 (trigger closure is exactly as written),
  C0 (observed-red before trusted), PIT-9/PIT-23 (redirect + census discipline).
- Minimal scope: no door opened, no abstraction, no new crate, no config knob.
- Every claimed verification has a named command and a named evidence row.

## 9. Review result

**OKAY (approved), zero mandatory fixes — Momus plan critic, 2026-10-21.** All
load-bearing references verified against the tree (bin + 4 tests + sabotage env,
n2 template, five check.sh legs, D97 trigger text, PIT-23/D84 rows, D98 free,
stage-4 §5 debt row, "FIVE" phrasing locations); steps executable as written,
scope minimal, no contradictions. One minor note (folded into §5 step 6):
README.md carries a separate stale "All four" leg claim - same drift class.
Ready for the P1-P5 ruling walk.
