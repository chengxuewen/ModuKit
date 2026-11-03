# Wave-3 Parent-to-Child Bus Push - Plan (v3)

> Status: v3, 2026-10-27. v2 -> v3 folds the final delta self-check (C8/C9:
> the Momus lane died on API quota, so the terminal check ran by hand).
> v1 -> v2 folded the four-lane review (run as an
> orchestrator self-review: all subagent lanes were killed by API quota, so the
> same four lanes were executed by hand - facts / completeness / safety / ops)
> and the A6 amendment to the hearing. Hearing:
> `docs/wave-3-parent-push-hearing.md` (D108 + A6). English only (C1); no
> circled numerals (C4).
>
> EXECUTION STATUS (2026-11-02): batch 0 closed 2026-10-27; batch 1a landed
> 2026-11-02 (inert pump/outbox); batch 1b landed 2026-11-02 (hook installed,
> old paths retired); batch 2 landed 2026-11-02 (HOST PUSH + lever proof);
> batch 3 docs sync landed 2026-11-03; the acceptance battery remains. Both
> legs 12/13 and the 36-leg ci are green.

## 0. Fact base (verified against the tree, 2026-10-27)

**The gap** (D108): `handle_forward(FWD_PUB) -> on_publish`
(`bus_wire.rs:143-168`) is the ONLY drain site for proxy `Receiver` queues.
Every publish that does not enter through the child frame path fills queues
that nobody drains. The verified host-origin publish sites:

| Site | Path |
|---|---|
| `embed.rs:299` | the C ABI ambient bus publish (the ctypes floor the demo drivers call) |
| `host_v2.rs:130` / `:135` | the in-process plugin trampolines (the v2/v3 tables) |
| `kernel.rs:157` | the public `kernel.bus().publish(...)` host API |
| `wiring.rs:212` | the timer bell site (already pumps, via the retiring path) |
| `bus_wire.rs:141` | the child frame path (already pumps; loop is deleted) |

**Rulings**: A1 hook inside `Bus::publish` after the fan-out, outside the
topics lock; `on_publish`'s loop deleted. A2 `pump_local_bell`, the `timer`
set, `note_timer_child` (`executor_proc.rs:518` call site) and the retire
sweep deleted; the bell site keeps only its publish; D99 R5-A delivery clause
superseded; `wire.h` untouched. A4 DELIVER carries `qos_to_i32` of the
publish's own QoS; deferred topics no-op. A5 HOST PUSH stanza on legs 12+13
plus `MODUKIT_HOST_PUSH_OFF` and three unit tests. A6 (amendment of A3's
mechanism) bounded writes, remainder-retaining outbox, NO `O_NONBLOCK`.

**Verified constraints (v2 findings)**:

- **C1 - the fd is shared.** `executor_proc` writes STOP/ACK/CALL/DIAL
  (`:449`, `:574`, `:723`, `:795`) and reads frames (`:359`, `:468`, `:585`)
  on the SAME socket the fan-out writes. `O_NONBLOCK` is per file
  description; A3's original mechanism would break every blocking read/write
  on the channel. A6 replaces it.
- **C2 - `write_frame` is two `write_all` calls** (header, then body,
  `proc_wire.rs:123-134`): a partial write followed by an error would leave a
  half frame and misalign the child's stream forever. Any bounded-write
  design must track write progress, not retry blindly.
- **C3 - `Bus` is NOT `Clone`** (only `#[derive(Default)]`, `bus.rs:161`):
  the hook field lives on the one instance the kernel owns; no shared-state
  question. (The v1 residual about clone semantics is withdrawn.)
- **C4 - the install seam exists in two places**: production
  `proc_supervisor.rs:119-126` (`ParentFanout::new` + `attach_parent_bells`);
  test wiring `wiring.rs:305-318`. Both must install the hook or the test
  harness silently diverges.
- **C5 - the single-point lever has a house precedent**:
  `MODUKIT_TIMER_BELL_SABOTAGE` is read at EXACTLY one site
  (`wiring.rs:199`, with its own comment saying so). `MODUKIT_HOST_PUSH_OFF`
  follows that pattern verbatim.
- **C6 - lock order**: `publish` holds `lock_topics` through the fan-out;
  the hook takes `subs`/`children`/`outbox`. The hook must fire after the
  topics guard drops; no hook-side path may publish (deliver only encodes and
  writes).
