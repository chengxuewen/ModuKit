# Wave-2 Benchmark Volume Plan - MediaServo discipline tier

> Date: 2026-10-23. Contract basis: D104 (docs/wave-2-benchmark-hearing.md, rulings
> E/B/B/A, maintainer-adjudicated one by one). Status: APPROVED 2026-10-23 (Momus OKAY + maintainer calibers A/A/A one-by-one).
> This plan lands code only through the three batches below; the delivery plane
> (frontend / docker / deploy CLI / SDK install face) is explicitly OUT - demand
> triggered, per the standing demotion lineage.

## 0. What was verified before writing this plan

All load-bearing anchors were re-read by the orchestrator on 2026-10-23:

- Our private-use bridge: cmake/ModuKitPolyOrchCrates.cmake:80-93 (two
  supplier-private calls + hand-rolled add_test per lane).
- Supplier CTEST keyword exists at our pin:
  PolyOrch/cmake/PolyOrchRustHelpers.cmake:1872-1894 (opts CTEST; knobs
  CTEST_NAME / CTEST_LABELS / CTEST_WORKING_DIRECTORY / CTEST_SKIP_RETURN_CODE).
- Supplier version property exists: PolyOrch/CMakeLists.txt:25
  (INTERFACE_POLYORCH_VERSION on PolyOrch::PolyOrch).
- Supplier standard bootstrap exists, both platforms:
  PolyOrch/scripts/pixi-init.sh (pin :20/:26, no-op-when-satisfied :66, lock-drift
  retry :8-9) + pixi-init.bat (57 lines).
- Twin's retirement note naming our precedent:
  .refinfo/MediaServo/cmake/MediaServoCrates.cmake:26-29; their floor grammar:
  MediaServoPolyOrch.cmake:39-44.
- COUNTER-EXAMPLE FOUND: their clippy.toml only sets the three allow-*-in-tests
  valves, and NO file in their whole repo enables unwrap_used/expect_used
  (grep count 0) - the production-code ban is decoration there, unfireable.
  Batch 2 ships the working version instead of copying the decoration.
- Ctest census today: 9 entries (5 lane + 1 demo lane + 3 cpp band).

## 1. Shape of the volume (diagram)

```
            BATCH 1 - THE WALL          BATCH 2 - RECONCILE+PINS     BATCH 3 - ENVIRONMENT
           (kill private reads)        (proof the checks can bite)  (authority + wrapper)
          +---------------------+     +------------------------+   +-----------------------+
 cmake    | lane test verb      |     | shim: version floor    |   | bootstrap.sh/.bat     |
 layer    |  gains CTEST keyword|     | check-embed-parity     |   |  = thin host wrappers |
          | bridge block DELETE |     | check-capability-ledger|   |  -> PolyOrch/scripts/ |
          |  (2 private calls + |     | check-doc-refs         |   |     pixi-init.{sh,bat}|
          |   hand add_test)    |     |  each gains --self-test|   | demo legs: runner-    |
          +---------------------+     | NEW lane-table gate    |   |  origin PROBE, loud   |
 result: 9 ctest entries, SAME        | clippy.toml + WORKING  |   |  fallback announce    |
  census/labels/SKIP-77, minted by    |  lints wiring (5       |   | CMakePresets.json     |
  one vector with the buttons.        |  crates opt in)        |   | deny.toml licenses+   |
                                      | rustfmt.toml /         |   |  bans legs (probe-    |
                                      |  rust-toolchain.toml   |   |  skip loud, absent    |
                                      +------------------------+   |  today)               |
                                                                   +-----------------------+
 EGRESS-ARMED (procedure written, nothing faked): pixi node+python install + lock warm,
 interpreter switch (probe already routes automatically), cargo-deny install, advisory
 leg, fresh-box bootstrap full run, Windows first-run of the .bat wrapper.
 DECLINED (recorded deviation): incremental fmt leg (ruling 3) - whole-workspace check
 stays; re-open when the repo outgrows the solo line.
```

## 2. Batch 1 - the wall

