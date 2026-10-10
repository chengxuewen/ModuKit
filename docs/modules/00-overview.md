# ModuKit Module Documentation — Index

Engineering deep-dives complementing [../architecture.md](../architecture.md). Adjudication
provenance: D2-D24 (`.agents/memorys/decisions.md`), walked one-by-one with the maintainer
on 2026-10-10.

| # | Document | Covers | Status |
|---|---|---|---|
| 01 | [plugin-placement-model.md](01-plugin-placement-model.md) | executor closed set, shadow-runner, switchability preconditions, asymmetries, trust veto | D2 |
| 02 | [service-model.md](02-service-model.md) | C+ registry: six invariants, canonical identity, dependency policies, job pattern, non-goals | D4/D6/D9 |
| 03 | [lifecycle-state-machine.md](03-lifecycle-state-machine.md) | six states + Failed edge, restart policy, boot orchestration, blocking asymmetry, update path, clock & timers | D3/D6/D8/D20 |
| 04 | [contract-and-codegen.md](04-contract-and-codegen.md) | `.proto` single door, protoc plugin (Rust/C ABI/fdset), two-stage views, cargo oneof, sidecar framing | D2/D4/D6/D7/D9/D12/D13/D14/D15 |
| 05 | [manifest-and-deployment.md](05-manifest-and-deployment.md) | manifest fields, three-tier residence, cross-check, aliases, deploy-time rules | D2/D3/D4/D7/D8/D9 |
| 06 | [topics-and-bus.md](06-topics-and-bus.md) | object table, topic QoS, payload lanes, instrumentation taps | D10/D12 |
| 07 | [transport-architecture.md](07-transport-architecture.md) | narrow seam, per-plane slots, descriptor access, bridge exchange, carrier selection, bridges | D11/D14/D15 |
| 08 | [end-to-end-walkthrough.md](08-end-to-end-walkthrough.md) | one video frame end to end: three sheets, cargo oneof, faces, template status | D14/D15/D23 |
| 09 | [recording-and-replay.md](09-recording-and-replay.md) | three-part header, gap stats, replay clocks, container policy | D16 |
| 10 | [observability-and-debugging.md](10-observability-and-debugging.md) | introspect trio, logger, edge ledger + counters + resources, testhost, analyzer, DAP/Foxglove lanes | D17/D18/D19/D21/D22/D24 |

Reading order for a newcomer: 03 -> 02 -> 01 -> 05 -> 04.
Reading order for a plugin author: 05 -> 02 -> 04 -> 06.
Communication stack reading order: 06 (semantics) -> 07 (mechanics).
End-to-end picture (video-frame worked example): [08-end-to-end-walkthrough.md](08-end-to-end-walkthrough.md).

These documents describe the adjudicated baseline, not implemented code — the workspace is
parked pending maintainer instruction (Stage-1 cut: architecture.md §10).
