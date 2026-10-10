# Reference Profile — Arrow Flight (via arrow-rs)

> **External reference.** Profiled from a blobless clone at `~/.cache/modukit-research/arrow-rs`
> (NOT under `.refinfo/`, not ModuKit content, not a dependency). Spec file `Flight.proto`
> fetched from upstream `apache/arrow` main as of research date.
> Research date: 2026-10-10 · Checkout: `dfef34e6` (= `60.0.0-214`, **2026-10-10 — same-day HEAD**)
> · Upstreams: <https://github.com/apache/arrow-rs> (Rust impl) +
> <https://github.com/apache/arrow> `format/Flight.proto` (protocol, 678 lines measured)
> · License: Apache-2.0 (both) · Rust **edition 2024**, MSRV 1.88 (workspace Cargo.toml)
> · Activity: 8,426 commits total, 1,416 in last 12 months (measured) — actively maintained at
> industrial cadence.
> Claims are measured on these artifacts; anything else is marked UNCERTAIN or industry knowledge.

## 1. What it is, in one line

Apache Arrow's official RPC framework: **protobuf carries the control plane (descriptors,
tickets, schema, actions), Arrow IPC bytes ride as an opaque tail in the data plane, gRPC is
the substrate** — the production standardization of exactly the "bill of lading + cargo" split
our D12 adopted as doctrine. This profile prices the technique and states plainly which parts
we already decided NOT to take.

## 2. Contract surface (measured: `Flight.proto:42-144`, service `ArrowFlightService`, 10 RPCs)

| RPC | Shape | ModuKit analog (already adjudicated) |
|---|---|---|
| `Handshake(stream↔stream)` | session negotiation | capability/ABI handshake (04 §3) |
| `GetFlightInfo(FlightDescriptor → FlightInfo)` | name → endpoints+schema+routes | discovery layer, D9 canonical ids |
| `PollFlightInfo(→ stream PollInfo)` | **long-running ticket w/ progress + completion** | **JobToken + event completion (D6) — first-class precedent** |
| `GetSchema(FlightDescriptor)` | schema by reference | `schema_digest` lane (D12) |
| `DoGet(Ticket → stream FlightData)` | **opaque handle** → data stream | handle-not-instance doctrine (D4 invariant 1); `Ticket{bytes}` is literally an untyped handle |
| `DoPut(stream FlightData →)` | client push stream | exe/remote upload lane |
| `DoExchange(stream FlightData ↔ stream)` | bidirectional stream | stream topics (D10 `StreamChannel` analog; AUDESYS D10 trichotomy reappears here) |
| `DoAction / ListActions` | control plane RPC | manifest policy hooks (start/stop/reload would be actions) |
| `ListFlights(Criteria)` | registry listing w/ filter | our *rejected-for-year-1* filter vocabulary (D4) — exists here as a typed `Criteria` message, not LDAP |

**Reading**: the service surface is a faithful map of our already-adjudicated object table —
nothing in Flight's shape requires a concept we haven't ruled, and two rulings (job tokens,
handle tickets) have production protocol files backing them.

## 3. The sidecar wire shape (measured: `Flight.proto` `message FlightData`)

```
FlightData {
  flight_descriptor = 1   // routing metadata (small proto msg)
  data_header       = 2   // Arrow IPC message header — "as described in Message.fbs"
                          // (i.e. the IPC header is itself FLATBUFFERS — spec comment)
  app_metadata      = 3   // opaque user bytes — our app-level envelope slot
  data_body        = 1000 // raw Arrow IPC batch bytes, tag 1000 = deliberately LAST
}
```

- **tag 1000 is the technique**: field numbers ordered so the heavy, self-delimiting tail sits
  last ⇒ a streaming reader parses the bill of lading first and hands the body off untouched.
  This is D12's "header first, tail never parsed" shipped in a spec, with field numbering
  (our three registry rules) doing the structural work.
