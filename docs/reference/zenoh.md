# Reference Profile — Eclipse zenoh

> **External reference.** This clone is a read-only third-party checkout under `.refinfo/`;
> it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `74051d0cc9a` (main, 2026-10-05) · Upstream: <https://github.com/eclipse-zenoh/zenoh> · Cargo version: 1.10.1
> All facts below are derived from this local clone unless marked otherwise.

## 1. Portrait

| Field | Value |
|---|---|
| Project | Eclipse zenoh — "Zero Overhead Pub/Sub, Store/Query and Compute" (README) |
| Language | Rust (reference implementation of the Zenoh protocol) |
| Governance | Eclipse Foundation project; ECA (Eclipse Contributor Agreement) required (CONTRIBUTING.md) |
| License | `EPL-2.0 OR Apache-2.0` dual (LICENSE, Cargo.toml `license` field) |
| Core org | ZettaScale Team (NOTICE.md); top 6-month committers: 5–7 named humans + release bot |
| Size | 39 workspace crates (`find . -maxdepth 3 -name Cargo.toml`), 4,826 total commits |
| Velocity | 401 commits / 27 distinct contributors in last 12 months (`git log --since=12.months.1`) |
| Maturity | 63 tags; 1.0.0 tagged 2024-10-21; latest 1.10.1 tagged 2026-09-07; API in stable 1.x |
| MSRV | Rust 1.75 (`rust-version` in Cargo.toml; dedicated `ci/zenoh-1-75` workspace job; HEAD commit is "restore Rust 1.75 … compatibility") |
| Roles in repo | `zenoh/` lib crate, `zenohd/` router daemon, `zenoh-ext/`, `plugins/`, `io/` (links+transport), `commons/` (20 infra crates), `examples/` |

## 2. Architecture in focus

### 2.1 Pub/Sub/Query/Reply model
One key space (`key_expr` with wildcard matching) carries both data streams and RPC-shaped exchanges:
- Pub/sub: `zenoh/src/api/publisher.rs`, `subscriber.rs` — declarer/builder pattern (`zenoh/src/api/builders/{publisher,subscriber}.rs`), callback/async handlers in `zenoh/src/api/handlers/`.
- Store/query/reply: `queryable.rs`, `querier.rs`, `query.rs`, `reply.rs` (+ `builders/` twins) — a `Queryable` answers a `Query` with `Reply`es; `zenoh-ext` adds `publication_cache.rs`, `querying_subscriber.rs`, `advanced_publisher/subscriber` for delivery guarantees.
- Session is the sole object: `api/session.rs`; entry point is a `Zenoh` builder (`api/mod.rs`); connectivity/scouting in `api/{connectivity,scouting}.rs` (multicast router discovery), liveness in `api/liveliness.rs`, in-band admin in `api/admin.rs`.
- Network topology modes: `router | peer | client` (`DEFAULT_CONFIG.json5` `mode`); `zenohd` is the router binary. Plugins extend the router; application nodes embed the `zenoh` crate.

### 2.2 Transports / links
Layered: `io/zenoh-transport` (sessions, fragmentation, QoS, batch) over `io/zenoh-link-commons` + `io/zenoh-link` over 10 pluggable link crates in `io/zenoh-links/`: `tcp, tls, quic, quic_datagram, udp, ws, serial, vsock, unixpipe, unixsock_stream`. Endpoint URIs carry per-link options (`?prio=`, `?rel=`, `#iface=`, `#dscp=` — `DEFAULT_CONFIG.json5`). Zero-copy shared memory lives in `commons/zenoh-shm` (posix_shm, cleanup, metadata) and is wired into `io/zenoh-transport` as an optional dep; `commons/zenoh-uring` shows an io_uring fast path.

### 2.3 Plugin system — prime Stage-1 material (real paths)
`plugins/zenoh-plugin-trait/` is a complete, shipped dynamic-plugin ABI for a Rust host:
- **Lifecycle state machine**: `Declared → Loaded → Started` (`src/plugin.rs` `PluginState`), plus `PluginReport{level,messages}` per state and `PluginDiff{Delete,Start}` enabling **runtime plugin add/remove via config diff**.
- **Generic trait**: `Plugin<StartArgs, Instance>` (`src/plugin.rs`); host specializes it — `ZenohPlugin: Plugin<StartArgs = DynamicRuntime, Instance = RunningPlugin>` (`zenoh/src/api/plugins.rs`).
- **Dynamic loading**: `src/manager/dynamic_plugin.rs` uses `libloading`; sources are `ByName` (search dirs, file naming `libzenoh_plugin_<name>.so` — `PLUGIN_PREFIX` in `zenoh/src/api/plugins.rs`, CLI `-p name:path` and `--plugin-search-dir` in `zenohd/src/main.rs`) or `ByPaths`.
- **The exported surface is a vtable**: macro `declare_plugin!(Ty)` (`src/manager/dynamic_plugin.rs`) emits exactly three `#[no_mangle]` symbols — `get_plugin_loader_version() -> u64`, `get_compatibility() -> Compatibility`, `load_plugin() -> PluginVTable<…>` (`src/vtable.rs`, 79 lines).
- **ABI negotiation**: `src/compatibility.rs` — `Compatibility{rust_version, zenoh_version, zenoh_features}` compared before any pointer is trusted; the comment itself states feature mismatch can break ABI "even if the structure version is the same". ABI-stable strings via **stabby** (`stabby = "72.1.8"` in root Cargo.toml). `static_plugin.rs` lets the same plugin compile into the host statically.
- **Shipped plugins as evidence**: `plugins/zenoh-plugin-rest/` (HTTP gateway), `plugins/zenoh-plugin-storage-manager/` (store/query at rest; its `src/replication/{log,digest,classification}.rs` is a real CRDT-style replication core), `plugins/zenoh-backend-traits/` + `zenoh-backend-example/` (a second-level backend-plugin SPI), `zenoh-plugin-example/`. Config schema per plugin is JSON5/JSON (`zenoh_config::PluginLoad`, `plugins/<name>/__required__|__path__` in `DEFAULT_CONFIG.json5`).

