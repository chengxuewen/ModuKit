# Wave-2 Benchmark Hearing - MediaServo discipline tier (D104)

> Date: 2026-10-23. Maintainer intent: full-line benchmark of .refinfo/MediaServo
> (build organization + PolyOrch consumption), adopted as the project's engineering
> discipline tier. Adjudicated one by one with the maintainer (four questions, one
> per message, plain-language protocol). Evidence base: three read-only analysis
> passes over .refinfo/MediaServo, PolyOrch at the pin, and this repo, with the
> load-bearing anchors re-verified by the orchestrator the same day.

## Roster of adjudications

1. Scope tier of this benchmark volume (main-hazard only / +discipline parts /
   +IDE-hygiene / backlog only / full discipline line).
2. Who guarantees the Python/Node interpreters the demo legs run on.
3. Whether the format leg switches to changed-files-only checking.
4. Whether machine bootstrap delegates to PolyOrch's standard init scripts.

## Rulings

| # | Decision | Ruling |
|---|----------|--------|
| 1 | Scope | **E - full discipline tier, three batches.** Adopt wholesale: private-function-use retirement, version-floor greeter, self-test hooks on reconciliation scripts, clippy config, toolchain/style pin files, interpreter-authority fix, bootstrap delegation, offline-capable supply-chain legs, lane-table reconciliation. NOT adopted: delivery plane (frontend, docker, deploy CLI, SDK install face - demand-triggered, per the demotion ruling lineage); no-regress list keeps llvm-cov (over their tarpaulin), the 80-line hard boundary (over their soft 50), the single check.sh entry (over their root check.sh name footgun), and workspace-dependency centralization (ours already stricter). |
| 2 | Interpreters | **B - pixi declared sole authority; probe-with-announced-fallback.** This box (offline) keeps today's interpreters (mise node, system python) but every demo leg must print which interpreter it is using and why it fell back; silent skip becomes a violation. The switch itself needs egress (pixi package download) - armed, not faked. |
| 3 | Format leg | **B - keep whole-workspace checking, add a style pin file.** The changed-files-only scheme cures a large-team many-PRs disease we do not have (solo line, four crates, the leg currently green); and our baseline (origin/master) trails HEAD by unpushed work, so "changed set" would be a fiction. Explicit deviation from MediaServo, recorded here so the benchmark ledger can re-adjudicate if the project grows. rustfmt.toml lands (max_width 100 = today's habit). |
| 4 | Bootstrap | **A - thin host wrapper, BOTH platforms.** bootstrap.sh/.bat shrink to: submodule-presence guard + delegate to PolyOrch/scripts/pixi-init.sh/.bat (auditable installer, pinned version, already-satisfied no-op, lock-drift retry - all supplier-owned) + our own mise and git-hook stages + a reserved slot for a China-mirror host policy (maintainer call, separate). The Windows first-run verification debt exists whether we wrap or not (the current .bat has never been verified here either) - so it is recorded, not hidden. |

## Batches of work (contract for the plan document)

- **Batch 1 - the wall (P0).** Lanes mint their ctest entries through the supplier
  CTEST keyword (supported at our pin, PolyOrch/cmake/PolyOrchRustHelpers.cmake);
  delete the two sanctioned-internal reads (cmake/ModuKitPolyOrchCrates.cmake:85-86)
  and the hand-rolled bridge lines. Keep the ctest census (9 entries), labels, and
  SKIP-77 posture identical. Red proof: a lane registered without CTEST must drop
  the count and fail the leg.
- **Batch 2 - reconciliation + config pins.** Shim version floor (read the supplier
  version, FATAL below the known-good line - MediaServoPolyOrch.cmake:39-44 grammar);
  --self-test planted-defect hooks on check-embed-parity/capability-ledger/doc-refs;
  NEW lane-table reconciler (cmake lane table vs Cargo workspace members, fatal on
  drift, with its own self-test - MediaServoCrates lane-table grammar); clippy.toml
  (production unwrap/expect prohibition made mechanical); rustfmt.toml;
  rust-toolchain.toml (channel declaration; pixi stays the real authority).
- **Batch 3 - environment authority.** pixi.toml declares node + python features
  (lock warming deferred to egress - armed); the four py/js demo legs gain
  interpreter-origin probing with loud fallback (fallback-but-silent proven red);
  bootstrap thin wrapper on both platforms; CMakePresets.json (mirror of the pixi
  configure/build/test tasks; check the .vscode pin zone - PIT-7 territory);
  deny.toml license+bans legs (cargo-deny binary itself needs egress - leg probes
  and announces skip, config lands now).

## Open ledger (explicitly not silently swallowed)

- Incremental fmt: DECLINED with reasons (ruling 3); re-open when the repo outgrows
  the solo line.
- Egress-armed switches: pixi interpreter install, cargo-deny binary, full-new-box
  bootstrap path, PolyOrch lock warming.
- Windows first-run verification of the wrapper (.bat delegation) - owed by any
  Windows-capable machine, debt recorded here.
- China-mirror step inside the wrapper: reserved slot, maintainer network-policy call.
- Delivery plane (www/docker/deploy/SDK install face): demand-triggered hearings,
  per the standing demotion lineage.

## Verified sources (orchestrator re-read the same day)

- Our side: cmake/ModuKitPolyOrchCrates.cmake:85-86 (supplier-private reads).
- Twin retirement note naming our precedent: .refinfo/MediaServo/cmake/MediaServoCrates.cmake:26-29;
  CTEST keyword usage: same file 108-119; floor grammar: MediaServoPolyOrch.cmake:39-44.
- Supplier scripts: PolyOrch/scripts/pixi-init.sh (:20 pin 0.78.0, :66 already-satisfied
  no-op, :8-9 lock-drift retry), pixi-init.bat (57 lines, twin present).
- Our ad-hoc stage: bootstrap.sh:51-58 (curl-pipe install, no pin/recovery).
- Interpreter status quo: mise.toml:5-6 (node 22, uv) - no python pin; demo legs skip
  silently when absent; pixi.toml has no node/python feature.

## Review trail

- [x] plan document landed - Momus OKAY 2026-10-23 (fresh session); calibers A/A/A
  ruled one by one; batches 1-3 landed (plan §9-§11 evidence rows)
