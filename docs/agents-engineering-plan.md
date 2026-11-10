# Agents engineering plan - hearing #38-43 execution (P12-P16)

> STATUS: PLAN OF RECORD, 2026-11-10 (revision 2, panel-reviewed). Executes
> the MediaServo engineering-face hearing recorded in `docs/ve-parity-plan.md` section 2.8
> (#38-43, ruled 2026-11-10, user-adjudicated one by one; donor pinned
> 9d22cba). Backlog rows P12-P16 carry the same scope one-by-one; this plan
> adds the batch order, per-batch red proofs, and the verification ledger.

## 0. Hearing anchors (re-read before each batch)

| # | Ruling | One-line scope | Batch here |
|---|---|---|---|
| 38 | B: leaf batch first | aggregate the leaf scripts under `scripts/agents/`; dispatcher chain waits for a safety net | B3 (batch one, 16 leaves) + B6 (batch two, chain of 4) |
| 39 | B: main chain only | `test_agents_cli.py`: sync idempotence + planted drift RED | B2 |
| 40 | B: variant transplant | `check-no-abs-paths.py` over `/home/` + `/Users/` literals | B4 |
| 41 | B: one aggregate leg | `run-selftests.sh` runner + one check.sh leg | B1 |
| 42 | A: full port | `MODUKIT_IDE_ROOT` prefix + FOLDER on every target + USE_FOLDERS ON | B5 |
| 43 | C: discard | nothing to execute | - |

Counting correction carried from the ruling text: the leaf set is **16**
scripts, not 14 (mcp-* 6, skill-lint 1, skills-* 2, rules-links 1,
constraints-index 1, memory-aggregate 1, rule-ref-* 2, check-memory-ownership
1, verify-codebuddy-host 1). The ve-parity section 2.8 row and the backlog
P12 row are corrected alongside this plan's landing.

## 1. Scope and OUT

IN: `scripts/` restructure (leaf aggregation), one behavior test, one hygiene
gate, one selftest runner leg, CMake IDE grouping via the existing PolyOrch
FOLDER channel, and the path-reference repairs each move forces.