## 3. Key capabilities
- Unified pub/sub + query/reply + storage + compute over one key space, across process/machine boundaries, with multicast scouting and liveliness.
- Three node modes and a router daemon with hot-loadable plugins (REST API, storage/replication) — the router is itself the reference modular host.
- 10 link types + optional SHM zero-copy + QoS/priority/DSCP per link; TLS/QUIC security out of the box.
- `no_std`/embedded story: `ci/nostd-check` workspace crate; pure-C `zenoh-pico` sibling for MCUs (README).
- Cross-binding serialization layer in `zenoh-ext` ("lightweight and universal for all zenoh bindings" — README).
- Dependency-license enforcement wired into CI (`deny.toml`, cargo-deny; `Cargo.toml` pins `jsonschema = 0.20` explicitly "because of invalid license").

## 4. Development & current state
- **Velocity**: 401 commits / 27 contributors in trailing 12 months; steady (no month <30 commits in last 6). Total history 4,826 commits since first commit 2020-03-05 ("Initial code contribution").
- **Concentration**: 6-month top humans ~30/11/9/7/5/5/4 commits (CY Kuo, M. Mazouz, D. Matsubara, M. Ilyin, J. Enoch, J. Perez, O. Hecart) + `eclipse-zenoh-bot` 41 — ZettaScale-driven with Eclipse branding.
- **Release train**: 63 tags; frequent ~monthly minors under stable semver 1.x since 2024-10. `1.10.1` (2026-09-07) is latest; HEAD (2026-10-05) is a compatibility fix, not a feature.
- **License history**: the LICENSE blob in the initial commit already reads `apache-2.0 / epl-2.0` — **no relicensing commit exists inside this clone** (`git log --all -- LICENSE | wc -l` = 1). Dual-license arrived with the "First commit of Rust re-write" metadata generation. Pre-public provenance is squashed away.
- **Hygiene**: rustfmt/clippy configs, `Cross.toml`, fuzz CI (`fuzz-scheduled.yml`), valgrind CI crate, codecov badge; MSRV 1.75 actively defended.
- **Gap**: `CHANGELOG.md` is a **0-byte file** — release notes live only in GitHub releases/roadmap repo.

## 5. Ecosystem
- **Binding siblings** (README "Language Support"): `zenoh-c` (FFI binding over the Rust core), `zenoh-cpp` (C++ wrapper **over the C library**, not Rust), `zenoh-python`, `zenoh-go`, `zenoh-pico` (independent pure-C). One Rust core → one C exit → thin native façades.
- **Roadmap/discussions** live in a separate `eclipse-zenoh/roadmap` repo (README badge).
- Plugin ecosystem in-repo is small (rest, storage-manager, examples) but the trait is public API: out-of-tree zenohd plugins share exactly the ABI analyzed in §2.3; backends (`zenoh-backend-traits`) form a plugin-of-plugin SPI with versioned `struct_version`/`struct_features` strings.
- Published on crates.io (`zenoh`, `zenohd`, `zenoh-ext`, `zenoh-plugin-*`); Eclipse Foundation IP/process under ECA.

## 6. Highlights & limitations
**Highlights**
1. The most complete working Rust dynamic-plugin ABI found locally: vtable + exported-symbol contract + rustc/version/feature compatibility negotiation before dereferencing anything (§2.3). Direct Stage-1 design material.
2. Config-driven plugin lifecycle with runtime add/remove (`PluginDiff`) and per-plugin JSON schema — a modular-host control plane already validated at scale.
3. Its polyglot strategy **is** ModuKit's strategy: single Rust core, single C FFI exit, generated/thin façades per language.
4. Stable 1.x API for two years with monthly releases; disciplined MSRV and CI gates.

**Limitations**
1. Heavy: a distributed pub/sub engine (routing, scouting, gossip-adjacent networking, 39 crates, async runtimes) — enormous surface if ModuKit only wants local plugin data paths.
2. Its plugin ABI is Rust↔Rust (stabby/libloading), which **violates ModuKit's C-ABI-only exit rule**: plugin must be rebuilt against the exact zenoh version/features — negotiation is effectively all-or-nothing.
3. EPL-2.0 leg is weak-copyleft; only the Apache-2.0 leg is freely combinable (see §8 gate).
4. Zero-copy/SHM is a bolt-on option to a network-first data model, not the core abstraction.
5. Corporate-critical mass is one company (ZettaScale); Eclipse governance mitigates but does not remove it.

