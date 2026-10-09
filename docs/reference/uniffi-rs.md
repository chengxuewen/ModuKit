# Reference Profile — uniffi-rs (Mozilla UniFFI)

> **External reference.** This profile was researched from a disposable blobless clone at
> `~/.cache/modukit-research/uniffi-rs` (not under `.refinfo/`); it is not ModuKit content and
> none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `5cba45b419f` (main, 2026-10-08) · Upstream: <https://github.com/mozilla/uniffi-rs> · Latest tag: `v0.32.2` · Pinned toolchain: Rust `1.94.0`
> All facts below are derived from this local clone unless marked **UNCERTAIN**.

## 1. Portrait

| Field | Value |
|---|---|
| Project | UniFFI — "a multi-language bindings generator for Rust" (README) |
| Language | Rust (authoring side); emits `extern "C"` scaffolding + foreign-language façades |
| Governance | Mozilla-owned; `docs/policies/`, `docs/release-process.md`; CODE_OF_CONDUCT; decisions tracked as MADRs in `docs/adr/` |
| License | **MPL-2.0** — file-level weak copyleft (LICENSE, per-file `Mozilla Public License, v. 2.0` header). **Not** permissive; see §8.3 license gate — this is the sharpest interaction with ModuKit's undecided license. |
| Size | 20 workspace crates (`uniffi`, `uniffi_bindgen`, `uniffi_core`, `uniffi_meta`, `uniffi_udl`, `uniffi_macros`, `uniffi_pipeline`, …); 2,262 total commits |
| Velocity | 47 distinct contributors / 12 months; 164 all-time; 167 commits / 6 months. Yearly trend: 2023=449, 2024=384, 2025=279, 2026=225 (YTD) — decelerating but still active (HEAD 2026-10-08). |
| Maturity | ~30 tags, all pre-1.0 (`v0.32.2` latest, 2026-09-08). README: "ready for production use, but a long way from a 1.0 release with lots of internal work still going on." |
| Real adoption | **Firefox** mobile + desktop (README: "used extensively by Mozilla in Firefox"; Kotlin/Swift faces from one Rust core). **LibreOffice**: see §7/§8 — commonly cited but **UNCERTAIN** from this clone (no in-repo evidence). |
| MSRV policy | `docs/policies/rust-versions.md` — tracks **mozilla-central "Uses"/"Requires"** versions, not a fixed floor. A Firefox-coupled MSRV, not a neutral one. |

## 2. Architecture in focus

### 2.1 Terminology correction up front
The brief refers to an "uniffi-ffi single ABI design". **No crate by that name exists.** The flat-C-ABI
machinery is spread across `uniffi_core` (shared runtime + ABI primitive types + metadata encoder) and
`uniffi_bindgen/src/scaffolding/` (per-build C-ABI codegen). What uniffi actually keeps "single" is not a
C ABI at all — it is a **typed metadata model** (`uniffi_meta`). The C ABI is *derived from it per build and
validated by checksum*, not authored and frozen. This distinction is the core of §8.

### 2.2 Metadata model (the real single source of truth)
`uniffi_meta/src/lib.rs` defines ~30 `pub struct`/`enum` metadata items: `NamespaceMetadata`, `FnMetadata`,
`ConstructorMetadata`, `MethodMetadata`, `TraitMethodMetadata`, `FnParamMetadata`, `RecordMetadata`,
`EnumMetadata`, `ObjectMetadata`, `CallbackInterfaceMetadata`, `CustomTypeMetadata`, wrapped by a `Metadata`
enum with integer item codes (`uniffi_core/src/metadata.rs` `codes`: FUNC=0, METHOD=1, RECORD=2, ENUM=3,
INTERFACE=4, NAMESPACE=6, CONSTRUCTOR=7, UDL_FILE=8, CALLBACK_INTERFACE=9, TRAIT_METHOD=10, UNIFFI_TRAIT=11).
The whole model carries a version: `pub const UNIFFI_CONTRACT_VERSION: u32 = 31;` (lib.rs:28). Any FFI-contract
change bumps it (docs/uniffi-versioning.md: renaming/calling/converting-FFI-functions = breaking).

**Two authoring surfaces feed one model** (`docs/adr/0001-mvp-webidl.md` chose WebIDL-based UDL as MVP;
proc-macros came later, ADR 0009):
- UDL path — `uniffi_udl` parses a `.udl` file (WebIDL-flavoured) → `uniffi_meta` types. Rust impls written
  separately; `uniffi::include_scaffolding!()` pulls in generated glue.
