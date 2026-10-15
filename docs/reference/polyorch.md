# PolyOrch — External Reference Profile

> Research date: 2026-10-15 | Checkout: 5b5ad89 (171 commits, last 2026-10-10;
> no tags) | Upstream: gitee.com/chengxuewen/PolyOrch (sibling project, same
> maintainer lineage — internal prior art, no third-party validation implied)
> External reference profile. The PolyOrch clone at repo root (and the nested
> copy under `.refinfo/VisiaEngine/PolyOrch/`, pinned 3cbc155 / 2026-10-14) are
> git-ignored; they are NOT ModuKit content. Claims below were measured on the
> root checkout plus the VisiaEngine consumer tree; anything not locally
> verifiable is marked UNCERTAIN.

## 1. Project portrait

| Attribute | Value |
|---|---|
| Name | PolyOrch — "a scalable build orchestrator for polyglot monorepos" (PBOS = Polyglot Build Orchestration System) |
| Developer | Same maintainer family (sibling project, like VisiaEngine/AccessBase); bus factor 1 |
| Maturity | Implementation underway; 171 commits in clone, no release tags, no semver |
| License | Apache-2.0 only (single `LICENSE`; ModuKit canon is MIT OR Apache-2.0 dual — compatible either way) |
| Language | CMake (9 helper modules, ~74 functions in `PolyOrchRustHelpers.cmake` alone) + shell test suites; **no compiled binary** |
| Deliverable | A CMake helper surface consumed via `add_subdirectory(PolyOrch)` + fixture tests (`tests/run.sh`, e2e flag, `tests/matrix.sh`) + `examples/` (20 lanes) |
| Positioning | Build-time orchestration over heterogeneous native build systems (cargo/cmake/meson/npm/pip), pixi-pinned environments; explicitly NOT a runtime, NOT a plugin host |

**Premise correction that shapes this whole profile**: "Polyglot" in PolyOrch is
a brand word, not a module — `Poly` = Polyglot (the monorepo has many
languages), `Orch` = Orchestration. All 88 "polyglot" hits are docs prose; zero
code matches. Nothing is "using PolyOrch at runtime"; what "polyglot" does is
**use PolyOrch at build time**: one build graph, native toolchain per project.
PolyOrch's "bridge" also means something different from ModuKit's usage: a
bridge is the **CMake ↔ native-build-system adapter**, not a language FFI shim.

## 2. Architecture in focus

Six design invariants outrank everything else (measured in
`docs/architecture.md:16-24`, each "earned" by a retired anti-design; the
D16 adjudication replaced the prototype-era Lua/Xmake-addon delivery form with
the shipped CMake helper surface):

1. **Anything that affects the build result is pinned in the repository** — no
   build-affecting config in user-global state.
2. **One build authority** (D16 reading): each project's native build system
   (cargo, cmake, meson…) owns its engine; PolyOrch drives it through the
   helper surface. Generated CMake/VS/Xcode projects are **insight projections**
   for IDEs, never a second build path.
3. **Zero intrusion**: no PolyOrch-generated file of any kind inside a bridged
   directory.
4. **Optional absence degrades and complains; a suspicious toolchain source
   fails hard.**
5. **PolyOrch does not re-implement an engine** — it owns only *how things
   connect*.

Consumption is `add_subdirectory(PolyOrch)` with the gitlink/checkout pin
**committed** in the consumer repo; there is deliberately no install/addon
mechanism (D16 retired it — "nothing is installed, so nothing can shadow").
The environment layer is pixi: declaration file + committed lock +
git-ignored per-user cache.

## 3. Key capabilities

Rust is the hub; every foreign-language face is a Rust crate artifact consumed
by a native consumer. Mechanisms verified in `cmake/` + `examples/`:

| Face | Mechanism | Evidence |
|---|---|---|
| Rust↔C++ | `cxx` (`#[cxx::bridge]`), trampolines generated at build time | `polyorch_rust_cxxbridge`, `examples/rust-bindings/crates/spine/src/bridge.rs` |
| Rust→C | hand-written `extern "C"` + **cbindgen** header | `polyorch_rust_cbindgen(… HEADER_NAME spine.h)` |
| Rust→Python | PyO3 cdylib, **abi3-py38** (no libpython link) | `polyorch_rust_pyext(MODULE …)`, `examples/rust-pyext` |
| Rust→Node | napi-rs, N-API stable ABI = plain cargo cdylib | `examples/rust-nodejs`; artifact renamed `.so`→`.node` |
| Rust→Web | wasm-pack `pkg/` npm deliverable (pkg-web/pkg-node) | `examples/rust-wasm` |
| pixi env | setup/run/lock helpers | `PolyOrchPixiHelpers.cmake`, 4 pixi examples |
| Packaging | `install(TARGETS … EXPORT)` emitting `find_package` Config | `polyorch_rust_install`, `examples/rust-install-export` |

**Generated vs handwritten boundary** (the part ModuKit cares about): the
surface *declaration* files (`bridge.rs`, `c_abi.rs`, PyO3/napi modules) and
the consumer-side reverse-callback glue are handwritten; only headers,
trampolines, and export config files are generated (build tree, never
committed). One rule is load-bearing (`docs/tutorials.md:182-194`, their
PIT-32): keep handwritten shims free of `rust::` types — generated `cxx.h` and
the crate can disagree under version skew. Note PolyOrch runs **two parallel
foreign surfaces from one crate** (cxx + cbindgen); this is NOT the
single-canonical-ABI doctrine ModuKit's whitepaper fixes.

**Absent**: wit/WASM component model (only `wasm32-*` bare targets), plugin
loading, service registry, lifecycle, ABI version negotiation, C# (appears
only in the frozen whitepaper's table of *Xmake* capabilities — no
implementation; no csbindgen precedent here).

**Naming grammar** (adopt-worthy): `LANGUAGE_PRODUCT <python|node>` on install
— "language-runtime products follow the HOST language's naming, not cargo's"
(python strips `lib` prefix; node maps `.so`→`.node`); bin-artifact name must
equal the crate name (enforced contract); target suffixes `-build/-run/-test/-cxx`
behind a `-build` mediator; `PolyOrch_RUST_TARGET_PREFIX` fusion namespace.

## 4. Development & current state

- Root checkout 5b5ad89 (2026-10-10, 171 commits, no tags); the VisiaEngine
  submodule pins a **newer** 3cbc155 (2026-10-14) — active daily-line velocity.
- Docs remain the specification layer; whitepaper is a **frozen v1.0 record**,
  `docs/architecture.md` + D16 are the working authority (their decisions.md).
- Test surface is the binding gate: offline units + fixture e2e + a generator
  matrix (`tests/run.sh`, `POLYORCH_TEST_E2E=1`, `tests/matrix.sh`); their
  C0–C6 conventions are shell-check gates, same culture ModuKit's C0 inherits.
- Known open hole in their own canon: package sources (vcpkg/conan) pinning is
  still global tool config — violates their invariant 1, tracked as O4.

## 5. Ecosystem & adoption

One real consumer observed locally: **VisiaEngine** (sibling project). Its
wiring is the reference adoption pattern:

- PolyOrch as a git submodule (pin committed); root `CMakeLists.txt` opens as
  an "Assembly layer": `include(cmake/VisiaEnginePolyOrch.cmake)` →
  `visiaengine_ensure_polyorch()` — a **D39 sentinel reuse/self-embed**
  (first-wins: if a superbuild already added PolyOrch, reuse its copy — this
  is what makes nested `add_subdirectory` safe).
- Per-lane wrappers: `visiaengine_ensure_polyorch_rust()` maps their `USE_PIXI`
  onto `polyorch_rust_setup(FROM pixi|system)`; `VisiaEnginePolyorchCrates.cmake`
  runs the workspace crate face through PolyOrch (dependency edges visible in
  one graph); `VisiaEnginePolyorchWasm.cmake` runs the js/wasm lane with a
  dedicated cache; the capi engine itself is built by PolyOrch (their T-C
  adjudication).