## 7. Historical lessons
1. **Clean-donation relicensing**: entering Eclipse left no visible relicensing churn in this clone — the initial squashed commit carries the dual header. For ModuKit: if relicensing ever happens, a version-tagged dual grant is the cheap path; the squashed import also shows provenance can become unauditable in one commit.
2. **0.x → 1.0 in one leap**: 0.11.0 (2024-06-04) → 1.0.0 (2024-10-21) shipped a redesigned builder/declarer API in ~4.5 months; the entire 0.x line was breakage. Lesson: publish the plugin ABI only when willing to freeze it; zenoh froze `PLUGIN_LOADER_VERSION` semantics and now version-negotiates instead of breaking.
3. **License regressions are live dependencies, not history**: the `jsonschema` pin comment (`Cargo.toml:129`) proves a transitive dep's license change forces a version freeze enforced by cargo-deny. ModuKit needs the same gate from day one.
4. **Empty in-repo changelog**: 63 releases, `CHANGELOG.md` = 0 bytes — release communication drifted off-repo and the repo copy rotted. Either generate changelog from tags or delete the file; a stub is worse than nothing.
5. **MSRV is a product promise with a CI cost**: the HEAD commit itself restores Rust 1.75 compat and a whole `ci/zenoh-1-75` workspace exists for it — decide ModuKit's MSRV policy before the first crate ships.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS** (conditional DEPENDENCY-CANDIDATE only if/when Stage-2 needs cross-machine transport; not for the local data path).

### 8.1 Compare vs iceoryx2 (Stage-2 fork, local evidence only)
| Axis | zenoh (this clone) | iceoryx2 (`.refinfo/iceoryx2`, 2026-10-08) |
|---|---|---|
| Center of gravity | network-first pub/sub/query; SHM is an optional link enhancement | shared-memory IPC is the whole product |
| License | EPL-2.0 **OR** Apache-2.0 | MIT **OR** Apache-2.0 (both legs permissive) |
| Maturity signal | stable 1.x, 63 tags, 1.0 in 2024 | pre-1.0: tags v0.x, workspace version `0.10.999` |
| 12-mo velocity | 401 commits / 27 contributors | 3,858 commits / 46 contributors |
| Fit to "in-process/local plugin data" | heavyweight indirection | direct fit |
For the whitepaper §8 Stage-2 fork: if the requirement is *local* zero-copy between plugin processes, iceoryx2-style SHM is the smaller honest dependency; zenoh only wins the fork if cross-host transport is actually on the near roadmap. Either way zenoh's §2.3 plugin ABI is portable design, not a dependency.

### 8.2 Stage matrix
| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Adapt** | Copy the shape of `zenoh-plugin-trait`: Declared/Loaded/Started state machine, report-per-state, config-diff hot add/remove, name-search dirs + `<name>:<path>` CLI, one-vtable export contract. Replace its Rust-ABI (stabby) core with ModuKit's mandated C vtable — zenoh's own C-exit philosophy applied to its blind spot. |
| 2 — transport/SHM fork | **Avoid-as-dependency** (for local path) | Adopt its *key-space semantics*: pub + queryable/reply is exactly the service-registry shape (`zenoh/src/api/{queryable,querier,reply}.rs`). Import zenoh-c only if a cross-machine stage gets promoted to the roadmap. |
| 3 — registry/services | **Adapt** | Its `zenoh-backend-traits` two-level SPI (host plugin → backend plugin, versioned struct features) is the pattern for a registry with pluggable backends; replication modules (`log/digest/classification`) inform consistency choices later. |
| 4 — compositor/UI | **Reference-only** | Nothing local; link QoS vocabulary (`prio/rel/dscp` in endpoint URIs) is a cheap idea to mirror. |
| 5 — polyglot bindings | **Adopt** (strategy) | zenoh validates the plan end-to-end: Rust core → single C lib (`zenoh-c`) → façades per language (`zenoh-cpp` wraps C, python/go likewise) + one universal serialization layer (`zenoh-ext`) so all bindings speak one payload format. The negative proof is `zenoh-pico`: a second implementation in C costs a whole sibling org — never hand-write the low layer twice. |

### 8.3 License gate
`EPL-2.0 OR Apache-2.0` → **take the Apache-2.0 leg** for any code or pattern-derived work (patterns/ideas are not copyright-encumbered regardless). EPL leg is weak file-level copyleft — irrelevant while Apache leg is chosen, but pin "Apache-2.0 (user choice of EPL-2.0 OR Apache-2.0)" in the future SBOM/NOTICE from day one. No relicensing risk observed in-clone (dual since import); ECA obligations bind contributors upstream, not ModuKit downstream. Gate action: any Stage-2 zenoh-c adoption must re-run `git log -- LICENSE` upstream at pin time; record result in `decisions.md` at the fork's resolution.
