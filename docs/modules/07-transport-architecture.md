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
| remote carrier | absent | zenoh-shaped (`iceoryx2.md:39` shipped-carrier precedent; `00-overview.md` §2 sheet); same-machine multiple host processes discover each other through the SAME carrier on loopback (D34 note - no separate local-discovery mechanism ever) |
| streaming link | absent | WebRTC (Stage 4) as another carrier; loopback note per D34 |
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

- **Frame clause (D69, promoted 2026-10-15):** this pool + descriptor channel is the ONLY path frame bytes take to a plugin, in ANY placement (media-base posture, D68) - on-host; off-host frames travel the D15 bridge exchange above, and the clause bans raw-frame-pointer contract arguments in both directions. Contract signatures carry handles, descriptors and registration ids - never raw frame pointers, never ad-hoc malloc'd foreign buffers (01 §3 rules 3/7). A `dl` plugin mapping the pool is allowed as a trusted-tier privilege; SDK consumers receive frames over the API (pull / borrowed views / caller-allocated out-buffers). Shapes absorbed, fragility not: opaque pool handles across in-process calls = legal; wild-lifetime frame arguments = refused.

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

**Terminology (D35): ModuKit has no gateway.** Three worlds, three answers: plugin-to-plugin = direct handle-table calls (no routing layer exists); foreign middleware worlds (DDS etc.) = bridge plugins, any size (the DDS gateway below is the precedent - a bridge can be big, but its identity is always a plugin, gated by D26 capabilities); external entry points (phone/cloud/diagnostic tools) = also bridge plugins (zenoh-to-HTTP and kin) - when traffic policy needs central management, deploy more bridges; never build a kernel component. Observation of cross-world traffic rides the same ring as everything else (D19): sinks that forward to remote aggregators are ordinary subscribers - multi-machine observability is sink topology, not a gateway.



Interop with ROS2 / DDS / SOME-IP / existing zenoh meshes runs as a **plugin** (it speaks
the same object API — dogfooding, not a privileged path): opt-in `export`/`import` lists in
the manifest gate what may cross; each bridge carries its type-mapping table (reverse-DNS
ids <-> their topic/type names) and QoS translation; the bridge boundary **copies** —
zero-copy never crosses it. Nothing is exported by default. (A ROS 2 exe
plugin's own DDS traffic is not interop - it is the plugin's declared network
surface under D56; the bridge rules govern what crosses into the object table.)

**Foxglove live bridge (D61)**: a bridge plugin embedding the foxglove-sdk
(MIT, Rust) WebSocket server speaks the Foxglove WebSocket protocol, so the
app connects unchanged with every stock panel lit. Writeback (publish /
service call / parameter set) crosses as an ordinary client through the
existing gates - object table + D26, normal resolution, D37 ConfigService
four-gate; no debug exemption (D44). Each bridge carries its own type-mapping
table (D11); dictionary-pack promotion fires at the second bridge/ecosystem.
foxglove-sdk lives only in bridge plugins, never the host link domain (D60
discipline); Foxlet/Data Platform upload is site-side (D61f).

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

## 8. As built (2026-10-19, S2 shipped)

Slot ① is live in `crates/modukit-transport` and `proc_wire.rs`. The control frame is
exactly the TL envelope — `u32 LE len | u8 ver | u8 type | payload` — with
`WIRE_VERSION = 1`: every frame carries the version byte and a foreign `ver` is a typed
protocol error, never a reinterpret. Nine kinds ride it today (HELLO, START, STOP, ACK,
REGISTER, CALL, RESULT, plus the S2-M3b dial pair DIAL/DUE); payload ceiling `FRAME_MAX`
(64 KiB) so a lying length prefix cannot make the host allocate unbounded. Descriptor
passing over `SCM_RIGHTS` (`fdpass.rs`) shipped as slot ①'s fd courier — §3's
"attach once, hot path zero syscalls" is now the mechanism, not the plan. The first pool
backend is `vendor-shm`: iceoryx2 pinned exact (`=0.10.0`), behind a non-default
vendor-neutral feature, confined to one module plus one dependency line per D76, with
dummy/vendor semantic-equivalence tests as the machine proof that the abstraction is not
captive. The self-built memfd ring shipped as the always-available backend. §2's
inventory otherwise holds: remote carrier and streaming slots stay empty (triggers
unchanged), and §7's blanket "no vendor linkage year 1" became the D76-gated exception.
The D74 dial pair (DIAL/DUE) is the wire-side rendezvous the §2 sketch left unnamed.
Nothing in §5 or §6 moved — bridges and the never-cross list are as adjudicated.
