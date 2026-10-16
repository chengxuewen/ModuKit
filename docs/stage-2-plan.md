# Stage-2 Plan — Process Placement, Wire Transport, Zero-Copy First Entry

> Status: DRAFT for maintainer adjudication + Momus review. Plan of record once accepted.
> Scope anchor: `docs/architecture.md` §12 item 2 (supervisor, shadow-runner, wire transport)
> + the §10 Out-list spine the maintainer opened on 2026-10-16 (ProcessExecutor /
> shadow-runner / cross-process bus / iceoryx2 first entry). §10 lists are quoted, not reinvented.
> Execution gate: nothing here is executed until the maintainer says go (Stage-1 precedent).

## 0. Goal and boundary

Deliver process placement exactly as modules/01 specifies it: `ProcessExecutor`
implementation (the interface shipped in Stage-1), the shadow-runner binary, and the
cross-process bus over the wire transport; then admit iceoryx2 as the first vendor
pool behind the D11 backend trait (D25 selection is already adjudicated — execution,
not re-debate). The dual-executable acceptance leg deferred since Stage-1 (D17 test-2
exe leg, M6 note "exe leg = Stage-2") closes here.

Explicitly in: ProcessExecutor + supervisor loop (spawn/handshake/restart-policy/
FAILED-on-crash/kill teardown), `crates/modukit-runtime` (bin: supervisor host + bin:
shadow-runner — whitepaper §2.3 name), SCM_RIGHTS fd-passing + wire framing
(transport seam cross-process), cross-process bus wiring with QoS against a real wire
(D10 re-review trigger), iceoryx2 exact-pinned tag behind a cargo feature
(D25; kernel core stays std-only), export-freeze hearing incl. the declined
external-consumer validation round (D5), dual-placement acceptance (`scripts/demo-stage2.sh`).

Explicitly out (trigger-gated): cgroup/rlimit supervision and witness-based
liveness (modules/01 §7); watchdog/safety surface (§10 Out; D3 re-review trigger:
"watchdog/ASIL contract with a real safety vendor"); Zenoh cross-host carrier
(trigger: first cross-machine topic link); OTA distribution; compositor; all
language bindings (Stage 5); wasm executor.
parallel_ok routing: canon D8/§11/modules/03 schedule routing WITH this stage's
exe-placement build and D71's own wording keeps that trigger alive - the exclusion
is NOT decided unilaterally here; call-sheet item A7 adjudicates it.
Recorder/replay (D16) and the CLI trio-face (D17) are candidate lanes - canon
(D16/D17/D23) currently schedules both for Stage-2; call-sheet item A5 decides
in-stage vs defer, and a defer verdict explicitly supersedes those schedules.

## 1. Call sheet — adjudications the maintainer owes at kickoff (walked one by one)

| # | Question | Recommendation |
|---|---|---|
| A1 | Cross-process clock discipline: does the supervisor drive the runner's Sim/Step clock over the wire (deterministic dual-placement replay), or does the exe leg run Real clock and the replay equality relaxes to event-order? | Drive Sim/Step over a control-plane clock field (extends D20: timers-as-jobs already make the runner's clock injectable; keeps zero-sleep testhost honest cross-process) |
| A2 | D5's declined external-consumer validation round: shadow-runner is a separate process consuming the C-ABI/contract unchanged — does that count as the first non-kernel consumer, i.e. run the round before the freeze hearing? | Yes — run it as part of A4's hearing (the exe placement suite IS the validation the year-1 risk note waited for) |
| A3 | Vendor-lock amendment: M5's vendor_lock test proves zero crates.io additions in modukit-transport. iceoryx2 must enter. Amend to: vendor deps allowed only behind a non-default cargo feature of modukit-transport, exact-pinned tag (D25), kernel core untouched std-only. | Adopt the feature-boundary amendment |
| A4 | Export-freeze hearing scope: C-ABI export list + D51 shared-vocabulary origin (M5 deferral) + integrity codes (M7 deferral) + BundleError hint() (M8 deferral) + ABI version bump policy. Freeze or defer-with-calendar? | Hold the hearing at S2-M5 after the dual-placement suite has run; freeze what the suite touched, calendar the rest |
| A5 | D16 recorder/replay + D17 CLI: build in Stage-2 as optional lanes S2-M6a/b, or defer? NOTE: canon currently SCHEDULES both for Stage-2 (D16 "implementation + dependency land with Stage 2", D17 "CLI (Stage-2, commands map 1:1 to the trio)", D23 "Stage-2 CLI subcommand") - a defer verdict supersedes those entries + needs a delta row | Defer unless the maintainer wants a demo-driven recording lane this stage; if built, the `mcap` crate entry needs its own A3-style amendment |
| A6 | Coverage hard gate (D50: 80% once interfaces stabilize): does the freeze hearing (A4) declare interfaces stable → hard gate switches at S2-M5 acceptance? | Yes — switch at A4 outcome, measured at S2-M5 |
| A7 | parallel_ok routing: D8 ("routing is a Stage-2 mechanism", §11 delta row, modules/03 "implementation Stage-2") and D71's surviving named trigger ("routing lands with exe placement") schedule version-affinity routing to THIS stage's exe-placement build, while D71 kept the field inert for the media-base scenario. Ship a minimal routing leg inside S2-M2/M3, or supersede the D8 schedule with a D-entry + §11/§12 delta row and a named re-activation trigger? | Adjudicate one by one at kickoff; do NOT decide by omission |