- **T1.1 CTEST migration.** In cmake/ModuKitPolyOrchCrates.cmake: extend the
  `polyorch_rust_test()` call (:64-71) with
  `CTEST CTEST_LABELS "rust;headless" CTEST_WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
  CTEST_SKIP_RETURN_CODE 77` (values copied from the current hand-rolled
  set_tests_properties :90-93 so the census is byte-identical); delete the whole W4
  bridge block (:80-93). The demo-lane entry (:147-168) STAYS hand-rolled - it execs
  the staged binary, not a cargo test vector, and touches only public POLYORCH_RUST_*
  variables. cpp-band entries untouched (native add_test).
- **Acceptance:** ctest census still 9 with same names/labels; `ctest --test-dir
  out/build-stage4` leg green (check.sh:169 zone); `grep -c "_polyorch_rust_"
  cmake/*.cmake` == 0; full check.sh green.
- **Red proof (C0):** remove the CTEST keyword from one lane -> ctest-stage4 leg
  must go RED on the missing entry; observed, then restored.
- Commits: `build(cmake): mint lane ctest entries via supplier keyword` +
  `docs(memory): batch 1 wall retirement landed`.

## 3. Batch 2 - reconciliation + config pins

- **T2.1 Shim version floor.** cmake/ModuKitPolyOrch.cmake: after the embed/reuse
  branch, read INTERFACE_POLYORCH_VERSION off PolyOrch::PolyOrch; `FATAL_ERROR`
  below floor `0.7.1` (the pin that carries the CTEST keyword we now depend on -
  same grammar as MediaServoPolyOrch.cmake:39-44). Red proof: raise the floor to
  `99.0` once, configure must die loudly, restore.
- **T2.2 --self-test hooks.** scripts/check-embed-parity.sh,
  scripts/check-capability-ledger.sh, scripts/check-doc-refs.sh each gain a
  `--self-test` mode: copy the real inputs to a scratch dir, plant one defect,
  run the check against the scratch copy, require non-zero, print
  `self-test OK`. No scratch side effects on repo files.
- **T2.3 Lane-table reconciler (NEW).** scripts/check-lane-table.sh: parse
  MODUKIT_RUST_LANES (cmake/ModuKitPolyOrchCrates.cmake:22-27) + the demo lane's
  package; assert (a) every lane package is a workspace member (Cargo.toml:3), and
  (b) every `crates/*` member has a lane or sits on the script's explicit
  exception list with a one-line reason (modukit-runtime = bins, modukit-js =
  bin-like node-side face per D103). Built-in `--self-test` (ghost lane planted,
  missing member planted). Wired into check.sh as a fast leg.