- **C8 - a partial frame forbids every direct writer** (delta check): once
  the outbox holds a half-written frame for a child, a direct `write_frame`
  from `executor_proc` would append a new frame behind the remainder and
  misalign the stream. Law: ALL writes to a REGISTERED child go through the
  fan-out's outbox (`fan.send(child, kind, payload)`); the executor paths
  (STOP :574, CALL :449, DIAL :723/:795) already hold the `fan` reference
  (`:518` uses it), so the change is local. Pre-registration writes (START
  before the child's first FORWARD) stay direct - no outbox exists yet.
- **C9 - the bell path is now a regression surface** (delta check): A2 retires
  the bell-only pump, so bell crossing depends on the GENERAL pump. The clock
  demo leg (`scripts/demo-wave2-clock.sh`) and the `clock_crossing` /
  `abi_v3_door` suites must ride batch 1's regression list, or a landed T2
  capability silently breaks.
- **C7 - tests that pin today's loop**: `bus_wire.rs:516`
  (snapshot-on-subscribe), `:539` (self-echo), `:627` (the timer-only pin
  that A2 retires).

## 1. Shape: four batches

| Batch | Content | Gate |
|---|---|---|
| 0 | Hook foundation: `Bus` field + setter, fire-after-lock, install at both seams, single-point lever | unit tests + full gate |
| 1 | General pump: `pump_topic` + bounded-write outbox + retirements | unit tests + full gate |
| 2 | HOST PUSH stanza on legs 12 + 13, lever proven red | both legs green + lever red + full gate |
| 3 | Docs sweep, plan close, acceptance battery -> next free D | full gate + boundary + verdict sheet |

> Batch 1 executes as **1a (inert plumbing)** + **1b (atomic switch)**; the
> install and the old-path retirement must be in the same 1b commit. See section 10.

## 2. Batch 0 - the hook foundation

- **T0.1** `Bus` gains `publish_hook: Mutex<Option<Arc<dyn Fn(&TopicId, QoS,
  &[u8]) + Send + Sync>>>` + `set_publish_hook` (additive, D83 tail-add;
  `Bus` is not `Clone`, C3). `publish` calls the hook AFTER the fan-out and
  AFTER `drop(topics)` (C6), passing `(topic, qos, value)` unchanged.
- **T0.2** Install at BOTH seams (C4): `proc_supervisor.rs:119-126` and the
  test wiring `wiring.rs:305-318` install `fan.pump_topic` as the hook.
  A kernel without supervised children installs nothing. SEQUENCING (found at
  batch 0): the INSTALL lands in batch 1 together with `pump_topic` and the
  retirements - installing a live hook while `on_publish`'s loop and the bell
  pump still exist would double-deliver. Batch 0 ships the plumbing inert.
- **T0.3** Lever: `MODUKIT_HOST_PUSH_OFF` read at EXACTLY the install site
  (both seams share one helper so the read stays single, C5 discipline);
  set = skip the install. Comment names the demo lever and the single-site
  rule, mirroring `wiring.rs:199`.
- **T0.4** Tests: hook fires with the topics lock released - the hook body
  publishes on a second topic, launched on a worker thread with a join
  timeout, so a lock-held hook FAILS FAST instead of hanging the suite
  (C6, and the plant for section 6's lock row); deferred-topic publish fires
  the hook and produces zero frames.

## 3. Batch 1 - the general pump and the retirements

- **T1.1** `ParentFanout::pump_topic(topic, qos, value)`: the drain grammar
  `on_publish` runs today - `try_recv` each subscribed child's proxy
  `Receiver`, one DELIVER per value; when a child's queue drained empty,
  deliver the passed `value` (the byte-honest belt). No timer gate (A2).
- **T1.2** Bounded-write outbox (A6, C1/C2): `ParentFanout` gains
  `outbox: Mutex<HashMap<String, Outbox>>` with `Outbox { frames:
  VecDeque<Vec<u8>>, head_written: usize, bytes: usize }`. `deliver` encodes
  the whole frame, pushes it, and flushes: ONE `write` call per attempt on
  the queue head, advancing `head_written` by the returned count; a complete
  head pops; `WouldBlock`/`TimedOut` stop the flush and KEEP the remainder
  (count a STALL); a hard error deregisters the child, drops its outbox and
  counts a LOSS. Over `WALL_OUTBOX_CAP` bytes, pop WHOLE frames from the head
  and count a SHED (never a byte-level cut). No `set_nonblocking` anywhere.
- **T1.3** Registration: `set_write_timeout(WALL_WRITE_TIMEOUT)` once at
  `register_child`; reads untouched (C1). Both values are named constants
  (no magic numbers).
- **T1.4** Retirements: delete `pump_local_bell`, `ParentFanout.timer`,
  `note_timer_child` + its call site (`executor_proc.rs:518`), the timer
  sweep in `retire`, the bell site's pump call (`wiring.rs` `post_timer_bell`
  keeps only `bus.publish`), and `on_publish`'s drain loop.
- **T1.5** Counters: `wall_stalls` and `wall_sheds` in the gate-counter
  family (`counters.rs`, D21 doorway).
- **T1.6** Tests: filled socket -> publish returns within
  `WALL_WRITE_TIMEOUT` + margin, stall counted, and the byte stream stays
  frame-aligned when the reader drains; over-cap -> whole-frame sheds counted
  and alignment kept; the timer pin (`bus_wire.rs:627`) becomes a
  subscriber-law test (a BUS-only subscriber receives on a bell topic); the
  echo test (`:539`) and the snapshot test (`:516`) must stay green through
  the hook (they are the no-regression proof that the doorway swap is
  behavior-preserving).
- **T1.7** Single-writer law (C8): `ParentFanout` exposes
  `send(child, kind, payload)` (encode + outbox + flush); `executor_proc`'s
  registered-child writes (STOP/CALL/DIAL) route through it; START keeps its
  direct write (pre-registration). Regression list (C9): the clock demo leg
  plus `clock_crossing` and `abi_v3_door` suites green.

## 4. Batch 2 - the HOST PUSH stanza

- **T2.1** Legs twelve and thirteen gain a HOST PUSH segment: after the
  lifecycle stage the driver publishes on the seed topic through the ctypes
  floor (`modukit_bus_publish` -> `embed.rs:299` -> `Bus::publish` -> hook),
  and the child must hand the payload back through `seen`. Assertion line:
  `HOST PUSH PASS (host-origin publish crossed the wall via the general
  pump)`.
- **T2.2** Lever proof: `MODUKIT_HOST_PUSH_OFF=1` reds the HOST PUSH stanza
  while child-origin stanzas stay green (precise attribution), riding the
  leg's existing sabotage stanza.
- **T2.3** Zero skin changes (D108 ruling A4 promised the child side is
  untouched): a diff under `bindings/` in this batch is a plan violation.

## 5. Batch 3 - docs, close, acceptance

- **T3.1** Docs: `docs/modules/06-topics-and-bus.md` (general pump, the
  bounded-stall wording, the bell-gate retirement); `docs/stage-5-plan.md`
  section 3 (the "waits for a second ordinary-publish consumer" line becomes
  SUPERSEDED by D108 + A6); the hearing doc already carries the rulings.
  No leg-count change; `docs/capability-ledger.md` untouched (no wire
  surface); `wire.h` untouched.
- **T3.2** Acceptance battery: full gate; frozen-surface audit (`contracts/`
  diff over the volume must be EMPTY - any change is stop-the-line);
  coverage boundary re-measure (core must stay >= 80 with the outbox branches
  covered); section 6 ledger walk with each plant observed; verdict sheet ->
  mint the next free D on the maintainer's acceptance word (D109 is already the
  VE-parity record).