- proc-macro path — `#[uniffi::export]`, `uniffi::Object`/`Record`/`Enum` derives. **Proc-macros cannot see
  the whole interface** (each only sees its own tokens — metadata.rs doc comment), so they *emit metadata as
  static byte arrays exported from the compiled library*; `uniffi_bindgen/src/macro_metadata/` + `loader.rs`
  read them back out of the `.so`/`.dylib` symbol table. `uniffi_parse_rs` parses `src:<crate>` directly when
  no build is available.

### 2.3 How it keeps ONE ABI while generating many faces (the direct ModuKit question)
Mechanism, from `docs/manual/src/internals/{rust_calls,foreign_calls,lifting_and_lowering,object_references}.md`
and `uniffi_core/src/ffi/`:

1. **Lifting/lowering** — the *only* thing that crosses the boundary are flat primitives. Non-trivial values
   (`String`, `Option`, records, …) are serialised into a `RustBuffer`:
   `#[repr(C)] struct RustBuffer { capacity: u64, len: u64, data: *mut u8 }` (rustbuffer.rs:53). Byte-slice
   borrows use `ForeignBytes { len: i32 (JNA-friendly), data: *const u8 }`. Object/callback instances lower to
   a bare `u64` **Handle** (`0` = invalid; `uniffi_core/src/ffi/ffiserialize.rs`).
2. **Uniform call convention** — every exported fn becomes one generated `#[no_mangle] extern "C" fn` taking
   lowered args + a trailing `*mut RustCallStatus` out-param; return is either a lowered value or void.
   `#[repr(C)] struct RustCallStatus { code: RustCallStatusCode, error_buf: RustBuffer }`,
   code enum: `Success=0, Error=1, UnexpectedError=2, Cancelled=3` (foreign_calls.md) — exceptions/`Result`
   cross as status+buffer, never as Rust panics.
3. **Checksums as the ABI seal** — `scaffolding/templates/Checksums.rs` emits, per exported symbol, a
   `#[no_mangle] extern "C" fn <name>() -> u16` returning a checksum of that symbol's metadata
   (`uniffi_core/src/metadata.rs:251` `checksum()` over a `MetadataBuffer`). A dedicated
   `ffi_uniffi_contract_version` symbol exposes `UNIFFI_CONTRACT_VERSION` (interface/mod.rs:487). Bindings call
   these at load; a mismatch means the façade was generated against a different model than the shipped `.so`.
4. **N faces from the same flat surface** — `uniffi_bindgen/src/bindings/{kotlin,python,ruby,swift}/templates/`
   (Askama templates) each render the *same* `ComponentInterface` into native code that `dlopen`s / links the
   *same* `extern "C"` symbols. **Python uses ctypes; Swift links the C functions directly** — a difference
   explicitly acknowledged in `foreign_calls.md`, invisible to the Rust side. Rust→foreign callbacks are the
   mirror: a `#[repr(C)]` **VTable** of `extern "C" fn` pointers the foreign side registers via
   `uniffi_init_<iface>_vtable(...)` (ADR 0003/0004 — interfaces must be `Send+Sync`).

So uniffi's answer to "one ABI, many faces" is: **a versioned, checksum-sealed flat `extern "C"` ABI whose shape
is machine-derived from a metadata model, not hand-authored**. The faces never diverge from the core because
neither is written — both are generated from `uniffi_meta`, and the checksum makes drift a load-time failure
rather than silent UB.

## 3. Key capabilities
- Rich object-model types over FFI: records, enums (with associated data / `recursive`/`indirect`), interfaces
  (objects), callback interfaces, errors/`Result`, nullable, external + remote types, custom types.
- `async`/futures across the boundary (`internals/async-ffi.md`, `RustFuture`/`RustFutureContinuationCallback`,
  `Cancelled` status); Python event-loop binding, Swift async.
- Zero-copy borrow paths: `[ByRef]`/`[ByMutRef]` `&[u8]`/`&mut [u8]` as `ForeignBytes` (v0.32/HEAD; `#2940`,
  `#2878`) — mutable-borrow ABI-identical to read-only, flag `by_mut_ref` bumps metadata to 31.