## 2. Milestone outline

| # | Milestone | Deliverable | Anchors | Gate / evidence |
|---|---|---|---|---|
| S2-M0 | Kickoff hygiene | call sheet A1-A7 walked one by one (adjudication-walkthrough protocol); decisions land `decisions.md` as new D-entries AND the numbering-discipline gate widens in the same change (status.md check currently allows D1-D73 - first D74 mints red until the grep bound is bumped); PolyOrch pin freshness check; push decision OPEN at kickoff (Stage-1 M0 precedent: push only on explicit maintainer instruction; DoD #5 then closes it) | §0, this file | every A-item has a recorded verdict before S2-M1 opens |
| S2-M1 | ProcessExecutor + supervisor loop | executor.exe branch in core: spawn via std::process, control socket handshake (ABI v1 recheck), restart-policy table per D3, crash-watch (wait + FAILED edge + zero-phantom-reload), graceful STOP then timeout→kill teardown (modules/01 §3: `exe` teardown is `kill`); switchability preconditions enforced at install-time where checkable | D3, modules/01 §1-4 | first-red: exe-spawn test fails pre-implementation; crash-injection test: host alive, FAILED edge + ledger record, bounded restart counts visible on counters |
| S2-M2 | modukit-runtime + shadow-runner | new workspace member `crates/modukit-runtime` (runner bin per the modules/01 §2 command shape `modukit-run <plugin.so> --fd=N`; supervisor bin name has no canon - chosen and recorded at M0); runner dlopens the same fixture cdylib the dl executor would load, pumps identical ABI v1, registers over the control socket (the `--fd=N` inherited end); example-plugin gains an exe-placement twin run in the SAME suite (§10 "contract suite structured for later dual-placement runs" becomes live) | D23 templates, modules/01 §2 | first-red runner-connect handshake; dual-run table: every contract-suite case green under both placements |
| S2-M3 | Wire transport + cross-process bus | unix-socket SCM_RIGHTS fd passing (documented-unsafe libc shim per the M5 memfd_create precedent, std-only kept), TL-versioned frame on the wire, bus publish/subscribe across process boundary with QoS fan-out tested against a real wire (D10 trigger); A1 clock verdict lands here | D9/D10/D11/D20, modules/07 | first-red fd-pass roundtrip between two test processes; QoS stress: drop-counters visible on both sides, late-join across the wire; D17 test-2 exe leg green (step-replay content equality across placements) |
| S2-M4 | iceoryx2 first entry | `vendor-iceoryx2` feature on modukit-transport, exact-pinned tag; ported pool + porter behind the D11 backend trait; dummy double stays the CI default; frames cross only as pool+descriptor handles (D69 honored — no raw pointer, no foreign buffer anywhere in the public seam); A3 amendment lands with the Cargo change | D11/D25/D69 | first-red: pool roundtrip over vendor backend equals dummy-backend semantics; vendor_lock test amended per A3 and still green on default features; memory/fd leak check after 10k roundtrips |
| S2-M5 | Freeze hearing + Stage-2 acceptance | run the dual-placement suite as the declined external-consumer validation (A2), then the hearing (A4): C-ABI export list, D51 origin ruling, integrity codes, hint() — freeze what the suite touched; coverage hard-gate switch if A6 says so (A6); `scripts/demo-stage2.sh` PASS gate + falsifiability probe; §10/§12 delta table re-audit vs architecture.md | D5/D50/D51, §10 | demo-stage2.sh prints STAGE2-DEMO PASS; tampered-runner probe REDs the gate; coverage table re-measured; check.sh + ctest regressions green |
| S2-M6a/b | Optional lanes (only if A5 says in) | recorder/replay per modules/09 (mcap neutral crate) / minimal CLI mapping 1:1 to the org.modukit.* trio with `--json` | D16/D17 | each lands with its own first-reds + demo hook; each vendor entry passes an A3-style amendment first |

## 3. Execution sequence

S2-M0 → S2-M1 → S2-M2 → S2-M3 → S2-M4 → S2-M5; optional lanes after M5 or never.
Parallelism: the memfd/SCM_RIGHTS shim prototype (M3 first half) is independent of M2 —
may run as a background lane once M1 merges. M4 starts only after A3 is recorded
(Cargo changes wait the verdict per the execution-gate convention).

## 4. What exists at Stage-2 entry / what does NOT

Exists: executor trait seam with dl branch, FAILED edge, restart-policy types,
transport trait + Dummy/SocketpairLane/MemfdBlob (single-process), in-process bus,
bundle install protocol with SHA-256, ABI v1 handshake, observability floors,
step-clock testhost. NOT (lands this stage): any child-process spawn, any cross-process
fd move, any vendor pool, any supervisor binary.

## 5. Decision dependencies

A1 gates M3 shape; A2+A4+A6 gate M5; A3 gates M4 (and the Cargo.lock leg); A5 gates M6
existence; A7 gates a minimal routing leg inside M2/M3 (or its supersession record).
New D-entries minted at M0 land in `decisions.md` with re-review triggers per
the lesson-memory format. No whitepaper/architecture edits without a §9/§10 delta row.

## 6. Risks and rollback

| # | Risk | Mitigation / rollback |
|---|---|---|
| 1 | Process zombies / fd leaks make CI flaky | shared reaper testutil; every spawn test asserts wait()+fd-count before/after; leak-check in M4 gate |
| 2 | SIGKILL teardown loses plugin-side state | graceful STOP message + timeout only; timeout value = named constant, no magic (coding-style) |
| 3 | Dual-process nondeterminism vs zero-sleep discipline | A1 verdict: supervisor-driven clock; if the verdict lands Real-clock, M3 acceptance relaxes explicitly in writing, not silently |
| 4 | iceoryx2 API churn between pins | D25 exact-pinned tag + feature off default; dummy path keeps the whole suite green if the vendor leg breaks |
| 5 | Freeze hearing premature | A4 rule: freeze ONLY what the dual-placement suite actually exercised; everything else gets a named re-open trigger |
| 6 | Scope creep toward bindings/compositor (Stage-3+ magnets) | §0 Out list quoted at every milestone close, Stage-1 style |

## 7. Non-goals (one breath)

Safety certification surfaces, remote/multi-host transport, OTA, polyglot bindings,
UI composition, wasm. These have homes in §12 stages 3-5; Stage-2 touches none.

## 8. Definition of Done (Stage-2)

1. Same contract suite green under BOTH placements (dl + exe) in one ctest run; D17
   test-2 exe leg green with the A1 clock verdict honored.
2. Crash injection: host survives, FAILED + restart-policy observable via counters/events.
3. iceoryx2 pool behind the feature; default CI unchanged (dummy), D69 re-verified by
   the fd/pointer-free public seam test.
4. Freeze-hearing record + new D-entries for every A-item; §10 Out list updated with a
   delta row per closed item.
5. check.sh + demo-stage2.sh green, coverage per A6 outcome, all memories synced,
   PolyOrch pin fresh, everything pushed once the M0 push decision is given.