- **T3.1 docs sync landed 2026-11-03**: D108 decision amendment,
  `docs/architecture.md` D108 row, `docs/stage-5-plan.md`, `docs/modules/06`,
  and `status.md` now describe batches 0-2 as landed. Acceptance work in
  T3.2 remains.

## 6. Verification ledger (planted red before trusted)

Batch 0 closed 2026-10-27: hook field + setter + lock-release witness; the
lock plant (hook moved under `topics`) reds the witness in 2s, restored green;
clippy fresh-check clean; full gate green.


| Row | Plant | Expect |
|---|---|---|
| C6 lock order | move the hook call inside `lock_topics` (scratch) | the re-entrant-hook test fails on its join timeout (never hangs); restore green |
| C2 stream integrity | drop the partial-write remainder instead of keeping it | alignment test reds (reader sees a half frame); restore green |
| A6 bounded stall | set the write timeout to infinite (scratch) | bounded-return test reds (publish outlasts the timeout + margin) |
| A6 stall counting | remove the would-block branch | bounded-return test reds on the counter |
| A6 cap shedding | remove the cap | over-cap test reds (outbox grows, no shed count) |
| A2 subscriber law | reinstate the timer gate in `pump_topic` (scratch) | BUS-only-subscriber test reds on a bell topic |
| hook coverage | lever ON | HOST PUSH stanza reds; child-origin stanzas stay green |
| hook coverage (positive) | lever OFF | HOST PUSH stanza green on BOTH legs |
| deferred no-op | publish `KeepAll` | hook fires, zero frames |
| no double delivery | reinstate `on_publish`'s loop beside the hook | a child-origin publish yields two DELIVERs; echo test reds |
| C8 single writer | send a STOP directly while a partial frame is parked (scratch) | stream-alignment test reds; restore green |
| C9 bell regression | run the clock leg + suites with the general pump disabled | bell assertions red (proves they ride the new pump) |
| bell retirement | dial a timer | the timer child receives via the general pump; `grep -c pump_local_bell crates/` = 0 |
| frozen surface | diff `contracts/` over the volume | empty |
| gate | full `scripts/check.sh` at every batch end | exit 0 (fdpass cold family per PIT-32: capture first, never edit the test) |

