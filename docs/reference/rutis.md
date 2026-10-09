# Reference Profile — rutis (arcships/rutis)

> **External reference.** This profile was researched from a disposable clone at
> `~/.cache/modukit-research/rutis` (remote verified `https://github.com/arcships/rutis`; not under
> `.refinfo/`); it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `a8733d4` (main, 2026-10-08) · Latest tag: `v0.8.0` (2026-10-08)
> · Upstream: <https://github.com/arcships/rutis>
> This is the commissioned follow-up of the canon audit (`whitepaper-cited-unvendored.md`, verdict
> "REAL & highly relevant") — the only live project named by the ModuKit whitepaper at Stage 1.
> All facts below are derived from this local clone unless marked **UNCERTAIN**.

## 1. Portrait

| Field | Value |
|---|---|
| Project | rutis — "A plugin runtime for programs that keep running" (README); Rust core · plugins in TypeScript/JavaScript and Python · across processes and machines |
| Language | Rust (core + host, tokio-based); separate **full language runtimes** in Node.js (`node/`, ESM, engines `>=24`) and pure Python (`python/`, `>=3.12`, **zero runtime dependencies** per `python/rutis/pyproject.toml`) |
| Heritage | "The model comes from Cordis in the TypeScript ecosystem. rutis is its idiomatic Rust implementation" (README); LICENSE carries `Copyright (c) 2021-present Shigma (Cordis)` + `Copyright (c) 2026 eric8810 (rutis)` |
| Governance | 2-person project; CONTRIBUTING.md; heavy dated design-doc culture under `docs/` (see §4); repo is bilingual Chinese/English (`.en.md` twins, Chinese code comments in `crates/rutis/src`) |
| License | **MIT** — permissive, no copyleft (LICENSE). License gate in §8.4 is trivially green |
| Size | 14 workspace crates (`crates/`: rutis, rutis-bridge, rutis-host, rutis-loader, rutis-sdk, rutis-dylib, rutis-dylib-meta, rutis-dylib-launcher, rutis-cli, rutis-agent, rutis-dsh, rutis-dev, aimux-llm, rutis-xtask) + `examples/` + dylib test fixtures + `node/` + `python/`; 83,731 `.rs` lines; 99 files under `tests/` dirs |
| Velocity | **729 commits — all inside 2026-08-18 → 2026-10-08 (52 days)**; 2 distinct git authors (eric8810: 387, scotares: 342). 8 core releases in that window (`v0.1.0` 2026-08-19 → `v0.8.0` 2026-10-08) |
| Maturity | Pre-1.0, ~7 weeks old at research date. Tag lineage shows a crate-naming/version consolidation mid-flight (`rutis-v0.3.0/0.5.0/0.6.x` prefixes, separate `interop-*` and `loader-*` trains, unified into one lockstep `v0.7.0/v0.8.0` train — `docs/migration-0.7-to-0.8.en.md`) |
| Adoption | ★91 and "pushed 2026-10-09" per the canon-audit snapshot (`whitepaper-cited-unvendored.md`); live repo API re-check was rate-limited on research day, so exact current stars are **UNCERTAIN**. Published on crates.io (`rutis`), npm (`@arcships/rutis`, `@arcships/rutis-host`), PyPI (`rutis`), docs.rs. No third-party adoption observable from the clone — the in-repo apps (`rutis-agent` TUI agent, `aimux-llm`, `rutis-dsh`) read as dogfood, not users |

## 2. Architecture in focus

### 2.1 The core: five pillars, dependency-driven fibers

`crates/rutis/src/lib.rs` states the model as five pillars (ported from Cordis): (1) a plugin is a unit
of assembly — one `apply` provides services, registers listeners, records cleanup; (2) each plugin runs
in its own **fiber**, a lifecycle container with a **six-state machine** — `Pending → Loading → Active`
on success, `Loading → Failed` with rollback on error, `Active/Failed → Unloading → Pending|Disposed`
(`FiberState`, `crates/rutis/src/fiber.rs`); (3) services live in a **typed-key registry** with isolate
scopes (`key.rs`: `Key`/`ServiceKey`/`TypeKey`/`InstanceId`; `Ctx::provide*` at `ctx.rs:842+`);
(4) an event bus with four dispatch semantics (emit/parallel/serial/waterfall, `bus.rs`); (5) **dependencies
drive reload** — when a provider is replaced, only its consumers are evicted and restarted. Cleanup is
exactly-once, LIFO, via `Disposer`/`Effect` (`effect.rs`); a failed load rolls back what it had already
registered, and child plugins cascade-unload with the parent (`ctx.rs:1120+`). Declaration is typed:
`typed.rs` (`Typed`, `Deps`, `Gate`) keeps declared dependencies and actually-used services in
agreement **at compile time**. `diagnostics.rs` exposes a first-class runtime-introspection surface
(`RuntimeDiagnostics`, `DependencyStatus`, `EventBacklog`).

