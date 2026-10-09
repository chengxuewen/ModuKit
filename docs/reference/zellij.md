# Reference Profile — Zellij

> **External reference.** Clone is a read-only third-party checkout under `~/.cache/modukit-research/zellij/`;
> it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `9dacbc19` (main, 2026-10-08) · Upstream: <https://github.com/zellij-org/zellij> · Workspace version: `0.46.0` (released tag `v0.45.1`)
> All facts below are derived from this local clone unless marked otherwise. Claims that could not be checked against this clone are prefixed **UNCERTAIN**.

## 1. Portrait

| Field | Value |
|---|---|
| Project | Zellij — a terminal workspace/multiplexer with a pluggable UI (README: "create plugins in any language that compiles to WebAssembly") |
| Language | Rust (single codebase; plugins are WASM, host is native Rust) |
| Governance | Independent OSS org (`zellij-org`); GOVERNANCE.md, no foundation; core mass is one maintainer (Aram Drevekenin) |
| License | MIT (LICENSE.md: "Copyright (c) 2020 Zellij contributors"; `license.workspace = "MIT"`) |
| Size | 22 `Cargo.toml` manifests, 19 workspace members, 3,386 total commits (since first commit 2020-07-13 "line wrap kinda working") |
| Velocity | 314 commits / 64 distinct contributors in last 12 months (`git log --since=12.months.1`) |
| Concentration | Aram Drevekenin = 208 of 314 trailing-12-mo commits (~66%); next contributor 24 — single-maintainer bus-factor |
| Adoption | 35,686 stars / 1,486 forks / 1,925 open issues (GitHub API, 2026-10-09) — largest community in this reference series |
| Maturity | 76 tags; still pre-1.0: `v0.1.0-alpha` … latest `v0.45.1` (2026-08-28); HEAD is `0.46.0` in-dev |
| Roles in repo | `zellij-utils` (config, IPC, plugin schema, protos), `zellij-server` (session host: PTY, panes, tabs, **plugins**), `zellij-client` (per-TTY attach UI), `zellij-tile`+`zellij-tile-utils` (plugin SDK), `default-plugins/*` (12 first-party wasm plugins), `xtask`, `zellij-integration-tests` |

## 2. Architecture in focus

