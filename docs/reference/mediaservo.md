# MediaServo — External Reference Profile

> Research date: 2026-10-09 | Checkout: `1b38f1ef` on `main`, clean tree, 2026-09-30 | Upstream: https://gitee.com/chengxuewen/MediaServo.git
> External reference profile. MediaServo lives in `.refinfo/` (git-ignored clone); NOT ModuKit content. Claims measured on this checkout; anything not locally verifiable marked UNCERTAIN.
> Dual purpose: (a) a worked sibling system for ModuKit's Stage 4 WebRTC / multi-machine line, and (b) the pattern source of this reference-doc series — `docs/reference/janus-gateway.md` is the 8-section template this whole series copies.

## 1. Project portrait

| Field | Value |
|---|---|
| Name | MediaServo — "real-time media servo platform", standalone-deployable video/media service platform |
| Upstream | `https://gitee.com/chengxuewen/MediaServo.git` (Gitee, not GitHub); HEAD `1b38f1ef` 2026-09-30 |
| History | 901 commits total: 210 in 2026-07, 500 in 2026-08, 191 in 2026-09. First commit 2026-07-17 ("initial commit — OMSPBase Phase 0→1") |
| Contributors | effectively single-author (`git shortlog -sn` returns empty — identity was unset in this checkout) |
| License | Apache-2.0 (`[workspace.package] license = "Apache-2.0"`, full-text `LICENSE`) |
| Size | 297 `.rs` files / ~94,000 lines outside `vendor/`+`3rdparty/`; 40 `.ts`/`.tsx` in `www/`; workspace edition 2024, resolver 3, version 0.1.1 |
| Positioning | independently deployable media infra: NVR capture+recording, video conference, remote desktop, teleoperation, live push/pull |
| Product surface | seven product capabilities; four deployment forms: Embed (static crate), Sidecar (container + napi-rs), Standalone (own backend, users/RBAC), Platform module (Docker, delegated RBAC/LDAP) |

## 2. Architecture in focus

12 root crates + 9 binding crates under one workspace. Verbatim members:

```
crates/  mediaservo-host     headless capture+encode+push
         mediaservo-client   cabin GUI consumer (Tauri v2), lib+bin
         mediaservo-server   signaling + relay + monitoring, optional SFU
         mediaservo-common   config, error, metrics, protocol, auth
         mediaservo-media    pipeline, broadcast, engine, transform
         mediaservo-webrtc   WebRTC abstraction, 3 backends
         mediaservo-codec    stub + FFmpeg + GStreamer
         mediaservo-link     frame bus + WS signaling client + auth
         mediaservo-deck     source/codec/record/playback, dual-form
         mediaservo-field    COMPOSITE: webrtc + link + deck
         mediaservo-weaknet  weak-network handling
bindings/c    -c for field, link, deck, client
bindings/cxx  -cxx for link, deck, field, client
bindings/node/rust  mediaservo-node
```

**C ABI is the single cross-language exit** (D227) — the same bet as ModuKit. C is the contract base; C++ header-only RAII and Python ctypes are thin wrappers; Node uses napi-rs, described as isomorphic to LiveKit's `rtc-ffi-bindings`. `soname = libmediaservo_<sdk>.so.<MAJOR>` with MAJOR = C ABI version (D241); per-SDK symbol prefixes `mediaservo_<sdk>_`; a `check-abi-drift` gate reconciles all four faces.

**Plugin system (`docs/modules/06-plugin-system.md`) — directly comparable to ModuKit's kernel.** Microkernel philosophy, decisions numbered D28–D30:

- **D28 `Plugin` trait** — a plugin is a registerable pipeline-node factory, explicitly modeled on OBS's `obs_source_info` struct-of-callbacks registration: `name()`, `version() -> (u16,u16,u16)`, `category() -> PluginCategory`, `capabilities() -> Vec<PluginCapability>`, `init(ctx)`, `shutdown()`. Marker: `Send + Sync`.
- **D30 capability declared at registration, never probed at runtime.** `PluginCapability { node_type, media_type, codecs, pixel_formats, priority: u8 }`; `NodeType { Source, Processor, Sink }`, `MediaType { Encoded, Raw, Both }`. Empty `codecs` vector means wildcard passthrough. Format matching is direct string/pixel-format comparison, not a caps lattice.
- **D29 `PluginManager` dual-mode loading** — `compile_time: Vec<Arc<dyn Plugin>>` via `inventory::submit!` build-time registration, and `run_time: Vec<Arc<DynamicPlugin>>` via `dlopen` + `extern "C" register()`. `find_nodes(node_type, media_type, &FormatQuery)` then priority sort, then `create_node(capability, config) -> Box<dyn AnyPipelineNode>`. Selection flow: `[NvencEncoder(255), VaapiEncoder(200), SoftwareH264(50)]` → pick highest priority.
- **Lifecycle** — `on_load()` for global resources (GPU context, decoder pool), `create_node()` on demand per pipeline instance, `on_unload()` for cleanup; runtime path ends in `dlclose`.
- **Explicitly rejected: GStreamer-style caps negotiation.** Documented comparison — GStreamer does caps intersection + `GST_QUERY_CAPS` + fixate; OBS does `output_flags` bitmasks. Phase 1 verdict: declarative capabilities cover every planned plugin, and caps negotiation is over-design for fixed-format chains. Dynamic negotiation is deferred to Phase 2+.
- **ABI safety is a sizeof check** (OBS mode), not a type system.
- **Plugin taxonomy** — Source (Host media producers), Processor (media transform), Sink (consume), Protocol (RTMP/HLS/SRT/RTSP over GStreamer; WebRTC over str0m|libwebrtc|webrtc-rs; RTCDataChannel), Relay (STUN/TURN via coturn; SFU via LiveKit/mediasoup).

**Client/host SDK layers (`docs/modules/04-sdk-layers.md`) — four SDKs, one-way acyclic dependencies:**

| SDK | Responsibility | C ABI prefix |
|---|---|---|
| `link` | frame bus (both ends + registry), WS signaling client, auth (reuses common PSK/JWT), dc (Phase 2) | `mediaservo_link_*` |
| `field` | composite: push/pull (webrtc) + re-export link + re-export deck(source) — one dependency, closed loop | `mediaservo_field_*` |
| `client` | consumer orchestration: VideoRenderer (GPU interop), multi-session, input forward + telemetry | `mediaservo_client_*` |
| `deck` | source (GStreamer), codec (FFmpeg static), record (mux to disk), playback | `mediaservo_deck_*` |

Direction: `field → webrtc + link + deck`; `client → field`; deck independent.

**`deck` is the interesting one for ModuKit** — it ships three link forms from one crate: `rlib` standalone, `rlib full` statically embedded into field (the default), and **`cdylib` `deck-full.so` for optional dlopen / OTA distribution**. That is a second, media-shaped dynamic-plugin path running parallel to D29's `PluginManager` dlopen mode.

**Transport / signaling split.** Three WebRTC backends behind one trait (D11):

| Backend | Feature | Size | Build | Role |
|---|---|---|---|---|
| libwebrtc via `webrtc-sys` | `backend-webrtc-sys` (default) | ~30 MB `.so` | cmake + Corrosion | public internet, weak-network, teleoperation |
| `webrtc-rs` | `backend-webrtc-rs` | ~2 MB | cargo | skeleton only (structs), future upgrade |
| `str0m` | `backend-str0m` | ~100 KB | cargo | Phase 2+, LAN P2P, embedded |
| `stub` | no feature | — | cargo | dev/test compile checks |

- **D31 `MediaTransport` trait is sans-I/O.** D32 = compile-time `cfg` dispatch (webrtc-kit pattern) with an explicit mutual-exclusion guard. D33 = the sans-I/O loop contract, "enforced by str0m".
- Signaling is layered and versioned: Phase 1 `SignalHandler` (`accept`/`handle_input`/`close`) operating sans-I/O and emitting `Vec<SignalOutput>` with `target: Direct(ConnId) | Room(RoomId) | Broadcast`; Phase 2 `RoomRouter` for topology-aware P2P/SFU/PubSub routing plus a mediasoup SFU bridge. Transport is axum WebSocket and is deliberately **not** traited. All wire messages are protobuf. MQTT 5.0 is Phase 2+ for vehicle-to-cloud (D74).
- **The SFU is deliberately external to the host.** `docs/modules/sfu-mediasoup-integration.md` covers worker lifecycle, router config, transport creation, producer/consumer, SdpAdapter, observer integration, crash recovery, and five enumerated transport failure scenarios (connect timeout, DTLS handshake failure, ICE restart, produce/consume failure, worker crash) with DTLS/SRTP security notes.
- Phase 1 relayed everything through the server on WebSocket; RTCDataChannel is a Phase 2 low-latency enhancement (D155 also downgrades DataChannel and fixes the GStreamer→WebRTC byte boundary). `docs/modules/07-protocols.md` fixes four-channel semantics and declares them invariant across that migration.