OUT (explicit): the donor CLI shape (rejected N7), the cmake eight-way split
(rejected N8), e2e-\* naming family (discarded #43), parent-literal checking
(#40's other half - returns on the #37 trigger day), any change inside
`PolyOrch/` (upstream flow per C2; we only consume its verbs).

## 2. Batches

### B1 - selftests runner leg (P15)

- T1.1 `scripts/run-selftests.sh`: explicit list of the 15 in-family
  self-tests (gate-i18n, skill-lint, check-msrv, book-check, rule-ref-lint,
  check-commit-shape, check-capability-ledger, check-numbering,
  check-doc-refs, check-embed-parity, check-lane-table, check-promises,
  check-wire-parity, gate-abi, check-memory-ownership). check-memory.sh is
  deliberately OFF the list (it lives outside check.sh by design). Each entry
  carries interpreter + args (panel O1): fourteen take `--self-test`, while
  check-memory-ownership.py takes the positional `self-test` (its pixi.toml
  invocation shape) - the list stores the arg per entry, no alias surface
  added. Any red reddens the leg; a missing-listed script is a FAIL, not a
  skip (explicit list = explicit contract).
- T1.2 check.sh: one leg `selftests` + one `--list` row (38 -> 39);
  CONSTRAINTS.md regenerates via agents sync.
- T1.3 gate-abi probe note: if its `--self-test` turns out to need a built
  cdylib (cold clone = unresolvable), the runner gives it skip-when-absent
  semantics with a loud skip line - measured at implementation time.
- Red proof: add a phantom name to the list -> runner FAILs (observed once,
  then removed).

### B2 - dispatcher behavior test (P13)

- T2.1 `scripts/tests/test_agents_cli.py`, harness pattern copied from
  `test_agent_config.py` (`MODUKIT_AGENT_ROOT` scratch root + subprocess).
  First step is a probe: confirm `agents-cli.py` / `agent-links.py` honor
  MODUKIT_AGENT_ROOT through the loader chain; if a layer reads repo paths
  directly, add the env hook there first (smallest change, family style).
- T2.2 Two assertions: (a) sync idempotence, measured at REPO level (panel
  E3) - run `agents sync` twice against the real tree and require a clean
  `git diff` (byte-equal generated layers, the same property #39 asks for,
  without rebuilding a full .agents fixture; gate-agents already carries this
  round trip, the test pins it explicitly; running this assertion standalone
  (outside gate-agents) may rewrite generated layers once - via gate-agents
  the tree is already synced, so the diff stays clean); (b) planted drift -
  corrupt the scratch `agent-targets.json` -> `check` must RED.
- Red proof: assertion (b) IS the planted-defect red (C0 satisfied in-test).
- Joins the existing unittest discover under gate-agents automatically.

### B3 - leaf aggregation, batch one (P12a, 16 scripts)

- T3.1 `git mv` the 16 leaves into `scripts/agents/` (list in section 0).
- T3.2 Repair path references - MECHANICAL INVENTORY FIRST (panel R4-1): the
  authoritative net is a full grep over the moved names,
  `grep -rn "scripts/mcp-\|scripts/skill-lint\|scripts/skills-\|scripts/rules-links\|scripts/constraints-index\|scripts/memory-aggregate\|scripts/rule-ref\|scripts/check-memory-ownership\|scripts/verify-codebuddy"`
  across the repo (excluding .pixi/, target/, node_modules/); the enumeration
  below is the known-category map, the grep is the contract. Known classes:
  `pixi.toml` tasks (mcp/gate-mcp/agents family), `scripts/check.sh` legs
  (skill-lint, gate-mcp, rule-ref-lint via gate-rules),
  `scripts/agent-links.py` loader table, `hooks/pre-commit`, `docs/env.md`,
  `CONTRIBUTING.md`, AGENTS.md command blocks, `.agents/skills/*/SKILL.md`
  path mentions, `scripts/tests/test_agent_config.py` MCP_SCRIPT path, and
  INTERNAL PATH CONSTANTS of consumers that stay top-level (panel H1):
  `scripts/gate-mcp.py` REGISTRY + two more `scripts/mcp-servers.json`
  literals (lines 36/147/176) - gate-mcp.py itself does NOT move (gates stay
  at scripts/ top level by convention, panel C4), but its data file does.
  Verified non-issue (panel E1): `.agents/mcp.json` command strings carry
  zero `scripts/` paths (grep 2026-11-10) - launches route through pixi task
  names, already inside the pixi.toml net; re-verify with the mechanical
  grep at implementation time.
- T3.3 The four chain files stay at `scripts/` top level (batch two).
- Red proof: mid-move, leave exactly one stale reference in place, observe
  the owning leg redden (check-doc-refs or gate-agents), then fix it. This
  proves the reference net actually holds the move.

### B4 - absolute-path hygiene gate (P14)

- T4.1 `scripts/check-no-abs-paths.py` at scripts/ TOP LEVEL with the
  check-* family (panel M2: gates live at top level - gate-mcp.py sets the
  precedent; scripts/agents/ holds generators and their data, not gates).
  Variant of donor `check_no_parent.py` @ 9d22cba: scan `scripts/` +
  `.agents/` for `/home/` and `/Users/` literals, with the donor's SKIP set
  inherited (panel M1: `__pycache__`, `node_modules`, and `.agents/memorys`
  - the memorys files legitimately quote absolute paths in incident notes;
  donor check_no_parent.py:26 skips exactly this); allowlist file for
  recorded exceptions, populated by a pre-scan at landing (expect a
  non-empty-but-small list, NOT the rev-1 "none expected" assumption);
  `--self-test` plants both literals in a scratch root and asserts RED +
  clean green.
- T4.2 One check.sh leg + `--list` row (39 -> 40); add the script to the B1
  runner list (16 entries).
- Red proof: in-test plants (C0); plus one live observation against the repo
  tree before trusting the green.

### B5 - CMake IDE grouping, full port (P16)

- T5.1 Probe first: read `PolyOrch/cmake/PolyOrchFindRust.cmake` (1428+),
  `PolyOrchCMakeHelpers.cmake` (688+), `PolyOrchNodeHelpers.cmake` (69,
  `PolyOrch_RUST_FOLDER_ROOT` knob) - record the exact knob semantics before
  writing any consumer line.
- T5.2 `CMakeLists.txt`: `set_property(GLOBAL PROPERTY USE_FOLDERS ON)` +
  `MODUKIT_IDE_ROOT` ("" when PROJECT_IS_TOP_LEVEL, else "modukit/") + the
  3.21-floor FATAL guard, donor shape.
- T5.3 Consume FOLDER at every declaration site: `cmake/ModuKitPolyOrchCrates.cmake`
  (lane rows), `cmake/ModuKitDemoTwins.cmake` (twin targets),
  `cmake/ModuKitGates.cmake` (gate bridges), examples/bindings faces.
- T5.4 Verification: reconfigure + `cmake-smoke` + `ctest` census unchanged +
  `check-lane-table` still green (it audits lane DATA, not IDE properties) +
  one human observation of the Qoder target tree (memory-grade evidence).
- Red proof: no new gate here; the proof burden is "nothing existing
  reddens" + the IDE tree visibly groups. The two existing cmake-smoke
  negative paths remain the configure-level guard.

### B6 - leaf aggregation, batch two (P12b, chain of 4)

- T6.1 Precondition gate: B2's test green on the moved layout simulation.
- T6.2 `git mv` agents-cli.py, agent-links.py, agent_common.py,
  agent-targets.json into `scripts/agents/`; repair loader paths, pixi tasks,
  hooks, docs (same net as B3, smaller).
- T6.3 Full verification: check.sh all green; `test_agents_cli.py` green on
  the new paths (it is the safety net this batch exists to prove).

## 3. Verification ledger

| Plant / action | Expect |
|---|---|
| B1 phantom list entry | runner FAIL (then removed) |
| B1 final | all 15 self-tests OK in one leg; --list 39 |
| B2 double sync (repo level) | clean git diff after the second run |
| B2 corrupted scratch targets.json | agents check RED |
| B3 one stale reference left mid-move | owning leg RED |
| B3 post-move mechanical grep | zero old-path hits repo-wide |
| B3 final | check.sh green; scripts/ top level 61 -> 47 files (16 moved, runner + gate added) |
| B4 planted /home/ and /Users/ literals | gate RED |
| B4 repo tree scan | green (allowlist = exactly the pre-scan set, small) |
| B5 reconfigure + ctest | census unchanged, cmake-smoke green |
| B6 chain move with test net | test_agents_cli green on new paths |

## 4. Order and dependencies

```
B1 (independent, smallest) -> B2 -> B3 -> B4 (extends B1 list) -> B5 -> B6 (needs B2)
```

B5 is independent of B3/B4 but scheduled after them so the scripts tree is
settled before touching cmake; B6 is last by design (batch two waits for the
safety net, hearing #38's own condition).

## 5. Risks

- The MODUKIT_AGENT_ROOT probe (T2.1) may surface a layer that reads repo
  paths directly; the fix is a one-line env hook per layer, family style -
  bounded, but unverified until probed.
- gate-abi's self-test dependency footprint (T1.3) is unmeasured; skip
  semantics are the fallback, decided at implementation time.
- The B3 reference net is wide (pixi tasks, hooks, skills docs); the one-stale-
  reference red proof is the sampling check, not an exhaustive one.
- IDE grouping evidence is partly human-observed (memory-grade); no machine
  gate asserts folder shape - accepted, it is sugar not a contract.

## 6. Gate plan

Full `bash scripts/check.sh` after each batch; `pixi run agents sync` before
each commit that touches .agents or generated layers; commit-shape gate owns
every commit (one batch = one commit, C3).

## 7. Deviations and debts

- none at plan time; deviations get logged here with DECLINED / owed /
  known-environment labels as they occur.

## 8. Change log

- 2026-11-10 revision 2: panel review (expert-panel-review, sequential track,
  4 lenses) folded in - H1 unmoved-consumer constants (gate-mcp.py REGISTRY)
  joined the B3 net; R4-1 mechanical grep replaces enumeration as the
  contract; M1 SKIP set inherited from the donor, allowlist expectation
  corrected; M2 the hygiene gate sits top-level with the check-* family;
  O1 runner stores per-entry args (positional style kept); E3 idempotence
  measured at repo level; E2 top-level count corrected 61 -> 47; E1 mcp.json
  clean-path verification recorded; C4 gate-mcp.py stays by convention.
  Re-review pass (core two) caught three self-inflicted inconsistencies and
  folded them in: ledger B4 allowlist line aligned with M1, the standalone-
  run side-effect note on T2.2(a), and the STATUS line wrap.
- 2026-11-10 revision 1: plan drawn from hearing #38-43 (ve-parity-plan
  section 2.8); leaf-count corrected 14 -> 16; six batches, ledger, risks.