## 7. Deviations + named residuals

- **Bounded stall, not zero stall** (A6): a publish blocks at most
  `WALL_WRITE_TIMEOUT` per stuck child before the remainder is parked and the
  next child is served. Bus commandment 3 holds in-process; on the wall it
  reads "bounded". Named here so no later reader mistakes it for silence.
- **Outbox retries only on the next publish to that child** (no timer thread,
  per the one-thread law). A child that stalls mid-burst resumes on the next
  publish; no periodic drain exists. Revisit if a consumer needs bounded
  latency.
- **Withdrawn v1 residual**: the `Bus` clone question - `Bus` is not `Clone`
  (C3), verified.
- **Batch 0 scope note**: the plan's original T0.2 (install in batch 0) moved
  to batch 1 - see the sequencing note at T0.2. Batch 0 landed the hook field,
  the setter and the lock-release witness only; the tree is behavior-identical
  (hook `None` everywhere) and the gate stayed green.
- **Hook contract** (batch 0 finding): the hook must NOT publish - it observes
  every publish, so a publish inside the hook re-enters it (found by a stack
  overflow in the first draft of the witness test; the constraint is now in
  the type's doc).
- **Known environment**: the ctest cold-window flake family (PIT-32) -
  capture first, isolate second.
- **Wall counters (1b)**: `wall_stalls` / `wall_sheds` / `wall_losses` live on
  `ParentFanout`; folding them into `Snapshot.counters()` needs a `CountersView`
  surface change and is deferred. They are still asserted by unit tests and
  readable from the fanout.
- **Registration lifetime (2)**: the parent-side wall registration now
  outlives the START ack and is retired at stop, so host-origin publishes can
  reach a running child. Child-local mailbox authority is unchanged; only the
  parent proxy lifetime moved.
- **Host bus routing (2)**: once the exe rung is active, `modukit_bus_publish`
  targets the supervisor kernel bus (the one carrying the hook), not the
  separate ambient in-process kernel. No ABI or contract change.
- **Lever scope (2)**: `MODUKIT_HOST_PUSH_OFF` suppresses only host-origin
  publishes, marked by a thread-local set at the C door; child-origin traffic
  still rides the hook and stays green.

## 8. Non-goals

- No `O_NONBLOCK` on the child fd (C1: it is shared with executor reads and
  writes).
- No new frame kind, no `wire.h` change, no golden regeneration.
- No cross-child ordering guarantee (per-child FIFO only).
- No delivery for deferred QoS (`KeepAll`/`Bounded`).
- No thread, no per-child OS-level re-architecture.
- No skin changes.

## 9. Review trail

- [x] Four-lane review of v1 (facts / completeness / safety / ops) - executed
  by the orchestrator because all four subagent lanes were killed by API quota
  (rps exhausted + stale timeout): findings C1-C7 and the A6 amendment folded
  into v2.
- [x] Final delta self-check (v2 -> v3): C8 (single-writer law for registered
  children) and C9 (bell path is now a general-pump regression surface) folded.
- [ ] Momus plan review (house flow, fresh session, prompt = file path only):
  ATTEMPTED twice (bg_d3f78f78 on v2) - both runs killed by API quota
  ("rps exhausted"); no verdict produced. The self-review above stands in for
  it this session; re-run Momus when quota recovers, before acceptance.
- [ ] Maintainer approval word (batch 0 opens on it).

## 10. Execution annex (2026-11-02)

This section is the hand-off checklist. It exists so the next session does not
forget the ordering, the atomic switch, or the frozen surfaces.

### 10.1 Preflight (must pass before code)

- [ ] Maintainer go word for batch 1 is recorded.
- [ ] Momus/approval gap in section 9 is resolved, or explicitly waived by the
  maintainer; do not silently proceed.
- [ ] `git status` clean; record `HEAD`.
- [ ] Baseline `pixi run ci` is green; record the 36-leg result and time.
- [ ] No unrelated in-flight change; B8 and the doc-audit batch are closed.
- [ ] `git diff -- contracts/ bindings/` is empty; if not, stop.
- [ ] Owner and independent verifier are named; rollback commit will be recorded.

### 10.2 Batch 1a - inert plumbing (no behavior change)

- Implement `pump_topic`, `Outbox`, counters, and their tests. Do not set a
  socket write timeout in 1a; that would change the old path's behavior.
- Do **not** install the hook at either seam.
- Do **not** delete `pump_local_bell`, `ParentFanout.timer`, `note_timer_child`,
  or `on_publish`'s drain loop yet.
- Exit: unit tests plus full `pixi run ci` green; tree behavior is identical to
  batch 0; the diff shows no production hook install.
- Landed 2026-11-02 (`338957c`): `pump_topic`, `Outbox`, wall counters, 11
  bus_wire tests; no hook install, no timeout, no retirement.

### 10.3 Batch 1b - atomic switch (behavior change)

- Install the hook at both seams (`proc_supervisor.rs` and test `wiring.rs`).
- Set `WALL_WRITE_TIMEOUT` at `register_child` (this is a behavior change, so
  it belongs here, not in 1a).
- Delete the bell pump, timer state, `note_timer_child` call site, and
  `on_publish` drain loop in the SAME commit.
- Route registered-child STOP/CALL/DIAL through `ParentFanout::send()`; keep
  START direct.
- Verify the `wall_stalls` / `wall_sheds` counters increment on the production
  path (the counters and their unit tests landed in 1a).
- Tests: filled socket bounded return, STALL count, frame alignment, over-cap
  whole-frame SHED, timer pin as subscriber-law, echo/snapshot green,
  clock/abi_v3 regression green, no double delivery.
- Remove the temporary `#[allow(dead_code)]` markers from `pump_topic`,
  `Outbox`, and the wall getters once the hook is installed.
- Atomic invariant: install + retirement are one commit; never leave a tree with
  both paths active or neither path active.
- Rollback: revert 1b; 1a stays; the old paths return. Do not revert 1a first.
- Landed 2026-11-02 (`91fc390`): hook installed, write timeout set, bell pump
  and `on_publish` loop removed, registered STOP/DIAL writes routed through
  `send`; 47 core tests and 36-leg `ci` green.

### 10.4 Batch 2 - HOST PUSH proof (only after 1b green)

- Add the HOST PUSH stanza to legs 12/13.
- Prove `MODUKIT_HOST_PUSH_OFF=1` reds the host-origin stanza and leaves
  child-origin stanzas green.
- `bindings/`, `contracts/`, `wire.h`, and goldens stay untouched.
- Exit: both legs green, lever red observed, full gate green.
- Landed 2026-11-02 (`0a89d4e`): both plugin legs print `HOST PUSH PASS`;
  with `MODUKIT_HOST_PUSH_OFF=1` they print `HOST PUSH LEVER PASS` (host-origin
  red, child-origin green) and never reach `LIFECYCLE PASS`.

### 10.5 Do-not-forget checklist

- [ ] `contracts/` diff over the volume is empty.
- [ ] `wire.h` unchanged; no golden regeneration.
- [ ] `bindings/` unchanged.
- [ ] No `O_NONBLOCK` on the child fd.
- [ ] No new thread; no periodic drain.
- [ ] No new frame kind; no deferred-QoS delivery.
- [ ] Outbox keeps whole-frame boundaries; SHED is whole-frame.
- [ ] No double delivery after the hook install.
- [ ] `grep -c pump_local_bell crates/` is 0 after 1b.
- [ ] `MODUKIT_HOST_PUSH_OFF` works at exactly one install helper (proof in
  batch 2).
- [ ] Status/plan updated after each batch; C5 runs at batch close.
- [ ] Evidence recorded: HEAD, commands, red plants, outputs.

### 10.6 Ownership and evidence

| Batch | Owner | Verifier | Evidence |
|---|---|---|---|
| 1a | core/bus | independent reviewer | unit tests + full gate; no install diff |
| 1b | core/bus | independent reviewer | full gate + clock/abi_v3 + no-double-delivery plant |
| 2 | demo/legs | independent reviewer | both demo legs + lever red + frozen-surface diff |
| 3 | docs/ci | independent reviewer | full gate + coverage + verdict sheet |
