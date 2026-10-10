# Reference Profile — AUDESYS

> **External reference — and a special one.** Read-only clone under `.refinfo/AUDESYS` (git-ignored);
> it is NOT ModuKit content and must not be edited or cited as ours.
> **Provenance flag: `origin = https://gitee.com/chengxuewen/AUDESYS.git` — same maintainer account as
> ModuKit's origin (verified via `git remote -v`).** This is the maintainer's own **sister project**:
> internal prior art, not third-party validation. Borrow claims are therefore strong (same author,
> Apache-2.0), but nothing here counts as "the ecosystem adopted it".
> Research date 2026-10-10 · Checkout: `412ed1e` (2026-08-11) · History: 404 commits, all inside the
> last 12 months (measured) · Upstream: gitee (above) · License: Apache-2.0 (LICENSE at root +
> `workspace.package.license`) · Rust **edition 2024**, 24-member workspace, ~36.5k LoC excluding
> generated code (38.8k including), 826 `#[test]`/`#[tokio::test]` attributes (measured) + a
> TypeScript/Eclipse-Theia front end (330 `.ts`) untouched by this profile.
> Claims below are measured on this checkout; anything beyond is marked UNCERTAIN.

## 1. What it is

"Automation Development Embedded System" — an industrial-control runtime simulation platform
(README:3-19): Studio IDE (Theia + Monaco + GLSP), a Runtime Engine running a 5-step PLC-style
cycle with **program hot-swap**, a Hardware Abstraction Layer (HAL), simulator, CNC motion,
six IEC 61131-3 compilers (ST/IL/LD/SFC/FBD) + G-code, Modbus/HART adapters, Prometheus
metrics, a DAP debug adapter, and a napi-rs bridge into the IDE.

It is **not a plugin framework** — no dlopen, no runtime plugin loading; every integration is a
static workspace crate. It matters to ModuKit for a different reason: it is a **live, same-author
prior-art demo of the exact seams this repo has just adjudicated** (middleware abstraction,
staged update semantics, closed protocol types, JS-embedded kernel surface).

## 2. AMW middleware — their D11, an rmw-shaped three-pole trait set

The crown jewel of this profile. Their own decision record (translated from
`.agents/memorys/decisions.md`, their D11, 2026-07-09): *"following the ROS2 rmw pattern, define
a three-pole middleware abstraction (transport / discovery / QoS) with interchangeable
implementations — do not marry Zenoh/DDS/MQTT; swap the implementation, keep the API. Phase 1
uses inproc, Phase 2 uses zenoh."*

Anatomy (measured anchors):

- **Traits live in `audesys-hal-core`**: `HalTransport: Send + Sync` (`src/transport.rs:30`),
  `HalDiscovery` (`src/discovery.rs:39`), `HalQoS` (`src/qos.rs:75`), and a **supertrait**
  `AmwMiddleware: HalTransport + HalDiscovery + HalQoS` (`src/middleware.rs:18`).
- **Backend 1 (real): `audesys-amw-inproc`** — doc header: implements all four traits for
  single-process deployments (`src/lib.rs:1-12`): `InprocTransport` (signal publish/subscribe/read
  + RPC), `StaticDiscovery` (registry listing, pattern match, watchers), `InprocQoS` (LockLevel
  progression, Config Barrier, liveliness, security domains), `InprocMiddleware` +
  `InprocFactory` + **AuditLog**, plus `create_stream_channel` (`src/stream.rs`).
- **Backend 2 (scaffold): `audesys-amw-zenoh`** — header is honest: *"current implementation uses
  in-memory storage with the Zenoh API surface prepared for future pub/sub + query integration"*
  (`src/lib.rs:1-8`). Planned key-expression grammar `audesys/{namespace}/signal/{name}` and
  `.../rpc/{method}`; constructor is site-scoped (`ZenohTransport::new("site-a")`) — site as a
  first-class naming dimension, worth noting for our multi-machine Stage-4.
- **Their D10 — communication primitives**: three orthogonal kinds covering four systems
  (LinuxCNC/OpenPLC/ROS2/dora): **Signal** (single-writer, latest-value overwrite),
  **StreamChannel** (buffered multi-writer/multi-reader queue), **RPC**. Their stated rationale
  (translated): *Signal and StreamChannel must never be merged — a decade of ROS2 lessons.*

