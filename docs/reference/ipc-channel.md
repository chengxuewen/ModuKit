# Reference Profile — ipc-channel (servo)

> **External reference.** This profile was researched from a disposable clone at
> `~/.cache/modukit-research/ipc-channel` (not under `.refinfo/`); it is not ModuKit content and
> none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `6326a1a` (= tag `v0.23.0`, 2026-09-01) · Upstream: <https://github.com/servo/ipc-channel> (verified via `git remote -v`) · Crate: `ipc-channel 0.23.0`
> All facts below are derived from this local clone unless marked **UNCERTAIN**.

## 1. Portrait

| Field | Value |
|---|---|
| Project | "an inter-process implementation of Rust channels (which were inspired by CSP)" — drop-in `std::mpsc` replacement across processes; serde on the wire (README) |
| Language | Rust only; single crate, per-OS platform backends (`src/platform/{unix,macos,windows,inprocess}/`) |
| Governance | servo org on GitHub; Mozilla-born (LICENSE-MIT header: "Copyright (c) 2012-2013 Mozilla Foundation"; first commit 2015-06-19; early merges `r=pcwalton`/`glennw` — Gecko/Servo provenance visible in history) |
| License | **MIT OR Apache-2.0** — dual, both full texts present at root (LICENSE-MIT, LICENSE-APACHE). Expected posture confirmed by direct file read (§8.4) |
| Size | 22 `.rs` files / 53,203 LoC incl. tests+benches; core ≈ 6,712 LoC (ipc.rs 1058, router.rs 300, backends: unix 1306, macos 1354, windows 2165, inprocess 529); 731 all-time commits |
| Velocity | 22 commits / 8 contributors / 12 mo; **6 mo: 2 commits / 2 contributors** (both dependency bumps). Yearly: 2023=19, 2024=28, 2025=32, 2026=17 YTD — flat-low, maintenance-mode |
| Maturity | 34 tags, all pre-1.0 (`v0.23.0` latest, 2026-09-01); release cadence ~quarterly *when it happens* (v0.21.0 2026-01-27 → v0.22.0 2026-04-30 → v0.23.0 2026-09-01); `rust-version = 1.86.0`, MSRV CI added 2025-12 (#428) |
| Real adoption | Historically the IPC spine of Servo and Gecko content processes (provenance above). **Current** production users outside Servo are **UNCERTAIN** — not measurable from the clone |

## 2. Architecture in focus

### 2.1 The typed-channel core

`ipc::channel() -> (IpcSender<T>, IpcReceiver<T>)` with `T: Serialize`/`Deserialize` (README's
explicit mapping table to `std::sync::mpsc`). Semantics differences are documented in-README: queues
are **always unbounded, `send()` never blocks**. The elegant trick: **`IpcSender`/`IpcReceiver`
themselves implement Serialize** — channels are messages, so a child can be handed new typed
endpoints over an existing endpoint (§ of the README: "send IPC channels over IPC channels freely").
Mechanically, serializing an endpoint records an index into thread-local side vectors
(`OS_IPC_CHANNELS_FOR_SERIALIZATION`, ipc.rs:34; the SHM analogue at :36); at `send()` the collected
`OsIpcChannel`s ride out-of-band with the bytes — file descriptors over `SCM_RIGHTS` on unix
(unix/mod.rs:305-306), out-of-line port+descriptor over `mach_msg` on macOS (`mach_sys.rs`'s
`mach_msg_ool_descriptor_t`), duplicated handles via `DuplicateHandle` on Windows
(windows/mod.rs:279-313). Deserialization re-attaches them on the far side
(`OS_IPC_CHANNELS_FOR_DESERIALIZATION`, ipc.rs:28). Framing comes free from
`AF_UNIX + SOCK_SEQPACKET` (unix/mod.rs:468) with in-OS fragmentation for large payloads
("Split up the packet into fragments", unix/mod.rs:426; `MSG_EOR` boundary marking, #444).

### 2.2 Router and bootstrap

- **Bootstrap**: `IpcOneShotServer::new() -> (server, port_name)`; the *name* is the only thing
  that must cross a pre-existing boundary (env/argv). Backends: unix creates a temp-dir
  `AF_UNIX SEQPACKET` listener (`temp_dir.path().join("socket")`, unix/mod.rs:704-709); macOS
  registers a launchd **bootstrap** port name, prefix configurable via `set_bootstrap_prefix`
  (macos/mod.rs:38, exported in lib.rs:45, made public #426); Windows uses a UUID'd named-pipe
  server name (windows/mod.rs, `CreateNamedPipeA` at :154). One accept → typed `IpcSender` → every
  further channel is delegated through it (§2.1).
- **Router** (`src/router.rs`, 300 LoC): `RouterProxy::add_typed_route<T>` (:89),
  `add_typed_one_shot_route<T>` (:110), `route_ipc_receiver_to_{new,crossbeam}_sender` (:159/:174),
  `shutdown` (:133), single `ROUTER` thread pumping all registered receivers → user closures.
  This is the supervisor read-loop shape: one thread, N typed endpoints, per-endpoint handlers.
- **Demux primitive** below the router: `IpcReceiverSet::select()` — mio-backed readiness over many
  OS channels (benches/ipc_receiver_set.rs exists); non-blocking variants added 2026-01 (#435).

### 2.3 SHM posture: opt-in payload type, not a transport

`IpcSharedMemory` (ipc.rs:542, doc at :528: "Shared memory descriptor that will be made accessible
to the receiver of an IPC message that contains the descriptor") is a **serializable field type**:
apps embed it in message enums (`from_bytes`/`from_byte`, ipc.rs:642/654). On unix the backing is
`memfd_create(MFD_CLOEXEC)` + `ftruncate` (`create_shmem`, unix/mod.rs:1210-1217), the fd crosses as
an `SCM_RIGHTS` descriptor, receiver `mmap`s it (`from_fd` → `map_file`, unix/mod.rs:944-954);
Windows mirrors with `CreateFileMappingA`/`MapViewOfFile` (windows/mod.rs:46,2002).
Deref = `&[u8]` zero-copy read; `deref_mut` is `unsafe` **with an explicitly documented single
reader/writer discipline** (unix/mod.rs:917-925); `take()` (copy-out-to-Vec, #433) for consume-once.
Crucially: **no automatic spilling of large messages to SHM** — big inline payloads fragment over
the socket; SHM only carries bytes the application deliberately wrapped. And each region is a
one-shot handoff — no pool, no epoch/lease recycling, no multi-consumer fan-out.

### 2.4 Platform transports, one surface

`src/platform/mod.rs` cfg-selects four backends behind an identical internal API
(`OsIpcSender/Receiver/OneShotServer/SharedMemory/ReceiverSet`, force-inprocess feature switches to
the dummy backend, lib.rs docs). Unix: SEQPACKET + `sendmsg/recvmsg` + CLOEXEC (FreeBSD fixes
#445/#446); macOS: mach ports + bootstrap namespace; Windows: named pipes + handle duplication.
53k LoC with three real OS backends + test backend, each a flat `mod.rs` — the per-OS seam ModuKit's
`modukit-platform-{linux,win,macos}` would replicate.

### 2.5 The wire format is an implementation detail that isn't

Serialization switched **bincode → postcard** in #432 (2026-01-09, first shipped **v0.21.0** —
`git tag --contains` measured), a silent wire-level incompatibility in a 0.x minor, alongside a
unified error model (`IpcError`/`TryRecvError`, #407). Nothing in the handshake negotiates serde
format or version; two peers of different crate versions connect, decode fails, and the message
drops. This is the same unsealed-seam pathology as zellij (§2.5) transplanted to the control plane.

## 3. Key capabilities

- Typed FIFO endpoints with **channel delegation** (send endpoints over endpoints) — process trees
  grow capability graphs, not registries.
- One-shot bootstrap server on every desktop OS + launchd bootstrap prefix for macOS sandboxing.
- Router/proxy event loop; `IpcReceiverSet` demux; `async` feature exposing `IpcStream`
  (futures `Stream`, asynch.rs).
- `IpcSharedMemory` payload type (memfd/filemapping) — handle-passing zero-copy blobs.
- `force-inprocess` test backend; drop-in API parity with `std::sync::mpsc`; criterion benches for
  send/select/shm paths.

## 4. Development & current state

- HEAD = v0.23.0 tag = `6326a1a` (2026-09-01, "Update windows to 0.62 and rand to 0.10", #455).
- Last 6 months: 2 commits / 2 contributors — both dependency bumps. Last 12 months: substantive
  but compact — an API-refresh cluster (postcard switch, error unification, non-blocking select,
  `IpcSharedMemory::take`, `set_bootstrap_prefix`, FreeBSD fixes) by ~5-6 recurring external
  contributors (Narfinger, Glyn Normington, nortti0, Jonathan Schwender, Christian Belloni…), then
  release trains run by the org.
- **No CHANGELOG file at all** (repo root: Cargo.toml, README, LICENSE×2, rustfmt.toml, src,
  tests, benches) — release truth is git tags + PR subjects only; old tag annotations are one-liners
  ("Bump version to 0.7.2").
- The 2015→2018 arc (pcwalton/glennw era) was Gecko-engine-driven development; the 2025/2026 arc is
  Servo-adjacent community maintenance: fixes land when a consumer trips over them (e.g. FreeBSD),
  features land when an external maintainer wants them. Nothing suggests active feature roadmap.

## 5. Ecosystem

- Provenance: extracted from Mozilla's Gecko/Servo content-process IPC (LICENSE-MIT copyright;
  early PR merges reviewed `r=pcwalton`). Whether Firefox/Servo still build against this crates.io
  crate today is **UNCERTAIN** from the clone (servo-tree consumers are external).
- Published as `ipc-channel` on crates.io (Cargo.toml metadata); benches/tests maintained in-tree.
- Related-but-not-this: `ipc-channel`-shaped Rust crates (crossbeam, tokio IPC) and the
  iceoryx2/zenoh pair already profiled — this crate occupies the **point-to-point typed RPC**
  niche neither fills (§8.1).

## 6. Highlights & limitations

**Highlights**
1. Channel-passing-as-serialization is the cleanest bootstrap pattern found in this series: one
   one-shot name in argv/env, then *all* further endpoints delegate typed (§2.1/§2.2). Directly
   liftable to ModuKit's supervisor↔plugin-process handshakes.
2. Three production OS backends behind one ~6.7k-LoC internal API (unix SEQPACKET+SCM_RIGHTS /
   macOS mach+bootstrap / Windows named-pipes+DuplicateHandle) — small, legible, platform-PAL shaped.
3. Handle-passing SHM done right for control-plane purposes: memfd created, fd moves via
   `SCM_RIGHTS`, receiver mmaps — zero intermediate copy of large blobs *when the app opts in*, and
   `deref_mut` documented under explicit single-writer discipline.
4. Router/RouterProxy: a minimal, typed, single-thread event pump with one-shot-route support — the
   supervisor-loop skeleton ModuKit needs, pre-designed.
5. Dual MIT/Apache (D1-compatible), MSRV policy + CI, benches, inprocess test backend.

**Limitations**
1. **Maintenance mode measured thin** (2 commits/6mo) — fixes arrive via consumer pain, not
   roadmap. For a crate 10 years old this is *stability* or *stagnation* depending on the week; the
   bus factor beyond the servo org is not evidenced.
2. **Unsealed wire format**: bincode→postcard changed the ABI of every running pair in a minor
   (§2.5) with no negotiation — exactly the failure ModuKit's C-ABI version handshake must prevent.
3. Always-unbounded, never-blocks `send` = **backpressure by OOM**; no bounded/flow-controlled
   endpoint exists (README admits the semantic gap vs std bounded channels).
4. SHM is one-shot, single-consumer blobs — no recycling pool, no epochs/leases, no multi-subscriber
   fan-out; wrong primitive for sustained I420 frame rings (§2.3).
5. No pub/sub, no discovery/registry, no request-response sugar, single-OS-instance only
   (README: "in a single operating system instance") — it is *endpoints*, not a *fabric*.

## 7. Historical lessons

1. **A typed control plane and a zero-copy data plane are different animals.** ipc-channel spends
   its complexity on endpoint delegation, bootstrap, and OS handle plumbing; it deliberately stops
   short of frames. The 10-year read: don't force one crate to do both — ModuKit-transport should be
   *shaped* like this for RPC and *sized* like iceoryx2 for pixels (§8.1).
2. **Wire formats are ABI; changing them is a breaking release.** The postcard switch (#432, shipped
   in a *minor*) silently partitioned the installed base. ModuKit lesson: pin the serde format inside
   the contract, stamp a wire-version byte into every envelope, and make cross-version connect()
   return a named error — the handshake this crate never built.
3. **Bootstrap by one short name + typed delegation beats registries for process trees.** The
   one-shot server → first sender → channel-over-channel pattern (no port scanning, no central broker)
   matches ModuKit's supervisor-spawns-plugin flow one-to-one; the macOS `set_bootstrap_prefix`
   (#426) even shows how to make it sandbox-compatible.
4. **Platform backends stay maintainable when the internal surface is 5 types.** `OsIpcSender/
   Receiver/OneShotServer/SharedMemory/ReceiverSet` is the whole porting contract; three OS backends
   × ~1.3-2.2k LoC each. ModuKit's platform PAL crates should copy this surface-narrowness, not
   mirror per-OS APIs.
5. **No-CHANGELOG longevity is survivorship, not policy.** This crate shipped wire-breaking minors
   because release notes lived only in tags. For a plugin host promising third parties a stable ABI,
   generated changelogs + labeled breaks (zellij's #5611 pattern) are the cheap countermeasure.

## 8. Value for ModuKit

**Verdict: BORROW-SHAPE (primary, Stage-2 control plane) + narrowly-scoped DEPENDENCY-CANDIDATE
(Rust↔Rust internal RPC only) — REFERENCE for the platform PAL. License MIT/Apache: gate green.**

### 8.1 Posture vs the Stage-2 incumbents (the fork-sheet question)

| Axis | ipc-channel | iceoryx2 (pub/sub fabric) |
|---|---|---|
| Topology | point-to-point endpoints; graphs built by delegation | any-to-many services; shared static/dynamic config space |
| Payload | serde typed messages, `send` copies or hands a one-shot memfd | zero-copy samples with leases, multi-consumer, in-place headers |
| Bootstrap | one-shot name + channel-over-channel | service registry / config-driven ports |
| Backpressure | none (unbounded) | explicit (sample pool limits = natural flow control) |
| Crash semantics | `Disconnected` on recv; no supervision | same story — neither supervises; ModuKit's host must |
| Fit | control plane: lifecycle RPC, registry events, handle hand-off | data plane: sustained frame/sample streams |

They **compose**, and this dissolves part of the Stage-2 fork: a `modukit-transport` where the
typed control plane follows ipc-channel's shape (bootstrap + Router + typed endpoints over a
handshaked envelope) while the frame payload plane stays iceoryx2-shaped (pool + lease + fan-out)
is the synthesis neither gives alone. When *ipc-channel-shape alone* is right for ModuKit: plugins
with small typed messages and no media path (Stage-2 minimal host). When *iceoryx2-shape alone* is
right: pure streaming topologies with static wiring. The HMI/cockpit target needs both planes.

### 8.2 Direct-liftable primitives (Stage-2 inventory)

1. `IpcOneShotServer` + port-name-in-env spawn handshake → supervisor↔plugin bootstrap (§2.2).
2. Channel-over-channel typed delegation → dynamic service endpoints without a name server (§2.1).
3. `Router/RouterProxy` single-thread typed pump → the host's event-loop skeleton (§2.2).
4. `IpcReceiverSet` demux + non-blocking select (#435) → per-plugin read multiplexing.
5. `SCM_RIGHTS` fd handoff of memfd regions (§2.3) → handing a child process a SHM buffer id is
   *this exact mechanism* on unix/windows/mach — ModuKit's compositor frame-handoff plumbing.
6. `force-inprocess` test backend → deterministic multi-process logic tests in one process.

### 8.3 Stage matrix

| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **N/A / test-shape** | No IPC needed in-process; borrow only the inprocess-dummy idea for kernel tests. |
| 2 — multi-process plugins & IPC | **Adopt (shape) / Depend (internal, conditional)** | The primary Stage-2 control-plane reference (§8.1/§8.2). Depending on the crate directly is viable *only* for Rust↔Rust internal edges where its generics can reach the C-ABI boundary — which violates ModuKit's C-ABI-only exit for anything plugin-facing; hence shape-borrow, re-implement over `modukit-c`, add wire-version handshake + bounded queues it lacks. |
| 3 — compositor/UI | **Borrow** | Frame-buffer/handle handoff to child processes = memfd-over-`SCM_RIGHTS` (§2.3) on the three desktops; the *payload discipline* (I420 rings) stays ModuKit/iceoryx2's. |
| 4 — WebRTC | **Reference-only** | Single-OS-instance by design (README) — nothing cross-host here. |
| 5 — polyglot | **Avoid** | Generic serde endpoints are Rust-only by construction; the anti-pattern to generated-from-C-ABI doctrine. |

### 8.4 License gate

**MIT OR Apache-2.0**, dual — verified by direct read of both files at root (Apache 2.0 full text;
MIT notice "Copyright (c) 2012-2013 Mozilla Foundation", i.e. the Mozilla-era import, plus
contributors). Fully compatible with ModuKit's D1 (MIT OR Apache-2.0): same polarity, user-selectable,
Apache leg carries the patent grant. Borrow-shape, reimplementation, or outright dependency all
green; retain the copyright notice in NOTICE/SBOM if any file or snippet is lifted. No copyleft
interaction anywhere in the series so far matches this posture — same class as zellij (MIT) and
iceoryx2 (MIT).

### 8.5 Not verified here (UNCERTAIN)

- Current production consumers (Servo itself, Firefox content processes, third parties) —
  provenance is evidenced in-clone; *present-day* usage is not measurable from it.
- Whether the 2-commits/6-months cadence reflects stability or drift toward caretaker status; a
  health call needs servo.org release traffic the clone cannot see.
- macOS mach backend correctness and Windows named-pipe edge behavior — code present and plausible,
  platform behavior untested here (Linux-only box; `cargo test` for macos/windows backends cannot
  run).
- crates.io download/adoption statistics — not verifiable from the clone.
