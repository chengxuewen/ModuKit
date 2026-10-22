# Wave-2 T2 plan - the clock crossing (wall legs + demo)

> STATUS: LANDED 2026-10-22 - the maintainer's open-T2 order executed;
> the two landing amendments (LOSSY_RING unification, boot generation
> field) and the two observation-surface findings (parent->child LIVE pump
> leg, child-local subscriber mirror) are recorded in docs/stage-5-plan.md
> CLOCK lane T2, docs/wave-2-clock-hearing.md §9 and
> tests/clock_crossing.rs's header. Originally:
> REVIEWED OKAY 2026-10-21 (Momus; six grills verified against the
> tree, zero blockers - the review's anchor line numbers serve as worker
> pointers). Execution delegated on the maintainer's open-T2 order.
> Contract: D99 R5-A (drawn B,A,A,A,A,A at docs/wave-2-clock-hearing.md §8) -
> "local schedule, due routed through the EXISTING frames both ways; zero new
> frame types, zero new talkers, TWO new call sites". T1 (door) landed
> 2026-10-21 (stage-5-plan CLOCK row).

## 1. Scope

Wire-side crossing for timer bells, both directions, riding landed frames:

- `proc_wire.rs` (FROZEN - pure insertions): `pub const HELLO_CAP_TIMER_V2:
  u32 = 2` + doc line (bitmask field already exists in `Hello.caps` -
  `parse_hello_caps`/`hello_payload_caps` unchanged shape, WIRE_VERSION stays
  1, regen-diff untouched: caps is raw-frame grammar, the proto ledger already
  records `capability@5`).
- `crates/modukit-runtime/src/main.rs`: the runner's HELLO declares
  `HELLO_CAP_BUS_V2 | HELLO_CAP_TIMER_V2` when bus is on (mirror the landed
  `bus_on` pattern at :269); ONE named demo lever (mirror how bus_on is read)
  drops only the TIMER bit - `DEMO_WAVE2_CLOCK_SABOTAGE=1` semantics:
  "timer capability off".
- `executor_proc.rs` (FROZEN - pure insertions): parse `timer_caps` at the
  same handshake site that sets `bus_caps` (:345 mirror); at child
  register/retire points, tell the fanout (`note_timer_child(id)` /
  retire-purge). Existing gates stay: no-bus-cap child is never spoken to.
- `bus_wire.rs` (not frozen): `ParentFanout` gains a timer-children set and
  `pump_local_bell(&self, topic, qos_i32, value)` - the SAME drain-and-deliver
  grammar `on_publish` already runs (proxy Receiver pull + DELIVER frame per
  value), restricted to timer-capable interested children. `retire()` purge
  extended. No new frame types: DELIVER is the existing one.
- `kernel.rs`/`wiring.rs` (insertions): kernel field
  `parent_bells: Mutex<Option<Arc<ParentFanout>>>` + `attach_parent_bells()`
  setter; the supervisor's existing fanout construction
  (`proc_supervisor.rs:119`, not frozen) also attaches the handle to the
  kernel. `post_timer_bell` (T1) grows its two outlets AFTER the local publish,
  still behind the plugin-token register check:
  - child side: `if let Some(b) = bridge() { b.publish(topic, 2, &payload) }`
    (the same call the mailbox door verb makes - FWD_PUB frame);
  - parent side: `if let Some(f) = &self.parent_bells { f.pump_local_bell(...) }`
    (the named landed absence from the D99 hearing review - this is its fix
    site, scoped to bells only).
  Host-owned timers still never bell. Locks stay leaves (plugin_timers,
  fanout maps, bridge mutex): no plugin-table lock is held during
  post_jobs/dial paths (one-thread host law).
- QoS capacity amendment (named, D83 tail-add-by-record): T1 posted bells at
  the literal `Lossy(16)`; the wire vocabulary decodes code 2 as
  `Lossy(LOSSY_RING = 64)` (embed_codec.rs:50,110). Landing unifies to
  `QoS::Lossy(codec::LOSSY_RING)` everywhere so wall-in and wall-out rings
  behave identically (ruling 6 law beats the hearing draft's round number;
  the amendment row goes to stage-5-plan + hearing §9 pointer, contract file
  stays as drawn).
- Generation field (D99 §9 "pin at landing"): `clock.rs` gains
  `boot_gen: u64` sampled from `SystemTime` (UNIX_EPOCH, ns) at `Clock::new` -
  a boot tag only, never dial semantics; bell payload becomes
  `gen u64 LE ++ token u64 LE ++ label UTF-8`. Kills the cross-revive alias
  (old ring entries carry the dead kernel's gen; fresh fires carry the new).
  T1's `decode_bell`/late-join test/fixture updated to decode gen (opaque
  equality: same-kernel entries share gen; never assert literal values).
  step_replay untouched (host-owned timers, no bell).
- `tests/clock_crossing.rs` (new, real-runner harness per process_executor/
  dual_placement precedents): leg OUT = exe-plugin bell reaches a parent-bus
  subscriber (gen+token+label decoded; child's own local subscriber also sees
  it - mirror law); leg DOWN = child FWD_SUBs a parent-plugin bell topic and
  receives the DELIVER after the parent's dial (assert surface from the named
  precedents - fixture walk-report like busv2, or frame-level like dial tests;
  acceptance is "the leg goes RED under the named lever", mechanism is the
  implementer's pick from the named set); legacy pin: a child declaring only
  BUS_V2 (no timer bit) receives NO bells (asserted, not assumed).
