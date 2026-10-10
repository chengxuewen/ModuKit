# 10 — Observability & Debugging

D17. The kernel publishes itself; every tool is a downstream consumer. No custom UI, no
custom protocol, no back doors.

## 1. Introspect trio (year 1, ordinary services, reserved namespace `org.modukit.*`)

| Service | Returns | Consumed by |
|---|---|---|
| `LedgerDump.dump()` | the exact fdset bytes this host loaded (+ host/ABI version stamps) | CLI `ledger`, recorder header (09), bridge type-mapping self-check |
| `Snapshot.all(filter)/one(handle)` | object-table cut: plugins (state/placement/version/restart counts), services (provider/properties/ranking), topics (pubs/subs + **drop counters**), in-flight jobs, open slots + holders | CLI `ps`/`inspect`, dashboards, debug launch presets |
| `EventStream.tail(since_seq)` | merged ring: lifecycle transitions, registry events, QoS drops, bridge/transport stats — monotonic seq, resumable | CLI `tail`, Foxglove live bridge, gap-audit tools |

Read-only, side-effect-free; events are trust-tier filtered (D2 table) — a vendor plugin never
sees the cockpit's full wiring. **Constitutional rule: if a tool wants data that these three
services and ordinary bus APIs cannot give, the tool is wrong, not the kernel.**

## 2. `ModuleLoaded` event (year 1, one enum variant)

`{plugin id, artifact path, load base, symbol hint}` — the cheap trigger that lets native
debug adapters do pending breakpoints at the exact right moment, and lets the CLI print a
symbol inventory for attach flows. Five lines, disproportionate payoff.

## 3. Acceptance tests, not tools (year 1)

- **churn**: 200 install/resolve/fail/restart cycles — `Snapshot` must equal the fold of
  `EventStream` since genesis (tables and events may never disagree);
- **step-replay**: identical content sequence, same fixture, `dl` and `exe` placements
  (timing NOT promised — D16 determinism claim);
- **ledger equality**: `LedgerDump` bytes == checked-in `fdset` (same bytes the CI
  regen-diff gate already enforces).

### 3.1 The test host (D24)