- Version single-source: CMake `project()` version is **read from the workspace
  `Cargo.toml`**, mismatch = hard configure error (an explicit anti-copy of
  iceoryx2's five-literal debt).
- `cmake_minimum_required(3.25)` = "PolyOrch embed floor" (their D39).

No public third-party adoption found (private gitee host) — external
validation: UNCERTAIN.

## 6. Highlights & limitations

Highlights:
- Invariant discipline is the export: pin-everything, zero-intrusion,
  native-authority, degrade-vs-fail-hard — directly compatible with ModuKit's
  D-record culture.
- Host-language artifact naming grammar and the `-build` mediator pattern.
- A real superbuild-safe embed pattern with sentinel first-wins (VisiaEngine D39).
- Insight-projection doctrine for IDE files (one build path, views are derived).

Limitations:
- **It is not a plugin host.** Everything ModuKit's core is (ABI governance,
  lifecycle, registry, loading) is out of PolyOrch's scope; adopting it solves
  only repo build organization.
- Two-surface FFI doctrine (cxx + cbindgen side by side) conflicts with our
  generated-from-one-C-ABI canon — adopt the orchestrator, not the examples'
  FFI shape.
- Apache-2.0 single license, no tags, bus factor 1, pre-1.0 surface churn
  (D16-style pivots have already changed the delivery form once).
- Hard dependencies it drags in: CMake ≥3.25 and **pixi** as the environment
  layer — see §8 tension with our mise pinning.

## 7. Historical lessons

- **D16 delivery-form pivot**: the addon/Lua form was retired in favor of a
  consumed-from-checkout CMake surface; the lesson transferred to us — pin the
  *tool by gitlink*, keep no install mechanism, and let the whitepaper freeze
  while `architecture.md` + decision ledger stay the live authority (same
  pattern ModuKit runs in docs/architecture.md §9).
- Their invariant 1 was "earned three times over" (global package manager,
  PATH-resolved engine, global tool config) — the three-times-earned phrasing
  matches our own PIT/C0 negative-proof discipline; treat global-state build
  config as a known trap class in Stage-1 gate tooling.

## 8. Value-for-us verdict

**ADOPTED** (D64-D67, user-adjudicated one by one 2026-10-15, votes A/A/A/A) —
as our build-organization layer; physical submodule landing on maintainer call.

Adoptable now, pattern-level: pin-by-gitlink consumption, sentinel first-wins
embed, host-language naming grammar, version single-source from `Cargo.toml`,
degrade-vs-fail-hard toolchain policy, `-build` mediator targets.

Adopted as dependency (D64, immediate policy): the kernel is still pure Cargo, but
the whitepaper's polyglot lanes (modukit-c/py/js/wasm, Qt examples) are the named
destination; at landing the VisiaEngine wrapper pattern - thin repo-local
`modukit_*` cmake functions delegating to `polyorch_*`, full-shape root assembly,
embed floor 3.25 - is the adopted integration shape (D66).

Original open tensions, now resolved in the ledger (2026-10-15):
1. **mise vs pixi** -> D65: split ownership - mise keeps host/agent tooling with no
   build-reproducibility promise; pixi owns build-time environments with the
   committed `pixi.lock` as the pin (`polyorch_rust_setup(FROM pixi)`); mirrors D57.
2. **CMake at the root** -> D66: full VisiaEngine-shape root assembly at landing;
   `crates/` stay pure Cargo under zero-intrusion; one graph, IDE files are projections.
3. **Doctrine boundary** -> D67: orchestration + naming grammar + version
   single-source adopted; the cxx+cbindgen parallel surfaces are refused - our
   emitters (modules/04) generate the single projection from one C ABI, cbindgen
   stays as the header generator inside that pipeline. Pinning: the submodule
   gitlink is the single version authority; no branch tracking, no second ledger.