This is the same design space ModuKit Stage 1 occupies (OSGi-style lifecycle + service registry), and
the closest live Rust embodiment found by the audit.

### 2.2 Polyglot faces via protocol, not FFI

rutis does **not** embed Node or Python, and there is **no napi-rs/PyO3 anywhere in the tree**
(grep of `crates/rutis-host/Cargo.toml`, `node/*/package.json`, `python/rutis/pyproject.toml`). Each
language face is a **complete independent runtime** running as its own process, attached to the Rust
host over a byte-channel carrying one versioned JSON session protocol:

```text
            one Rust host process                              plugin processes (spliced by rutis-loader rows)
 ┌────────────────────────────────────────┐     ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐
 │ crates/rutis  core                     │     │ node runtime     │  │ python runtime   │  │ remote node      │
 │  fiber FSM · typed registry · event bus│     │ @arcships/rutis  │  │ `python -m rutis`│  │ (any of these,   │
 │  ctx.provide/use · dispose LIFO        │     │ +rutis-runtime   │  │ (pure stdlib)    │  │  over TLS)       │
 │                                        │     └────────┬────────┘  └────────┬────────┘  └────────┬─────────┘
 │ crates/rutis-bridge                    │              │                    │                    │
 │  session/protocol.rs  Frame: hello/    │   fd:3 inherited  unix:<path> /  │ tcp:+token (win)  │ listen:ws(s)://
 │    invoke/call/get/set · WireValue     │ ◀────────────┘ channel/mod.rs    ◀───────────────────◀─────────────┘
 │  runtime/{spawn,unix,process,local}    │   ordered reliable duplex bytes · identity out-of-band (ChannelInfo)
 └────────────────────────────────────────┘
```

Mechanics, from `crates/rutis-bridge/src/session/protocol.rs`, `channel/mod.rs`, `runtime/*`,
`python/rutis/rutis/__main__.py`, `node/rutis-runtime/src/runner.mjs`:

- **Frame protocol**: serde-tagged JSON frames (`op` field, `deny_unknown_fields`): `hello` (version 2
  "compat" vs version 3 "endpoint" format, which also negotiates `Implementation` and `capabilities`),
  `invoke` (call a service method by path), `call` (a function reference), plus object get/set.
- **Live references**: `WireValue` is `undefined | data | list | record | signal | reference` — a
  `reference` carries `{ id: u64, home: bool, kind: function|future|object }`, so objects and callbacks
  **stay alive across the boundary**; `signal` propagates caller cancellation (JS `AbortSignal` semantics)
  into the callee. Errors cross as named `Failure { name, message, graph }` — including a dedicated
  `SyncWaitCycle` error for cyclic synchronous waits detected across the wire.
- **Channel/transport split** (`docs/design-protocol-channel-decoupling-2026-10-03.md`): a `Channel` is
  one ordered, reliable, duplex message stream knowing nothing of the protocol; implementations are
  inherited fd sockets (`fd:3`), `unix:<path>`, `tcp:host:port` with a `RUTIS_CHANNEL_TOKEN` (the Windows
  process-attach path), in-memory channels, and WebSocket+TLS (rustls), with endpoint-takeover close
  code 4002 and out-of-band identity via `ChannelInfo`.
- **Multi-node**: `LinkPlugin` dials/listens and reconnects; over one session, nodes `Export`/`Import`
  services, run plugins **for each other** (`HostPlugin`), and forward events (`EventsPlugin`)
  (`crates/rutis-bridge/README.md`). A plugin cannot tell if `llm` is Rust, Python, or another machine.