**Convergence with ModuKit**: their Signal/StreamChannel dichotomy is the same split as our D10
`latest-value` vs `keep-all` QoS classes — arrived at independently, cited to the same ROS2
battle scars. Their three primitives map cleanly onto our message lanes (service call / topic /
job+event).

**The divergence to watch**: their abstraction is **rmw-style by admission** (transport+discovery+QoS
trait trio = a worldview slice), while our D11 deliberately kept the seam one level lower
(porter only; kernel owns semantics). At the scaffold stage both shapes look identical; the pair
will pay differently once a real vendor exposes features that exist on one side of the seam but
not the other. Their tree is the live experiment to observe.

**Caution flag (dummy drift)**: the in-memory "zenoh" backend is exactly the seam-placeholder risk
our own doctrine warns about — a test double that could calcify into the product. Their header
discloses it; ours should keep D11's vendor slot equally honest.

## 3. Hot-swap + Config Barrier — their engine version of update semantics

Measured in `audesys-runtime/src/engine.rs` (1,072 lines):

- `prepare_swap()` — deserialize program bytes, `is_well_formed()` validation, stage into
  `pending_swap` (`:532-545`);
- `commit_swap()` — applied **at the next cycle boundary** (`:205` marks the in-cycle check
  "Config Barrier, D17"); `rollback_swap()` (`:553`) with a test proving pending state clears
  (`:1038`);
- config layer hot-reloads YAML with file watching (`src/config.rs:1`).

Their D17 (translated): *all configuration changes are queued and applied at RT cycle boundaries;
LockLevel becomes config-permission grading; at Run level all RPC config writes are rejected — a
multi-process supervisor can RPC at any moment, developer self-discipline is not a mechanism.*

This is the **engine-domain twin of our D8** (staged validate → commit at a safe point → declared
permissions; never "change the wheel while the car is moving"). Two vocabularies worth stealing if
HMI runtime config changes ever need grading: `LockLevel` progression and the Run-level-rejects-
writes rule. Note also: program bytes ride **bincode** (`engine.rs:534`) — positional encoding is
fine for same-version blobs but is precisely the failure class dora documented persisting across
versions (`docs/reference/dora.md:166-171`); do not copy that half.

## 4. Type system and serialization — one productive lane, one cautionary lane

- **Their D12 — closed protocol vocabulary**: exactly 14 types — 11 scalars (Bool/S8..F64) +
  String + Blob + Array\<T\> — chosen to cover IEC 61131-3 fully; wide strings deliberately
  excluded (UTF-8 only); Blob kept out of type inference. Deciding the *closed* type list before
  building the bus is the discipline pending-adjudication-11 (C-protobuf) formalizes.
- **Schema-first binary lane**: `hal_value.fbs` (namespace `audesys.hal`; scalar thin-wrapper
  tables `:27`, `union HalValueData` `:54`, `table HalValue` `:73`) with a **real cross-language
  test**: generated C++ headers under `tests/fb_cross/cpp/generated/` (1,247 LoC of generated
  code alone). A same-family proof that one schema compiling into multiple languages works in
  production-shaped code — supporting the derived-registry direction of adjudication 11.
- **The anti-pattern, in their own hand**: `audesys-runtime/src/ipc.rs` is 1,861 lines and carries
  the marker comment (`:256`, translated): *"manual HalValue→JSON; switch to serde once HalValue
  derives it"* — hand-written serialization at the protocol boundary, living in the repo as debt.
  That is the exact cost the fdset + generated-code ruling retires. Honest footnote: the comment's
  `ponytail:` prefix is the same maintainer convention used in this repo's rules.

## 5. Supervisor — a measured cost data point for adjudication 1

`audesys-agent/src/main.rs`, **297 lines total** (`:1-8` doc): monitors child processes listed in
a YAML config, **auto-restarts on exit with exponential backoff**, pushes status to the controller
over a Unix-domain socket (best-effort fire-and-forget), token-authenticated connect
(`connect_and_auth`, `TOKEN_WIRE_SIZE` `:70`), graceful shutdown SIGINT/SIGTERM → SIGKILL
children → wait → exit. This is a real-world proof of the adjudication-1 claim that a supervision
plane is "a few hundred lines, not systemd" — and our D3 restart-policy fields
(`max_retries/backoff/cooldown`) mirror what they converged on operationally.

## 6. The polyglot surface that already shipped