## 3. Key capabilities

1. **Capture → encode → push pipeline** on the Host (camera, microphone, desktop; GStreamer capture chosen on the evidence that it covers all target platforms including Jetson CSI/RTSP).
2. **Hardware codec first, software fallback** — Nvenc > Vaapi > software H264, selected by the priority field described above.
3. **Pull → decode → render** on the client with GPU interop in VideoRenderer.
4. **Signaling server** with pairing relay, room model, JWT + PSK auth, allowlist authorization, room discovery (`GET /api/rooms`), and room-grouping semantics (one vehicle room + N stream sub-items).
5. **Multi-stream consumers** — `Consumer` handle per producer with independent frame pumps, bounded close ≤250 ms, session state snapshots + cumulative state callbacks, machine-readable errors (`code`/`wire_code`/`retryable`), and control-channel backpressure observation (`ready_state`/`buffered_amount`).
6. **Recording/playback** with dual encoding (main + secondary stream, the NVR industry standard) so the push-side bandwidth ladder cannot pollute recorded quality.
7. **Weak-network handling** as its own crate, plus start/stop conflict diagnosis that names the occupying pid and self-verifies process-table zeroing after stop.
8. **Polyglot bindings** — C, C++11 header-only RAII, Python ctypes wheel, Node napi-rs — four consumption paths off one C ABI.

## 4. Current state

- **Phase 3 complete** per its own `AGENTS.md` (self-described as of 2026-07-23: 7-crate workspace, webrtc triple-backend, codec triple-backend). Workspace grew to 12 crates + 9 binding crates by 2026-09-30.
- **901 commits in 11 weeks.** 2026-08 alone is 500 commits — roughly one commit every two working hours, all from one identity.
- **Memory files are enormous and still being appended to**: 192 `## PIT-` entries and 87 `## D` entries as of HEAD.
- **Docs are the dominant commit type**: `docs(memory)` 76, `docs` 71, ahead of `feat(host)` 51 and `feat(cli)` 33.
- **Binding churn is the busiest feature area**: 19 `feat(bindings)`, 14 `feat(sfu)`, 18 `feat(admin)`.
- **Still unreleased**: root `Cargo.toml` workspace version 0.1.1; `CHANGELOG.md` has one `Unreleased` section carrying breaking changes.
- **UNCERTAIN**: no public release tags, no issue tracker signal, no CI run counts were verifiable from this clone.

## 5. Ecosystem

Reference peers it names explicitly in its own docs: **OBS** (the `obs_source_info` struct-of-callbacks pattern and sizeof-check ABI idiom), **GStreamer** (capture + protocol plugins + the caps negotiation it deliberately did not copy), **Janus Gateway** (the subject of the reference template this ModuKit series copies), **LiveKit** (`rtc-ffi-bindings` as the napi-rs template; SFU relay plugin), **mediasoup** (SFU worker/router/transport), **webrtc-rs**, **str0m**, **coturn** (STUN/TURN), **Janus**, **FFmpeg**, plus **ROS 2** (a bridging sample for device-day) and **OpenCV/Boost** (the single-package-multi-component CMake consumer pattern).

Deployment peers: Tauri v2 for the cabin GUI, Caddy for TLS termination, Docker/docker-compose for the platform-module form, pixi as the dev toolchain, and `docker-cargo.sh` to run cargo inside a container so the host machine needs no toolchain.

## 6. Highlights and limitations

**Highlights**

- **The plugin contract is unusually small and unusually argued.** D28–D30 fit in one module doc, each decision names the alternative it rejected (GStreamer caps negotiation) and states the phase that would reopen it. That is exactly the discipline ModuKit's kernel needs.
- **Capability declaration instead of runtime negotiation** — `codecs`/`pixel_formats`/`priority` as plain data at registration time. Deterministic selection, trivially testable, no format lattice.
- **Capability declared once, enforced by a gate**: symbol prefixes reconciled across four binding faces by `check-abi-drift`, version linkage checked against `cargo metadata` truth rather than hand-copied.
- **One C ABI, four surfaces, and the C ABI itself is versioned in the soname.** That is the whole ModuKit binding strategy, already executed.
- **Every architectural constraint has a failure story attached** — 192 PITs, 139 of them referenced inside commit subjects.
- **Docs organized by Diátaxis** and split by purpose: `reference/` is "facts, look-up, per product module, restrained, authoritative, unambiguous" and every new entry must be registered in the README; `research/` is "historical research archive, not part of active work".

