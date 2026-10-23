# Wave-2 JS Volume Plan (D102 rail)

STATUS: REVIEWED OKAY 2026-10-23 (trail in §8). Nothing opens before the go order.
Rulings of record: D102 (docs/wave-2-js-hearing.md, drawn 2026-10-23, A,A,A,A).
THE CLAUSE: this volume lives or dies at M0's measured probe. Everything downstream is
drawn for BOTH outcomes - proceed or stand down with a named evidence row.

## 1. Goal and success (probe-gated)

A Node 22 program on this box, with ZERO installed packages, embeds the kernel through the
same C door as C++/Python: all 24 verbs, byte-identical service payloads across the three
languages, with every claim falsifiable. Success lines (valid only if M0 goes GREEN):

- `bash scripts/demo-wave2-js.sh` -> `JS-DEMO PASS` (16-verb walk through the generated bridge)
- `bash scripts/demo-wave2-js-sup.sh` -> `JS-SUP-DEMO PASS` + `FIDELITY PASS` + orphan census
- node:test harness green (facade pinned against the real .node artifact)
- dual goldens byte-stable across regens; structural pins green (bridge list == EMBED_VERBS)
- capability-ledger: four section headers gain the `JS` column; 24 verb rows filled
  planned->generated as the milestones land
- `bash scripts/check.sh` all green with ELEVEN demo legs
- coverage boundary: modukit-js joins the denominator as a fourth lib region - the pinned
  script must WIDEN to match (M3 edit, see below), >= 80 or a named honest exception at
  acceptance (D84a excludes only bins/fixtures; a real lib never gets silent exclusion)

Stand-down branch (M0 RED): discard the scratch dir (it lives outside the repo - nothing
to revert), record the exact failing command +
error in stage-5 as "js volume probe blocked, door stays armed (D102)", no further
milestones. That closes the round with zero waste.

## 2. Fact base (imported from the hearing; all re-verified there)

- Contracts (frozen, consumed not touched): 24 verbs = 16 rung-0 + 8 sup
  (contracts/modukit_embed.h:128,:139-235,vtable :246-282,sup :308-358); all exported by
  the kernel; dlopen-able libmodukit_core.so exists (crates/modukit-core/Cargo.toml:16
  cdylib since D100 M0); ambient kernel first-call init (embed.rs:92-96).
- Toolchain (measured 2026-10-23): node v22.23.3 (mise; npm 10.9.9; process.versions.napi=10;
  ABI modules=127); builtinModules ffi count = 0; node:test + node:assert PRESENT; system
  node 10.19.0 and node-gyp unusable (never needed - napi builds through cargo);
  ~/.cargo/registry cache holds napi-2.16.17 + napi-derive-2.16.13 + napi-derive-backend +
  napi-build-2.5.0 + napi-sys-2.4.0 + libloading-0.8.9/0.9.0 (7 real entries); libloading
  is already a core dependency (crates/modukit-core/Cargo.toml:19) - its offline
  availability is proven IN TREE.
- Codegen seams: the --embed_header door pattern (crates/modukit-codegen/src/main.rs),
  the header parsers (embed.rs:177-316 pub(crate)), the freshest emitter+test template
  (py.rs / tests/py_emitter.rs).
- The tri-language byte constant: "sup-fidelity-payload" (examples/cpp/host_sup_demo.cc:139,
  asserted by examples/py/host_sup_demo.py K_FIDELITY_PAYLOAD). JS asserts THE SAME bytes.
- Ledger checker is existence-only (scripts/check-capability-ledger.sh:9-10) - the JS
  column touches ledger text rows only, never gate code.

## 3. Milestones

### M0 - THE PROBE (martial clause; half a day, throwaway, nothing committed)
- Scratch crate OUTSIDE the repo tree (e.g. /tmp/jsprobe): `[lib] crate-type=["cdylib"]`,
  deps napi + napi-build (build.rs), exported `fn version() -> u32 { 1 }` via #[napi].
- Build with `cargo build --offline` under pixi; expect the .so/.node produced resolving
  ONLY cached crates. Then copy the artifact to *.node and load it in node:
  `node -e "const m=require('./jsprobe.node'); console.log(m.version())"` -> expect `1`.
