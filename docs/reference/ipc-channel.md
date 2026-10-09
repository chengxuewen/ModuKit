# Reference Profile — ipc-channel (servo)

> **External reference.** This is a read-only third-party checkout under `~/.cache/modukit-research/ipc-channel`
> (research clone, not `.refinfo/`); it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `6326a1a` (v0.23.0, tag HEAD, 2026-09-01) · Upstream: <https://github.com/servo/ipc-channel>
> All facts below are derived from this local clone unless marked UNCERTAIN. `gh` CLI and outbound `github.com` were
> unavailable in this environment (git `ls-remote` timed out, `gh: command not found`), so stars/forks/open-issue counts
> are **UNCERTAIN** and only git-history metrics are reported.

## 1. Portrait

| Field | Value |
|---|---|
| Project | `ipc-channel` — "A multiprocess drop-in replacement for Rust channels" (Cargo.toml `description`, README) |
| Language | Rust (single crate, no workspace) |
| Governance | Servo project spin-out; `authors = ["The Servo Project Developers"]`; LICENSE-MIT header reads "Copyright (c) 2012-2013 Mozilla Foundation" |
| License | `MIT OR Apache-2.0` dual permissive (Cargo.toml `license`; `LICENSE-APACHE` 10,847 B + `LICENSE-MIT` 1,067 B both present) |
| Size | 16 `.rs` files, 52,620 total src lines — **but 42,654 of them are auto-generated `src/platform/macos/mach_sys.rs`**; the hand-authored crate is ~7k lines |
| Velocity | **22 commits / 8 distinct contributors in last 12 months** (`git log --since=12.months.1`); 731 total commits, 76 all-time contributors, 34 tags |
| Maturity | 0.x forever; latest `v0.23.0` (2026-09-01), `v0.22.0` (2026-04-30), `v0.21.0` (2026-01-27); first tag `v0.2.1` (2016-03-09) |
| MSRV | Rust 1.86.0 (`rust-version`), **explicitly pegged to Servo**: comment "This version may be `<=`, but not higher than the MSRV in the main servo project"; `edition = "2021"` |
| Roles in repo | `ipc.rs` (typed API) · `router.rs` (in-process demux thread) · `asynch.rs` (`async` feature) · `error.rs` · `platform/{unix,macos,windows,inprocess}` (backends) · `tests/`, `benches/` |
| Positioning | Point-to-point typed channels + one-shot bootstrap + fd/handle passing across processes on ONE OS instance; it is **not** a bus, **not** pub/sub, **not** cross-host |

## 2. Architecture in focus

Three cooperating pieces. Real paths throughout.