- `data_body`'s self-description comes from Arrow IPC's own flatbuffers header — the spec
  comment referencing `Message.fbs` confirms fbs as a *header encoder* pattern: when our
  trigger-gated fbs cargo arrives, this is house-aligned precedent, not dialect drift.
- Implementation honesty (measured): Rust encoder tests compare baseline vs optimized
  `data_body` sizes (arrow-flight `encode.rs:826-836`, `:2159` accounting sums) — i.e.
  **one-copy-optimized assembly at framing level** in this checkout; pointer-level zero copy
  across language boundaries remains the C Data Interface lane, not Flight. "Flight is
  zero-copy everywhere" is an overclaim — UNCERTAIN beyond what we measured, treat as one-copy.

## 4. Implementation anatomy (measured, arrow-rs @ 60.0.0)

- `arrow-flight`: **19,672 LoC, 132 test attributes**; deps pinned `tonic 0.14 / prost 0.14`
  with `default-features = false` hygiene (our mise/deny.toml world).
- `src/sql/` adds **7,135 LoC** = Flight SQL — a whole *metadata service product*
  (catalogs/schemas/tables, prepared statements) expressed *on top of* Flight; commands/results
  wrap in protobuf `Any` with typed helpers (`sql/mod.rs:123-140 as_any()`) — precedent for
  "typed escape hatch with a registry" over Blob-style anonymous bytes (contrast: AUDESYS
  `Blob` excluded from type inference; Flight wraps typed `Any`).
- Generated proto Rust checked in (`src/arrow.flight.protocol.rs`) + `gen/` regeneration crate —
  derived-artifact discipline done right (their version of our sidecar/fdset regen gate).
- Reference binaries: `flight_sql_server`/`flight_sql_client` — dogfooding examples in-tree.

## 5. Verdict — what ModuKit takes, what it refuses

| Object | Verdict | Note |
|---|---|---|
| Bill-of-lading/cargo split (FlightData shape, tag-1000 ordering) | **CONFIRMS D12 — technique adopted** | already written into modules/04 §2 as our sidecar rule |
| `Ticket` as opaque handle; descriptor→info→ticket flow | **BORROW-PATTERN (shape validation)** | our D4 handles + D9 ids map 1:1; the 3-step discovery handshake is a good checklist for slot ③ |
| `PollFlightInfo` | **BORROW-PATTERN** | long-running-job-with-progress as protocol citizen = D6 vindicated at spec level |
| `DoExchange` bidirectional streams | **BORROW-SHAPE** for remote topic lanes | our D10 keeps the vocabulary kernel-owned (no DDS-style worldview import) |
| gRPC/tonic substrate (http2, tokio, codegen stack) | **REJECT (as decided)** | D6 sync-boundary + D11 porter-seam already declined this universe; Flight's *framing* is portable without its *transport* |
| `Criteria` listing filter | **REFERENCE-ONLY** | evidence that filter vocabulary can arrive *later as pure addition* (our D4 reservation path stays cheap) |
| Flight SQL | **NOT RELEVANT (product layer)** | a vertical service built ON the framework — closest sibling analogy: "what an HMI signal-service plugin might look like"; study for Stage-4/5, no kernel debt |
| Dependency adoption | **NO** | we adopt patterns; arrow-rs as cargo-format library (arrow-ipc) enters only at a cargo-lane trigger, then it's a workspace-member decision per our DEPENDENCY gates |

## 6. Sharpest single finding

Flight's `FlightData` puts its giant payload at **proto tag 1000** — the field-numbering
discipline we mandate (D12 rule ①) used not for compatibility but as *streaming architecture*:
ordering metadata before body in the number line is what lets every reader be lazy about the
cargo. One integer choice, whole performance story — the cheapest possible advertisement for
"tags from day one".

## 7. Open / unverified

Wire-level throughput/latency numbers not benchmarked here (needs stage-2 traffic shape);
tonic codec's internal copy count not audited; Flight.proto read from upstream main (2026-10-10
raw fetch) not a tagged release; Python/Java Flight client health: not checked (UNCERTAIN).