- GREEN (builds + loads + calls) -> proceed, and the probe's Cargo.toml dep lines become
  the pinned versions for the real crate. RED -> STAND DOWN per §1, evidence row, done.
- Record which node ABI/modules version the .node matched (informational, pin note).

### M1 - bridge + facade for the 16 rung-0 verbs, with six gates' first five
- Emitter: new crates/modukit-codegen/src/js.rs - reuses the SAME header parsers; emits
  TWO artifacts: (1) bridge Rust source: a napi module whose body holds one forwarder per
  rung-0 verb; every forwarder calls through libloading against the libmodukit_core.so
  path handed in at init (ONE dlopen site, one typed symbol table - the C door, no other);
  (2) JS facade: index.js exposing named functions + enum mirrors + the i32 error
  narrowing (data only, semantics-free - D63 floor rule; sugar stays out).
- New workspace member crates/modukit-js: Cargo.toml (name per the whitepaper register
  :43), src/ hosts the GENERATED bridge (checked-in face goes to testdata/golden like the
  py face; the live build regenerates through the pipeline), build.rs calling napi-build.
- Wiring: --js_out door beside --embed_header in main.rs; regen-golden glob + *.py's
  sibling *.rs-face + the .js artifact; py_emitter.rs is the pattern for tests/js_emitter.rs
  (bridge list == EMBED_VERBS rung-0 set; facade list == bridge list; enum value-for-value;
  golden byte-compare x2; determinism x2; node:test harness file examples/js/pin_test.js
  run via `node --test`).
- Demo: examples/js/host_demo.js mirrors host_demo.cc scenes [1]-[11] (node:test asserts
  in-file or alongside; final line `JS-DEMO PASS`); lever DEMO_JS_SABOTAGE=1 flips ONE
  named scene; leg scripts/demo-wave2-js.sh (auto-skip when node/.so/build toolchain
  absent; cargo builds core + js bridge inside the leg) -> tenth check.sh leg; README
  gallery row; ledger: four section headers gain `| JS |` + the 16 rung-0 rows' JS cells
  planned->generated.
- Commits: `feat(codegen): js bridge + facade emitter` then
  `feat(js): workspace bridge crate + 16-verb demo leg`.

### M2 - sup rung (8) + tri-language fidelity + orphan census
- Emitter grows the 8 sup forwarders/facade entries (same parse sweep; the rung split is
  data, computed from the header).
- Demo: examples/js/host_sup_demo.js mirrors host_sup_demo.cc: sup install -> dial ->
  READY walk; FIDELITY stanza asserting the SAME "sup-fidelity-payload" bytes as the cpp
  stanza (:139) and the python K_FIDELITY_PAYLOAD; SIGKILL the child host -> /proc cmdline
  census via node fs walk, zero strays -> `JS-SUP-DEMO PASS` + `FIDELITY PASS`. Levers:
  DEMO_JS_M2_SABOTAGE=1 (census-blind) + fidelity-corrupt arm - both observed RED.
- Leg: scripts/demo-wave2-js-sup.sh -> ELEVENTH check.sh leg; ledger's remaining 8 JS
  cells flip; goldens regen stable.
- Commit: `feat(js): sup rung and tri-language fidelity for D102 M2`.

### M3 - closing sweep
- Nine->eleven wording everywhere (AGENTS, README sentence, status verification row);
  stage-5 lane row; architecture/status phase rows; coverage-boundary.sh WIDENS to four
  regions the same commit adds the crate to the denominator (grep whitelist + explicit
  modukit-js awk branch + the "exactly three rows" assertion becomes four - all three
  spots or the run stays a false green); its region % reported honestly; if < 80 it
  joins the acceptance verdict as a named discussion item, not a hidden exclusion.
- Commit: `docs(memory): js volume closed with eleven-leg sweep`.

## 4. Files inventory

New: crates/modukit-codegen/src/js.rs; crates/modukit-codegen/tests/js_emitter.rs;
crates/modukit-js/{Cargo.toml,build.rs,src/lib.rs}; examples/js/host_demo.js,
host_sup_demo.js, pin_test.js; scripts/demo-wave2-js.sh; scripts/demo-wave2-js-sup.sh;
golden faces for both artifacts. Edited: codegen main.rs/lib.rs (door + module); root
Cargo.toml (workspace member); scripts/regen-golden.sh (glob); scripts/check.sh (two
legs); docs/capability-ledger.md (four headers + 24 cells); README.md (two gallery rows
+ legs sentence); status/AGENTS/architecture/stage-5 at M3;
scripts/coverage-boundary.sh (M3: widen grep + explicit modukit-js branch + the
three->four row assertion). NOTHING under contracts/,
nothing under crates/modukit-core/src/ (D87; the .so is consumed as-is).