**Limitations**

- **Single-author, 11-week, unreleased.** No external review, no released version, no tags. Architectural claims are untested by third parties.
- **The plugin ABI is the weakest part of the design** — a sizeof check is all the safety. `dyn Plugin` + `dlopen` means any ABI drift is a segfault, not a compile error.
- **Static trait dispatch only.** `Plugin: Send + Sync` with no async — fine for media pumps, wrong shape for anything needing a runtime or I/O.
- **The two plugin paths (D29 dlopen vs `deck` cdylib) are parallel and not unified.** Which one a new dynamic plugin uses is a project decision, not a documented rule.
- **Backend multiplicity has real cost**: ~30 MB `.so`, cmake + Corrosion cross-compiles, a vendored `[patch.crates-io]` fork of `webrtc-sys` (PIT-76) carrying an upstream-missing `request_key_frame` field.
- **Memory files have grown to 192 PITs and 87 Ds in one project month.** Unsearchable by a human without tooling; the retrieval burden is being solved with agent infrastructure rather than structure.
- **UNCERTAIN**: whether `sfu-mediasoup`, JWT allowlist authorization, and the admin dashboard are production-grade. The docs treat them as current; no external verification is possible from this clone.

## 7. Historical lessons

Pulled from `git log` and `.agents/memorys/pitfalls.md` at `1b38f1ef`. These are the lessons that matter most for ModuKit's own C ABI and binding pipeline.

**PIT-71 — link-time symbol collision hidden by `cargo check` (2026-08-07).** An e2e test introduced a `mediaservo-server` dependency into the host crate to spawn the SFU in-process, so the host test binary linked `webrtc-sys` (libwebrtc with embedded OpenSSL) and `mediasoup-sys` (static openssl-3.0.8) together: `duplicate symbol: X509_PUBKEY_it`. `cargo check` passed — it only checks types — and the file had `#![cfg(target_os = "linux")]` while CI ran on macOS, so "compiles" was an illusion. Fix: delete the dependency, drop the in-process branch, drive the SFU externally over WebSocket and assert through WS signaling only. **For ModuKit: two static OpenSSL/Cryptopp surfaces must never meet in one link unit, and a compile check that does not link is not evidence.**

**PIT-210 — build artifacts written into the source tree (2026-09-29, HEAD).** `build bindings` produced `bindings/node/mediaservo.node` and `bindings/python/mediaservo/{build,_libs,*.egg-info}` *inside the git tree* — 180 MB of build product in the node directory, `git status` noise, accidental-commit risk. Root cause: napi/pip build flows used the source directory as their working directory, predating the `out/` delivery root. Fix: a `target/bindings-staging/` staging area; the `.node` is copied there, and the Python wheel is built from a staging mirror with cwd at the staging root while assembly still reads real sources from the tree. Verification: `git status bindings/` zero diff after a build. The entry's explicit prohibition: **never cover artifact paths with `.gitignore` as the hygiene solution — that masks the problem instead of solving it.**

**Artifact-tree discipline.** A `target/ → out/` two-stage layout was adopted and then extended: `out/` is the single delivery root mirroring host bins and bindings (lib + versioned `.so.<major>` + node + include headers), `target/` is left alone. `deploy bindings` defaults to `<out>/bindings`. `build:deploy` in the same tree hits `SameFileError` and needed an idempotency guard.

**Version-chain drift.** The bindings version was hand-copied to three places, then four crates got independent version sources with a CI literal gate; the frame-metadata version drifted as a floating literal and was replaced by generation from `frame.rs` (`WIRE_VERSION = 0`) asserted at two points. Lesson: **any version string that is not generated is a drift source.**

**Distribution weight.** Fat Python wheel 528 MB → 79.9 MB from `strip --strip-unneeded` at two points plus a deliberately lean FFmpeg build (no openssl/network features — local processing needs no HTTPS, and that also removes the last symbol overlap with BoringSSL, resolving the PIT-71 family at the source). Package splitting into four half-zones (host/server/sdk-field/sdk-client) driven by a single `_TARGET_CRATE` variable, with a deliberate explicit error when a cabin-side consumer requests device-side components.

