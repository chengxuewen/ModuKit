# Stage-5 — wave-2 ledger target (plan of record)

> STATUS: HEARING HELD 2026-10-21 - the v2 plugin-facing bus door was ruled six-by-A
> on maintainer word ("open the hearing", agenda letter 4); ledger record D96.
> Implementation lanes below were SPEC'd pre-opening; each opened on its own
maintainer word (D94/D96/D97/D99/D100-D104) - the rows carry current state.
> The schedule-timer agenda was HEARD and DRAWN 2026-10-21 (D99,
> docs/wave-2-clock-hearing.md); its lane T1 (the door) LANDED the same
> day; its lane T2 (the crossing) LANDED 2026-10-22 - row below. The §3
> ordinary-publish pump row stands (T2 wired bells only, at the due-funnel).

## 1. v2 bus hearing - the six rulings (all A, walked one by one, plain speech)

1. **Scope** - publish/subscribe + the cross-process topic wire are drawn here;
   schedule-timer is deferred to a hearing of its own (D82's trio: two drawn now,
   the third keeps its §6 slot with a named agenda).
2. **Version door** - a separate `HostAbiV2` table, negotiated by declared ABI major
   at the handshake; `HostAbiV1` stays byte-identical forever. This executes D83's
   own sentence ("appended verbs enter the next version candidate") verbatim -
   no reinterpretation, no first precedent of bending D87.
3. **Transport** - the existing TL control connection carries two NEW frame talkers
   (forward / deliver); no second pipe, no data plane. Zero-copy/K3 triggers stay
   untouched - big-flow creditors were not invented today.