- `audesys-theia-bridge`: **32 `#[napi]` exports** (`src/lib.rs:104-177+`) — a JavaScript process
  embedding the Rust kernel. This is ModuKit's deferred **product face #2** ("JS app embeds
  kernel") already alive next door, and direct evidence that the napi lane is cheap once the C
  ABI/typed surface exists (our JS dual-face queue item has sibling precedent for face 2;
  face 1 — ModuKit hosting JS plugins — still has no precedent in either repo).
- Two Tauri desktop apps excluded from the workspace (`apps/studio`, `3rdparty/AUDEDeck`).
- `audesys-dap-adapter`: standard **Debug Adapter Protocol** over stdio (`src/main.rs:1-12`) —
  their debug story is editor-native, not a bespoke TUI. For our queued adjudication 13, this
  is the maintainer's own revealed preference.

## 7. Governance (process evidence only)

Same lineage as this repo's agent system: `.agents/memorys/` with their own dated D10-D17 records
(in Chinese — the sister project carries no C1 rule), `AGENTS.md`, `SKILL.md`, `openspec/`,
`qa/`, `docs/{architecture,modules,specs,plans,research}`, `bootstrap.sh/.bat` twins. Toolchain
pinning differs: they run **pixi** (`pixi.toml` + `pixi.lock` + `pixi.sh/.bat`) where ModuKit
chose mise — recorded as divergence, not endorsement. Lint gates present (`deny.toml`,
`clippy.toml`, `rust-toolchain.toml`). README self-reports "239 SDD specification items" and
"799+ tests" (measured 826 attributes — claim and reality agree here, which is worth saying).

## 8. Verdict — per ModuKit decision and stage

| Object | Verdict | Note |
|---|---|---|
| AMW three-pole trait set (their D11) | **CONTRAST-CASE** | Their admitted rmw-shape vs our porter-seam (D11): same at scaffold stage, pays differently when vendor features hit the seam — observe |
| Signal/StreamChannel/RPC trichotomy (their D10) | **BORROW-VOCABULARY** | Independent re-derivation of our D10 dichotomy; strengthens our D10 adjudication wording; "site" naming dim feeds Stage-4 key design |
| Hot-swap prepare→commit→rollback at cycle boundary + Config Barrier (their D17) | **PATTERN-ADOPT** | Engine-level sibling of D8 staged commits; `LockLevel` permission grading is stealable when HMI runtime config needs tiers; **do not copy** bincode-for-persisted-blobs |
| Closed 14-type protocol (their D12) | **STRENGTHENS adjudication 11 (C-protobuf)** | Type vocabulary decided *before* bus construction, twice in-family |
| flatbuffers cross-language lane (fbs → C++ tested) | **CONFIRMS derived-artifact doctrine** | Schema compiles to multiple languages in real cross-tests — the registry-artifact model, demonstrated next door |
| Hand-written JSON seam (ipc.rs:256 `ponytail:` TODO) | **ANTI-PATTERN EVIDENCE** | The exact debt generated bindings retire; keep quoted in adjudication 11's landing |
| 297-line YAML supervisor | **COST EVIDENCE** | For D2/D3 supervision-plane sizing |
| 32 napi exports | **PRECEDENT** | For the JS dual-product-face queue item (face 2 proven; face 1 open) |
| Static adapters (Modbus/HART behind HAL traits) | **CONTRAST** | Their compile-time integration is the credible alternative to pluginizing device access — supports keeping D2's axis open AND the "static profile" idea cockpit customers may demand |
| Naming hazard | **AVOID** | Their `hal-binding-gen` is an ST→IR *compiler*; when borrowing, keep our "runner/SDK generation" terminology clean |
| zenoh backend currently in-memory | **HONEST-FLAG** | Not a hidden failure (their header says so), but "amw_zenoh is production" must never be implied; dummy-drift exemplar for our own seam doctrine |

## 9. Unverified / out of scope

The Theia/TS side (330 `.ts`, 6 editors, HMI designer, GLSP) was not profiled — README claims
only. Their test pass/CI state not run. Whether any AUDESYS runtime is deployed anywhere outside
the maintainer's own machine: **UNCERTAIN**. Their decisions.md holds records beyond D10-D17 not
exhaustively read (threading D13, doc structure D14/D15 — only D10/D11/D12/D17 cited here).

**Sharpest single finding**: the maintainer has already built, in Rust, edition 2024, with 826
tests, the middleware seam + staged hot-swap + closed type vocabulary this repo spent today's
walkthrough adjudicating — and their tree contains, in visible `ponytail:` comments, the exact
hand-written-serialization debt the pending IDL ruling exists to prevent.