- Demo leg (house grammar): `examples/rs/wave2_clock_demo.rs` (n2 template:
  build fixture cdylib + runner through cargo, staged manifest, real host +
  supervisor; scenes: dl plugin schedules on the parent, exe plugin schedules
  in the child, dial, assert bells cross BOTH ways, `STAGE-WAVE2-CLOCK PASS`
  line; `DEMO_WAVE2_CLOCK_SABOTAGE=1` documented at the ONE named point);
  `scripts/demo-wave2-clock.sh` (40-line n2 template, SKIP-when-no-toolchain,
  RED grammar); `check.sh` gains the SEVENTH demo leg (same PASS/SKIP block).
- Ledger sweep (current-state claims only): "SIX demo legs" -> SEVEN in
  AGENTS.md x2 + status.md gate row + README sentence; stage-5-plan CLOCK lane
  gains the T2 row (incl the LOSSY_RING + gen-field amendments); hearing §9
  pointer flips T2 -> LANDED with the amendment notes; T2 plan banner flips.

## 2. Non-scope

- The GENERAL parent->child live-pump for ORDINARY publishes - the
  stage-5-plan §3 row stays armed (this plan touches bells only, at the
  due-funnel; the pump method lives on ParentFanout and is called from the
  bell site, NOT from Bus::publish).
- No new frame types, verbs, proto messages, embed-face changes, table
  changes. v3 door (host_v3) untouched - the crossing is kernel/wire-side.
- No plugin-side dedup library (gen field is grammar; riding it is plugin
  sugar per D63).
- Cross-revive TEST matrix beyond the legacy-pin (restart census stays with
  the existing restart.rs family; the gen field's purpose is documented by
  the alias comment + payload test, not by a new crash harness).

## 3. Verification / done-when

1. New tests green (counts from redirected logs, PIT-9); T1 suite still green
   after the gen/qos amendments (decode helper updated atomically).
2. Three reds observed IN THE ORCHESTRATOR'S EYES: leg OUT red + leg DOWN red
   under the runner lever; demo script RED under `DEMO_WAVE2_CLOCK_SABOTAGE=1`
   with no PASS line; all restored green.
3. Frozen audit: `git diff --numstat` deletions=0 on abi.rs/kernel.rs/
   executor.rs/proc_wire.rs/executor_proc.rs.
4. check.sh all green incl SEVEN demo legs + regen-diff (fdsets unchanged -
   proof the wire schema truly wasn't touched) + parity trio.
5. Suite census: coverage-cited numbers move only at the next boundary (no
   llvm-cov run in this lane); test inventory rows updated.
6. Docs land WITH the code (D99 trigger honored: amendments named in
   stage-5-plan row + hearing §9 pointer; contract file not rewritten).

## 4. Risks

- Drain re-entrancy: pump_local_bell from the dial site while a FORWARD
  arrives - the one-thread host law serializes both in the M3b pump loop;
  tests run the dial and the wire read in one thread; a hang is RED by
  construction (first test exercises it).
- QoS unification changes T1's pinned late-join ring depth (16->64): the
  pinned test asserts replay-of-own-firings, not depth - stays green; if a
  depth assertion exists, update it in the same commit (atomically).
- Retire-purge miss class: a parked/parked-forever child must leave the
  timer set (retire() already runs at START ack + death; extend in place).
- boot_gen under SystemTime failure (clock before epoch): saturating/unwrap-
  or-0 pattern per repo error style; value never asserted literally.

## 5. Review checklist for the critic

Contract fidelity to D99 R5/R6 (frames reused, two call sites, zero new
grammar); frozen-file insertion discipline incl the RIGHT files; the
LOSSY_RING amendment is record-named, not silent; gen field kills the
documented alias without touching dial semantics; every wall leg has a
lever with its own eyes; the §3 row (ordinary-publish pump) stays deferred.

## 6. Review result

PENDING - Momus.

## 7. Execution notes (landing round, 2026-10-21)

- **FINDING, dispositioned by the orchestrator (accepted as a structural
  gap, not worked around)**: the parent->child LIVE bell pump is not
  end-to-end observable on today's tree. Three-part derivation, verbatim in
  tests/clock_crossing.rs and examples/rs/wave2_clock_demo.rs headers:
  (1) a supervised kernel boots the `exe` executor and hosts no dl plugin -
  no parent-originated bell producer exists on any tree that also owns a
  fanout; (2) the child's wall interest lives only inside the START window
  (retire at the ack sweeps it - the mailbox authority law, kept); (3) the
  parent dial of its own kernel cannot run concurrently with a child start
  (Kernel holds a `Box<dyn Executor>` trait object - not Send/Sync).
  The pump method still lands PROVEN at every observable level:
  bus_wire.rs::tests (frame-level: grammar, timer-target filter, retire
  purge) + wiring.rs::bell_crossing_tests (the REAL bell site pumping a
  wired fan-out; legacy-pin receive half) + the demo's [S3] in-window
  self-echo (wall-IN direction on the same DELIVER writer). End-to-end
  DOWN needs the §3 neighbor row to unlock (a mixed-placement parent kernel
  or a post-start DELIVER consumer in the runner) - named, not silent.
- QoS capacity: bells unified to `embed_codec::LOSSY_RING` (64) as planned -
  amendment recorded in the stage-5 T2 row.
- boot_gen field: landed as the payload prefix (gen ++ token ++ label),
  self-equality assertions only, per plan.
- Reds observed by worker AND orchestrator: door suite 3 FAILED under
  MODUKIT_TIMER_BELL_SABOTAGE, crossing suite 2 FAILED, demo scene [S4]
  RED with no PASS line; all restored green.
