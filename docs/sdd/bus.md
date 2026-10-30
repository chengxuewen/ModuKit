# Bus Contract (SDD)

> Test annotation: `// spec: BUS-01`; gate: `scripts/spec-trace.sh` plus the demo leg.

## BUS-01: cross-wall publish/subscribe

A parent-bus subscriber sees a topic published by a child executor after the same
walk: the child's publish crosses the wall as `MSG_FORWARD`, the parent re-publishes
it as `MSG_DELIVER`, and the subscriber receives the payload. With host push disabled
the runner is never spoken to and the parent bus remains at zero deliveries; no
silent fallback path is allowed.
