# WASM Plugin Hosts — External Reference Survey

> **External reference.** This survey was researched from disposable blobless clones at
> `~/.cache/modukit-research/extism` and `~/.cache/modukit-research/wasmtime` (not under `.refinfo/`);
> it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Extism checkout `d5da297` (main, 2026-09-02, tag `v1.40.0`) ·
> <https://github.com/extism/extism> · Wasmtime checkout `89b18a7b3` (main, 2026-10-08, latest release `v49.0.0` 2026-09-21) ·
> <https://github.com/bytecodealliance/wasmtime>
> All facts below are derived from these local clones (plus GitHub repo pages for stars, fetched 2026-10-09)
> unless marked **UNCERTAIN**. Cross-reference gap: `docs/reference/zellij.md` (kibit-wasm plugin host)
> does not exist yet — that comparison is **pending**, deferred to the zellij profile author.

## 1. Portrait

| Field | Extism | Wasmtime |
|---|---|---|
| Project | "the WebAssembly framework" — universal wasm plugin engine with per-language host SDKs (README) | "a lightweight WebAssembly runtime that is fast, secure, and standards-compliant" |
| Role in this survey | Batteries-included **plugin host layer** on top of Wasmtime | The **runtime** itself + typed component model (the layer plugins are built ON) |
| Stars / forks / open issues | 5.8k / 168 / 42 (GitHub page, 2026-10-09) | 18.7k / 1.9k / 773 (GitHub page, 2026-10-09) |
| License | **BSD-3-Clause** (LICENSE, "Copyright 2022 Dylibso, Inc.") | **Apache-2.0** (LICENSE) |
| Size | 6 workspace crates (`runtime`, `manifest`, `convert`, `convert-macros`, `libextism`, `extism-maturin`) + `kernel` (separate wasm); 692 commits | `cranelift` + `pulley` + `winch` trees, ~37 entries under `crates/`; 17,340 commits |
| Velocity (12 mo) | 8 commits / 6 authors — maintenance mode | 1,763 commits / 185 authors — peak-class activity |
| Yearly commits | 2022=250, 2023=302, 2024=99, 2025=35, 2026=6 (YTD) | 2023=1861, 2024=1502, 2025=1515, 2026=1533 (steady ~1.5k/yr) |
| Contributors (all-time) | 39 | 798 |
| Release model | Tags `v1.9.x`...`v1.40.0` (2026-09-02); each minor = a pinned Wasmtime major bump | Monthly majors; every 12th = LTS with 24-month support (`docs/stability-release.md:11-12`); measured: `v36.0.17`, `v48.0.5`, `v49.0.2` all patched 2026-10-02 in parallel |
| Governance | Single vendor (Dylibso); EIP process via `extism/proposals` tracker | Bytecode Alliance: RFC process, OSS-Fuzz 24/7 fuzzing, formal security policy, ADOPTERS.md |
| Pushed (HEAD) | 2026-09-02 | 2026-10-08 |

## 2. Architecture in focus: the host-guest ABI shape

### 2.1 Extism — a flat C ABI on BOTH boundaries

Extism's polyglot story is literally ModuKit's doctrine applied to wasm. Two boundaries, both untyped
byte-and-handle contracts:

**(a) Host-facing C ABI** — `runtime/src/sdk.rs` exports **33 `#[no_mangle] extern "C"` functions**
(`libextism` is a one-line cdylib wrapper: `pub use extism::sdk::*;`, `libextism/src/lib.rs:3`).
The surface: `extism_plugin_new(manifest_json, ..., imports, host_count)` / `extism_plugin_call` /
`extism_plugin_output_data` + `extism_plugin_output_length` / `extism_plugin_error` / `extism_plugin_free`,
host-callback construction via `extism_function_new` + `extism_function_free`, cancellation via
`extism_plugin_cancel` + `extism_plugin_cancel_handle`, fuel-gated variant
`extism_plugin_new_with_fuel_limit`, memory access for callbacks
(`extism_current_plugin_memory_alloc/_free/_length`), logging (`extism_log_file/_drain/_custom`),
`extism_version`. **Every one of the 15 host SDKs (Go, Python, Ruby, PHP, Java, .NET, OCaml, Haskell,
Elixir, Zig, C++, JS...) is a thin wrapper over this C ABI** — exactly the "bindings generated from one
C ABI, never hand-written per language" shape, one exit, many faces.

