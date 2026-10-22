# Stage-5 — wave-2 ledger target (plan of record)

> STATUS: HEARING HELD 2026-10-21 - the v2 plugin-facing bus door was ruled six-by-A
> on maintainer word ("open the hearing", agenda letter 4); ledger record D96.
> Implementation lanes below are SPEC'd, not open - each needs its own go order.
> The schedule-timer agenda was HEARD and DRAWN 2026-10-21 (D99,
> docs/wave-2-clock-hearing.md); its lane T1 (the door) LANDED the same
> day - row below. T2 (crossing) awaits its own go order.

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

## 2. Landing split (lane shape per the C++-4 risk-split precedent; NOT open)

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

- **T1 - the clock door**: LANDED 2026-10-21. `HostAbiV3` = the byte-identical
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
- Python bindings, live-attach debug tooling depth, JS dual-face product surface -
  all remain armed in `docs/stage-4-plan.md` §6, each needing its own word and round
  (the regen-tool nit on that list CLOSED 2026-10-21, commit 00a5290).

- Debug-reader first landing 2026-10-21: the adjudicated door shipped as a minimal
  reader-only `modukit-debug-reader` self-check bin in `modukit-runtime`. It uses the
  existing `TestHost` data plane, adds no ABI verb or live attach, and is not a
  standalone `check.sh` demo leg. Plan: `docs/wave-2-debug-reader-plan.md`.