**Gate culture.** `139` commit subjects cite a `PIT-n` (through PIT-210); decision references run to `D289`. Three-stage CI (D-CI-01: Check → Test → Build & Package), a single `check.sh` that auto-detects pixi and delegates to dockerized cargo, and a `review-hardcode` skill. Every PIT entry carries symptom / root cause / solution / verification / prohibition — the same five-part shape ModuKit already inherited.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS — primary Stage-4 mirror (WebRTC / multi-machine), and the origin of this series' doc discipline.** Adopt its plugin-contract shape with one hardening, adopt its binding-pipeline rules wholesale, avoid its unsafe plugin ABI.

**Adopt — Stage 1 kernel**

- **Declare capabilities at registration, not runtime probing.** Copy D30 directly: `node_type` + `media_type`-style category + supported formats + `priority: u8`, matched by direct comparison. Empty format list = wildcard. Do not build a caps/fixate lattice. The `NodeType { Source, Processor, Sink }` ternary is media-specific; the *shape* — kind + formats + priority — is generic and works for arbitrary service plugins.
- **Adopt D28's registration surface as a template for the minimum viable `Plugin` trait.** `name`, `version`, `category`, `capabilities`, `init`, `shutdown` is six methods and it is complete. Resist growth.
- **Adopt dual-mode loading (D29) from the start** — `inventory::submit!` for compile-time and `dlopen` for runtime behind one `PluginManager` with `find_*` then `create_*`. MediaServo shipped compile-time in Phase 0 and dlopen in Phase 2; building both in the shape of one manager from day one is cheaper than retrofitting.
- **Adopt the priority field.** Hardware-over-software selection is a real, recurring requirement and `priority: u8` settles it without a solver.

**Adapt — the parts that are not portable as-is**

- **The plugin ABI.** `sizeof` check is not enough. For ModuKit's stated goal of cross-language plugins over one C ABI, a versioned, generated ABI with an explicit layout assertion and a generated C header is the difference between a segfault and a compile error. MediaServo never needed this because all its plugins are same-language Rust; ModuKit's whole premise does.
- **`deck`'s dual form.** The `rlib`-default + `cdylib`-for-OTA pattern is worth keeping as a documented second path, but only after deciding which of the two dynamic paths is canonical. MediaServo left it undecided.

**Adopt — Stage 5 binding pipeline**

- **One C ABI, everything else a wrapper.** C = contract base; C++ RAII and Python ctypes are thin wrappers; the C ABI version lives in the soname (`libmodukit_<pkg>.so.<MAJOR>`).
- **Per-language symbol prefixes** (`modukit_<pkg>_`) reconciled by a generated drift check.
- **Generate every version string.** No hand-copied literals; assert at two points.
- **A staging area between build and delivery.** Build into `target/<pkg>-staging`, assemble into `out/`, never write artifacts into the source tree, and never treat `.gitignore` as the fix (PIT-210's explicit prohibition).
- **Verify by linking, not by `cargo check`.**
- **Per-language package boundaries are explicit and error loudly** when a consumer requests a component it should not have.

**Avoid**

- Vendoring a patched upstream dependency to add one field (PIT-76) — it survives for months and outlives the feature.
- Multiple WebRTC/transport backends with compile-time mutual-exclusion guards as a first move. Pick one. ModuKit's whitepaper names `webrtc-rs` / getstream-rtc for Stage 4; MediaServo went the other way — libwebrtc via `webrtc-sys` as the default, with `webrtc-rs` demoted to struct skeleton. A documented counter-signal, not a reason to multiply backends.
- A memory tree that grows to 192 entries in one month. Structure beats volume.

**Ancestor note on the agent tree.** ModuKit's `.agents/` was ported from PolyOrch. MediaServo carries the same family in its mature form: the `rules/` tree is **identical** — all 15 entries match (14 language packs + `web` + `common` + `README.md`). Of ModuKit's 10 skills, 9 exist in MediaServo; MediaServo carries 22, adding `ci-cd-automation`, `test-harness`, `review-hardcode`, `security-hardening`, `source-driven-development`, `openspec-explore`, `openspec-propose`, `browser-testing`, and others ModuKit does not yet have. ModuKit's own `adjudication-walkthrough` has no MediaServo counterpart. Treat MediaServo's `.agents/` as the reference implementation of the scaffolding ModuKit inherited — especially `memorys/pitfalls.md`, which is what makes 139 PIT-citing commit subjects possible.