**(b) Guest-facing import ABI** — the guest is a raw wasm core module importing from two namespaces
(`runtime/src/plugin.rs:13-14`): `extism:host/env` (built-ins) and `extism:host/user` (host callbacks).
The memory primitives live in the **PPE kernel**: `kernel/src/lib.rs` `#[no_mangle]` exports
`alloc` (:360), `free` (:374), `length` (:421), `load_u8/u64` (:435/:445), `store_u8/u64` (:477/:487),
`input_load_u8/u64`, `input_set` (:501), `output_set` (:517), `input_length/input_offset` (:531/:537),
`output_length` (:543) — all `i64` handles into linear memory. Data flow per call: host serializes input,
writes it into guest memory, sets the input handle; the PDK reads via `input_offset`/`input_load_*`;
the guest returns by `output_set(handle, len)`; host reads output bytes back through the exported
`extism_plugin_output_data`. The kernel itself is a compiled wasm artifact embedded in the host
(`runtime/src/manifest.rs:36` `include_bytes!("extism-runtime.wasm")`) and registered into the linker as
the env module (`plugin.rs:398`), alongside native builtins linked with `linker.func_new`
(`plugin.rs:342-361`): `config_get`, `var_get/var_set`, `http_request/http_status_code/http_headers`,
`log_warn/info/debug/error/trace`, `get_log_level`. Host-controlled HTTP with no WASI at all —
the guest has no sockets, only `http_request` gated by `allowed_hosts` in the manifest.

Manifest (`manifest/src/lib.rs`): JSON schema — `Wasm` sources file/url/data/http (:140-196) with an
optional integrity `hash` (:102), config map (:271), `allowed_hosts` (:276), `allowed_paths` (:282),
`timeout_ms` (:286), `MemoryOptions{max_pages, max_http_response_bytes, max_var_bytes}` (:11-45).
Typed ergonomics above the raw ABI come from the `convert` crate (Rust `IntoExtism`/`FromExtism` for
JSON/msgpack/etc.) and **xtp-bindgen**, a schema (OpenAPI-inspired IDL, `example-schema.yaml`) →
generated PDK bindings for TypeScript/Go/Rust/Python/C#/Zig/C++ (README "Generating Bindings").

### 2.2 Wasmtime — raw core ABI vs typed component ABI

Wasmtime offers two boundary shapes and pushes new plugin work onto the typed one:

- **Core wasm**: untyped. `Linker<T>` (`crates/wasmtime/src/runtime/linker.rs:85`) registers
  `(module, name) -> FuncType` entries by hand; everything crossing the boundary is
  `i32/i64/f32/f64/reference` values with manual memory marshalling — the same raw form Extism built
  its env namespace on.
- **Component model**: typed, interface-first. `crates/wasmtime/src/runtime/component/` implements
  component instances (`instance.rs`), a typed linker (`linker.rs`), **resource handles with
  ownership/drop semantics** (`resources.rs`, `resource_table.rs`), interface matching between the
  WIT-declared world and the component's actual imports (`matching.rs`), and async concurrency
  (`concurrent.rs`). The `bindgen!` macro (`crates/component-macro`) generates the Rust host bindings
  **from the WIT contract** — single source of truth again, but a typed schema rather than a frozen C ABI.
  Canonical ABI lifting/lowering carries lists, records, variants, error unions across the boundary.
