# Stage-2 Plan — Process Placement, Wire Transport, Zero-Copy First Entry

> Status: call sheet A1-A7 USER-ADJUDICATED one by one 2026-10-16 (verdicts: D74-D80);
> plan adjusted accordingly below. Execution gate remains CLOSED until maintainer go.
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
parallel_ok routing: SUPERSEDED per A7/D80 - mechanism deferred, declaration field
gains a typed deferral at the hot-swap door (inside S2-M2/M3); D8 triggers stand.
Recorder/replay (D16) and CLI (D17/D23): DEFERRED per A5/D78 - the Stage-2
scheduling superseded with an explicit record; lanes S2-M6a/b struck from this stage.

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

### Verdicts (2026-10-16, user-adjudicated one by one)

- **A1 = A** (D74): supervisor drives the Sim/Step dial over the control socket on the test/replay track, timeout guard; production Real. D17 test-2 lands as canon wrote it.
- **A2 = A** (D75): the exe dual run IS the declined external-consumer round; friction log feeds the hearing; true-third-party trigger stays alive.
- **A3 = A amended** (D76): vendor-neutral feature name `vendor-shm`; factory confined to one module + one dep line; default-purity check + feature CI lane + dummy/vendor equivalence suite; core std-only.
- **A4 = A** (D77): hearing after the suite, freeze door-by-door (only exercised doors welded); the three IOUs + ABI bump policy are agenda items with named dates/triggers.
- **A5 = B** (D78): recorder and CLI both deferred out of Stage-2 by explicit supersession; no mcap entry; re-activation triggers recorded.
- **A6 = A** (D79): 80% hard gate workspace-wide at S2-M5, default features; codegen 72.7 debt paid down or exempted at the hearing; anti-gaming clause.
- **A7 = C** (D80): routing superseded; typed deferral replaces silent ignore at the swap door (~100 lines, joins S2-M2/M3); demand data becomes the reactivation brief.

## 2. Milestone outline

| # | Milestone | Deliverable | Anchors | Gate / evidence |
|---|---|---|---|---|
| S2-M0 | Kickoff hygiene | PARTLY DONE 2026-10-16: call sheet A1-A7 walked and verdicts minted as D74-D80 (numbering gate widened same day, negative proof observed); PolyOrch pin freshness checked clean (5945f2a == tip); REMAINING: push decision (OPEN - maintainer) and supervisor bin name chosen at M2 start | §0, this file | verdicts recorded = done except maintainer-side push call |
| S2-M1 | ProcessExecutor + supervisor loop | LANDED 2026-10-16 (explicit go; agent-built, orchestrator re-verified 116/0 + check.sh + demo regression): exe arm spawns std::process, socketpair control at fd-0 stdin (01 §1 socketpair/stdio shape; arbitrary-fd lands M3), HELLO = ABI v1 + strict self-report (M7 precedent), Supervisor poll/revive drives crash->FAILED->D3-budget->respawn through existing doors only (zero phantom reloads asserted), graceful STOP -> STOP_KILL_GRACE -> SIGKILL; envelope {len,type,payload} with clock variant reserved by comment for D74/M3; counters process_crashes/restarts folded CountersView; kernel now placement-agnostic (executors police their own kind). ESCALATE adopted: E1 host loop drives poll() (M2/M3), E2 fd-0 convention revisit at M3 SCM_RIGHTS, E3 crash code -> D77 agenda IOU-4. | D3, modules/01 §1-4, D74/D80 | first-reds quoted (E0432 unresolved Supervisor/Revive/STOP_KILL_GRACE + E0599 missing variants); 8 new tests incl. fd-hygiene + zero-zombie; independent re-run: 116 passed, clippy -D warnings, check.sh green, STAGE1-DEMO PASS |
| S2-M2 | modukit-runtime + shadow-runner | new workspace member `crates/modukit-runtime` (runner bin per the modules/01 §2 command shape `modukit-run <plugin.so> --fd=N`; supervisor bin name has no canon - chosen and recorded at M0); runner dlopens the same fixture cdylib the dl executor would load, pumps identical ABI v1, registers over the control socket (the `--fd=N` inherited end); example-plugin gains an exe-placement twin run in the SAME suite (§10 "contract suite structured for later dual-placement runs" becomes live) | D23 templates, modules/01 §2 | first-red runner-connect handshake; dual-run table: every contract-suite case green under both placements |
| S2-M3 | Wire transport + cross-process bus | unix-socket SCM_RIGHTS fd passing (documented-unsafe libc shim per the M5 memfd_create precedent, std-only kept), TL-versioned frame on the wire, bus publish/subscribe across process boundary with QoS fan-out tested against a real wire (D10 trigger); D74 clock verdict lands here (supervisor-driven dial + due-jobs return + timeout guard); D80 typed deferral wiring joins here at the swap door | D9/D10/D11/D20/D74/D80, modules/07 | first-red fd-pass roundtrip between two test processes; QoS stress: drop-counters visible on both sides, late-join across the wire; D17 test-2 exe leg green (step-replay content equality across placements, dialed by the supervisor) |
| S2-M4 | iceoryx2 first entry | `vendor-shm` neutral feature on modukit-transport (D76: factory name confined to one module + one pinned dep line, never in the public seam); ported pool + porter behind the D11 backend trait; dummy double stays the CI default; frames cross only as pool+descriptor handles (D69 honored); equivalence suite dummy-vs-vendor per D76; vendor_lock amended to default-purity + feature CI lane | D11/D25/D69/D76 | first-red: pool roundtrip over vendor backend equals dummy-backend semantics; amended vendor_lock green on default features; feature-lane CI green; memory/fd leak check after 10k roundtrips |
| S2-M5 | Freeze hearing + Stage-2 acceptance | run the dual-placement suite as the external-consumer validation round (D75), then the hearing (D77): C-ABI export list door-by-door, D51 origin ruling, integrity codes, hint(), crash-code ownership (IOU-4 from S2-M1 E3), ABI bump policy; coverage 80% hard gate measured workspace-wide in the same run (D79); `scripts/demo-stage2.sh` PASS gate + falsifiability probe; §10/§12 delta table re-audit vs architecture.md | D50/D51/D75/D77/D79, §10 | demo-stage2.sh prints STAGE2-DEMO PASS; tampered-runner probe REDs the gate; coverage >= 80% default-features (or recorded exemption at the hearing); check.sh + ctest regressions green |
| ~~S2-M6a/b~~ | ~~Optional lanes~~ | STRUCK 2026-10-16 per A5/D78 - recorder and CLI deferred out of Stage-2 by supersession record; re-activation triggers live in decisions.md | D78 | - |

## 3. Execution sequence

S2-M0 (nearly closed - push call remains) → S2-M1 → S2-M2 → S2-M3 → S2-M4 → S2-M5.
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

All seven verdicts landed 2026-10-16 (D74-D80);
dependencies now read as facts:
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