The kernel ships these acceptance-test building blocks as a public `test-support` face (the idiom
`modukit-testhost`): in-process host + dummy transport (D11 slot ⓓ) + step-domain clock
with `advance()` (D20's dial) + the fold(events)->snapshot checker + the mcap-fixture feed
point (D16). v1 surface discipline: **only** host construction, clock turning, and
snapshot/event reads — no mock services, no patching APIs; production code path, swapped
water sources. Plugin authors import it like any crate; the example template (§9, 08) ships
its stub. True-process e2e (death/restart/backpressure) stays with the Stage-2
dual-placement suite — this is the determinism layer, not the honesty layer.

  [precedent: ipc-channel ships exactly an inprocess test backend for deterministic
  multi-process logic — ipc-channel.md:96,205]

## 4. Tool faces by stage (all downstream, none privileged)

- **CLI (Stage 2)**: `ps / inspect / ledger dump|diff / tail / topic stats / record / play /
  call / serve --ws` — every command maps 1:1 to §1 services or ordinary clients;
  `--json` everywhere (unix composability).
- **Foxglove (Stage 2; D16 acceptance target)**: a recording must open in Foxglove Studio
  (mcap + protobuf encoding + fdset-in-schema-header); `serve --ws` is just an
  EventStream forwarder. Failure at adoption triggers the own-container fallback (09 §4).
- **DAP**:
  - *native plugins (dl & exe)* — **off-the-shelf adapters** (codelldb/gdb/debugpy) wrap
    real debuggers; the kernel ships only the §2 event + presets/launch-configs generated
    from introspect data. exe placement additionally gets fast restart-under-debugger
    (crash isolation is a debug feature) and **rr time-travel** for free (its DAP entry
    exists; pairs with D16 replay doctrine).
  - *script runners (Stage 5)* — thin engine adapters (lua hooks, QuickJS debugger) behind
    the reserved `debug_adapter()` runner hook point; VS Code/Zed F5 for script authors.
  - *documented limits*: no cross-boundary stepping script<->native frames; DAP shows
    process state, never bus semantics (that is §1's job).
- **Never**: a bespoke ModuKit debug protocol; an rqt-style UI-plugin-framework-inside-the-
  plugin-framework (its maintenance ledger is the warning label).

## 5. The plugin mouth — logger (D19)

Plugins `acquire::<Logger>()` — the same posture as acquiring any service (01 §3 forbids
ambient authority; the ban on stray `printf` implies a formal mouth). A log record =
`{level, component = plugin id, msg, kv[], seq, ts}` and lands in the EventStream's
`LogEvent` variant — the **same monotonic ring as frames and lifecycle events**, so:
recordings replay with narration (D16), `graph --diff` has an audit trail, and the
Foxglove log panel fills from the bridge for free (mcap log encoding; general knowledge,
verify at adoption). Sinks (stdout, rotating file, journald) are ordinary subscribers —
D17 constitution again. Per-plugin rate limit with an explicit `dropped` counter (overflow
is visible, never silent). Visibility is trust-tier filtered like all events (§1).
OTLP export, when a real fleet/collector need arrives (P2), becomes a sink-subscriber of
this ring: the OpenTelemetry world consumes B; B never depends on it.

## 6. Two ledgers, one analyzer (D18)

The kernel keeps one live ledger: `(caller plugin, service, method) -> {count, last-use}`,
recorded at the single legitimate doorway — the generated client stub (D13): one relaxed
atomic add per call. Entries retire with their provider (`EdgeRemoved` event; long history
belongs to recordings, never to RAM). `Snapshot.edges()` exposes it.

Paired with the **declared ledger** (manifest `requires`/`provides`, 05 §3), every graph
question becomes a pure walk over the two ledgers:

- `callers(service)` / `uses(plugin)` — the daily "will I break anything?" answer;
- `impact(plugin)` — declared minus observed: who starves if it is removed;
- orphans: provided-but-never-called (deprecation evidence), declared-but-never-used (pruning);
- **declared-vs-observed diff** — dead dependencies vs. undeclared use. D18 stance:
  detect and warn (`graph --diff`); hard rejection of undeclared use is a deployment
  policy knob (maintainer chose B over B+, default = warn);
- cycles: already a boot-time error (D3 §3); the analyzer only re-reports for humans.

The analyzer is a §1-class consumer (CLI `graph` exporting dot/mermaid/json, or a library)
— zero privileged access, zero kernel growth beyond the ledger itself.

### 6.1 Counters at the gate (D21)

No new doorway — the existing ones get richer:

- edge entries grow `{count, last}` -> `{count, last, sum-latency, max-latency, slow-count}`
  (threshold per deployment; all relaxed atomic adds on the D18 stub path);
- the bus publish/deliver gates accumulate `{published, delivered, dropped, queue-high-water}`
  per topic — the 06 §2 "drops must be visible" commandment made mechanical;
- reads: `Snapshot.counters()` + periodic `CountersTick` events onto the shared ring, so
  recorder, plots and future exporters (Prometheus/OTLP — P2 subscriber list) all eat the
  same feed; per-gate counting is switchable deployment-side.
- Rejected in D21: in-kernel quantiles (t-digest/HDR belong downstream of the ring) and
  direct OTel-metrics adoption (same async-smuggling/dependency-tree verdict as D19 — and
  OTel would only re-aggregate these very numbers).

### 6.2 The resource books (D22)

The supervisor piggybacks: periodic `/proc/<pid>/stat` sampling plus `wait4` final
accounting for `exe`-placed plugins — `Snapshot` plugin entries gain
`{pid, cpu_total, rss_peak}` and `ResourceTick` events join the shared ring (D21's
consumers eat them identically). `dl` entries read **`n/a` by design**: shared-address
accounting is theater, not measurement (01 §4). Quota enforcement (cgroup/rlimit) is a
P2 trigger item — the bell rings with the first untrusted vendor plugin; the manifest
`resources{}` field will sit beside the reserved `rt_policy` family (05 §2).

## 7. Division of labor

Timeline of the bus -> §1 services + 09 recordings. Process state -> DAP/lldb lanes.
Performance -> perf/tracy lane (out of scope, stays out). Files -> Foxglove. The kernel's
own debug surface is complete the day §1+§2+§3 exist — which is year 1.