### 2.1 Process / session model — read this first (it is NOT the dual/tri-mode the brief assumed)
Zellij runs **two native processes** per session, not a plugin-process zoo:
- **server** (`zellij-server`) — one detached session process: owns PTYs (`pty.rs`), panes, tabs, and the **wasm plugin host**. Everything a plugin does happens *inside this one process*, in the wasm engine.
- **client** (`zellij-client`) — one process per attached terminal; reads stdin, renders the server's output; talks to the server over `interprocess::local_socket` (unix socket / Windows named pipe) with **protobuf framing** (`zellij-utils/src/ipc.rs`, `zellij-utils/src/client_server_contract/{client_to_server,server_to_client,common_types}.proto`). This is the "its own socket protocol" in the brief — but it is **client↔server**, *not* host↔plugin.
- **External-process channel = CLI pipes**: `zellij pipe` / `zellij action` let a plain OS process feed a plugin via `pipes.rs` + the `ReadCliPipes` permission (CHANGELOG #5537 "release a CLI pipe when the plugin handling it crashes, instead of blocking the `zellij pipe` client"). This is the only place a *non-wasm* process participates, and it is a data pipe, not a linked plugin.

**Correction to the brief**: the premise of a **"native dynamic-libload (libloading/cdylib) plugin mode, ABI-coupled to the host version"** does **not exist in this clone**. Verified: `grep -rn 'libloading\|Library::new\|cdylib'` over `zellij-server/src`+`zellij-utils/src` is empty; no plugin crate sets `crate-type`; `git log --all -S 'libloading::'` returns zero commits; no commit subject mentions "native plugin". What Zellij *actually* ships is a **dual*mode*: **embedded-wasm built-in plugins** (§2.4) + **external-wasm plugins** (§2.2/2.3). *UNCERTAIN*: an early native-dylib mode may have existed upstream before this blob-filtered clone's reachable history, or the brief conflated the two — treat as unverified; the pivot **to wasm** is the point, and the in-clone evidence supports it (everything is wasm).

### 2.2 The plugin ABI seam = a protobuf byte-stream schema, NOT a symbol table
This is the load-bearing design decision for ModuKit:
- **Contract lives in `.proto` files**: `zellij-utils/src/plugin_api/*.proto` — `event.proto`, `plugin_command.proto`, `action.proto`, `command.proto`, `pipe_message.proto`, `message.proto`, `plugin_permission.proto`, `resize.proto`, `style.proto`, `shared_plugin.proto`, … → prost generates `generated_plugin_api.rs`. Rust structs and wasm-side structs are *the same schema*, compiled on both sides.
- **Transport over WASI pipes, not FFI pointers**: the plugin calls host via stdout (`shim::object_to_stdout(&msg.encode_to_vec())`) and receives via stdin (`shim::protobuf_bytes_from_stdin()`); host writes to the plugin via `wasi_write_object` (`zellij-server/src/plugins/zellij_exports.rs`). No shared Rust types cross the boundary — only protobuf bytes. A host-ABI break (rustc version, struct layout) therefore *cannot* corrupt a running plugin.
- **Narrow import surface**: instead of exposing hundreds of host functions as wasm imports, `zellij_exports.rs` wraps a thin import set and dispatches a `PluginCommand` enum inside the host (the `func_wrap` count is small; the real surface is the protobuf `plugin_command` union). Compare zenoh's 3-symbol vtable (`get_plugin_loader_version`/`get_compatibility`/`load_plugin`) — same instinct: **few entry points, versioned payload**.

### 2.3 Version / ABI coupling — how they handle it, and where it bit them
- There is **one version string, the host's** crate version: `zellij-utils/src/consts.rs:13` `pub const VERSION: &str = env!("CARGO_PKG_VERSION")`, re-exported into the SDK (`zellij-tile/src/prelude.rs:3 pub use zellij_utils::consts::VERSION`).
- The plugin **reports its own version** through the export the `register_plugin!` macro emits: `#[no_mangle] pub fn plugin_version() { println!("{}", prelude::VERSION) }`. The SDK's `VERSION` is whatever `zellij-tile` version the plugin was compiled against — so an external plugin embeds the *host version it was built for*.
- A `PLUGIN_MISMATCH` const string (zellij-tile/src/lib.rs) is the user-facing message when the SDK and host diverge.
- **No negotiation in the schema itself** — the protobuf contract carries no version/compat handshake (unlike zenoh's `Compatibility{rust_version, zenoh_version, zenoh_features}`). Compatibility is enforced *out of band* by (a) a version-specific cache path (§7 lesson 3) and (b) all-rebuild-per-release, not by runtime feature negotiation. This is the single most important negative result for ModuKit's C-ABI policy (§8).

### 2.4 Built-in vs external plugin sources
- **Built-in** = the 12 `default-plugins/*` (bars, strider, session-manager, plugin-manager, layout-manager, configuration, about, prompt, share, context-menu, multiple-select, fixture-plugin-for-tests) are **workspace members compiled at the host version** and **embedded in the zellij binary** (`input/plugins.rs`: `distribution::builtin_plugin_bytes(name)`, `is_builtin`; CHANGELOG #5622 "not copying the builtin plugins onto the heap"). Because they build in-workspace, their SDK version always equals the host — they *cannot* mismatch. **The version-coupling pain is reserved for the external ecosystem.**
- **External** plugins are located by URL/path schemes `file:` / `http(s):` / `zellij:` (`input/plugins.rs` `RunPluginLocation`, `InvalidUrlScheme` error text), downloaded (`zellij-utils/src/downloader.rs`), and cached to disk. `watch_filesystem.rs` supports hot reload during dev.

### 2.5 Lifecycle + sandboxing primitives
- **SDK trait surface** (`zellij-tile/src/lib.rs`): `ZellijPlugin: Default { load(config: BTreeMap), update(Event)->bool, pipe(PipeMessage)->bool, render(rows,cols) }`; `update`/`pipe` return "should I render?" so the host only redraws on demand. `ZellijWorker` (`register_worker!`) = background task off the render path. `ZellijSharedPlugin` (`register_shared_plugin!`) = one wasm instance serving many clients/tabs (slot_added/removed, client_connected — the "multiple-pane-single-instance" optimization, CHANGELOG #5662).
- **The `register_plugin!` macro** is `macro_rules!`, not a proc-macro: it injects `thread_local! STATE`, a `main()` with a custom panic hook (`shim::report_panic`), and the `#[no_mangle]` exports `load/update/pipe/render/plugin_version`. *No `plugin-macros` proc-macro crate exists in this clone* (task brief named one — **not-found**).
- **Capability/permission gate**: `plugin_api/plugin_permission.rs` `PermissionType{ReadApplicationState, ChangeApplicationState, OpenFiles, RunCommands, OpenTerminalsOrPlugins, WriteToStdin, WebAccess, ReadCliPipes}`, enforced in the host (`zellij_exports.rs:1305 check_permissions(...)`). Filesystem is sandboxed by WASI preopens to per-plugin `data`/`cache` dirs (`plugin_loader.rs create_plugin_fs_entries`, `builder.preopened_dir`), and memory is capped by `StoreLimitsBuilder` (`create_optimized_store_limits`). Crash isolation is per-instance: a plugin panic is caught by the wasm panic hook and reported as a `LoadingIndication`, not a session crash.

## 3. Key capabilities
- A genuinely polyglot plugin contract: README markets "any language that compiles to WebAssembly"; the wire is protobuf so the SDK is not the ABI — only the Rust SDK (`zellij-tile`) is first-class, non-Rust wasm plugins hand-roll the protobuf encode/decode against the same `.proto`s.
- In-process wasm sandbox with interpreter isolation (`wasmi`, no JIT), per-plugin WASI preopens, store memory limits, and a permission model gating ~8 capability classes.
- Background workers + shared/multi-client plugin instances (one wasm module, many pane slots) — an explicit density optimization.
- Hot-reloadable external plugins from disk/URL; built-ins embedded in the binary.
- A separate client/server socket protocol (protobuf over local socket) already supporting multi-client attach, remote-attach (`zellij-client/src/remote_attach`), and a web client (`web_client.rs`) — i.e. the same session, several front-ends.

## 4. Development & current state
- **Velocity**: 314 commits / 64 contributors trailing 12 months; total 3,386 since 2020-07. Release train ~monthly minors; latest tag `v0.45.1` (2026-08-28), HEAD `0.46.0`.
- **Concentration risk**: dominant maintainer ~66% of recent commits — decisions about the plugin ABI effectively flow through one person (context for how "break external plugins" calls get made quickly).
- **Engine churn is recent and visible** (§7): the wasm host moved wasmer→wasmtime→wasmi across 0.38→0.44; the wire moved serde→protobuf at 0.38.
- **License**: MIT, unambiguous, `license-file = ["LICENSE.md","4"]` (Cargo.toml). No relicensing churn visible in-clone.
- **Hygiene**: rustfmt+clippy configured; `rust-toolchain.toml` pins `channel = "1.95.0"` / `rust-version = "1.95"`; integration-tests workspace; e2e in CI. CHANGELOG.md is **maintained** (unlike zenoh's empty one) and richly records plugin/API changes per PR.
- **Still pre-1.0**: no public stable plugin-ABI guarantee; the contract is documented at zellij.dev/documentation/plugins and re-cut frequently.

## 5. Ecosystem
- Plugin ecosystem is **external** to the repo: out-of-tree community wasm plugins (strider-style, status-bar, session tooling) built against published `zellij-tile` + `zellij-utils` versions on crates.io and loaded via `file:`/`http:`/`zellij:`. `rust-plugin-example` is the referenced template (zellij-tile/src/lib.rs docs).
- `example/distribution/plugin/` in-tree is the packaging/distribution reference for shipping plugins through a custom distribution channel.
- The `.proto` plugin API is the *de facto* cross-language contract surface: anything that can emit the protobuf bytes and answer the `load/update/pipe/render` exports can be a plugin.
- Publishing: `zellij`, `zellij-tile`, `zellij-utils`, `zellij-server`, `zellij-client`, `default-plugins/*` on crates.io (workspace-shared version).

## 6. Highlights & limitations
**Highlights**
1. **It proves the C-ABI-exit idea by proxy**: the plugin boundary is a *serialized byte schema* (protobuf), never a linked symbol table. Host recompiles / rustc bumps / struct-layout changes are inert to a running plugin. That is the property ModuKit's "one C ABI, generated bindings" rule wants — Zellij achieves the *isolation* goal via wasm+schema instead of via a C ABI.
2. **Single narrow dispatch surface** (protobuf `PluginCommand` union over stdio) is a clean, versionable contract — a real alternative shape to a fat function vtable.
3. **Built-in vs external split is the correct answer to "who eats version breakage."** In-workspace plugins always match; only out-of-tree authors pay, and the mitigation (version-keyed cache, hot-reload, downloadable plugins) is documented in-repo.
4. **Sandboxing vocabulary is concrete and shippable**: WASI preopens per-plugin, store memory limits, an explicit permission enum gating capability classes, panic-hook crash containment. Directly reusable concepts for ModuKit Stage-5 wasm + Stage-2 crash isolation.
5. **A maintained CHANGELOG + dated tags** makes the whole ABI breakage arc auditable — the discipline ModuKit wants (its own CHANGELOG.md is currently non-existent).

**Limitations**
1. **No runtime version/compat negotiation in the contract.** protobuf payload has no handshake; compat is enforced by "rebuild everything per release" + version-keyed cache path. zenoh's explicit `Compatibility` check is the *better* mechanism for a Rust↔Rust ABI; ModuKit's C ABI needs neither, but must add an explicit contract version it currently lacks.
2. **Its own ABI churned three times (engine) plus once (wire format)** — a stability anti-pattern even though each move was individually justified. The lesson is that "the sandbox" (engine) and "the contract" (schema) are *separate* axes that both need freezing.
3. Still **pre-1.0**, no plugin-ABI guarantee, and effectively single-maintainer — the breakage model is "acceptable" only because the author controls both sides. ModuKit with third-party maintainers cannot copy that governance.
4. Plugins are **in-process** wasm — good for isolation-via-interpreter, but it is *not* the multi-process model ModuKit Stage-2 targets. The CLI-pipe mechanism is a workaround, not a full out-of-process plugin ABI.
5. First-class SDK is **Rust-only**; other languages must speak raw protobuf against stdio — no generated façade exists for them.

## 7. Historical lessons — the plugin-ABI breakage arc (commit/tag evidence)
1. **The wire-format cutover broke every external plugin at once — commit `1bedfc90` (#2686), first tag `v0.38.0`.** "use protocol buffers for serializing across the wasm boundary" replaced the prior serde/JSON boundary wholesale. No shim, no dual-read — plugins compiled against the old boundary simply failed to decode. Lesson for ModuKit: *the wire schema is the ABI*; a schema format change is a hard cutover. Freeze the C-ABI *serialization* story (ModuKit chose one C ABI → generated bindings) and never hand-edit the on-wire form the way Zellij swapped JSON→protobuf.
2. **The wasm engine churned three times — `wasmer 3.1.1` (`7c726c13`, #2706) → `switch from Wasmer to Wasmtime` (`7d7848cd`, #3349, tag `v0.41.0`) → `Migrate from wasmtime to wasmi` (`889bd06f`, #4449, tag `v0.44.0`), then wasmi 0.51→1.1 (`77d97e3c`, `38a9aba8`/`113ff707`).** Each engine swap re-touched WASI handling, limits, and the calling assumptions in `plugin_loader.rs`. Because wasm bytes are engine-agnostic, published `.wasm` survived — the *host* churned, not the plugins. Lesson: the sandbox engine and the plugin contract are **two independent stability axes**; ModuKit should pin both (wasmtime version + contract version) and treat an engine swap as a compatibility event with its own test, not a routine dep bump.
3. **They formally admitted artifacts are version-coupled — commit `fb1af39a` (#2836, 2023-10-12, author Thomas Linford): "add zellij version to cached artifact path."** Pre-compiled/cached plugin artifacts were moved into a version-specific subfolder. The reason is exactly ModuKit's C-ABI fear: an artifact compiled/cached against host N must not be silently reused on host N+1. This is the *right* instinct; the cheaper ModuKit equivalent is an ABI version stamp in the artifact filename/dir from day one, so a mismatch is a cache miss (re-fetch), never a wrong-layout load.
4. **Version mismatch handling was itself buggy and repeatedly patched** — `75801bdb` (#1838) "Improve error handling on plugin version mismatch" *after* the mechanism existed, and `4e5717ec` "fix(plugin): mismatch JSON format on `get_zellij_version`" (the host→plugin version answer had the wrong encoding). Lesson: the version-negotiation path is where crashes hide — it needs its own explicit tests (ModuKit: a `abi_version()` handshake with a red/green negative test before Stage-1 ships), not an incidental const string + a `PLUGIN_MISMATCH` message.
5. **Internal plugins were made breakage-proof by co-versioning them** — moving default-plugins *into the workspace* and embedding the wasm bytes in the binary (CHANGELOG #5622 perf note, `builtin_plugin_bytes`) means the shipped UI bars/tab plugins never mismatch. Every ABI break now lands only on out-of-tree authors. Lesson for ModuKit: **ship first-party plugins from the same build as the host**; expose the *stable* C ABI to third parties, and keep internal plugins on an *unstable/internal* contract you may break at will. That is the C-ABI-stability policy in one line: stable surface = only what third parties bind to; internal ≠ obligated.
6. **The pivot to wasm was a pivot *away from symbol-table coupling*** — the README's promise ("any language that compiles to WebAssembly") is only achievable because the boundary is serialized bytes + interpreter isolation, not a native ABI. The native-dylib model (the brief's "internal, ABI-coupled" premise) is precisely what they did not ship here. For ModuKit, whose rule is "one C ABI, bindings generated, never hand-written per language", Zellij is the counter-example that shows a **schema-over-transport** seam buys both polyglot-ness *and* host-decoupling that a raw C symbol ABI does not — so the C ABI must carry an explicit version/negotiation to reach parity.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS + CAUTIONARY-TALE.** Not a dependency (Zellij is a terminal multiplexer; nothing to import). But it is the *closest production wasm-plugin host with an auditable breakage history* in this series, so it earns two seats at the table: (a) a positive model for sandboxing + built-in-vs-external split (Stage-5, Stage-1), and (b) a negative model — what "no contract version negotiation" costs a growing plugin ecosystem — feeding ModuKit's C-ABI stability policy directly.

### 8.1 Compare vs zenoh-plugin-trait (the other plugin ABI in this series)
| Axis | Zellij (this clone) | zenoh-plugin-trait (`zenoh.md` §2.3) |
|---|---|---|
| Boundary mechanism | protobuf byte schema over WASI stdio, inside a wasm interpreter | Rust vtable over `libloading`, same-process dylib |
| Symbol coupling to host | **none** (plugin never links host symbols) | **total** (must rebuild vs exact zenoh/rustc/features) |
| Runtime compat negotiation | **absent** (version-keyed cache + rebuild-everything) | **present** (`Compatibility{rust_version,version,features}` checked before any pointer) |
| Polyglot reach | any wasm language (Rust SDK first-class only) | Rust only |
| Engine/host stability axis | churned 3× (wasmer→wasmtime→wasmi) | stable (stabby vtable frozen) |
| Crash isolation | interpreter + panic hook (in-process) | none (in-process dylib can take down host) |
The synthesis: zenoh solves *Rust↔Rust* safety with an explicit `Compatibility` handshake; Zellij solves *cross-language* + *isolation* with schema-over-transport but has **no** handshake. ModuKit's C ABI should take **both halves**: Zellij's "serialized bytes, no symbol table" seam *and* zenoh's "negotiate version/features before trusting anything" gate — put the handshake **in the C contract** so a C-ABI host is safe with *generated non-Rust bindings*, which neither project gives cleanly.

### 8.2 Stage matrix
| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Adapt** | Adopt the built-in-vs-external split + lifecycle vocabulary (`load/update/pipe/render`, worker-as-side-task, `update->bool` render-on-demand). Copy the *narrow dispatch* shape (one command union, not a fat vtable) but implement it over ModuKit's C ABI, and add the version handshake Zellij lacks. Ship first-party plugins in-workspace at the host version (their lesson 5). |
| 2 — multi-process plugins | **Reference-only** | Zellij is the *opposite* choice (in-process wasm). Its only out-of-process surface — CLI pipes (`pipes.rs`, `ReadCliPipes`, crash-releases-pipe #5537) — is the right granularity for a supervisor-managed external plugin channel; study it, but ModuKit's real crash-isolation model is iceoryx2/dora, not Zellij. |
| 3 — registry/services | **Reference-only** | No service registry here; plugin discovery is URL/path-based, not typed-service. |
| 4 — compositor/UI | **Reference-only** | Its TUI compositor is unrelated to ModuKit's frame/SHM compositor; only the *client↔server socket + protobuf frame* shape (interprocess local socket) is a cheap analogue for the host/router split. |
| 5 — wasm / polyglot bindings | **Adopt (pattern) + Avoid (governance)** | The wasm sandbox stack is the Stage-5 template: `wasmi`/wasmtime + WASI preopens per-plugin + `StoreLimits` + a `PermissionType` enum + panic-hook isolation. But copy it as *contract-first*: schema is the ABI (their `.proto`-equivalent = ModuKit's generated C headers), pin the engine version separately, and stamp every artifact with a contract version (lesson 3) so Zellij's "rebuild everything on every bump" never becomes ModuKit's policy. |

### 8.3 License gate
MIT — no gate. `license-file = ["LICENSE.md","4"]`; patterns, `.proto` schema *design*, and the sandboxing vocabulary are freely adoptable, and even verbatim code snippets are usable with a license-header/attribution line. No copyleft, no relicensing risk observed in-clone (single MIT file since 2020). Gate action: none required for borrowing; if any `.proto` text or Rust is copied, retain the MIT copyright line in the future NOTICE/SBOM. This is strictly friendlier than zenoh's EPL/Apache dual and than CppMS/CTK's runtime-license gates — Zellij is the lowest-friction reference in the series for legal reuse.