- **Conformance discipline**: the same conformance suites run against every face — Rust
  `tests/{node_conformance,python_runtime,memory_mux,local_soak,multihop,process_exit}.rs` and
  `python/rutis/tests/conformance_*.py`.
- **Cordis mount**: the `cordis` feature generates Rust bindings **at build time** (syn) for published
  Cordis JS plugins — a third integration mode beyond "in-Rust" and "over protocol".

### 2.3 Native plugins: the C ABI they deleted, and the shared-SDK ABI they chose

`docs/design-dylib-sdk-2026-09-24.en.md` §2 records that a first hot-plug experiment (**cdylib + C ABI +
JSON strings**, commit `14f8c53`, deleted) was abandoned: it "incurred serialization without isolation"
and `dyn PluginFactory` could not cross the boundary. The replacement for trusted first-party plugins is
the opposite polarity of ModuKit's doctrine: a **shared Rust-ABI SDK dylib**:

- `crates/rutis-sdk` is `crate-type = ["dylib"]` re-exporting `rutis`, `tokio`, `serde_json`, one
  `#[global_allocator]` (forced `System` — with a dynamic libstd a custom allocator crashes on
  Mach-O/Windows/Linux, documented at `src/lib.rs:17-22` with rust-lang issue refs), plus compile-time
  guards (`compile_error!` under `panic = "abort"`).
- Compatibility is **verified before execution**: a 512-byte boot blob (`BOOT_MAGIC
  "RUTIS_PLUGIN_BOOT_V1\0"` + `sdk_id` + `sdk_artifact_sha256` + plugin id/version, packed by const-fn
  `boot_meta`) is embedded pre-`dlopen`; `rutis-dylib-meta` parses ELF/Mach-O/PE to check the dependency
  chain statically; an independent launcher (`rutis-dylib-launcher`) verifies artifact hashes; the only
  `extern "C"` symbol is a diagnostic — `rutis_sdk_boot_id(buf, cap)` (`rutis-sdk/src/lib.rs:30`).
  Design goal 3 in words: *"do not use `dlopen` succeeded as a compatibility test."*
- Because host and every plugin link **the same** `librutis_sdk.so` (same compiler, same build config),
  Rust `TypeId`s agree across the boundary and arbitrary Rust types (`Box<dyn PluginFactory>`, `Ctx`,
  services) move through **without serialization**; registry keys stay type-identity based instead of
  becoming stable strings.