- External-binding-generator API (`uniffi_bindgen` lib + `docs/manual/src/internals/*`) so third parties ship
  new language faces without touching core (`docs/adr/0007-enable-implementing-bindings-separately.md`).
- First-party backends in-tree: **Kotlin, Swift, Python, Ruby** (`ls uniffi_bindgen/src/bindings/`).

## 4. Development & current state
- HEAD `5cba45b` (2026-10-08, "Alexey"). Release train v0.29→v0.30→v0.31→v0.32.2 (v0.32.0 2026-06-30, v0.32.1
  2026-09-08). CHANGELOG.md is the live ledger (unlike zenoh's empty file), managed by `cargo release`.
- **v0.32.0 breaking batch** (CHANGELOG): async primary-constructors rejected for Kotlin/Python; Ruby forces
  named enum-ctor params; `--config` moved to a *global config* format (old `uniffi.toml` → warn+ignore);
  `[ByRef] bytes` UDL args remap `&Vec<u8>`→`&[u8]` (Kotlin callers switch `ByteArray`→direct `ByteBuffer`);
  pipeline bindgen reworked ("any external binding generators using this will [need changes]"). Pre-1.0 churn is
  real and lands in minors.
- Removed-API notes: "Removed the previously deprecated library-mode API: `BindgenCrateConfigSupplier`" — an
  explicit deprecation→removal cycle, tracked in CHANGELOG (see §7 on what "deprecated" means here).
- New surface: `uniffi-bindgen-kotlin-jni` crate (in-tree) + remote-types (`#[uniffi::export(remote)]`).
- UDL is **not** deprecated. Both authoring surfaces are maintained in lockstep (every v0.32 change is written
  for *both* proc-macro and UDL). See §7.

## 5. Ecosystem
- **First-party:** Kotlin/Swift (Firefox's original need), Python, Ruby. README lists exactly these four.
- **Third-party binding generators** (README §Third-party, each its own repo/maintainer, via the ADR-0007 API):
  JavaScript/React-Native (jhugman), Kotlin Multiplatform ×2 (Gobley, UbiqueInnovation), **Go** (NordSecurity),
  **C#** (NordSecurity), **Dart** (NiallBunting), Java (IronCoreLabs), Node ×2 (livekit, criccomini),
  Haskell (mercury). **Health of these is UNCERTAIN** — not verifiable from this clone.
- **Alternative tools** (README): Diplomat (C/C++-focused), Interoptopus. ADR 0000 rejected SWIG (no Kotlin/Swift)
  and Djinni (C++-core, "explicitly in maintenance mode") — the same "why not adopt X" reasoning ModuKit faces.
- Adjacent tooling: `uniffi-dl` IDEA plugin (UDL), `cargo-swift`, `cargo-ndk` Gradle plugin, `uniffi-starter`.

## 6. Highlights & limitations
**Highlights**
1. A *proven-at-Firefox-scale* embodiment of ModuKit's exact doctrine: one Rust core, one flat `extern "C"`
   ABI, N generated language faces — no hand-written per-language bindings (§2.3).
2. The **metadata model + checksum seal** is the mechanism ModuKit lacks a word for: version the *interface
   description* (`UNIFFI_CONTRACT_VERSION=31`), emit a `u16` checksum per symbol, fail at load on drift. Cheap,
   robust ABI-safety, directly portable.
3. Clean layering worth mirroring: `uniffi_meta` (types) / `uniffi_core` (runtime + ABI primitives) /
   `uniffi_bindgen` (codegen) are separate crates; the shared runtime is a dependency both sides link.
4. External-generator API (ADR 0007) is how they cap backend-maintenance cost — languages live in owners' repos
   with owners' CI, decoupled from core releases.

**Limitations**
1. **MPL-2.0**, not Apache/MIT. File-level copyleft: any *modified* uniffi source file must be open-sourced under
   MPL. Generated *output* is theirs to disclaim, but vendoring their runtime crates is contagious per-file. (gate §8.3)
2. Its "single ABI" is **derived + version-pinned**, not a *hand-authored, hand-frozen contract*. ModuKit's stated
   model (start FROM a hand-authored C ABI as the contract) is the opposite authoring polarity — see §8.2.
3. Pre-1.0 forever-so-far: breaking changes every minor (v0.32.0 above), and the "single ABI" checksum is
   **all-or-nothing** (any model change invalidates all faces) — same brittleness ModuKit must avoid re-inventing.
4. `RustBuffer`-centric = **serialise-across-every-call**; it is not a zero-copy transport (the v0.32 byte-borrow
   work is fighting this, narrowly). Fine for API binding, wrong shape for ModuKit's Stage-2 SHM/frame data path.
5. Firefox-coupled MSRV (`policies/rust-versions.md`) + Mozilla-staff-driven — decelerating velocity (§1) and a
   single-critical-mass sponsor, same governance risk as zenoh/ZettaScale.

## 7. Historical lessons
1. **Proc-macro vs UDL — the "UDL deprecated?" premise is FALSE.** uniffi began on UDL (ADR 0001, WebIDL) and
   *added* proc-macros (ADR 0009); **neither is deprecated.** Both are maintained in lockstep and every v0.32
   change is expressed twice (proc-macro + UDL form). What *was* removed is an old **library-mode API**
   (`BindgenCrateConfigSupplier`), not UDL. Lesson for ModuKit: a macro-in-Rust surface and a standalone IDL
   surface are not mutually exclusive — the two feed **one** `uniffi_meta`, so you don't deprecate one when you
   add the other. Deprecating a whole authoring model is a mistake this project demonstrably did *not* make.
2. **Backend maintenance cost is the real tax — pay it by externalising, not by pruning.** First-party backends
   never rotted away (git history shows zero *deleted* bindings dirs), but the team's ceiling is exactly **4**
   (Kotlin/Swift/Python/Ruby); everything else — Dart/Go/C#/Java/Node/Haskell/KMP/JS — lives in third-party repos
   behind the ADR-0007 generator API + a version-pinned contract. Lesson: **the number of languages you maintain
   in-tree is a hard budget.** uniffi chose to grow by contract stability (checksum/version), not by adding
   maintainers. ModuKit's "bindings generated from one C ABI" plan inherits this same trap at Stage 5.
3. **The ABI is versioned as data, and sealed by checksum — not frozen as a header.** `UNIFFI_CONTRACT_VERSION`
   + per-symbol `u16` checksum turn "the C ABI drifted from the façade" into a load-time error. Lesson: whatever
   ModuKit's hand-authored C ABI is, **carry an integer contract version + a content checksum across it** — that
   is the cheap, proven mechanism that makes a flat C ABI trustworthy.
4. **Serialise-by-default is a design commitment, not a free lunch.** `RustBuffer`-for-everything is simple and
   safe but costs a copy per call; the 2026 `[ByRef]`/`[ByMutRef]` bytes work (v0.32, `#2878`/`#2940`) is a
   multi-release scramble to claw back zero-copy for the hot path — and it forced cross-language caller churn
   (`ByteArray`→`ByteBuffer`). Lesson for ModuKit's SHM/I420 frame path: get zero-copy into the *first* ABI
   design, or you will relive this migration.
5. **Pre-1.0 churn lands on users via minors.** Two years of v0.x with breaking-minors (config format, byte
   args, pipeline rework) — README still says "long way from 1.0". Lesson: freeze the ABI contract *deliberately*
   (a D-record) before promising external binding authors a stable target; uniffi's contract version exists but
   its *meaning* (what counts breaking) only got documented after years of churn.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS (primary, Stage-5 polyglot) — REFINE our doctrine; do NOT adopt as a dependency.**

### 8.1 Direct hit on the "bindings generated from one C ABI, never hand-written" doctrine
**Validated** in its core claim: uniffi *is* the proof that one Rust core → one flat `extern "C"` ABI → many
generated faces, with zero hand-written per-language bindings, works at Firefox scale (§2.3, §6). Our doctrine's
"never hand-write the low layer twice" is exactly uniffi's thesis, and its layering
(`uniffi_meta` / `uniffi_core` runtime / `uniffi_bindgen`) is a reference shape for our `modukit-c` + generated
`modukit-{py,cs,js,wasm}` crates.

**But it challenges the phrasing "start FROM a hand-authored C ABI as the contract"**, and that contrast is the
finding (§8.2). uniffi does *not* hand-author the C ABI — it **derives** it from a richer typed model
(`uniffi_meta`) and seals it with a version+checksum. Two authoring polars:

### 8.2 Authoring polarity — uniffi vs ModuKit
| Axis | uniffi-rs | ModuKit stated plan |
|---|---|---|
| Single source of truth | Rust impl + `uniffi_meta` typed model (UDL or proc-macro) | a **hand-authored C ABI** header |
| C ABI origin | *generated* from the model, per build | *authored first*, everything generated from it |
| Faces derived from | the metadata model (via `ComponentInterface`), then the same flat C symbols | the C ABI directly |
| Stability mechanism | `UNIFFI_CONTRACT_VERSION` + per-symbol `u16` checksum, fail-on-drift | (unspecified — must be added) |
| Richness location | in Rust + model (records/enums/errors/async) | in the C contract we hand-write |

Reading: uniffi's model is *strictly richer* than a bare C header, because a C header cannot express records,
enum-with-data, `Result`/error, object lifetimes, or async — so ModuKit, starting from a hand-authored C ABI, must
**either (a) design a real IDL/metadata layer above the C ABI** (re-deriving uniffi's `uniffi_meta` insight, likely
landing on a UDL-like surface), **or (b) accept a much lower-ceiling contract** (functions + buffers + status only,
à la zenoh/iceoryx2). The comparison says: **our "C ABI is the contract" doctrine is weaker than "a typed
interface model is the contract; the C ABI is its generated projection."** Recommend ModuKit adopt the latter: a
small `modukit-meta`/IDL as the single truth, hand-authored *or* proc-macro-described, that generates both the C
scaffolding and every façade — and carries uniffi's version+checksum seal across the boundary.

### 8.3 Stage matrix
| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Adapt** | Its callback **VTable** (`#[repr(C)]` fn-pointer struct, registered via `uniffi_init_*_vtable`, `u64` handles, `Send+Sync`) is a second shipped example of a Rust-host dynamic ABI alongside zenoh-plugin-trait; the handle+status+`RustCallStatus` convention is directly liftable. |
| 2 — transport/SHM | **Reference-only** | `RustBuffer`-serialise-everything is the anti-pattern for our zero-copy frame path; watch their `[ByRef]`/`[ByMutRef]` bytes migration as a cautionary cost. |
| 3 — registry/services | **Reference-only** | Not a service registry; skip. |
| 4 — compositor/UI | **N/A** | Nothing here. |
| 5 — polyglot bindings | **Adopt (patterns) / study hardest** | THE closest analogue to our entire Stage-5 plan. Lift: metadata-as-contract, version+checksum seal, generated scaffolding + N Askama-style faces, ADR-0007 external-generator split to cap maintenance. |

### 8.4 License gate (the sharpest ModuKit interaction — flag prominently)
**MPL-2.0**, *not* Apache-2.0/MIT like zenoh/iceoryx2/CppMS in this series. Consequences:
- **File-level weak copyleft** (MPL §3.2): any *uniffi source file* ModuKit copies or modifies (e.g. vendoring
  `uniffi_core`) must remain MPL-licensed and its changes disclosed — contagious per-file, not per-work.
- **Generated output**: uniffi generates code but the MPL text carries no explicit output-rights grant;
  generated-binding ownership is a *project-policy assumption*, **UNCERTAIN** from license text alone. Do not
  assume uniffi-generated bindings are MPL-free without reading their template headers.
- **Interaction with ModuKit's TBD license** (README "License: TBD"): if ModuKit ships under a permissive license,
  MPL components still impose their per-file obligations downstream. If ModuKit ships as a **host that loads
  uniffi-built plugins across a C ABI**, that is *use* of uniffi (bindings + core compiled into the plugin), not
  combination in ModuKit's own binary — the C-ABI boundary is the isolation line MPL respects.
- **Gate action**: ModuKit should NOT vendor uniffi crates into a permissively-licensed core. Prefer
  **pattern-borrowing** (ideas/interfaces are not copyright-encumbered) + a ModuKit-authored ABI. If any file is
  ever lifted, pin "MPL-2.0" in SBOM/NOTICE and record in `decisions.md`; re-run `git log -- LICENSE` upstream at
  pin time. **Resolve ModuKit's own license with MPL-in-mind before Stage-5.**

### 8.5 Not verified here (UNCERTAIN)
- **LibreOffice adoption**: widely repeated in the Rust-bindings conversation, but **no evidence in this clone**;
  could not confirm via web during research. Mark **UNCERTAIN** — do not cite in whitepaper until sourced.
- **External binding-generator health** (Go/C#/Dart/KMP/JS/Haskell repos, activity, MPL-vs-their-own license):
  not checkable from this clone; verify per-repo before depending on any at Stage 5.
