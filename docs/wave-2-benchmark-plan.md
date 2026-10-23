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

## 8. Review trail

- [x] Momus review 2026-10-23 (fresh session): OKAY - every §0 anchor independently
  re-verified (incl. pin project VERSION 0.7.1 matching the floor constant, CTEST knob
  values byte-matching the current hand-rolled properties); one cosmetic nit (bat 57
  not 50 lines) fixed in place; no blockers.
- [x] Maintainer approval 2026-10-23 - three caliber calls walked one by one,
  ruled A / A / A: day-one full clippy switch-on (ladder retired at measured debt),
  floor 0.7.1 + bump-note, RUNNER line joins the eleven-leg verdict rule (the gate
  change carries its own planted-defect red proof).