- **T2.4 Working clippy (NOT the twin's decoration).** Root Cargo.toml gains
  `[workspace.lints.clippy] unwrap_used = "deny", expect_used = "deny"`; the five
  shipped crates (core/runtime/transport/codegen/js) opt in with
  `[lints] workspace = true`; fixtures/examples do NOT opt in. clippy.toml lands
  with `allow-unwrap-in-tests = true`, `allow-expect-in-tests = true` (inline
  #[cfg(test)] modules and tests/ dirs stay usable). Red proof: plant one
  `.unwrap()` in modukit-core non-test code -> clippy leg -D warnings must die;
  remove, green. (Caliber ruled A 2026-10-23: measured production-zone debt is
  exactly three sites - codegen 2, js 1 - so all five shipped crates switch on
  day one with the three cleaned in-batch; the ladder clause is retired. A
  blanket allow stays forbidden.)
- **T2.5 Style pin.** rustfmt.toml: `edition = "2024"`, `max_width = 100`
  (matches today's habit - the fmt leg must stay green with zero reflow; if a
  file disagrees, that file is a bug the leg will show). Red proof: plant one
  120-char line, fmt --check red, remove.
- **T2.6 Toolchain declaration.** rust-toolchain.toml `channel = "stable"` with a
  one-line comment: pixi remains the real authority on this box (D65); the file is
  intent signage for rustup hosts. No leg (not enforceable offline; decoration
  rule C0 says don't mint unfireable gates - this is a declaration, not a check).
- Commits: one per task (5) + docs(memory) close note.

## 4. Batch 3 - environment authority (landable-now parts)

- **T3.1 Runner-origin probes.** The four py/js demo legs
  (scripts/demo-wave2-{py,py-sup,js,js-sup}.sh): before running, print exactly one
  `RUNNER: <interp> <version> (<source>)` line, source ∈ {pixi-env, PATH}.
  Resolution order: pixi env interpreter first (when present), else PATH; when
  neither, the existing loud SKIP stays. check.sh's eleven-leg zone gains the
  census rule: each of the four legs must show a PASS line AND a RUNNER line -
  PASS-without-RUNNER is red (sabotage lever: strip the RUNNER echo once, watch
  the leg die, restore).
- **T3.2 Bootstrap thin wrappers, both platforms.** bootstrap.sh: keep
  submodule-guard (`test -f PolyOrch/scripts/pixi-init.sh || git submodule
  update --init --recursive PolyOrch`), replace the self-owned pixi install
  block (:51-58 curl-pipe) with `bash PolyOrch/scripts/pixi-init.sh
  --manifest-path "$ROOT/pixi.toml"`; keep our mise stage, git-hooks stage,
  and a commented reserved slot for the China-mirror host policy (maintainer
  call, not bundled). bootstrap.bat: same guard + `call PolyOrch\scripts\pixi-
  init.bat` + our hooks/mise equivalents. On this box today both delegation
  paths execute the supplier's already-satisfied no-op branch - test that it
  is genuinely no-op (observed) before trusting it. Red proof: hide the
  supplier script once -> the guard must fail loudly, restore. Windows
  first-run verification = recorded debt (any Windows machine settles it).
- **T3.3 CMakePresets.json.** Configure presets mirroring the pixi tasks
  (binaryDir out/build, -G Ninja, MODUKIT_USE_PIXI=ON) + a test preset pointing
  the same dir; coexists with pixi tasks (both routes keep working); cross-check
  the .vscode cmake pin so the IDE route is not shadowed (PIT-7 territory -
  verified by running `cmake --preset` configure + `ctest` once, green, and the
  .vscode route re-opened in the IDE check is handed to the maintainer if
  unavailable here).
- **T3.4 deny.toml + probeable legs.** Deny config lands (allow list derived
  from the actual Cargo.lock license census - task step one enumerates it;
  baseline expectation MIT/Apache-2.0/BSD/ISC/Zlib family); pixi task
  `deny = "cargo deny check licenses bans"`; check.sh leg: tool present -> run;
  absent (today, offline box) -> print `SKIP cargo-deny (tool not installable
  offline)` - a loud skip, never a silent one. Advisory leg stays armed (egress).
- **T3.5 Egress procedure (written here, executed when network returns).**
  Add pixi features `node` (nodejs 22 line) + `python` (3.12 line) to default
  env; `pixi lock`; warm caches; T3.1 probes then auto-route to pixi-env;
  install cargo-deny; arm advisory leg; run bootstrap.sh end-to-end on a box
  that can download. Each armed switch gets its own landing note in §7 - none
  is faked today.
- Commits: one per task (4) + docs(memory) close note.

## 5. Verification ledger (every gate observed red once)

| Leg / check | Planted defect | Expected |
|---|---|---|
| ctest-stage4 census | drop CTEST keyword, one lane | RED (missing entry) |
| grep private-use | restore bridge block | RED (count > 0) |
| shim floor | floor raised to 99.0 | configure FATAL |
| three --self-test hooks | their own mode | each prints self-test OK via a real caught defect |
| lane-table gate | ghost lane / unregistered member | RED |
| clippy deny | one .unwrap() in core | RED |
| fmt leg | one 120-char line | RED |
| eleven-leg census + RUNNER | strip one RUNNER echo | RED |
| bootstrap guard | hide supplier script | loud FATAL |
| deny leg | (absent tool) | loud SKIP line, never silent |

Full `bash scripts/check.sh` at the end of every batch; numbering/C1/size lines
run per commit batch as usual. C3 shape per commit.

## 6. Non-goals / refusals (stated so nobody re-litigates by accident)

- Delivery plane (www/docker/deploy/SDK install face/SOVERSION): trigger-gated.
- No regressions: llvm-cov stays (not their tarpaulin), 80 hard boundary stays
  (not their soft 50), single check.sh entry stays (not their root-check.sh name
  footgun), [workspace.dependencies] centralization stays (theirs is looser).
- Their clippy.toml decoration is NOT copied (counter-example §0) - we ship the
  working wiring instead.

## 7. Deviation + debt ledger

- Incremental fmt: DECLINED (ruling 3). Re-open on team growth.
- Windows first-run of bootstrap wrapper: owed to any Windows-capable machine.
- Egress-armed switches: T3.5 list (pixi interpreters, cargo-deny, advisory,
  fresh-box full run).
- China-mirror slot in wrapper: maintainer network-policy call.
- clippy deny: day-one full switch-on ruled (measured three-site debt, cleaned
  in-batch); ladder retired; blanket allow forbidden.
- floor bump-note rule: adopted at caliber ruling (A/A/A 2026-10-23).
- Floor version 0.7.1: re-check at every future PolyOrch bump whether new API is
  relied on (bump commits gain a body bullet naming the floor if raised).

- [x] Momus review 2026-10-23 (fresh session): OKAY - every §0 anchor independently
  re-verified (incl. pin project VERSION 0.7.1 matching the floor constant, CTEST knob
  values byte-matching the current hand-rolled properties); one cosmetic nit (bat 57
  not 50 lines) fixed in place; no blockers.
## 9. Batch 1 landing notes (2026-10-23)

- Migration landed as designed; the post-change census was captured byte-identical
  to the pre-change baseline (9 entries, same names/labels/SKIP-77).
- C0 reds observed on the REAL gate, both plants, restored after each:
  - plant A (stray unknown token where CTEST sat): the supplier parse layer
    LOUDLY rejects at reconfigure - gate red via the build branch. Bonus fact:
    extra-arg drift can never be silent on this verb.
  - plant B (the census-only failure mode, whole CTEST arg-group absent,
    call otherwise legal): configure clean, five lane entries vanish, plain
    ctest would stay green on the four survivors - the new census assertion
    caught it: "FAIL ctest-stage4 census (missing: <5 names>)", gate exit 1.
  - the shared verb call loses all five lanes at once (single call site);
    one-lane precision would need per-lane calls and was not worth it - the
    census names any missing subset.
- Supplier-internal grep (exact names) in cmake/: 0 hits. The four remaining
  "_polyorch_" matches are our own modukit_ensure_polyorch_rust function name.
- check.sh ctest-stage4 leg now checks three phases: build, per-name census,
  full run (replacing the single run that could be green on a shrunk list).

## 10. Batch 2 landing notes (2026-10-23)

- Landed: version floor both-mode + single-constant (lane A); --self-test hooks on
  all three reconcilers; new lane-table gate wired into check.sh (5 lanes <-> 11
  members, 2 reason-carrying exception rows); working clippy deny across the five
  shipped crates; rustfmt/toolchain pin files.
- CALIBER CORRECTION (honest metric rule): the day-one ruling measured production
  debt as 3 sites by an awk heuristic; the compiler truth was 5 (core 3, codegen 1,
  js 1). All five fixed properly per crate style. The ruling (day-one full switch-on,
  no blanket production allow) held; the number in the ruling is corrected here.
- VALVE-SCOPE FINDING: clippy 0.1.99 allow-*-in-tests exempts #[test]/#[should_panic]
  FUNCTION BODIES ONLY - plain helper fns inside tests/ targets still lint. The plan
  T2.4 parenthetical assumed tests/ dirs were covered; they are not. Resolution:
  file-scoped #![allow(clippy::unwrap_used, clippy::expect_used)] headers on tests/
  files, each carrying the measured rationale; test logic untouched; production
  zones stay denied. Recorded deviation from the plan text.
- CACHE FALSE-GREEN: one residual expect at js/src/lib.rs:399 passed an orchestrator
  re-verification because the warm clippy fingerprint served a stale green; the mtime
  churn of a later probe forced a real recompute and exposed it. Fixed (typed napi
  error arm - panics never cross the FFI boundary). Filed as PIT-30: post-edit
  revalidation gets fresh compiles, not warm-cache verdicts.
- Reds observed (C0): floor raised to 99.0 -> configure FATAL; four --self-test modes
  each caught their planted defect; planted unwrap -> clippy leg exit 101; planted
  120-char line -> fmt --check red. All restored; final full gate green.
- Cold flake: modukit-transport-test failed once, first run after the build; 9 warm
  iterations + 2 subsequent full gates green; the output file was overwritten before
  capture. Cause TBD - next occurrence captures output-on-failure immediately.
- Parallel-delegation noise: the three concurrent lanes intermittently saw injected
  content inside tool results (fake system directives, unrelated tasks) and shared
  task-store cross-talk; every lane ignored them and stayed on brief; the
  orchestrator judged on filesystem ground truth + gate reruns, never on narrative.

## 11. Batch 3 landing notes (2026-10-23)

- Runner-origin census LANDED (caliber ruling 3): the four py/js legs each print
  one RUNNER line (pixi-env first, PATH fallback announced, loud SKIP untouched);
  check.sh now FAILs PASS-without-RUNNER. Sabotage red observed (echo stripped ->
  leg FAIL, gate exit 1), restored green. The pixi-env branch arms with T3.5.
- cargo-deny leg LANDED: deny.toml allow list is the real license census (48 pkgs:
  MIT/Apache-2.0/ISC/Unicode-3.0 closure, each with bearer comment); bans stanza
  intentionally empty (unverifiable offline, C0 says no unfired gates); leg today
  prints the loud skip note; pixi task "deny" added.
- Bootstrap wrappers LANDED: guard + delegation to PolyOrch/scripts/pixi-init.sh
  (default pin 0.78.0; box runs pixi 0.82.0 so the ready-branch no-op is live-
  verified; lock-drift retry now supplier-owned); rsproxy reserved host-policy
  slot; mise/hooks stages unchanged; e2e idempotence md5-proved (rc-files + pixi
  binary untouched). Guard red observed (supplier script hidden -> loud exit 1).
  bootstrap.bat mirrors the shape; WINDOWS FIRST-RUN DEBT RECORDED - never executed.
- CMakePresets.json LANDED (schema v3, >=3.25 floor): default/stage4 configure +
  build + test presets mirror the pixi tasks exactly; live green chain configure
  -> build -> census 9 -> ctest --preset. Finding: out/build held a stale Unix
  Makefiles tree that the pixi configure task ALSO could not reconfigure - the
  presets structurally end that drift class. No .vscode change needed (values
  mirror 1:1; auto-selected preset reproduces the IDE flow). Red observed via
  version 999 rejection.
- EDIT-REVERSION INCIDENT (amended same day, true cause): the js kernel() fix
  "vanished" TWICE because lib.rs is GENERATED - the demo-wave2-js legs re-emit it
  from the codegen template every gate run (its own banner says DO NOT EDIT, D63).
  The environment was innocent; the hand-fix sat in the wrong layer. Root cure:
  fixed the emitter (js.rs), re-regenerated the golden, skin copied per leg
  grammar; post-gate byte-identity + forced-fresh clippy prove it stayed. PIT-31
  amended with the correction; the post-commit tree-verification discipline is
  kept (it caught the drift fast - it just named the wrong culprit).

## 12. Review trail

- [x] Maintainer approval 2026-10-23 - three caliber calls walked one by one,
  ruled A / A / A: day-one full clippy switch-on (ladder retired at measured debt),
  floor 0.7.1 + bump-note, RUNNER line joins the eleven-leg verdict rule (the gate
  change carries its own planted-defect red proof).
- [x] Acceptance QA caught a real finding (maintainer): test-plugins rs/cpp targets
  not band-separated in the IDE tree. Root cause: the shelf never had an rs band on
  disk while examples does; the position-mirror folder rule faithfully reproduced
  the asymmetry. Maintainer ruled A (move the boxes, keep the mirror honest): five
  fixture crates into test-plugins/rs/, members/deps/lane-fields/docs follow, full
  gate green with the 9-entry census byte-identical. (The "PIT-31 eviction"
  wording here predates the true-cause finding - see the amendment above: the
  reversion was the js leg re-emitting the unfixed template, since cured at the
  emitter.)