## 5. Risks and named limits

- The M0 probe is the load-bearing unknown; both branches pre-drawn (§1). No improvising.
- napi version pin: the probe pins whatever builds offline; the crate inherits those
  exact versions (shortcut: offline cache is the version oracle, revisit when egress exists).
- node ABI modules=127: the .node is only loadable by node 22.x on this box - named in
  the leg header comment (upgrade when the toolchain moves; that re-arms R2, D102 trigger).
- Generated bridge must avoid macro sprawl: ONE napi module init, mechanically emitted
  forwarders; anything the macros cannot express in generated form is a plan amendment
  before code, not during.
- GIL-equivalents/threads: all demos single-threaded; async facade is OUT (D6).

## 6. Out of scope (D102 closures)

Face 1 / script-host (own hearing; stage-4:134 is the Python instance of that separate-
case rail, the JS carve follows it by analogy). npm publishing, koffi/ffi-napi (offline-
dead). TS type emission (D48 lane). Hand-written floor (D63). Child-process shapes (not
the embed face). Frozen surfaces (D87).

## 7. Execution notes (landing round, 2026-10-23)

- M0 probe GREEN both steps: offline cargo build of a scratch napi cdylib (pinned
  napi-2.16.17 / napi-derive-2.16.13 / napi-build-2.5.0) + `PROBE 1` through
  process.dlopen. The scratch also de-risked the emitter first: no tuple ToNapiValue,
  no u64/usize conversions -> create_object returns + i64 crossing policy, both cited
  in the generated banners.
- M1: bridge + facade emitted from the frozen header (rung-0 first); tenth leg green
  (23 scene checks, `JS-DEMO PASS`), lever red at scene [2] observed, node:test 4/4,
  goldens regen-stable. The supply-chain fence caught the napi tree as designed (its
  set-leg observed RED in the first full gate) and was amended per its own rule: exact
  probe pins + closure admission + one-door confinement test.
- M2: filter site inverted (both rungs in, banners header-computed); sup rung walked
  the exe chain; `FIDELITY PASS (24 typed bytes out, payload byte-exact)` asserts the
  SAME constant cpp cites at host_sup_demo.cc:139 and python pins at K_FIDELITY_PAYLOAD
  - first tri-language byte agreement; both levers observed RED, restored; eleventh leg
  green; ledger JS column complete (24 generated).
- Deviations honored as drawn: node-parent gates via status-file + liveness probe
  (streams can't blocking-fgets); facade live path target/js (bindings/js unignored,
  no .gitignore edit); LD_PRELOAD of the pinned-env libstdc++ (node boots the system
  3.4.28, the cpp fixture chain needs 3.4.29) - all documented at their sites.
- Outstanding for acceptance: the modukit-js coverage-region reading (generated
  forwarders, no cargo-side tests - the pin surface is node-side); the boundary run
  with four regions reports it honestly and the maintainer rules (classify bin-like
  per the runtime precedent, or demand in-crate tests).

## 8. Review trail

- Full grill (fresh session): everything tree-true EXCEPT ONE BLOCKER - the plan claimed
  "modukit-js joins the coverage denominator" while coverage-boundary.sh is hard-wired to
  exactly three regions (grep whitelist, awk else-swallow, three-row assertion) and the
  script was absent from the file inventory - a false-green trap of the kind D101's
  re-review trigger exists to catch. Plus three nits (two " - "-chain commit subjects vs
  C3 prose; "revert" for a crate that lives outside the repo; stage-4:134 cited as if it
  governed JS when it is the Python instance of the same rail). All fixed same-day.
- Increment recheck (fresh session): OKAY - the three script spots named in M3 match the
  file's actual lines 41/42-45/54; all four commit subjects re-measured clean (41/51/56/
  52, no " - "); replaced regions read end-to-end with no orphans. Every M3 executor must
  still touch all three spots in one commit - the plan says it, the script will prove it.
