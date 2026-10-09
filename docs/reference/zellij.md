# Reference Profile — Zellij (zellij-org)

> **External reference.** This profile was researched from a disposable clone at
> `~/.cache/modukit-research/zellij` (not under `.refinfo/`); it is not ModuKit content and
> none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `9dacbc19` (main, 2026-10-08) · Upstream: <https://github.com/zellij-org/zellij> (verified via `git remote -v`) · Latest tag: `v0.45.1` (2026-08-28) · In-tree workspace version: `0.46.0` (unreleased)
> All facts below are derived from this local clone unless marked **UNCERTAIN**.

## 1. Portrait

| Field | Value |
|---|---|
| Project | Zellij — terminal workspace/multiplexer; "plugin system allowing one to create plugins in any language that compiles to WebAssembly" (README:48) |
| Language | Rust host; plugins target `wasm32-wasip1`, executed by an in-server **wasmi** interpreter |
| Governance | zellij-org; effectively single-maintainer — Aram Drevekenin authored **208 of 314** commits in the last 12 months (66%); next contributor 24 |
| License | **MIT** (LICENSE.md: "MIT License, Copyright (c) 2020 Zellij contributors"). Permissive; see §8.4 |
| Size | Workspace: `zellij-{client,server,utils,tile,tile-utils,integration-tests}` + 11 `default-plugins/` + `xtask`; 466 `.rs` files, 407,599 LoC (incl. tests); 3,386 commits since 2020-07-13 |
| Velocity | 314 commits / 65 distinct contributors / 12 mo; 147 commits / 44 contributors / 6 mo. Yearly: 2023=373, 2024=323, **2025=172 (measured one-year lull)**, 2026=305 YTD. HEAD 2026-10-08 — active again |
| Maturity | 76 tags, all pre-1.0. The plugin engine has had **three backends** (wasmer → wasmtime → wasmi) since v0.41 (§2.4) |
| Real adoption | Ubiquitous terminal multiplexer (commonly cited); adoption scale **UNCERTAIN** — no in-repo evidence measurable from the clone |

## 2. Architecture in focus — a production Rust plugin host, three coexisting modes

### 2.0 Terminology correction up front

The research brief lists "three plugin modes: internal version-locked, **Kibit-wasm external**,
IPC/socket client". Two exist as described; **"Kibit" does not exist here**: `grep -rli kibit` over
`.rs`/`.toml`/`.md` = 0 hits, `git log --all --grep kibit` = 0 hits, and a GitHub search
`q=kibit+zellij` (api.github.com, measured 2026-10-09) returned **0 items**. Any "Kibit format" is
defunct, renamed, or elsewhere — **UNCERTAIN**, excluded below. The actual measured triad:

### 2.1 Mode A — built-in plugins: lockstep-compiled, embedded in the binary

`zellij-utils/src/distribution.rs` defines `ZELLIJ_BUILTIN_PLUGIN_NAMES` (11: bars, strider,
session-manager, configuration, plugin-manager, about, share, multiple-select, layout-manager,
prompt, context-menu) and embeds each via `include_bytes!("…/assets/plugins/<name>.wasm")`.
The wasm is built from the **same workspace at the same version**
(`zellij-tile = { path = "../../zellij-tile" }`, strider/Cargo.toml) — version-lock *by
construction*: host and plugin share one protobuf schema compiled together, so this tier can never
drift. Debug builds may load from `target/wasm32-wasip1/debug` (`plugins_from_target` feature);
release installs unpack the embedded bytes into the user config dir.

### 2.2 Mode B — external wasm plugins: interpreter-hosted, protobuf RPC

Runtime lives in `zellij-server/src/plugins/` (`wasm_bridge.rs`, `plugin_loader.rs`,
`plugin_worker.rs`, `pinned_executor.rs`, `zellij_exports.rs`):

