# iceoryx2 — External Reference Profile

> Research date: 2026-10-09 | Checkout: v0.10.0-180-gedd21c0c3 | Upstream: https://github.com/eclipse-iceoryx/iceoryx2.git
> External reference profile. iceoryx2 lives in .refinfo/ (git-ignored clone); NOT ModuKit content. Claims measured on this checkout; anything not locally verifiable marked UNCERTAIN.

## 1. Project portrait

| Field | Value |
|---|---|
| Name | Eclipse iceoryx2 (`iceoryx2`, CLI `iox2`, homepage https://iceoryx.io) |
| Developer | Eclipse Foundation project; dominant lead Christian Eltzschig (3,805 commits), Jeff Ithier (1,440), Mathias Kraus (1,432); 65 contributors total |
| First release | repo "Initial commit" 2023-12-12; first tag v0.1.0 dated 2023-12-14 (2 days later) |
| Current version | v0.10.0 (2026-09-18); main = `0.10.999`, 18 tags, HEAD 2026-10-08 |
| License(s) | dual `MIT OR Apache-2.0` (`[workspace.package] license` in root `Cargo.toml`; `LICENSE-APACHE` + `LICENSE-MIT`, SPDX per file) |
| Language mix | ~271k lines `.rs`, ~74k lines C/C++/headers (bindings + PAL); measured with `wc -l` on this checkout |
| Positioning | "Zero-copy lock-free IPC with a Rust core" — service-oriented middleware for ultra-low-latency inter-process data exchange (`README.md` intro) |
| Target users | automotive / embedded / robotics / HFT-style workloads needing predictable latency at any payload size; ROS 2 and Zenoh integrations signal robotics + distributed-SOA ambitions |

## 2. Architecture in focus

Layered workspace, each layer small and replaceable (all crate names verbatim from root `Cargo.toml` members):

```
 iceoryx2 (core: service / port / node / waitset / config)
   ├── iceoryx2-cal   pluggable-backend contracts  ── per-concept: trait + implementations + recommended default
   ├── iceoryx2-bb    building blocks (posix, shm, lock-free, memory, …)  #![no_std]-compatible
   ├── iceoryx2-pal   OS ports (os-api, posix, concurrency-sync, configuration)
   ├── iceoryx2-ffi   c (cbindgen) · python (PyO3) · ffi-macros   ← C ABI is the single exit
   │    iceoryx2-c / iceoryx2-cxx  packaged C & C++ bindings (C++ wraps the C ABI)
   ├── iceoryx2-link  host-to-host: link core + backend{tunnel|gateway} + carrier/adapter contracts
   ├── iceoryx2-services/discovery   attribute-based service registry
   └── iceoryx2-cli / iceoryx2-log / iceoryx2-userland(record-and-replay) / integrations/{zenoh,ros2}
```

- **Shared-memory pub/sub**: a *service* is a named SHM realm. `iceoryx2/src/service/` splits `static_config/` (immutable per pattern: `publish_subscribe.rs`, `event.rs`, `request_response.rs`, `blackboard.rs`, `message_type_details.rs`, attribute-based matching) from `dynamic_config/` (live port counts, connect/disconnect state). `iceoryx2/src/port/` implements one port type per role (`publisher.rs`…`writer.rs`, `client.rs`…`server.rs`, `notifier.rs`/`listener.rs`) plus `backpressure_strategy.rs` and `port_lifetime_tag.rs`.
- **Zero-copy semantics**: samples live in SHM; `iceoryx2-cal/src/shm_allocator/` offers switchable allocators (`bump_allocator.rs`, `pool_allocator.rs`, `pointer_offset.rs`); `iceoryx2-cal/src/zero_copy_connection/` is the ring between publisher and subscribers; every payload carries a fixed `header/` (`payload_header.rs` + per-pattern headers) so metadata reads never touch the payload. Types crossing SHM must satisfy the generated `ZeroCopySend` trait/proc-macro (marked done in `ROADMAP.md` "Building Blocks").
- **Transport abstraction (`-cal`, the key idea)**: every OS primitive is a trait with multiple backends selected at runtime — `shared_memory/{posix,file,process_local}.rs`, `event/{trigger,…}`, `static_storage`, `dynamic_storage`, `reactor`, `serialize`, `hash`, `monitoring`, `named_concept.rs` (backend naming registry). `process_local.rs` + `local.rs`/`local_threadsafe.rs` under `src/service/` mean the *same API* serves intra-thread, intra-process, and cross-process transport.
- **Notification**: `iceoryx2/src/waitset.rs` — reactor-based WaitSet multiplexing data + signals across ports (roadmap: "WaitSet - event multiplexer based on reactor pattern", done).
- **Host-to-host**: `iceoryx2-link/README.md` defines link (local discovery/bridges/propagation) over a `backend` contract; `tunnel` connects iceoryx2↔iceoryx2 via a pluggable `carrier` — the shipped carrier is **Zenoh** (`integrations/zenoh/link-tunnel-cli/Cargo.toml` depends on `zenoh`); `gateway`+`adapter` bridge to foreign middleware (ROS 2 WIP, #1742).
- **Platform matrix** (`README.md` "Supported Platforms"): Linux x86_64/aarch64/32-bit, Windows, macOS, FreeBSD **tier 2 (done)**; QNX 7.1/8.0 **tier 3 (done, not CI-tested)**; Android, bare-metal (`no_std` PoC), VxWorks **PoC**; FreeRTOS/ThreadX/iOS/RTEMS/Redox **planned**. Tier 1 = full safety+security features — *no tier-1 platform yet*, target is to lift Linux/QNX.
- **`no_std`/embedded story**: `-bb`/`-cal` crates are `#![cfg_attr(not(any(test, feature="std")), no_std)]` with dedicated `tests-nostd` crates per module (`doc/development-setup/nostd-*.md`); roadmap moonshots `no_std` on stable across tier-1 marked **done**; bare-metal PoC works except event pattern.

## 3. Key capabilities

- Messaging patterns: publish/subscribe ( configurable history per subscriber, #1185; user headers; backpressure), event, request/response, blackboard, pipeline (planned) — one service API, pattern-specific port factories.
- Cross-language communication on one C ABI: Rust, C, C++, Python shipped; C# in separate repo; Zig documented (`doc/user-documentation/use-iceoryx2-with-zig.md`); Go/Java/Kotlin/Swift/Lua/TS planned (`README.md` bindings table).
- Decentralized runtime: no daemon crate exists in the workspace; node management + `stale_resource_cleanup.rs` are in-process; only an *optional* health-monitoring example spawns a central daemon (`doc/how-to-write-end-to-end-tests.md:184`).
- Service discovery with attribute matching + type fingerprinting (services from hosts with differing descriptions are not bridged, #1960); `iceoryx2-services/discovery` crate; `iox2` CLI inspects nodes/services.
- FlatBuffers payload support for pub/sub + request/response (#1745); dynamic-storage/SHM resize ("completely dynamic setup with dynamic shared memory", roadmap checked).
- Ops surface: TOML config (`iceoryx2/src/config.rs`), pluggable loggers (`iceoryx2-log`), signal-handling modes, WaitSet, SIGBUS/corruption recovery documented in `FAQ.md`.
- Benchmarks in-tree (`benchmarks/`: publish-subscribe, event, request-response, queue) with README latency charts vs other mechanisms/architectures; exact µs numbers only in SVG plots — UNCERTAIN from checkout.

## 4. Development & current state

- Velocity: `3,858` commits in the last 12 months (`git log --since=12.months.1`); `46` distinct contributors active in that window (`shortlog -s`), 65 all-time.
- Release cadence: v0.1.0→v0.10.0, 18 tags since 2023-12; recent minor releases roughly quarterly (v0.9.x patch train 0.9.1–0.9.3 → v0.10.0 2026-09-18); structured release notes per version under `doc/release-notes/`.
- Toolchain facts (root `Cargo.toml`): `edition = "2024"`, workspace `rust-version = "1.89"`; build systems Cargo + Bazel + CMake (first-class, `iceoryx2-cmake-modules`); `justfile` task runner (recent E2E migration to `just`, PR #2072).
- Health signals: active CI-referenced clippy/MSRV maintenance commits (e.g. "compatible with MSRV 1.85", #1714); SECURITY.md + Eclipse governance; commercial support listed (ekxide); VxWorks PoC lives in the ekxide fork — funding-driven platform work is the stated model (`ROADMAP.md` "Main Focus ... depends on ... funding").
- Pre-1.0: version `0.10.999` on main; breaking restructuring still happening in 2026 (link/gateway overhaul landed in v0.10.0).

## 5. Ecosystem & adoption

- Local evidence of ecosystem: separate `eclipse-iceoryx/iceoryx2-csharp` bindings repo; `integrations/ros2` gateway WIP; `integrations/zenoh` tunnel; `iceoryx2-userland/record-and-replay`; bazel examples; component-tests shared with the C++ bindings — a binding-ecosystem strategy, not a single-language library.
- GitHub metrics (API, 2026-10-09): 2,585 stars, 198 forks, 245 open issues.
- Lineage: successor to eclipse-iceoryx (v1, C++); v1's industrial adoption (automotive, ROS 2 middleware option) is well known but NOT verifiable in this checkout — UNCERTAIN here.
- Beyond the iceoryx umbrella: no independent adopters verifiable locally — UNCERTAIN.

## 6. Highlights & limitations

Highlights
- One Rust core → every language through a generated C ABI (cbindgen + type-erasure storage pattern, `iceoryx2-ffi/c/README.md`): exactly the "generate, don't hand-write" shape ModuKit wants.
- `-cal` layer = every OS mechanism behind a runtime-switchable trait with a `recommended.rs` default and conformance suites (`iceoryx2-cal/conformance-tests`) — portability without compile-time platform forks.
- Consistent low latency regardless of payload size; decentralized (daemonless) design removes the single point of failure and deployment burden.
- `no_std` on stable down to bare-metal PoC; 3 OS families beyond the big three (FreeBSD, QNX, 32-bit Linux).
- Operational FAQ keyed by exact error names (`SIGBUS`, `HangsInCreation`, `IncompatibleTypes`) — field-debuggable.

Limitations
- Pre-1.0 churn: crate/API renames still landing in v0.10.0; no stable ABI promise; pin-and-wrap required.
- MSRV 1.89 + edition 2024 is aggressive for toolchain-frozen consumers.
- Safety tier 1 not achieved on any platform yet; QNX untested in CI (tier 3); mixed-criticality + IAM still open checkboxes (`ROADMAP.md`).
- SHM operational hazards: `/dev/shm` layout quirks, stale resources after crashes, cross-user permission failures (`FAQ.md` has entries for each — the existence of the entries is the warning).
- `iox2_*_storage_t` type erasure means the C ABI must be manually kept size/alignment-correct — a footgun their ffi-macros automate; hand-porting this pattern is costly.
- C++/C# bindings lag main; Python lacks some surfaces (port counts only exposed in v0.10, #1151).

## 7. Historical lessons

1. **Full rewrite in a fresh repo, not refactor-in-place.** iceoryx2 began 2023-12-12 ("[#1] let there be code", 2023-12-13) as a new workspace, explicitly "overcoming past technical debts … enabling the modularity we've always desired" (`README.md`). The v1 pain (C++ core, central Roudi daemon, rigid layering) is the upstream narrative — locally only its *absences* are provable (no daemon crate; modularity-first crate split). Lesson for ModuKit: when the canon changes (in-process kernel vs IPC substrate), a clean-tree restart with the old repo kept as reference is viable and cheap if you split layers from day 1.
2. **The rewrite's prize was pluggability, not features.** The v1→v2 delta that shows in code is the `-cal` backend-contract layer and `-pal`/`-bb` splits — every capability v1 hardcoded became a runtime-selected trait. The feature list (pub/sub, events) is nearly v1 parity ("support at least the same feature set and platforms as iceoryx", `README.md`). Lesson: rewrite for seams, port features later.
3. **Pre-1.0 naming churn is real and must be budgeted.** v0.10.0 release notes rename whole crate families (`iceoryx2-gateway* → iceoryx2-link*`, table in `doc/release-notes/iceoryx2-v0.10.0.md`); the very first post-initial commits fixed crate-name style (`[#2] Use - instead of _ in crate names`, 2023-12-13). Lesson: freeze *names and layer boundaries* later than behavior; ModuKit's whitepaper §2.3 namespace plan should allow one post-0.x rename pass without treating it as failure.
4. **Host-to-host was redesigned twice before settling.** Gateway (v0.8-era) → link{backend=tunnel|gateway, carrier, adapter} (v0.10) with conformance suites per contract (`iceoryx2-link/README.md`). They chose to *use* Zenoh as one carrier rather than compete with it. Lesson for the whitepaper §8 Stage-2 fork: iceoryx2 and Zenoh compose (verified: `integrations/zenoh`), so the fork is narrower than "either/or".
5. **Ops failure modes became documentation.** `FAQ.md` grew per-error entries (stale resources, SIGBUS, corrupted services, fd exhaustion) — the price of decentralized SHM is user-visible cleanup paths; they also shipped `Node::force_remove_service` (#1584) as the escape hatch. Lesson: ModuKit's multi-process stage needs the equivalent "operator override" API designed in, not bolted on.

## 8. Value for ModuKit

Verdict: DEPENDENCY-CANDIDATE (primary) + BORROW-PATTERNS (bindings + backend-contract layer)

**License gate (for dependency use):** dual `MIT OR Apache-2.0` — permissive, user-selectable; Apache-2.0 leg carries an explicit patent grant, MIT leg is maximally compatible with any future ModuKit license (TBD then; resolved D1 dual the same day). SPDX headers per file; Eclipse Foundation governance. **Outcome: PASS for direct dependency, vendoring, and (via C ABI) for generated-binding redistribution.** No copyleft exposure. Only caution: `iceoryx2-cxx`/`-c` package their own copies of the same dual license — no conflict.

**Adopt directly (Stage 2):** depend on the `iceoryx2` crate as the zero-copy IPC substrate for multi-process plugins — pub/sub for data-flow plugins, event pattern for lifecycle signals, WaitSet as the cross-process readiness multiplexer. Its service discovery with attribute matching is close enough to ModuKit's registry semantics to reuse before reinventing. Adopt `iceoryx2-cal`-style runtime backend selection only if ModuKit targets non-Linux SHM-less platforms.

**Adapt (Stage 2/5):**
- C ABI generation recipe: `iceoryx2-ffi/c` (cbindgen config + `ffi-macros` + two-stage `iox2_*_storage_t`/`iox2_*_t` type erasure) is a working instance of ModuKit's "bindings generated from one C ABI" doctrine — copy the pattern, plus their cross-language e2e suite (`examples/cross-language-end-to-end-tests`, `component-tests/`).
- `static_config` vs `dynamic_config` split and `port_factory` builder pattern → shapes ModuKit's Stage-1 OSGi-style service registry (service descriptor immutable; live port state separate).
- `stale_resource_cleanup.rs` + `force_remove_service` + crash-tracking roadmap items → template for plugin-crash recovery in the multi-process kernel.

**Avoid:** pulling `iceoryx2-cxx`/hand-written binding layers (violates the generate-from-one-ABI rule); depending on `iceoryx2-link`/zenoh tunnel now — it is actively restructuring and belongs to a later ModuKit transport stage; hard-coupling core types to `ZeroCopySend`/service API before Stage-2 is gated — keep iceoryx2 behind an adapter trait so the unadjudicated §8 fork (iceoryx2 vs Zenoh) stays reversible until Stage 2 opens; consuming its pre-1.0 breaking surface without a pinned `Cargo.lock` (C-constraint: lockfiles commit).

**Stage mapping:** S1 in-process kernel → REFERENCE-ONLY (registry/builder/waitset shapes). S2 multi-process + zero-copy IPC → DEPENDENCY-CANDIDATE, the profile's core case. S3/S4 (UI compositor/SHM/transport scope) → BORROW-PATTERNS (`-cal` shared-memory/resizable-shm backends). S5 polyglot bindings → BORROW-PATTERNS (ffi layer is the strongest local evidence that the C-ABI-only exit scales to 3+ languages).