- Declared non-goals (§1 of that doc): third-party/untrusted/crash-isolated plugins go to the protocol
  face instead (`docs/design-protocol-plugins-2026-09-25.en.md`, #46–48); `dlclose` is not v1 (§9); no
  `abi_stable`-style cross-rustc compatibility; updates have an unload/load gap (#51).

### 2.4 Zero-Rust hosting and the loader

`crates/rutis-loader` expresses "which plugins run" as layered, reconciled configuration; `rutis-host`
(`crates/rutis-host`, npm/uvx/cargo-install/binary distribution) runs TypeScript/JavaScript/Python plugins
from a `rutis.json` with live-reload `dev`, a static `check` (describe every row, fail on what cannot
run), and node linking — "run plugins without a line of Rust" (README). Template projects ship for both
languages (`crates/rutis-host/templates/{node,python}/`).

## 3. Key capabilities

- Dependency-gated lifecycle with exactly-once LIFO cleanup, failed-load rollback, cascade child unload,
  and provider-swap → consumer-evict-and-reload locality (the whole README "Features" section).
- Hot config update and provider swap without stopping unrelated plugins.
- Polyglot services called by name across Rust/TS/JS/Python with live object/function references,
  futures, and cancellation signals over one JSON session protocol (version-negotiated, `deny_unknown_fields`).
- Multi-machine mesh: services shared, plugins remotely spawned, events forwarded, reconnect after drop.
- Typed plugins: compile-time agreement between declared and used dependencies (`typed.rs`; the npm face
  checks `inject`/`provides` shapes the same way).
- Native dylib hot-update of trusted first-party plugins through a content-addressed, pre-verified SDK
  boundary (design doc + loader/launcher/meta crates in-tree; first version without `dlclose`).
- A runtime-introspection/diagnostics API (`PluginDiagnostics`, `ResolvedDependency`, `ServiceAccess`).
- `rutis-host check` — fail-at-describe-time validation of plugin configurations before launch.

## 4. Development & current state

- HEAD `a8733d4` (2026-10-08, PR #172 "docs/design-philosophy-rewrite"); `v0.8.0` tagged the same day;
  every published artifact (crates, npm, PyPI) is at 0.8.0 in one train.
- Release cadence: 8 core tags in 52 days; migration guides per jump (`docs/migration-0.1-to-02.en.md`
  … `migration-0.7-to-0.8.en.md`) — breaking-change documentation at near-daily frequency.
- 0.7→0.8 was a *coordination* release, not a semantic one: the core "moves from 0.6.1 straight to 0.8.0
  with no code changes" to stop users mixing two mismatched core copies; wire protocol unchanged, with
  documented partial back-compat (0.7 runtimes still run rows outside instances).
- Design docs are dated and cross-referenced (`design-dylib-sdk-2026-09-24`, `design-protocol-plugins-2026-09-25`,
  `design-protocol-channel-decoupling-2026-10-03`, `design-dual-core-2026-08-20`, research notes on hot
  reload and Rust async, Windows dylib probes under `docs/probes/windows-dylib/`).
- CI: `ci.yml`, `dylib-windows.yml`, `stress.yml`, `release.yml`, `release-cli.yml` (`.github/workflows/`).
  Remote branches named `claude/*` and `codex/*` — the project is openly AI-assisted in its workflow.
- Honest age read: first commit "extracted from min-cordis" (2026-08-18); everything else — the
  `interop`→`bridge` rename, the deleted hotplug prototype, the version-train unification — happened in
  the same 7 weeks. Direction and API shape are still moving (§7.5).

## 5. Ecosystem

- **Upstream**: Cordis (`shigma/cordis`) — the TS model; rutis mounts published Cordis plugins into Rust
  via build-time binding generation, so it inherits Cordis's plugin corpus as a (TS-side) ecosystem day one.
- **Distribution surfaces**: crates.io (`rutis`, `rutis-bridge`, `rutis-loader`, `rutis-host`, `rutis-sdk`),
  npm (`@arcships/rutis`, `@arcships/rutis-host`, `@arcships/rutis-runtime`), PyPI (`rutis`), GitHub release
  binaries. Badge wall in README; docs.rs live.
- **Adjacent names**: `aimux-llm` (LLM-provider multiplexer plugin) and `rutis-agent` (agent loop + TUI +
  bash/replace-text tools) are in-repo showcases, not external ecosystem; `rutis-dsh` is an internal app
  whose purpose was not fully surveyed — **UNCERTAIN**.
- **No observable third-party users** from the clone; download/adoption stats **UNCERTAIN** (registry
  queries not performed — API rate limits hit during research).
- ModuKit's own canon now places rutis beside CppMicroServices as the Stage-1 reference pair
  (`whitepaper-cited-unvendored.md`, recommended action 1) — CppMS supplies the semantics, rutis the live
  Rust-shape comparison.

## 6. Highlights & limitations

**Highlights**

1. **The only live Rust embodiment of our exact Stage-1 thesis** — dependency-driven lifecycle, typed
   service registry, exactly-once LIFO cleanup, provider-swap reload locality — shipped, benchmarked
   (`crates/rutis/benches/`), and soak-tested (`tests/local_soak.rs`) in 7 weeks by 2 people.
2. **Compatibility-as-data on the native boundary**: boot blob (magic + sdk_id + artifact sha256) readable
   *before* `dlopen`, binary-metadata dependency-chain verification, independent pre-execution launcher,
   one `extern "C"` diagnostic symbol. "dlopen succeeded" is explicitly rejected as a compatibility test —
   the same load-time-seal lesson uniffi encodes as checksums (§ uniffi-rs.7.3), with a *pre-execution* twist.
3. **Protocol-first polyglot scales absurdly cheaply**: three faces (Node, Python, remote Rust nodes) with
   zero binding generation and a zero-dependency Python package, because every face implements one small
   versioned JSON frame protocol over an abstract ordered channel. Cross-language conformance suites keep
   the promise honest.
4. **`WireValue` reference model** — function/object/future references by `u64` id with `home` ownership +
   `AbortSignal` propagation + named remote errors — is a complete, proven answer to "callbacks and live
   objects across a boundary" that our C-ABI plan still has to design.
5. **Documentation discipline**: dated design docs, per-jump migration guides, `check` command as static
   fail-fast, probes/experiments directories recording failed approaches (deleted hotplug commit cited by
   hash) — the failure ledger culture ModuKit's `.agents/` rules mandate, externalized.

**Limitations**

1. **Seven weeks old, bus factor 2, ★91**: pre-1.0 with near-weekly breaking minors; the whitepaper
   "only LIVE named project" status cuts both ways — it is evidence the shape is buildable, not that it is
   durable. Do not depend; profile and borrow.
2. **Its native ABI is anti-C-ABI**: the shared-SDK design *requires* the same compiler and build
   configuration on both sides (rustc-version compat explicitly out of scope), so `libstd-<hash>.so` +
   `librutis_sdk.so` pinning buys rich cross-dylib Rust types at the price of total toolchain lock-in —
   the exact failure mode ModuKit's "C ABI as the only cross-language exit" exists to avoid.
3. **Every cross-language call serialises JSON** (`WireValue`/serde): right for a control plane, wrong
   shape for our Stage-2 I420/SHM frame path — identical to the `RustBuffer` critique in uniffi-rs §6.
4. **Polyglot = subprocess-per-language, not in-process embedding.** No PyO3/napi means their managed faces
   can never share our host address space; crash isolation is free but per-call latency and process sprawl
   are paid. Their untrusted-code story (protocol plugins) is design-doc + issues (#46–48), not shipped.
5. **Windows is a second-class local face**: "Local runtimes need Linux or macOS" (`rutis-bridge/README.md`);
   the Windows attach path is TCP-loopback processes, and Windows dylib work lives under probe docs.
6. **Naming/scope churn**: `interop` → `bridge`, split version trains → lockstep 0.8, deleted experiments —
   the API surface a Stage-1 adopter would mirror is still consolidating.

## 7. Historical lessons

1. **They tried our doctrine and deleted it.** A cdylib + C ABI + JSON-strings hot-plug prototype shipped
   and was removed (`14f8c53`) as "serialization without isolation" (`design-dylib-sdk §2`). Reading for
   ModuKit: a flat C ABI *between Rust peers* gives you neither Rust-type richness nor isolation — it is
   the wrong tool for same-language extension; it earns its place only at the language frontier. Design
   our C ABI for the frontier, and give same-language plugins a better door (their answer: shared SDK;
   ours could be a stable trait-object + contract-version scheme).
2. **TypeId identity is the hidden blocker for registry-backed dylib plugins.** Their fix: one shared SDK
   dylib so `TypeId`s agree, so registry keys can stay type-based (`design-dylib-sdk §2`, research-hot-reload
   §4.4). ModuKit's "type-safe service registry" over `cdylib`s must decide *up front* between (a) shared-runtime
   identity or (b) stable string/symbol service keys — uniffi's profile arrived at the same fork from the
   metadata direction. rutis picks (a); a C-ABI-only design forces (b).
3. **Verify before you execute; put compatibility in file data.** Boot blob + artifact sha256 + static
   ELF/Mach-O/PE dependency-chain parse + independent launcher = incompatible plugins rejected "with a
   readable reason" before any SDK code runs. This is directly liftable to `modukit-c`: a static,
   parseable identity section beats symbol-probing at `dlopen`, and beats "it loaded, hope it's right".
4. **Polyglot via one protocol is dramatically cheaper than polyglot via generated bindings — if
   subprocess cost is acceptable.** Node + Python + remote faces in one repo with zero codegen (except the
   optional Cordis mount) and a stdlib-only Python package. Our Stage-5 plan should split by trust and
   locality: in-process generated bindings where embedding matters, protocol faces where isolation matters
   — rutis proves the protocol half can carry the whole ecosystem.
5. **Cross-language version skew is a self-inflicted outage.** Two cores whose "types do not match" was a
   real user failure mode, cured by a lockstep train + per-jump migration guides + protocol-version
   negotiation with documented partial back-compat (`migration-0.7-to-0.8.en.md`). ModuKit's release plan
   (crates + npm + PyPI + C ABI) inherits the same hazard; adopt train-versioning and `hello`-style
   capability negotiation from day one.
6. **Six states beat four.** rutis kept Cordis's six-state fiber (`fiber.rs`) rather than the four-state
   OSGi textbook; `Failed` as a *reloadable* state (dependency back → `Unloading` → `Pending`) is what
   makes provider-swap reload locality clean. Our Stage-1 state machine should model failure as live,
   not terminal.
7. **Canon audits pay rent immediately.** This profile exists because `whitepaper-cited-unvendored.md`
   checked one whitepaper name against GitHub and found the *only* cited Stage-1 project that actually
   lived (7 names queried, 2 real). See `sysplugin.md` for the counter-case. Never inherit an unverified
   reference.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS (high priority, Stage-1 semantics + ABI-safety mechanisms) — watch closely, do
NOT adopt as a dependency (age × bus factor).**

### 8.1 Stage 1 — the live Rust shape of our kernel

Map directly: our planned install/start/stop/update/uninstall ↔ their six-state fiber; service registry ↔
`TypeKey`/`ServiceKey` + isolate scopes; dependency injection ↔ `inject`/`provides` + consumer-eviction;
leak prevention ↔ `Disposer` LIFO exactly-once + load rollback. Two concrete lifts: (1) provider-swap →
only-consumers-restart as the *update* primitive (their "change without downtime"); (2) a first-class
diagnostics object (`RuntimeDiagnostics`) instead of log-soup, plus a `rutis-host check`-style static
row validation. Compare against CppMicroServices' 6-state + lease semantics (`cppmicroservices.md`) —
with two such different lineages converging on ~6 states and dependency-driven reload, our Stage-1 spec
should stop treating OSGi 4-state as canonical.

### 8.2 Stage 5 / ABI doctrine — the second counter-evidence

uniffi said "the C ABI should be *generated* from a typed model"; rutis says "for same-language
extension, don't cross a C ABI at all — share one SDK and check identity before execution." Both
challenge the whitepaper's "C ABI as the only cross-language exit" phrasing from opposite sides.
Synthesis recommendation for the next whitepaper revision: keep a hand-authored **flat C ABI as the
language frontier** (foreign faces), add a **pre-execution compatibility identity** on the Rust-side
door (blob + hash, §7.3), and decide the TypeId fork explicitly (§7.2) as a D-record.

### 8.3 Stage matrix

| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Borrow (primary)** | Fiber FSM incl. reloadable `Failed`, typed registry, LIFO disposers, dependency-driven reload, diagnostics + `check`; the closest live analog to design against, alongside CppMS. |
| 2 — transport/SHM | **Reference-only** | JSON serialise-per-call + ordered byte-channel abstraction; control-plane lessons yes, frame path no. Their Windows TCP-token attach is a sane fallback pattern for our multi-process plugins. |
| 3 — registry/services | **Borrow (narrow)** | Isolate scopes, `InstanceId` vs type keys, service `intercept`/`ServiceWriter` (controlled mutation of provided services). |
| 4 — compositor/UI | **N/A** | Nothing UI. |
| 5 — polyglot bindings | **Study as rival design** | Protocol faces vs our generated-binding faces: cost model proven (§7.4); `WireValue` reference/signal model liftable to our C ABI callback layer; Cordis-mount build-time codegen is a third option worth naming in the Stage-5 decision. |

### 8.4 License gate

**MIT** — the cheapest file in this series: permissive, no copyleft, no SBOM condition beyond keeping
the copyright notice. Code-level borrowing (patterns and mechanisms, not copies) carries no obligations;
if any snippet is ever lifted verbatim, retain both copyright lines (Shigma/Cordis 2021-present,
eric8810 2026) in NOTES. Re-run `git log -- LICENSE` upstream at any future pin time. No interaction with
ModuKit's undecided license.

### 8.5 Not verified here (UNCERTAIN)

- **Live stars/activity/adoption**: ★91 and pushed-date are the 2026-10-09 audit snapshot; GitHub repo API
  was rate-limited during this research and npm/PyPI download counts were not queried.
- **Third-party users**: none observable in the clone; the in-repo apps are dogfood.
- **Dylib variant shipping status**: loader/launcher/meta crates exist and Windows is CI'd, but the
  "optional host build variant" language + probe docs suggest partial — check before citing as shipped.
- **Protocol-plugin (untrusted) maturity**: design doc + issue range (#46–48), implementation state not
  surveyed.
- **`rutis-dsh` purpose**: not fully read; listed as internal app only.