### 2.1 Typed endpoints (`src/ipc.rs`, 1,058 lines — the public surface)
- `channel<T>() -> (IpcSender<T>, IpcReceiver<T>)` (`ipc.rs:70`); `bytes_channel()` for raw `[u8]` (`ipc.rs:114`).
- **`IpcSender<T>`/`IpcReceiver<T>` are themselves `Serialize`+`Deserialize`** (`ipc.rs:239`, `ipc.rs:252`, `ipc.rs:352`-region) — channels are sent over channels (the README's "drop-in" claim, and the mechanism behind delegation/spawn).
- `recv` / `try_recv` / `try_recv_timeout` (`ipc.rs:199-231`); `to_opaque` → `OpaqueIpcSender`/`OpaqueIpcReceiver` (`ipc.rs:783-858`) for the type-erased wire.
- **Channels are always unbounded and `send()` never blocks** (README "Semantic differences") — no backpressure, no bounded queue. The one deliberate divergence from `std::sync::mpsc`.
- `IpcReceiverSet` (`ipc.rs:400`, `add`/`add_opaque`/`select`/`try_select`/`try_select_timeout`) — the epoll/mio-style multiplexer over many receivers; `IpcMessage::to<T>()` (`ipc.rs:752`) deserializes a raw selection lazily.
- **Crash detection lives in `error.rs`**: `IpcError::Disconnected` is returned by `recv` once all senders are gone and the pipe/socket is closed ("Disconnect non-empty channel" documented as behaviour, #437). Detection, not recovery — see §7.

### 2.2 The bootstrap / router handshake
- **`IpcOneShotServer<T>`** (`ipc.rs:888-920`): `new() -> (server, name)`, `accept()` blocks and returns `(IpcReceiver<T>, first_message)`; the client calls `IpcSender::connect(name)` (`ipc.rs:311`) — **at most once per server** (README "Bootstrapping channels"). The `name` is a filesystem path bound to a Unix socket on unix, a generated name on macOS/Windows. You pass `name` to the child out-of-band (env var / argv) — `spawn_one_shot_server_client()` in `tests/integration_test.rs`, `cross_process_embedded_senders_fork()` in `src/test.rs` show both exec-spawn and `fork()`.
- **`ROUTER`** (`src/router.rs`): a global `LazyLock<RouterProxy>` that spins up **one dedicated OS thread** (`router.rs:54` `thread::Builder…spawn(Router::new().run())`) running a blocking `ipc_receiver_set.select()` loop (`router.rs:226-230`). It converts `IpcReceiver<T>` into crossbeam `Sender`/`Receiver` (`add_typed_route`, `route_ipc_receiver_to_new_crossbeam_receiver`) so callers never park a thread per fd. This is the design ModuKit's supervisor needs: many child channels, one poller. `asynch.rs` is the futures-flavored twin (`IpcStream<T>`, own router, `async` feature).

### 2.3 Platform transports (`src/platform/`, cfg-dispatched in `mod.rs:73`)
- **unix** (`unix/mod.rs`, 1,306): `AF_UNIX` `SOCK_SEQPACKET` sockets + **`SCM_RIGHTS` fd passing**; `mio` eventing. Message carries channel fds AND shm fds in the ancillary cmsg; `MAX_FDS_IN_CMSG = 64` (`unix/mod.rs:42`), `RESERVED_SIZE = CMSG_SPACE(64*8)` (`:54`). Fragment size is derived from the kernel `SO_SNDBUF` (`get_system_sendbuf_size`, `:214`), large payloads split across fragments (MSG_EOR boundary, #444).
- **macOS** (`macos/mod.rs` 1,354 + `mach_sys.rs` 42,654 generated): Mach ports; `SendData::OutOfLine` for big payloads (`macos/mod.rs:370,381`); `set_bootstrap_prefix` (#426) for bootstrap-server namespacing. **42k of 52k repo lines are the raw Mach FFI binding** — the true hand-written cost is ~1.3k.
- **Windows** (`windows/mod.rs`, 2,165 + `aliased_cell.rs`): **named pipes** (`CreateNamedPipeA`, `PIPE_BUFFER_SIZE = MAX_FRAGMENT_SIZE + 4 KiB`, `:129`) with overlapped IO; SHM via `CreateFileMappingA`/`MapViewOfFile` (`:46`, `:2002`, `:2023`).
- **inprocess** (`inprocess/mod.rs`, 529): dummy backend for `force-inprocess` and for android/ios/wasi/unknown where there is no cross-process primitive.

### 2.4 The shared-memory path for large payloads — the key comparison
`IpcSharedMemory` (`ipc.rs:542`) wraps an OS region and `Deref`s to `[u8]` (`ipc.rs:547`); `from_bytes`/`from_byte`/`take`/`deref_mut` (`ipc.rs:637-675`, `:560`). The wire trick:
1. `from_bytes` on the sender makes a fresh **`memfd_create(MFD_CLOEXEC)` + `ftruncate`** region (`unix/mod.rs:1212`, `create_shmem`) and **memcpy**s the bytes in.
2. On `Serialize` (`ipc.rs:605`), the region is pushed into a **thread-local side-channel** `OS_IPC_SHARED_MEMORY_REGIONS_FOR_SERIALIZATION` (`ipc.rs:36`) and **only its integer index** is encoded into the `postcard` payload (`usize::MAX` = empty).
3. The actual **fd rides `SCM_RIGHTS` out-of-band** with the socket write (`unix/mod.rs` send path collects `shared_memory_region.store.fd()`).
4. `Deserialize` (`ipc.rs:578`) pops the region from the matching thread-local and mmaps it; the receiver reads it in place via `Deref`.

**Posture vs iceoryx2 (`.refinfo/iceoryx2`, v0.10.0):** this is **"copy-in, hand-over-the-handle, per message"** — a memoization of large blobs on a point-to-point channel, single-consumer, no pool and no ring, no zero-copy *write* (you still `memcpy` into the new memfd). iceoryx2 is the opposite: a **pre-allocated SHM pool with loaned samples**, multi-subscriber fan-out, a true zero-copy ring, WaitSet multiplexing, per-pattern headers. `benches/ipc_shared_mem.rs` makes the split explicit — it benches a `deref_mut` in-place path vs a `to_vec`-copy path (`MUT` const-generic), i.e. ipc-channel lets you *reuse* one mapping you already own but does not *loan* a buffer to peers. **When ipc-channel is the right size:** an occasional/bursty large buffer you already hold in memory (a decoded frame, a serialized doc) moving one hop parent↔child, where you also want to hand back file/socket/channel fds and want typed-channel ergonomics. **When it is the wrong size:** sustained high-frequency frame rings with fan-out — that is iceoryx2's whole reason to exist.

## 3. Key capabilities
- Drop-in `std::sync::mpsc` shape extended across processes; channels-over-channels are first-class.
- One-shot server for bootstrap; typed `IpcSender`/`IpcReceiver`; opaque + `bytes` variants; `IpcReceiverSet` multiplexing with blocking and non-blocking select (`#435`).
- fd/handle passing (`SCM_RIGHTS` / Mach send-once / Windows handle dup) — the same path carries SHM regions, so big payloads cost one handle copy + an mmap, not a socket-sized serialization.
- `async` feature (`asynch.rs`, `IpcStream`) and `force-inprocess` for tests / platforms lacking real IPC.
- 4 platform backends + an in-process stub; CI covers mac/linux/windows.
- Deliberately minimal dependency set (see §4) — no tokio, no runtime, no allocator coupling.

## 4. Development & current state
- **Long-term velocity is a maintenance heartbeat, not a product train.** Yearly commits (`git log --since=$y-01-01 --until=$y-12-31`): 2015:75, 2016:223, 2017:76, 2018:47, 2019:41, **2020:10**, 2021:153, **2022:9**, 2023:19, 2024:28, 2025:32, 2026:17 (YTD). Two near-dead years (2020, 2022), one big-rework spike (2021), a genuine 2025→2026 revival.
- **Concentration**: 12-month top committers Narfinger (7), nortti0 (5), **Glyn Normington (3)**, Schwender (2), Sam (2) — a handful of Servo-adjacent maintainers; no corporate release bot.
- **Substantive recent work, all in 2025-2026** (from the 22-commit log): error unification `IpcError`/`TryRecvError` (#407, 2026-01-09), **bincode→postcard switch (#432)**, `Switch from fnv to rustc_hash (#430)`, non-blocking `select` (#435), `IpcSharedMemory::take (#433)`, MSRV CI job (#428), `set_bootstrap_prefix (#426)`, FreeBSD CMSG/`RESERVED_SIZE` fixes (#445/#446), `windows 0.62 / rand 0.10 (#455, 2026-09-01 = HEAD)`.
- **Dependency hygiene**: current direct deps are `crossbeam-channel 0.5`, `libc 0.2`, `postcard 1.1` (default-features off, `use-std`), `serde_core 1.0`, `thiserror 2.0`, `uuid 1 (v4)`; per-OS `mio 1`/`rustc-hash`/`tempfile` (unix), `rand` (macos), `windows 0.62` (win). **No Cargo.lock is committed** (lib crate; `ls Cargo.lock` → NO_LOCK). No `deny.toml`, no cargo-deny observed.
- **Docs hygiene**: `#![doc = include_str!("../README.md")]` (lib.rs) keeps rustdoc == README; there is **no `CHANGELOG.md`** (NO_CHANGELOG) and **no `docs/` dir** (NO_DOCS) — release notes live only in GitHub tags/PRs. `rustfmt.toml` present; single `.github/workflows/main.yml` (test matrix + `msrv_build` on `dtolnay/rust-toolchain@1.86`).

## 5. Ecosystem
- Consumed by **Servo and the wider Rust browser/embedded-IPC ecosystem** historically; `force-inprocess`, `async`, and `win32-trace` are its only feature flags (Cargo.toml `[features]`). Adoption breadth today is **UNCERTAIN** (no network / `gh`); crates.io download and dependent counts could not be read locally.
- It is a **leaf library, not an ecosystem** — no binding siblings, no separate CLI, no plugin ABI, no schema. Unlike zenoh/iceoryx2 it never exits to C; the API is Rust generics over `serde`, which is exactly why it cannot be ModuKit's shipped cross-language contract.
- Serialization: `postcard` is the current wire format; the crate is **`serde_core`-based** (#419, 2025-09-27) to compile in parallel with `serde` itself.

## 6. Highlights & limitations
**Highlights**
1. The cleanest local model of *typed process/channel plumbing*: unbounded typed `IpcSender<T>`/`IpcReceiver<T>` where the endpoints are themselves serializable, so a parent can delegate channels down a spawn tree — ModuKit Stage-2's exact bootstrap need (`ipc.rs:239-252`, `IpcOneShotServer`).
2. A working **single-router-thread + `IpcReceiverSet::select` demux** (`router.rs`) that scales to many child channels without a thread-per-socket — the supervisor's blocking read side, ready to copy in shape.
3. `Disconnected`-as-`recv`-error gives **free crash/peer-death detection** on every typed endpoint (`error.rs`); no heartbeat protocol to invent.
4. The **SHM-via-thread-local-index + fd-passing** trick (`ipc.rs:605` + `unix/mod.rs:1212`) moves large payloads + OS handles over the *same* channel with no separate transport — the cheap middle option between "serialize everything" and "full iceoryx2".
5. Truly minimal, runtime-free deps and an active MSRV gate; ~7k hand-written lines once you exclude `mach_sys.rs`.

**Limitations**
1. **No C ABI and no language story** — the whole API is Rust generics + serde; it cannot be the generated-façade contract ModuKit's C-ABI-only exit rule demands.
2. **No pub/sub, no registry, no fan-out, no request/response** — point-to-point unbounded channels only; it is a *transport primitive*, not the service layer.
3. **No backpressure by design** (`send()` never blocks, README) — a stalled child means unbounded memory growth on the sender; ModuKit must add flow control at the supervisor level.
4. **Zero-copy is shallow**: one `memcpy` into a fresh `memfd` per send, single-consumer, no buffer loan/pool — far from iceoryx2's rings.
5. **Maintenance fragility**: 22 commits / ~3 active humans per year, `0.x` for a decade with no stable-ABI promise, two dead years in history; MSRV is chained to Servo's, not ModuKit's.
6. Platform-specific kernel surface (`SCM_RIGHTS`, Mach, named-pipe fragments) is where the real complexity/bugs live (recent FreeBSD/`RESERVED_SIZE` fixes) — cross-platform correctness is not free.

## 7. Historical lessons
1. **Servo spin-out pattern.** `ipc-channel` was extracted from `servo/servo` into a standalone `servo/ipc-channel` crate — this repo's "Initial commit" is 2015-06-19 and cross-process sending already worked by 2015-06-23. Lesson: a component whose job is well-scoped enough to be *the browser's IPC layer* is a good candidate to own as its own crate; ModuKit's `modukit-transport` local plane can be seeded the same way (extract, standalone, keep the parent as a consumer).
2. **Maintenance-mode is survivable but you must budget the dips.** 2020 (10) and 2022 (9) show a real library used in production going quiet for a year when the driving org's priorities move. Lesson: pin `Cargo.lock` to a tag and treat a 12-month quiet period as *normal*, not a signal to fork — but do not depend on it for a feature roadmap either.
3. **Serialization-dep churn is a decade-scale risk, not a one-off.** bincode was adopted 2015-08-08 ("Use bincode instead of JSON"), served for ~10 years, then was **replaced wholesale by `postcard` on 2026-01-09 (#432)**, followed immediately by a `heapless`-default fix (#440). `serde` also had to be split to `serde_core` (#419) to coexist. Lesson for ModuKit: pick a serialization format that is *not load-bearing on your public ABI* — the wire format changed under a stable-ish crate and consumers still had to recompile; keeping serde/postcard behind ModuKit's own codec trait (not the C surface) contains this blast radius.
4. **The 42k-line `mach_sys.rs` is the portability tax.** 81% of the repo is generated Mach bindings needed only for macOS. Lesson: any ModuKit local-IPC backend that claims cross-platform support will spend most of its bulk on per-OS primitives; scope Stage-2 "local plane" to Linux (`memfd`/`SCM_RIGHTS`) first and treat mac/win bindings as later, isolated work.
5. **No CHANGELOG in a 10-year, 34-release library.** Release truth lives only in git tags + PR merge subjects. Lesson (echoes the zenoh profile): either generate changelog from tags or don't create the stub; ModuKit should generate from tags in CI from day one.

## 8. Value for ModuKit

**Verdict: BORROW-SHAPE (primary).** Not a dependency candidate for ModuKit's shipped transport contract — its API is Rust-generic serde with no C-ABI exit, which fails ModuKit's C-ABI-only rule; and it offers no pub/sub/registry. But it is the single most directly *portable* local reference for the three primitives modukit-transport's **local plane** must define: typed endpoints, one-shot bootstrap, and SHM-via-handle-passing. iceoryx2 remains the dependency for the zero-copy *data* plane; ipc-channel is the borrow-shape for the typed *control* plane that sets it up.

### 8.1 Dependency-candidate vs borrow-shape
| Axis | ipc-channel | verdict for ModuKit |
|---|---|---|
| Typed cross-process channel + channel delegation | first-class, ~7k real lines | **Borrow-shape** — reimplement the `IpcSender/IpcReceiver` + `IpcOneShotServer` contract over ModuKit's C ABI |
| Crash/peer-death detection | `IpcError::Disconnected` on `recv` | **Adopt the semantics** in the supervisor; add restart (they have none) |
| Large-payload SHM | copy-in + fd/handle passing, single consumer | **Borrow for control plane only**; use iceoryx2 (already D-pending) for the multi-subscriber zero-copy data plane |
| Demux many child channels | `ROUTER` thread + `IpcReceiverSet::select` | **Adopt the shape** for the supervisor's read loop |
| Language story | Rust-only, serde generics | **Avoid as the shipped contract** — cannot be the generated-façade surface |
| Backpressure | none (never blocks) | **Avoid the default** — ModuKit needs bounded/flow-controlled endpoints for plugins |

### 8.2 Stage matrix
| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Reference-only** | In-process kernel needs neither IPC nor SHM; `inprocess` backend is a stub, nothing here informs lifecycle. |
| 2 — multi-process plugins & IPC | **Adapt (primary)** | Port the *shape*, not the crate, into `modukit-transport` local plane: (a) `IpcOneShotServer` handshake = the parent↔child bootstrap where the server name is passed via env/argv; (b) `ROUTER` + `IpcReceiverSet::select` = the supervisor's single-threaded read side over N child channels; (c) `Disconnected`-on-`recv` = crash *detection* (ModuKit adds restart/supervision it deliberately lacks). |
| 3 — UI compositor | **Borrow-shape** | Compositor needs to *hand child processes SHM frame buffers + fds*; ipc-channel's `IpcSharedMemory`+`SCM_RIGHTS` is a working demonstration of exactly that on Linux (`unix/mod.rs:1212`) and Windows (`CreateFileMappingA`/dup). The typed "send me a handle" message is the shape; the payload discipline (I420 rings) is ModuKit's own. |
| 4 — remote WebRTC | **Reference-only** | Single-OS-instance only (README); no cross-host transport; nothing to borrow. |
| 5 — polyglot bindings | **Avoid** | Its generics+serde surface is the anti-pattern to ModuKit's one-C-ABI-exit doctrine; do not model the public transport API on it. |

### 8.3 License gate
`MIT OR Apache-2.0` (both files present, permissive, user-selectable) → **green for borrow-shape, reimplementation, and (if ever vendored for a Rust-only internal edge) redistribution** with no copyleft exposure. The Apache leg carries a patent grant; the MIT leg is maximally compatible with ModuKit's still-TBD license. SBOM/NOTICE: pin "MIT OR Apache-2.0 (user choice)". No relicensing risk observed (`git log --all -- LICENSE-MIT` traces to the 2015 import; Mozilla-copyright MIT header is stable). Gate action: any Stage-2 borrow that links the crate (rather than reimplements) must re-run `git log -- LICENSE*` and `cargo deny` at pin time and record the result in `decisions.md`; note this repo ships **no Cargo.lock and no deny.toml**, so ModuKit must supply both.

### 8.4 The iceoryx2 fork — where ipc-channel actually lands
The whitepaper §8 Stage-2 fork was framed as iceoryx2-vs-Zenoh. ipc-channel adds a **third, orthogonal** option that dissolves part of the fork: it is neither a SHM ring (iceoryx2) nor a network bus (zenoh), but the **typed control channel that bootstraps and coordinates** either data plane. Concrete composition ModuKit can adopt: *ipc-channel-shaped typed control plane (reimplemented over C ABI) sets up, negotiates, and supervises; iceoryx2-shaped zero-copy SHM carries the sustained frame/data payload.* Neither ipc-channel nor iceoryx2 alone gives crash isolation + typed bootstrap + zero-copy fan-out — the lesson this profile contributes to `00-overview.md`'s Stage-2 decision sheet.