1. Load: wasmi compiles the user `.wasm`; **16 MB** memory cap per plugin
   (`plugin_loader.rs:516`); WASI + **one host function import** —
   `zellij:host_run_plugin_command` (`zellij_exports.rs:439`).
2. Invoke: the host calls plugin exports **by string name** — `get_func(…,"_initialize")`
   (plugin_loader.rs:332,444), `get_typed_func::<(),i32>("update")`,
   `get_typed_func::<(i32,i32),()>("render")` (wasm_bridge.rs:3388,3403). Payloads are prost-encoded
   protobuf against `zellij-utils/src/plugin_api/*.proto` (plugin_command, event, context,
   pipe_message, shared_plugin, prompt, file…).
3. **No load-time ABI seal exists.** The plugin can *ask* the host's version
   (`get_zellij_version`, zellij_exports.rs:1355, returning the full semver string) — informational
   only. The mismatch text `PLUGIN_MISMATCH` (zellij-tile/src/lib.rs:123: "…the plugins aren't
   compatible with the current zellij version") is displayed **after** a plugin fails to decode an
   event — detection is post-hoc, not negotiation. A grep for `incompatib` finds no plugin path at
   all. Stale plugins are loaded, run, and fail mid-stream. §2.5 tracks what replaced enforcement.
4. Scheduling: `pinned_executor.rs` — "a dynamic thread pool that pins jobs to specific threads
   based on plugin_id": one OS thread per plugin, pool grows/shrinks. Plugin panics are caught and
   rendered into the plugin's pane (`handle_plugin_crash`, wasm_bridge.rs:3457; 5 call sites); the
   server survives. Synchronous `update`/`render` means a *hang* stalls that pinned thread.

### 2.3 Mode C — the IPC/socket process model: session servers + CLI pipes

Genuine multi-process lives at the **session** level: `zellij-client` (terminal front) talks to a
detached per-session `zellij-server` over `interprocess::local_socket` streams
(`zellij-utils/src/ipc.rs:8`) with a protobuf contract —
`zellij-utils/src/client_server_contract/{client_to_server,server_to_client,common_types}.proto`.
External processes act on sessions through the **pipes/actions API**:
`PipeSource::{Cli,Plugin,Keybind,PromptRequest}` (`zellij-utils/src/data.rs:3656`; the Cli variant
carries a pipe_id for block/unblock) and `plugin_api/pipe_message.proto` — a `zellij pipe`/
`zellij action` process can be held blocked until a plugin consumes it (crash-release fix f4d38d65,
#5537: "release a CLI pipe when the plugin handling it crashes, instead of blocking the client").
Nested sessions (#5589: guest session reports input mode/keybinds to its host session) extend the
same socket contract recursively. **Any process that speaks the socket protocol is a session actor**
— the cheapest plugin-adjacent extension lane they built.

### 2.4 Engine churn on a live seam

Measured via `git log -S` on Cargo deps + `git tag --contains`:

| Event | Commit | First shipped |
|---|---|---|
| wasmer → wasmtime | 7d7848cd (#3349, 2024-06-28) | **v0.41.0** |
| crash wave attributed to that runtime change | f16ee084 (#3776, 2024-11-15, "various crashes due to invalid state exposed by the recent wasm runtime change", CHANGELOG:306) | v0.41.x |
| wasmtime → wasmi (+ jemalloc, pinned executor) | 889bd06f (#4449, 2025-10-21, CHANGELOG:137) | **v0.44.0** |
| self-labeled tile breaks with a migration command | #5611 ("(BREAKING CHANGE zellij-tile) `NestedListItem::new`…", "`Text::new`→`Text::from`", ast-grep one-liner shipped) | v0.46.0 unreleased |

`grep -c 'BREAKING CHANGE' CHANGELOG.md` = **13** entries since v0.14 (2021), including v0.34.3's
"(BREAKING CHANGE) performance: change plugin data flow to improve render speed". Two engine swaps
in ~16 months plus a standing ledger of tile breaks = the repo-side proof that external plugin
compatibility is repeatedly lost at version bumps. (User-visible incident threads are outside the
clone — **UNCERTAIN**.) Plugins mostly rode through engine swaps because the *wasm module + WASI is
itself the ABI*; the real breakage vector is protobuf/tile drift, which no handshake converts into a
clean error (§2.2-3).

### 2.5 The version-negotiation history — built, patched, weakened, removed-then-reverted-to-string

The strongest local evidence for ModuKit's ABI-seal requirement is that Zellij *had* rudimentary
version machinery and *deconstructed* it:

| Commit | Date / era | What it shows |
|---|---|---|
| 4e5717ec `fix(plugin): mismatch JSON format on get_zellij_version` | 2021-12-01, → v0.22.0 | host→plugin version answer existed early and its **encoding was buggy** |
| 75801bdb `plugins: Improve error handling on plugin version mismatch` (#1838) | → v0.32.0 | mismatch handling needed its own fix round |
| 11b0210d `plugins: rework plugin loading` (#1924) | → v0.34.0 | loading (and its caching) restructured wholesale |
| ef365edc `remove version mismatch error` | 2023-08-08; **in no stable tag** (`git tag --contains` empty — an unmerged/branch commit) | explicit intent to *delete* the mismatch error surfaced |
| HEAD state | 2026 | `get_zellij_version` = informational query; `PLUGIN_MISMATCH` = post-hoc decode-failure string; **no load-time check** |

Lesson embedded: the negotiation path is where crashes hide; a const error string is what remains
when nobody owns the handshake with dedicated tests.

### 2.6 Isolation model — the honest correction

The brief's premise "plugins in separate processes" is **inverted** in the shipped design: plugins
are in-process, interpreter-sandboxed wasm on pinned threads inside the session server. Process
isolation exists at the **session/client boundary** (§2.3), not at the plugin boundary. ModuKit
Stage 2 should mirror the split that exists: per-session detached server + socket-attached clients
+ protocol-gated external actors; and read the wasm tier as ModuKit's *sandbox/script* tier, not its
process tier.

## 3. Key capabilities

- Polyglot plugin surface via wasm32-wasip1; `zellij-tile` Rust crate is the ergonomic façade
  (`shim.rs` marshals protobuf both ways).
- Per-plugin permission gating (`PermissionType`, event subscriptions; #5143 tightened command
  permissions), own data/cache dirs per plugin (`plugin_loader.rs:49-56`).
- Plugin UI toolkit (StyledText/Table/NestedList), real layout participation (#5590: plugin can
  yield its layout space back), floating/stacked/hover panes.
- Pipes + prompts: bidirectional CLI↔plugin messaging with blocking semantics
  (`pipe_message.proto`, `prompt.proto`, `prompt_requests.rs`, `watch_filesystem.rs`).
- Shared plugins (`shared.rs`, `call_update`, `module_with_exports(&["load","update","render"])`).
- Session ops: detached servers, resurrection, nested sessions, web/mobile client (#5441, #5630),
  and "Zellij as a library" distributions (#5630).

## 4. Development & current state

- HEAD `9dacbc19` (2026-10-08). Release train: v0.44.3 (2026-05-13) → v0.45.0 (2026-08-20) →
  v0.45.1 (2026-08-28); workspace 0.46.0-unreleased with a busy Unreleased block.
- 2025 lull (172 commits vs 373 in 2023) followed by 2026 recovery (305 by October): a
  single-maintainer burnout window measured directly in the commit stream.
- Breaking-change etiquette visibly improved: Unreleased entries are now labeled
  `(BREAKING CHANGE zellij-tile)` **and ship a migration one-liner** (#5611) — the labeling exists
  because the breaks are chronic.

## 5. Ecosystem

- In-tree: 11 built-in plugins (incl. a plugin-manager that downloads external ones + test fixture),
  `example/distribution/`, `docs/THIRD_PARTY_INSTALL.md` for distro packagers.
- Out-of-tree community plugins and the zellij.dev docs site — health **UNCERTAIN** from this clone.
- Yazelix (graphics fork) merged upstream in 2026 (#5630) — fork-and-remerge proven on this tree.

## 6. Highlights & limitations

**Highlights**
1. A shipped, daily-driver Rust plugin host with **three coexisting compatibility tiers** (embedded
   lockstep, sandboxed external, socket-protocol external) — the same mode-triangle ModuKit plans.
2. Honest policy where sealing is absent: version-lock what you build; sandbox + memory-cap +
   trap-catch what you don't; protocolize the rest as separate processes.
3. `pinned_executor.rs` + trap-to-pane UX (`handle_plugin_crash`) + per-plugin dirs/caps = a concrete,
   small crash/hang containment design.
4. Client/server protobuf contract + blocking CLI pipes = external processes as first-class session
   actors through a protocol-only lane — ModuKit Stage-2's supervisor bus in miniature.
5. MIT license; CHANGELOG discipline with labeled breaks; e2e suite (`zellij-integration-tests`).

**Limitations**
1. **No load-time ABI seal — ever** (§2.2/§2.5): no version constant exchange, no rejection path,
   mismatch only as post-hoc error text. The 13 BREAKING entries and two engine swaps are the bill.
2. Plugin hang = pinned-thread stall; synchronous host→plugin calls; the pipe block/unblock
   machinery exists precisely because plugin calls block clients (#5537).
3. 66% single-maintainer commit share; the one-year lull is bus-factor risk materialized, not
   hypothetical.
4. wasm-in-process excludes native plugins, GPU, and zero-copy frames — ModuKit's I420/compositor
   ambitions need the other plane entirely (see ipc-channel.md, iceoryx2.md).
5. One protobuf schema set mixes host-internal dataflow with the plugin-facing API, so internal
   event tweaks become external breaks (visible pattern across the CHANGELOG).

## 7. Historical lessons

1. **A half-built handshake is worse than none — it launders the absence.** Zellij shipped
   `get_zellij_version` (2021), patched its encoding, patched its error handling, then converged on
   an *informational query + after-the-fact error string* (§2.5). ModuKit's C ABI must negotiate
   **before any pointer crosses**: one `modukit_abi_version()` + per-symbol checksum, load-time
   reject, red/green negative test — the uniffi seal pattern with the plugin-side enforcement Zellij
   never built.
2. **Version-lock what you control; sandbox what you don't; protocolize the rest.** The three modes
   (§2.1-3) carry different compatibility burdens, and the product works *because* they don't
   pretend to be one loader. ModuKit's cdylib / multi-process / script tiers should adopt the same
   asymmetry — including "first-party plugins compile in the same build, period".
3. **A stable module boundary makes the engine a swappable dependency.** Two wasm engine swaps
   (#3349, #4449) were each one host-glue PR; plugin binaries mostly rode through because the wasm
   module *is* the boundary. ModuKit's native path has no such buffer — the C ABI is the module
   boundary — so the seal that wasm gives for free must be hand-built (contract version + checksum).
4. **Breakage ledgers need migration tooling to be survivable.** The #5611 pattern — labeled
   `(BREAKING CHANGE zellij-tile)` plus a runnable ast-grep rewrite — converts an ecosystem break
   into a 2-minute author chore. Cheap; ModuKit's C-ABI churn should adopt label + migrator per break
   as policy.
5. **Synchronous plugin invocation leaks stalls into users.** Zellij's answer (pinned executor,
   pipe blocking states, crash-release #5537) is all *containment*, none *preemption*. ModuKit's
   supervisor needs pre-baked watchdog semantics (timeout → quarantine → restart) rather than
   growing them under incident pressure.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS (production plugin-host shape, Stages 1/2/5) + the series' strongest
negative proof for ModuKit's ABI-seal requirement. MIT: license gate green.**

### 8.1 What the "Stage-2 mirror" actually is

The closest Stage-2 analogue is not their plugin layer but their **session plane**: detached
per-session server, socket-attached clients, protobuf'd action bus, protocol-gated CLI actors,
crash→pane error UX. That is ModuKit's supervisor + multi-process plugin channel at terminal
fidelity. Their in-process wasm tier maps instead to ModuKit's *script/sandbox* tier (Stage 5).
Reading them swapped — "sandbox in-process, isolate over sockets" — is the composition both repos
converge on: control plane in-process-friendly, data plane out-of-process (pairs with
`ipc-channel.md` §8 and the Stage-2 fork sheet in `00-overview.md`).

### 8.2 Mode-by-mode mapping

| Zellij mode | ModuKit counterpart | What transfers |
|---|---|---|
| Built-in lockstep wasm (§2.1) | First-party plugins shipped with host | Co-build version-lock as a legitimate tier; internal contract may break at will — never promise it |
| External wasmi plugin, protobuf RPC (§2.2) | Stage-5 WASM/script sandbox host | Event subscription + `PermissionType` gating, 16 MB cap, one host-import surface, trap→UI error |
| Session/pipe socket model (§2.3) | Stage-2 multi-process plugins | Socket contract + pipe block/crash-release semantics; per-actor error reporting |

### 8.3 Stage matrix

| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Adapt** | Permission-gated subscriptions; lifecycle errors surfaced per-instance (LoadingIndication pattern); builtins-vs-external as kernel policy from day one; narrow one-command-union dispatch (`host_run_plugin_command` + protobuf enum) instead of fat vtables. |
| 2 — transport / multi-process | **Adapt (shape) / avoid (payload)** | Per-session server + local-socket protobuf bus + pipe semantics lift to the supervisor/control plane. Synchronous pinned-thread calls and copy-per-message protobuf are the wrong shape for I420 frames — keep frames on the iceoryx2/SHM plane. |
| 3 — compositor/UI | **Reference-only** | TUI compositor; only the client↔server socket frame split is analogous to a host/router compositor bus. |
| 4 — WebRTC | **N/A** | Nothing media-plane (share plugin is socket remote-control, not streaming). |
| 5 — polyglot/script | **Adopt (pattern) / study hardest (governance failure)** | Lift the sandbox stack (wasmi + WASI caps + shim crate + permission enum); the unsealed seam is the D-record ModuKit must *not* replicate — add the handshake wasm users get implicitly and C-ABI users must author explicitly. |

### 8.4 License gate

**MIT** (LICENSE.md). No copyleft, no output obligations, no interaction with ModuKit's D1
(MIT OR Apache-2.0): code snippets, schema *designs*, and vocabulary are freely adoptable with an
attribution line in NOTICE/SBOM if files are ever copied. Greenest gate in this series alongside
iceoryx2 (MIT) and ipc-channel (MIT/Apache).

### 8.5 Not verified here (UNCERTAIN)

- **"Kibit" plugin format** — 0 hits in-clone (tree, history, CHANGELOG) and 0 GitHub search hits
  (probe 2026-10-09); if it refers to a future/third-party runtime it is outside this repo and must
  be sourced before any whitepaper citation.
- Adoption scale and external-plugin ecosystem health (no in-repo evidence).
- GitHub-issue-level "plugins broke on version bump" incident threads — the CHANGELOG/tag/commit
  ledger above is the strongest local proof; the mass-break anecdotes themselves are **UNCERTAIN**.
- Whether ef365edc ("remove version mismatch error") ever merged to a release — `git tag --contains`
  returns nothing for it; treated as an unmerged signal of intent, not shipped history.
- v0.46.0 direction: labeled breaks + migrators are improving honesty; still no sign of a load-time
  handshake.
