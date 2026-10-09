# VisiaEngine — External Reference Profile

> Research date: 2026-10-09 | Checkout: b14d555 (293 commits, all within 12 months;
> no tags) | Upstream: gitee.com/chengxuewen/VisiaEngine (private mirror)
> External reference profile. VisiaEngine lives in `.refinfo/` (git-ignored
> clone); it is NOT part of ModuKit. Claims below were measured on this
> checkout during a dedicated four-lane survey + a five-member adversarial
> adjudication; anything not locally verifiable is marked UNCERTAIN.

## 1. Project portrait

| Attribute | Value |
|---|---|
| Name | VisiaEngine — embeddable 2D/2.5D/3D spatial visualization engine kernel |
| Developer | Same maintainer lineage as this repo family (sibling project, not third-party OSS) |
| Maturity | Phase-1 MVP declared complete (2026-09); 293-commit squashed local history, no release tags |
| License | Dual MIT / Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE` at root) |
| Language | Rust (133 `.rs` core) + C/C++/Qt example faces + wasm |
| Stack | wgpu pipeline (D4 decision: self-built, conda-forge rust via pixi, D5), CMake consumer facade |
| Positioning | Engine kernel, explicitly not a game engine; embeds into Qt/Flutter/C#/Web via a 49-function C ABI |
| Target users | Host applications needing map/point-cloud/glTF/tile rendering without owning a render loop |

## 2. Architecture in focus

Layering with two statically-enforced invariants (from `crates/*/Cargo.toml`
descriptions and `docs/architecture.md`):

```
visiaengine-core (1,031 LoC: scene graph = Slab handles + dirty flags,
  coordinates, resources)          invariant 1: core never depends on rendering
visiaengine-geo (853) GeoJSON/projection/tiling
io-* family (glTF 351, hdr 267, points 459, text 259, tiles 1,104) — all CPU-side
visiaengine-render (1,793: RenderBackend trait + command IR, zero GPU types)
visiaengine-render-wgpu (3,589: default backend; invariant 2: wgpu types stop here)
bindings/c/visiaengine-capi (49 pub extern "C" fn in src/ffi.rs)
bindings/{cpp,qt,js} — mirror faces
```

C-ABI discipline (the part ModuKit cares about; all verified in `src/ffi.rs`):
two-channel errors (domain `int` codes; panics fenced to `VE_ERR_PANIC = -4`,
line 18); the single cross-ABI struct self-describes via a `struct_size`
first field (line 29-33, contract CAPI-05) with `visiaengine_abi_version()`
(line 164) as runtime pin; generation-checked handle table (no raw pointers);
caller-allocated out-buffers; **pull-model loop** — host keeps its own event
loop, engine exposes pump/render at host cadence; cdylib-first packaging with
SONAME (static link closure rejected as explosive). Header `visiaengine.h` is
cbindgen-produced then human-reviewed; C++/Qt/wasm are hand-written mirrors
kept equal by an ABI-mirror gate with an explicit rename-alias table.

## 3. Key capabilities

glTF/GeoJSON/PLY point clouds (EDL stroke), MVT + raster tiles with layered
basemap stacking (E817 pixel-assert pair), GGX PBR + IBL, GPU instancing
(100k blocks, single draw), shadow/SSAO/bloom/tonemap post chain, CJK-subset
label rendering, section clipping, mini-map + click navigation, 2D↔3D
projection morph, picking, measurement, far-coordinate rebasing (D7,
pixel-level verified). SDK install tree: 9-file manifest, relocatable
`find_package` + pkg-config.

## 4. Development & current state

- Velocity: 293 commits in the clone, entire visible history within 12 months;
  no tags → release cadence is doc-batch based ("Phase 1 MVP + batch 4
  render + batch 5 docs re-home"), not semver. Contributor count: single
  maintainer visible (shortlog empty due to squashed author metadata — treat
  bus factor as 1).
- Machine-enforced corpus (measured): 199 contract headings in `docs/sdd/*.md`
  per file-tally (capi 41 + core 21 + geo 27 + gltf 11 + io 18 + render 47 +
  wgpu 34); **one survey reported 152 — unresolved conflict, re-derive via
  `bash scripts/spec-trace.sh` before quoting**; 279 `// spec:` tag lines.
- C ABI: 49 exported functions verified via grep of `pub extern "C" fn`
  (gate-abi's `nm` cross-check needs cargo — unavailable in this environment).
- Gates: 27 scripts in `scripts/`; CI chain `fmt lint check test audit
  gate-style gate-trace gate-abi gate-docs check-promises cmake-smoke`.
- Health caution: prose numbers drift (README vs measured package/crate counts
  differ; measured tree = 12 packages incl. capi/wasm, README text varies).

## 5. Ecosystem & adoption

No public GitHub presence found (private gitee mirror); stars/adoption
UNCERTAIN by design. Ecosystem analogues for its tech choices: wgpu, pixi,
conda-forge, Qt. Its own `docs/reference/` profiles 18+ external projects —
VisiaEngine practices reference-analysis on others; this series reciprocates.

## 6. Highlights & limitations

**Highlights.** Gate culture where every check must be observed red at least
once (negative proof embedded in script comments); docs-only gates run
sub-second in pure bash; overclaim lint polices "delivered vs promised"
language; decision records (`D5`, `D6`, `D7`) carry trigger clauses for
revisit; evidence memos are dated and immutable.

**Limitations.** Ritual density tuned for a GPU SDK may over-engineer a plugin
kernel (the tribunal's central YAGNI finding); hand-written mirror faces
conflict with ModuKit's generated-only canon; heavy pixi/conda lockstep pinning
raises onboarding cost; single-maintainer velocity means doc↔reality drift is
policed by the author's own gates (works, but self-referential); upstream
gate scripts carry CJK comments and GNU bashisms — unportable into ModuKit
without a rewrite.

## 7. Historical lessons (from its own recorded history)

1. **Vacuous-assertion incidents, twice recorded**: a hard-coded clear color
   made a pixel coverage assertion constant-true and pass CI for a whole
   feature band (its PIT-45); a silent `ctest -R` empty match would have
   "passed" a family of missing tests. Both became gate rules.
2. **Doc drift is real even with gates**: README/AGENTS/package metadata numbers
   disagree across surfaces until locked by derived-count checks — proof that
   prose numbers need truth-holder tooling (ModuKit's future `gate.sh numbers`).
3. **Marketing prose smuggles into sibling canon**: this repo's own research
   draft inherited Visia's "host-loop contract" phrasing and presented it as
   ModuKit doctrine — caught only by adversarial re-reading (tribunal
   prosecutor, R1). Cross-project citation needs provenance discipline.
4. **Feature-gate before API-freeze**: scene tree deliberately kept out of the
   C ABI until semantics settle — the opposite of aspirational-API drafting.
5. **Escape hatches get ledgers, not silence**: D5 (pixi-single-source) records
   its own exceptions (win-64 rustup, embedded cross-compile) with named
   conditions — the pattern ModuKit should copy for any "we don't use X,
   except" decision.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS** (sibling method source; not a dependency —
ModuKit does not render, so no code reuse; no license gate issue for
patterns-only adoption).

- **Adopt directly (now, docs-only)**: gate SKIP/RED/empty-set exit discipline
  (V1 — codifies our C0); doc-liveness link check (G1); D-record mechanics +
  ID-ledger law + supersede-don't-rewrite (G5/G6); gap-closure census with
  permanent-rejection section (G8); loaded-instruction rent audit (G10);
  evidence-pointing discipline for every published number (G2/P2); overclaim
  lint with /tmp-staged self-test (G4); the fixed reference-profile template
  itself (G3 — this document is its first ModuKit use).
- **Adapt (principle yes, mechanism no)**: spec-trace becomes D/PIT citation
  liveness until code exists (V3); cargo-authority/facade purity as principle
  only (V5 — no cmake in our stack); cmake-smoke negative-path states as a
  rule, not scripts (V8); capability gating stays a freeze-order rule, no API
  shape (A8).
- **Adopt when code lands**: handle/`struct_size`/borrow-buffer ABI clauses at
  Stage 5 (A2/A4/A5 — contract text now, asserts with cargo); bidirectional
  spec-trace full corpus at Stage 1 tests; E-band dual-mode examples at
  Stage 1 examples; own-numbered size budget at first artifact (P1).
- **Avoid (adjudicated 5-lens REJECT)**: hand-written per-language binding
  mirrors with alias-table gates (A6 — violates our generated-only canon;
  binding wording: "Hand-written binding layers are prohibited; generated
  only from the single C ABI"), copying its size numbers (A7/P1's 7.25 MB),
  llms.txt with hand-typed counts (G7), the gallery/card machinery (V7).
- **Full adjudication trail**: 30-candidate dossier and five-lens verdict
  tables were kept in `/tmp/opencode/visiaengine-r1-pool.md` and
  `visiaengine-r2-pool.md` (scratch, ephemeral — the distilled verdicts
  above are the durable form).
