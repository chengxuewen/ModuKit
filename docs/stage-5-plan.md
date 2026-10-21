# Stage-5 — wave-2 ledger target (plan of record)

> STATUS: HEARING HELD 2026-10-21 - the v2 plugin-facing bus door was ruled six-by-A
> on maintainer word ("open the hearing", agenda letter 4); ledger record D96.
> Implementation lanes below are SPEC'd, not open - each needs its own go order.
> The schedule-timer agenda stays armed in `docs/stage-4-plan.md` §6.

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

- **N1 - the door**: `HostAbiV2` (append-only in abi.rs; existing lines zero-touched),
  HELLO capability field (proto tail-add + fdset regen-diff), the four mailbox verbs
  for in-process (dl) plugins, codegen v2 face + goldens, fixtures + unit gates
  (parity mirror grows a v2 trio, tripwire, capability-ledger rows).
- **N2 - the crossing**: the two wire talkers in proc_wire/proc_supervisor + the
  child-side shadow queue, subscribe-snapshot + self-echo frames, the wall-
  transparency dual-suite (same assertions in-process vs across), demo legs
  (planted-red sabotage hooks, C0), fresh-tree ctest counts (PIT-17).
- Done-when per lane mirrors the M-lane grammar exactly: red-planted hooks, ledgers
  and parity move with the table, orchestrator re-measures independently (PIT-14).

## 3. What this file deliberately does NOT decide

- Timer signatures (deferred by ruling 1 - named future agenda, not a silent drop);
- Python bindings, debug reader CLI, JS dual-face product surface, regen-tool nit -
  all remain armed in `docs/stage-4-plan.md` §6, each needing its own word and round.
