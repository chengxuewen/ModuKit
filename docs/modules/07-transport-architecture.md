# 07 — Transport Architecture: Narrow Seam, Per-Plane Slots, Bridges

D11. Kernel owns the communication semantics; vendors are swappable porters.

## 1. The rule

Semantics (object table D10, naming D9, TL-versioned frames, descriptor message kinds) live
in the kernel. What is swappable behind a small trait is **how bytes and buffers move**.
The rmw disease — abstracting an entire middleware worldview, ending in least-common-
denominator features and N maintained bridges — is avoided by keeping the abstraction one
level below worldviews, the same level iceoryx2 chose internally (`-cal`: one trait,
thread/in-process/cross-process backends; `docs/reference/iceoryx2.md:37`).

## 2. The seam (sketch)

```
trait TransportBackend {            // ~10 methods, not a middleware spec
    fn open_channel(...)   // rendezvous incl. descriptor negotiation
    fn send(&Frame) / fn poll(...) / fn close(...)
    fn pass_descriptor(...)        // fd / slot handoff semantics per OS
    fn capabilities(...) -> { zero_copy_local, remote, ... }
}
```

Slot inventory (all simultaneous, per plane, no favorites):

| Slot | Year 1 | Stage 2+ |
|---|---|---|
| local frames | socketpair/pipe + memfd (self-built) | first pool backend = iceoryx2, exact-pinned to the nearest stable tag at integration
   (D25; deny.toml window; upgrades are budgeted, never silent). Self-built memfd ring =
   trigger-gated fallback: (1) upstream breaks the pin twice in a row or stalls;
   (2) port-glue cost vs dynamic topics beats the ring budget; (3) win/mac demand before
   PAL maturity. All glue stays inside the adapter (five verbs + name-mapping cache);
   the registry never learns vendor vocabulary. |
| remote carrier | absent | zenoh-shaped (`iceoryx2.md:39` shipped-carrier precedent; `00-overview.md` §2 sheet) |
| streaming link | absent | WebRTC (Stage 4) as another carrier |
| **dummy** | test double (inprocess-lineage per `ipc-channel.md:96`) | deterministic supervision tests keep this slot forever |

**Framing precedent (D12)**: Arrow Flight ships exactly the shape we standardize — small
proto bill of lading, opaque Arrow IPC tail (sidecar), zero/one-copy discipline. We adopt
the *technique* over our own TL framing in slot ③; we do **not** adopt Flight's gRPC
substrate (D6 sync-boundary doctrine; slot ③ candidates stay zenoh-shaped per §2 sheet).

## 3. Descriptor access protocol (D14)

Tokens are **self-sufficient**: every descriptor-bearing envelope (e.g. `VideoFrameMeta`)
carries `slot: u64`, `epoch: u32`, and the full `layout` (plane offsets/strides or buffer
geometry). The reader computes addresses by pure addition — **no per-frame metadata query,
no lockstep with the writer ever**.

- **Attach once, hot path zero syscalls**: at subscribe-resolve the pool fd is handed over
  exactly once via UDS (`SCM_RIGHTS`) — slot ① remains the only fd courier (D7); the
  consumer mmaps the pool and caches the base. Per frame: receive envelope → slice =
  base + slot×cell + offset → release = one atomic refcount decrement.
- **Liveness is enforced by types, not by docs**: local views are borrows (Rust
  `FrameView<'a>` releases on Drop; C pairs `slot_view/slot_release`; Python uses
  memoryview scope). A stale token returns `EPOCH_EXPIRED` as an ordinary error value —
  never UB. Dead consumers leak nothing: pool holds are keyed by process and UDS
  disconnect reclaims them (the supervisor's death detection, same mechanism).
- **Slow readers never push through writers** (06 commandment, now mechanically): a
  writer hitting a referenced cell rotates to the next; `Bounded` pools wait per QoS,
  `Lossy` drop the oldest unreferenced. Stale views fail with `EPOCH_EXPIRED` — the
  honest cost of bounded memory.
- **Cross-host — the bridge exchange (D15):** leaving the host, a kernel bridge thread
  rewrites `local -> tail` — materializing the waybill's tail arm (one honest copy; line
  compression is a deployment axis, `encoding` a type axis — never conflated). Inbound
  consumers receive the tail arm directly; **no ticket ever crosses a network**. Zero copy
  stops at the host boundary and does not pretend otherwise (D2 honesty table, API edition).
- **Views borrow the frame, never the arena (D15):** the frame object owns the ticket (or
  the buffer); planes are views into whatever the frame owns. Release is the frame's
  destructor — Rust lifetime, C++ RAII (+ debug assert), C release pairing: one ledger,
  three manners.

## 4. Carrier selection is deployment, not type (D14)

`encoding` (RAW | H264 | LZ4 | ...) is a property of **what the cargo IS** — declared in
`.proto` because it identifies the bytes. Which carrier moves them (local SHM pool —
always preferred on-host — / compressed line / WebRTC / zenoh) is a property of **how
they travel** — declared in the manifest carrier slot and executed behind the §2 seam.
Rule: the type side never spells out how to move; the deployment side never reinterprets
what it is. Swapping carriers later = config; swapping formats = a new (or bumped)
message type in the ledger. Pool geometry (cell size, depth — the third sheet, D15) is
likewise deployment-side (05 §6): the `local` arm only means anything where the pool says
how big the cells are.

## 5. Foreign middleware = bridge plugins, never a heart swap

Interop with ROS2 / DDS / SOME-IP / existing zenoh meshes runs as a **plugin** (it speaks
the same object API — dogfooding, not a privileged path): opt-in `export`/`import` lists in
the manifest gate what may cross; each bridge carries its type-mapping table (reverse-DNS
ids <-> their topic/type names) and QoS translation; the bridge boundary **copies** —
zero-copy never crosses it. Nothing is exported by default.

**Bridge toolbox** (D12, trigger-when-needed, unverified here): row-encoded external streams
may be transcoded wire->columnar directly (`ptars` Rust, `pybufarrow`, `arrowpb` —
self-reported benchmarks, no claims made in this repo); the kernel itself only ever sees
envelope + cargo formats.

**Upgrade trigger** (narrow): an OEM contract requiring native RTPS graph visibility inside
their existing DDS discovery. Answer = a bigger bridge (gateway plugin), still not a core
middleware swap. Recorded as the D11 re-review trigger.

## 6. What never crosses the seam

Handles (object tables are per-side), trust decisions (kernel policy, 01 §5), naming
governance (D9), and the QoS vocabulary — bridges translate it, they do not define it.
Vendor libraries' threads/allocators are quarantined inside a backend implementation.

## 7. Deliberately not year 1

Any vendor linkage; DDS mappings; full QoS matrices; cross-host discovery (the kernel's
network story is transport-level until a real multi-machine need).