- **Plugin story**: wasmtime deliberately ships **no first-party plugin framework**. Its canonical answer
  is the WASI 0.2 "plugins" world: `docs/wasip2-plugins.md` + `examples/wasip2-plugins/` (a calculator
  whose plugins are components, with `wit/`, a `c-plugin/` and a `js-plugin/` — multi-language guests
  under one typed world; added by `63f094dfc` #11848, migrated by `3fa985acb` #12268).

### 2.3 Versioning

- **Extism**: PDK/SDK skew is a first-class detected failure — the host walks every guest import from
  `extism:host/env` and bails with a named diagnostic if the env module can't satisfy it: "This may
  indicate that the PDK that was used to build this plugin has additional features that aren't available
  in this version of the SDK" (`plugin.rs:372-391`). Upstream engine pin is per-release: measured
  `runtime/Cargo.toml` — v1.9.0 pinned wasmtime `>=26,<27`, v1.10 `>=27,<30`, v1.13/v1.20 `37`,
  v1.30 `43`, v1.40 `48` (LTS). Plugin-visible ABI, however, stays stable across those bumps.
- **Wasmtime**: version *lifecycle* is institutional — monthly majors, every 12th major is a 24-month
  LTS with security backports guaranteed, bugfixes volunteer-backported (`docs/stability-release.md:11-13`,
  LTS RFC: bytecodealliance/rfcs#42). Feature maturity is tiered (`docs/stability-tiers.md`,
  `docs/stability-wasm-proposals.md`). Measured evidence it works: `v36.0.17` (2026-10-02) still patched
  while `v49.0.0` shipped 2026-09-21.

### 2.4 Resource limits

- **Extism** pre-bundles all three enforcement axes: memory page cap (`MemoryOptions.max_pages` →
  wasmtime `Module` limits), **wall-clock timeout** (`timeout_ms`, enforced by a dedicated monitor
  thread that cancels running plugins, `runtime/src/timer.rs:62-124` + `extism_plugin_cancel`), and
  **fuel** (deterministic instruction meter; `fuel`/`initialization_fuel` options `plugin.rs:54`,
  C API `extism_plugin_new_with_fuel_limit`, out-of-fuel error path fixed in `c2866a7` #762). Plus
  byte caps per channel (max_http_response_bytes, max_var_bytes) and host-gated network/filesystem.
- **Wasmtime** provides the primitives, opt-in per `Config`/`Store`: `Config::consume_fuel`
  (`config.rs:643`), `Config::epoch_interruption` (`config.rs:788`) with a ticker, and
  `Store::limiter` (`runtime/store.rs:933`, `runtime/limits.rs`) — a `ResourceLimiter` callback capping
  memory growth, tables, instances, and component-specific resources. Fuel is deterministic (same
  module + same fuel = same trap point), which is the audit/replay property a sandbox wants.

### 2.5 Host-function distribution

- **Extism**: namespace model — built-ins under `extism:host/env` (memory, vars, config, http, logging),
  user callbacks under `extism:host/user`, linked by name from the manifest's `imports` + the C ABI's
  `extism_function_new` (with an optional namespace setter `extism_function_set_namespace`). Untyped
  `i64` in / `i64` out; type mapping is PDK-side per language.
- **Wasmtime**: core = manual `Linker::func_new/define` (engine-checked FuncType only); components =
  `bindgen!`-generated typed host functions from WIT, with real resource ownership. System services are
  distributed as **separate typed crates implementing standard worlds**: `crates/wasi` (preview1+preview2),
  `wasi-http`, `wasi-keyvalue` ("Wasmtime implementation of the wasi-keyvalue API"), `wasi-nn`,
  `wasi-config`, `wasi-tls`, plus `wiggle` (macro impl of WASI from IDL) and the
  `wasi-preview1-component-adapter` (run legacy preview1 binaries as components).

```
EXTISM HOST-GUEST BOUNDARY (v1 runtime, measured)

host process                                        wasm sandbox (guest core module)
+------------------------------------+             +--------------------------------------+
| language SDK (Go/Py/Ruby/Java/...) |             | plugin code                          |
|   wraps libextism C ABI (33 fns)   |  manifest   |  + PDK + PPE kernel (linked in)      |
| sdk.rs: extism_plugin_new(...)     |---- JSON ---->  exports: <your functions>          |
|   extism_plugin_call(name,in,len)  |  call name  |  imports "extism:host/env":          |
|   extism_plugin_output_data/length |===========> |    alloc/free/load_u64/store_u64     |
|   extism_function_new (callbacks)  |<==========> |    input_offset/output_set/length    |
|   extism_plugin_cancel             |  i64 handles|    var_get/var_set/config_get        |
| host reads/writes guest linear mem |  + bytes    |    http_request/log_* (host-gated)   |
+------------------------------------+             |  imports "extism:host/user":         |
      |                                             |    <host callbacks by namespace>     |
      v                                             +--------------------------------------+
wasmtime engine underneath (one pinned major per Extism release)

WASMTIME contrast: components replace the flat handle ABI with a WIT-typed world
(exports/imports typed incl. resources with drop ownership); host bindings are
bindgen!-generated from the WIT, not hand-registered name-by-name.
```

## 3. Key capabilities

**Extism**: universal host SDK layer over one C ABI; sandboxed plugins with host-owned HTTP, vars
(persistent module-scope state), config, logging; memory/fuel/wall-time limits declarable per manifest;
wasm integrity hash in manifest; cancellation; compiled/cached plugin path (`extism_compiled_plugin_new`,
AOT-adjacent); PDKs for 10 guest languages; schema-driven typed bindings via xtp-bindgen; browser/JS host
where wasmtime cannot go (`extism-runtime.wasm` kernel is the shared abstraction; **UNCERTAIN** — JS
host internals not in this clone).

**Wasmtime**: three execution engines (Cranelift optimizing, Winch baseline, Pulley portable interpreter
— top-level `cranelift/`, `winch/`, `pulley/` trees); AOT compile + serialize; fuel/epoch/limiter
triad; full component model incl. async, resources, streams; standardized WASI 0.2 worlds incl.
http/keyvalue/nn/config/tls; core-dumps + gdb debugging (`docs/examples-debugging-*`, `crates/debugger`);
`wizer` pre-initialization tool (fast plugin cold start); profiling (perf/vtune/samply); multipass
compiler; security posture: fuzzing, RFC review, formal-verification collaborations (README "Secure").

## 4. Development & current state (12-month window, measured)

- **Extism**: 8 commits / 6 authors in 12 months; last release `v1.40.0` = "Upgrade to Wasmtime 48 (LTS)"
  (`d5da297`, 2026-09-02). The commit stream since 2024 is almost exclusively dependency bumps
  (prost/ureq/cbindgen), small fixes (pool thread-safety `85fce56` #893, fuel error path `c2866a7` #762)
  and LTS-tracking releases. **Assessment: maintenance mode, single-vendor funded, ABI surface stable.**
- **Wasmtime**: 1,763 commits / 185 authors in 12 months; releases v37..v49 in the window; v50-rc1 already
  out 2026-10-05; three LTS lines patched simultaneously. Yearly counts flat (1861/1502/1515/1533).
  **Assessment: one of the most actively engineered runtimes anywhere; release discipline is codified.**

## 5. Ecosystem / adoption evidence

- **Extism**: 15 host SDKs + 10 PDKs enumerated in the README table (verified repo URLs: go-sdk,
  python-sdk, ruby-sdk, php-sdk, java-sdk, dotnet-sdk, ocaml-sdk, haskell-sdk, elixir-sdk, zig-sdk,
  cpp-sdk, perl-sdk, js-sdk...); in-repo SDK delivery plumbing (`extism-maturin` = Python wheel packager,
  `nuget/` = .NET packaging); used by the XTP platform (README). Named external production adopters:
  **not verifiable from this clone — UNCERTAIN**; the dylibso commercial dependency is visible in LICENSE
  copyright + README footer.
- **Wasmtime**: `ADOPTERS.md` in-repo, production badges: **Akamai, Cosmonic/wasmCloud, DFINITY
  (Internet Computer canisters), Embark Studios (game engine), Fastly Compute, Huawei Cloud, InfinyOn,
  Microsoft (AKS WASI node pools in preview since Oct 2021), Redpanda, Shopify Functions, SingleStore**.
  Official embeddings: Rust, C, C++, Python, .NET, Go, Ruby; community: Elixir, Perl (README).
  Component-model-adjacent platforms (Spin/fermyon, wasmCloud) — wasmCloud is in ADOPTERS.md; Spin itself
  is **UNCERTAIN** from this clone.

## 6. Highlights & limitations

**Extism** — highlights: the only project here that ships the *whole* plugin product (ABI + limits +
host-gated IO + polyglot SDK/PDK matrix + schema bindings) as one coherent system; manifest-declared
sandbox posture (auditable: pages, timeout, hosts, paths); PDK-skew diagnostic UX. Limitations: activity
decays to 6 commits in 2026 (bus-factor on Dylibso); no typed interface at the wasm boundary (i64 handles
+ JSON/msgpack convention); pinned one-wasmtime-major-per-release model means the env surface hides a
moving engine underneath; no first-class component model — a core-module-only design; fuel/epoch
semantics are its own wrapper, not standard knobs.

**Wasmtime** — highlights: standards-grade typed boundary (WIT worlds, resources with drop, streams,
async); the limits triad is deterministic and composable (fuel = auditable replay); LTS contract makes it
a defensible long-term dependency; security engineering culture (RFCs, OSS-Fuzz, advisories, tiering);
proven at hyperscale edge (Fastly, Shopify, DFINITY). Limitations: *no plugin framework* — KV, vars,
host-gated HTTP-as-plugin-capability, cancellation UX are yours to build; monthly-major churn (mitigated
by LTS but real for trackers); preview1→preview2 migration burden persists via the
`wasi-preview1-component-adapter` crate; component tooling (`wit-parser`, adapters, `wasm-tools`) pulls
a bigger supply chain than "embed one runtime".

## 7. Historical lessons (citable)

- **Extism's ABI stability experiment failed the easy way and then moved the problem into wasm.** The v0
  contract (guest exports `alloc`/`free`/`store_data`/`load_data` — the WASI-experimental-era shape,
  tags v0.0.1 2022-11-29 through v0.5.2 2023-09-21) was replaced wholesale by the v1 host-namespace
  design after `3da5262` (2023-07-27, #384 "Implement parts of the extism runtime in WebAssembly") moved
  runtime semantics into the portable kernel. Timeline shows a genuine dual-track migration:
  `v1.0.0-rc0` 2023-10-16 while v0.5.3/v0.5.4 still shipped (2023-10-24/25); `v1.0.0` landed 2024-01-05.
  Lesson for ModuKit: even a deliberately tiny flat ABI gets regretted *at the direction level*
  (guest-exports vs host-provides) — freeze the direction, not just the symbols; and note the fix they
  chose: make the boundary implementation itself portable-artifact (wasm kernel) so all hosts share it.
- **Extism also froze its host-facing C ABI across a full internal rewrite.** The 33-function `sdk.rs`
  surface survived v0→v1 (manifest-JSON-in, call-by-name-out) — proof the "one C ABI as the stable spine"
  doctrine can outlive engine swaps underneath. But it then outsourced typed ergonomics to xtp-bindgen
  rather than evolving the C ABI itself: flat C ABI for transport, generated typed layer for DX.
- **Wasmtime's component migration took years and left a permanent bridge.** preview2 was prototyped in
  branches, unified into the main tree by `5aee0b7ff`/`b90731fab` (PRs #6374/#6391, 2023-05-19) and first
  released in **v10.0.0** (measured via `git tag --contains b90731fab`); the preview1→preview2 adapter
  crate is still maintained at HEAD in 2026. Lesson: typed-contract migrations need an adapter story
  from day one, priced in years, not releases.
- **Wasmtime's churn answer was to institutionalize support windows.** `docs/stability-release.md` cites
  the LTS RFC (bytecodealliance/rfcs#42) amending an earlier every-release-is-breaking regime; the
  measured 2026-10-02 triple-patch (v36/v48/v49) shows the machine running. v1.0.0 "fast, safe, production
  ready" (tag 2022-09-20, #4930) was itself the pivot from experimental velocity to product posture.
- **wasmer-vs-wasmtime churn** (the frequently-cited lesson: wasmer's engine swap in 2.x/3.x and its
  system/plugin ABI rewrites) — **UNCERTAIN from these clones** (no wasmer checkout here); the citable
  local contrast is that wasmtime searched clean of any dlopen-style plugin ABI in current history
  (probes on `crates/c-api` and `-i --grep=dlopen` find no plugin-ABI add/remove trace) — wasmtime's
  plugin bet is the component world, not a bespoke host ABI. Do not cite wasmer specifics from this file.

## 8. Value for ModuKit

Whitepaper mapping (verified lines): §2.3 crate plan `modukit-wasm # WASM host & sandbox` (line 44);
tech stack "WASM host | wasmtime / wasmer" (line 147); "Untrusted plugins: WASM sandbox behind WASI
interfaces" (line 184); security row "**WASM sandbox for untrusted plugins, resource limits and audit**"
(line 230, Stage script/sandbox row).

- **Wasmtime — Verdict: DEPENDENCY-CANDIDATE.** It is already the named first choice in the whitepaper's
  host row; the clone confirms why: Apache-2.0, LTS contract (pin one 12th-major at a time, e.g. v48),
  fuel + epoch + `ResourceLimiter` are exactly the "resource limits and audit" primitives line 230
  requires (fuel determinism additionally gives replayable audit). Component model is the right Stage-5
  wasm face: WIT = ModuKit's "single source of truth, never hand-written per language" already formalized.
  Accept the cost: you build the plugin framework yourself (capability registry, cancellation, manifest
  semantics) — which is the actual roadmap work anyway.
- **Extism — Verdict: BORROW-PATTERNS (high-value design map; do NOT vendor).** Maintenance-mode
  velocity (6 commits in 2026) disqualifies it as the spine, but it is the best public blueprint of the
  sandbox posture line 230/184 describes: declarative manifest limits (pages/timeout/hosts/paths/http
  byte caps), host-gated HTTP instead of raw WASI sockets, fuel-capped compiled-plugin path, PDK-skew
  diagnostics, and wasm-artifact integrity hashing — lift these as `modukit-wasm` design requirements.
  Its xtp-bindgen (schema → per-language PDK bindings) validates ModuKit's generated-bindings doctrine
  end-to-end and can be cited when ratifying the A6 "generated-only bindings" wording.
- **The deep connection (record prominently):** Extism's entire polyglot reach is delivered by **33
  `extern "C"` functions** (`runtime/src/sdk.rs`) — one C ABI, fifteen wrapped SDKs, zero hand-written
  per-language engines. That is ModuKit's single-exit doctrine, independently proven at wasm scale.
  Where Extism/Wasmtime stop (untyped handles in core wasm; typed worlds only for components), ModuKit's
  baseline choice — hand-authored C ABI + generation — is the v0-era path; the historical lesson in §7
  says: freeze the *direction* (host-provides env functions, guest-exports entry points) in the first
  ratified header, because Extism paid a full v0→v1 rewrite for getting that direction wrong.
- **Stage script/sandbox**: adopt the two-axis limiter default (fuel for determinism/audit + wall-clock
  cancel for worst-case) — Extism's `timer.rs`+`cancel` pair shows the UX both axes need, wasmtime's
  `config.rs:643/788` provides the engines.
- **Stage 5 wasm face**: component-model-first, with a `wasi-preview1-component-adapter`-style
  compatibility story planned from day one (§7, cost measured in years).

### 8.1 Not verified here (UNCERTAIN)

- wasmer's plugin/ABI churn specifics (no local clone) — §7 keeps the claim external and unnamed.
- Extism's browser host internals, named external production adopters, and KV store implementation
  surface (`kv_store` did not appear in `runtime/src/*.rs` greps; env builtins are config/var/http/log).
- zellij/kibit-wasm comparison — `docs/reference/zellij.md` **pending** (clone exists at
  `~/.cache/modukit-research/zellij`, profile not yet written by its author).