4. **Compatibility** - a capability field tail-added to the HELLO proto message;
   bus talkers are spoken only to capable peers; `WIRE_VERSION` stays 1 (vocabulary
   growth never bumped a transport promise - the K7 note's public claim holds).
5. **Plugin-facing shape** - the mailbox grammar mirroring the host face exactly:
   `publish(topic, qos, bytes)` / `subscribe(topic) -> queue-id` /
   `recv(queue, buf, cap -> need-reported)` / `unsubscribe(queue)`. Zero
   reverse-direction function pointers in the ABI; doorbell ergonomics is
   plugin-side sugar per D63's constrained-sugar tier, never a contract direction.
6. **Wall semantics** - the wall is transparent: same-name = same-topic, publisher
   self-echo included, retained-latest snapshot delivered on subscribe across the
   wall, per-mailbox drop counters stay honest. Verification grammar = one
   assertion suite run wall-in AND wall-out (the dual_placement precedent), so a
   semantic discount cannot hide in only one column.

## 2. Landing split (lane shape per the C++-4 risk-split precedent; header wording
predates the openings - see the rows

- **N1 - the door**: LANDED 2026-10-21. `HostAbiV2` (v1 prefix byte-identical + four
  mailbox slots; abi.rs 111/0, kernel.rs 14/0, executor.rs 39/0 - pure insertions,
  frozen audit by the orchestrator), plugin.h mirror (52/0), v2 trio parity leg,
  HOST2 ledger rows (47), fixtures busv2/liar + door suite, codegen pluginface emitter
  (C++ Host view, bus forwarders behind a size-guard; golden via banner recipe).
  Numbers orchestrator re-measured: 231 tests / 50 suites, check.sh all green,
  four v1 demo legs PASS, planted reds 4x logged. Scope correction recorded here:
  the HELLO capability field is WIRE-side - N2 owns it (in-process needs no proto).
- **N2 - the crossing**: LANDED 2026-10-21. Exactly two talkers (MSG_FORWARD 10 /
  MSG_DELIVER 11, sub/unsub ride FORWARD's op grammar), capability = optional u32 LE
  tail on the HELLO wire + the proto ledger records Hello{abi..capability@5},
  WIRE_VERSION stays 1. Parent fan-out is the SAME code path as in-process (one bus,
  one fan-out law); child-side shadow queues reuse host_v2 grammar verbatim.
  Orchestrator re-measured: 239/51, five demo legs green incl the new
  demo-stage5-n2 (sabotage RED in own eyes), zero deletions across all frozen files
  (proc_wire 166/0, executor_proc 64/0, host_v2 60/0, bus 48/0, supervisor 8/0);
  runner main.rs +38/-7 (not frozen, pre-declared).
- Done-when per lane mirrors the M-lane grammar exactly: red-planted hooks, ledgers
  and parity move with the table, orchestrator re-measures independently (PIT-14).

### CLOCK lane (D99)

- **T1 - the clock door**: LANDED 2026-10-22 (drafted 10-21; commit d7434af - date unified by doc-audit 2026-10-23). `HostAbiV3` = the byte-identical
  V2 prefix + `timer_schedule`/`clock_now` tail slots (frozen audit by the
  orchestrator: abi.rs 84/0, kernel.rs 59/0, executor.rs 147/0 - pure
  insertions; proc_wire/executor_proc/embed.h zero touch). The five prefix
  slots reuse the v2 trampolines VERBATIM through the shared v3 context (one
  mailbox implementation, mirror law); the per-plugin token register is
  kernel-side new state (authority-law grammar; the REGISTER outlives the
  start call, the table does not), released at stop AND uninstall with
  never-re-fire censuses. The due bell posts at the ONE due-funnel
  (`wiring.rs::post_jobs`) on `org.modukit.timer.<plugin-id>`, payload
  token u64 LE ++ label UTF-8, QoS `Lossy(16)`; host-owned timers keep the
  ring posting untouched; the sabotage lever is read at exactly the bell
  site. Late-join ring replay PINNED as behavior; topic grammar asserted
  before wiring; liar-shape guard cells (false major 3 keeps the landed
  shapes, major 4 stays AbiMismatch) in executor.rs `#[cfg(test)]`. plugin.h
  / parity trio / capability-ledger section D / codegen v3 face all move
  with the table (golden idempotent x2). EXE INTERIM, PINNED NOTE: the
  child runner loads through the SAME shared door - a v3 plugin in an exe
  child schedules on its own D74 clock LOCALLY; its bell reaches child-local
  subscribers only and crosses NOTHING (no capability bit, no bridge
  routing - T2 scope by D99 R6). Orchestrator re-measured: 254 tests /
  55 suites (was 239/51), planted red observed in the orchestrator's own
  eyes (both bell legs FAILED under sabotage, restored 6/6), check.sh all
  green incl the six demo legs. Plan: docs/wave-2-t1-clock-door-plan.md.
- **T2 - the crossing**: LANDED 2026-10-22 (maintainer open-T2 order; plan
  docs/wave-2-t2-clock-crossing-plan.md). `HELLO_CAP_TIMER_V2 = 2` rides the
  landed caps tail (WIRE_VERSION stays 1; no proto/fdset touch, regen-diff
  byte-clean); the runner declares BUS|TIMER, with ONE named lever
  (DEMO_WAVE2_CLOCK_SABOTAGE=1) dropping only the timer bit. The executor
  parses timer_caps beside bus_caps and notes the fan-out's bell-target set
  at the child's register site; `retire` sweeps it with the window. The
  crossing dial (`executor_proc::dial_child_crossing`, the supervisor's dial
  rewired to it) absorbs the child's MID-DIAL FORWARD bells into the
  fan-out: existing FWD_PUB/DELIVER frames, TWO new call sites at the
  due-funnel (`post_timer_bell`'s outlets), zero new frame types, no general
  Bus::publish pump (§3 row untouched). AMENDMENTS NAMED (D83
  tail-add-by-record): bell capacity unified to `embed_codec::LOSSY_RING`
  (64 - the wire vocabulary's decode of qos code 2 beats the hearing draft's
  round 16; the amendment names itself in the wiring.rs module doc) and the
  bell payload grew the boot generation (`gen u64 LE ++ token ++ label`,
  D99 §9 pin, the cross-revive alias kill) with T1's decode_bell / late-join
  / fixture docs updated atomically (gen asserted by self-equality only,
  never literally). FINDINGS (the plan's stop clause - reported, not worked
  around): the parent->child LIVE pump leg is NOT end-to-end observable
  today (a fanout-owning Supervisor kernel hosts no dl plugin producer;
  the child's wall interest lives only inside the start window; Kernel is
  not Send/Sync so no concurrent dial exists), and the leg-OUT child-local
  subscriber mirror has no post-start observer either; the pump method
  landed with frame-level proof in `bus_wire.rs::tests` and the REAL bell
  site in `wiring.rs::bell_crossing_tests`. Evidence: leg OUT green through
  the real runner + supervisor dial (parent-bus decode, DUE-agreeing token,
  late-join byte-equal); the timer-cap lever reddens the leg (observed, the
  suite's own column) and the demo (observed RED + restored green); demo
  leg SEVEN `scripts/demo-wave2-clock.sh` landed in check.sh; ledger sweep
  SIX->SEVEN done (AGENTS x2 + scripts line, status.md gate row, README).

- **PYTHON volume (D100)**: LANDED 2026-10-23 (go order 2026-10-22). M0 one-line
  cdylib lane (staticlib precedent); M1 generated-ctypes floor over the 16 rung-0
  verbs - goldens + pins + eighth demo leg + lever, step-0 measured RED on protobuf
  (demo-only fixture codec, floor untouched); M2 sup rung + byte-FIDELITY pinned to
  host_sup_demo.cc:139 shared bytes + /proc-census orphan check, ninth leg, two
  levers all red-proved by the orchestrator's own eyes; ledger's Python column
  flipped planned->generated for all 24 rows. The S3 equal-capability line now
  holds for C++ AND Python on both rungs. Plan: docs/wave-2-python-volume-plan.md §7.
- **Volume ACCEPTED 2026-10-23 (D101)**: four-point DoD green - goals re-measured by
  the orchestrator, freeze audit zero-touch, coverage via the NEW pinned recipe
  (`scripts/coverage-boundary.sh`: core 86.18 / transport 88.29 / codegen 85.74 - the
  first boundary run with a machine-checkable census), docs swept incl the
  EINTR-sentinel hardening.

- **JS volume (D102)**: LANDED 2026-10-23 (go order same day). M0 martial probe GREEN -
  napi builds offline from cache, node loads the .node; the scratch probe settled the
  type-shape quirks first (no tuple returns, no u64 crossing - create_object + i64 policy).
  M1 generated bridge + facade over the 16 rung-0 verbs - tenth leg JS-DEMO PASS (23 checks,
  lever red at scene [2], restored), node:test pin_test 4/4, AND the supply-chain fence
  amended per its own words: the napi tree admitted with exact probe pins + one-door
  confinement (set-leg was observed red before the admission - C0 honored). M2 sup rung (8)
  + byte-FIDELITY against the SAME pinned constant as cpp (host_sup_demo.cc:139) and python
  + /proc-census orphan proof - eleventh leg, both levers red-proved. Ledger JS column:
  24 rows generated. Face-1 (JS-authored plugins) stays the script-host separate case.
  Commits 687e7ea..87aae6e; plan docs/wave-2-js-volume-plan.md. ACCEPTED 2026-10-23
  (D103): the js region ruled bin-like metric-EXEMPT (0.00% displayed, vote skipped -
  the generated face's proofs are the two node-side legs + pin_test; hand-written logic
  in that crate would void the exemption and re-arm the region).
- **Benchmark volume hearing DRAWN 2026-10-23 (D104)**: MediaServo discipline tier
  adopted (E/B/B/A, walked one by one). Three batches: wall retirement (CTEST-keyword
  ctest minting, supplier-private reads deleted), reconciliation + config pins (shim
  floor, --self-test hooks, lane-table reconciler, clippy/rustfmt/toolchain files),
  environment authority (pixi declares interpreters with loud fallback; bootstrap
  thin wrapper on both platforms; presets; deny license+bans legs). Format-leg
  increment DECLINED as recorded deviation; delivery plane stays trigger-gated.
  plan APPROVED (Momus OKAY + calibers A/A/A); BATCHES 1-3 LANDED 2026-10-23;
  ACCEPTED 2026-10-23 (D105; fresh boundary green + same-day doc-audit swept);
  debts on plan §7 ledger (egress switches, Windows first-run, mirror slot).

- **Plugin door (D106) batches 0-3 LANDED (2026-10-23 python leg twelve; 2026-10-27
  node leg thirteen)**: batch 0 the install-door exemption (runner-bearing non-.so entries
  skip the dlopen probe, identity defers to the spawn-time HELLO cross-check -
  script_door.rs); batch 1 the wire ledger `contracts/modukit_wire.h` + 14 golden
  frames + the fast `wire-parity` leg; batch 2 the hand-written python skin
  `bindings/py/plugin_runtime.py` (all 11 kinds, 3.8 floor, stdlib) with fixtures
  `test-plugins/py/py_echo.py` / `py_liar.py` and leg twelve
  `scripts/demo-wave2-plugin-py.sh` (conformance-before-launch, FIDELITY +
  BUS ROUNDTRIP + liar typed error + orphan census, lever red-proved inline);
  batch 3 the hand-written ESM node skin `bindings/js/plugin_runtime.mjs`
  (BigInt u64, net.Socket fd-0 full duplex, pure stdlib) with fixtures
  `test-plugins/js/js_echo.mjs` / `js_liar.mjs` and leg thirteen
  `scripts/demo-wave2-plugin-js.sh` (the python leg's five-stage mirror, lever
  red-proved inline). Plan: docs/wave-2-plugin-door-plan.md.

## 3. What this file deliberately does NOT decide

- Parent->child live-push for ORDINARY publishes (named by the D99 CLOCK
  hearing review, 2026-10-21): a parent-originated bus publish never pumps
  down today - `ParentFanout.deliver` fires only on inbound-child FWD_PUB
  (:142) and subscribe-snapshot (:167). The D99 timer due-site wiring (T2)
  carries the clock case with its own planted red; the general pump waits
  for a second ordinary-publish consumer - named here, not a silent drop.

- Timer signatures (deferred by ruling 1 - named future agenda, not a silent drop);
- STEADY-STATE crossing: mailbox authority rides the start window (01 section 3 rule 2),
  so post-start bus APIs and live child-to-child overlap are a FUTURE agenda the moment
  a real consumer needs them - named here, not a silent drop (fan-out already serves
  all interested children; parent-side behavior is unit-pinned);
- live-attach debug tooling depth - remains armed in `docs/stage-4-plan.md` §6, needing
  its own word and round (Python bindings LANDED 2026-10-23; JS face 2 LANDED 2026-10-23
  above; JS face 1 = script-host separate case)
  (the regen-tool nit on that list CLOSED 2026-10-21, commit 00a5290).

- Debug-reader first landing 2026-10-21: the adjudicated door shipped as a minimal
  reader-only `modukit-debug-reader` self-check bin in `modukit-runtime`. It uses the
  existing `TestHost` data plane, adds no ABI verb or live attach, and is not a
  standalone `check.sh` demo leg. Plan: `docs/wave-2-debug-reader-plan.md`.
