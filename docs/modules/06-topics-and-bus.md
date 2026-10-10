# 06 — Topics and the Object Table

D10. Cross-plugin communication is one object table with four kinds of handles.

## 1. One table, four handle kinds

| Kind | Meaning | Where defined |
|---|---|---|
| service | named typed object answering calls | 02 |
| job | in-flight work with token + completion event | 02 §4 |
| **topic** | named typed multi-party data stream | this document |
| descriptor | capability to a buffer (fd / shm slot) | 07 |

What crosses any boundary is only ever: schema-shaped data, a numeric handle naming an object
on some side's table, or an OS descriptor. Never a pointer, never a language future (D6).
The registry and the bus share the same late-join discipline: snapshot then delta events
(invariant 5 shape).

## 2. Topic semantics

- Named by canonical reverse-DNS id (D9); payloads typed by the same single-source schemas
  (04 §1) — one definition, no per-plane dialect. Topics are declared by the
  `(modukit.topic)` option stamp in the authored `.proto` (D13); the manifest only wires
  them (who/where/policy).
- Per-topic QoS declared at publish: `latest-value` (default — the cockpit signal model:
  speed/gear/touch have a current value, history is a burden), `keep-all`, `lossy(n)`,
  `bounded(backpressure)`.
- Three commandments: topics carry **no request/response** (that is a service — topics have
  no return channel by construction); **no ordering** across publishers unless declared;
  a slow subscriber **never** pushes through the publisher (drop policy is per-subscription,
  visible in the manifest).
- Late join on `latest-value` topics receives the current value immediately.
- Payload formats per the D12 lanes: signals ride the proto envelope; batch/series topics
  declare Arrow cargo (04 §2 sidecar rule); media planes use raw descriptors (07 §2).

## 3. The bus is the instrumentation point

The recorder (module 09, D16) and the debugger (D-queue 13 — expected to merge into 09) are ordinary bus subscribers with an
`export`-gated capability — no side channels. Recorder frames store descriptors as slot
tokens, so a recording replays under the same contract suite (04 §4) as a fixture stream.

## 4. Placement over time

Year 1: in-process bus only (broadcast channels + latest-value, hundreds-of-lines scale) —
services and topics are both local objects already. Stage 2: topics cross the wire through
the transport seam (07) like everything else. Stage 4: cross-machine topic federation is a
carrier/bridge concern, not a kernel concept.

## 5. Deliberately not

DDS worldviews; per-publisher sequencing guarantees; persistent/durable queues; topics for
RPC; QoS vocabulary borrowed from any vendor (the kernel owns the list, bridges translate).
