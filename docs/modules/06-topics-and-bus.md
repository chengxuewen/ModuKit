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

**Boundary and naming discipline (D36)**: four combinations, all name discipline, zero new machinery. (1) Same-host groups: topics under a component prefix (`safety/*`, `cockpit/*`) with subscribe-discipline; hard group permissions stay behind the multi-vendor trigger (same philosophy as D34 rate limits). (2) Global broadcast: topics designed fleet-wide carry unqualified names (`fleet.telemetry`) - publishing from any machine reaches all via the carrier; wanting global is the author's explicit decision, expressed in the name. (3) Same-app clusters: application-prefix namespaces (`app-a/*`) interconnect, others do not subscribe - pure convention. (4) Runtime renaming stays rejected (see 02 §1, D36). Deliberate violation of naming discipline is not a naming problem - it is malice, handled by capability gates and trust tiers (D26/D2).

System traffic - including the future config-change events for `config_live` keys (D29) -
rides `org.modukit.*` reserved names (D26 namespace discipline).

System traffic - including the future config-change events for `config_live` keys (D29) -
rides `org.modukit.*` reserved names (D26 namespace discipline).

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

## 3. Tags: a topic discipline, not a mechanism (D42/D43)

Industrial point/tag workflows ride the bus unchanged:

- **Envelope by convention**: `modukit-api` (D52) ships a `TagEnvelope<T>` template -
  {value, quality good|uncertain|bad, capture timestamp} - with generated TS/Lua/Python
  counterparts. Using it is voluntary (the bus never inspects payloads - D13's door is
  for interfaces, not policing), but enveloped topics render uniformly in panels, trends
  and replay for free.
- **Run values write by publishing**: a downlink (setpoint 101.3) is an ordinary publish
  on the tag topic, gated by D26 capability + D37 RBAC. Run values and configuration
  (D29) are two lanes that never merge: transient shout vs durable intent - restarts
  keep config, forget run values.
- **Deadbands are the publisher's discipline** - the bus stays dumb and fast (no
  kernel-side change filtering).
- **Tag breakpoints belong to debuggers** (D17: DAP for native, runner hooks for
  scripts); the bus never grows a stop-the-world hook. "If value then act" is a
  watchdog plugin (ordinary subscriber), not a rule engine (config-mounts-code lesson,
  D38).
- **Observation** (D43): the tag table and trend curves are named bays of the D39 web
  blueprint (live half via EventStream, history half via the tool below). Historical
  lookup is a TOOL, not a resident store: `modukit history get <tag> --from --to` reads
  the mcap containers (D16), emits CSV/JSON for panels and scripts alike - CLI wagon
  (D23); the host stays warehouse-free (D17). Single-tag first-step trace (who
  produces/consumes this name) is an ordinary introspect query over the handle table;
  full causal chains are trigger-gated.

## 4. The bus is the instrumentation point

The recorder (module 09, D16) and the debugger (D-queue 13 — expected to merge into 09) are ordinary bus subscribers with an
`export`-gated capability — no side channels. Recorder frames store descriptors as slot
tokens, so a recording replays under the same contract suite (04 §4) as a fixture stream.

## 5. Placement over time

Year 1: in-process bus only (broadcast channels + latest-value, hundreds-of-lines scale) —
services and topics are both local objects already. Stage 2: topics cross the wire through
the transport seam (07) like everything else. Stage 4: cross-machine topic federation is a
carrier/bridge concern, not a kernel concept.

## 6. Deliberately not

Publisher-side rate LIMITS are not built in year 1 (D34): D21 counters make floods visible
(the drops/queue-high-water commandments), and year-1 handling is observation + manual
intervention. An enforcement policy (per-publisher caps, backpressure) is trigger-gated on
the first real topic-storm incident.

Also deliberately not: DDS worldviews; per-publisher sequencing guarantees; persistent/durable queues; topics for
RPC; QoS vocabulary borrowed from any vendor (the kernel owns the list, bridges translate).
