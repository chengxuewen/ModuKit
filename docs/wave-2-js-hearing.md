# Wave-2 JS Bindings Hearing (D102)

STATUS: DRAWN 2026-10-23 - all four items walked one by one, rulings A,A,A,A (user-
adjudicated). Minted as D102. Volume plan follows on the standard rail.
Volume: JS/Node bindings - the "face 2" door (a JS program embedding the kernel through the
C gate), armed since the C++ volume closed.
Trigger: maintainer word 2026-10-23 opening the JS door, after the Python volume
accepted (D101) - the
same "own word and round" rail the armed list names (docs/stage-5-plan.md, armed-lines
paragraph; docs/stage-4-plan.md:186, :196-197).

## 0. How to read this

Four items decide this volume. Each is one plain question; the walkthrough takes them one
at a time; the star points at what the facts lean toward, nothing is decided until the
maintainer rules. Every number below has a file:line or a live command you can check now.
One honesty note up front: unlike Python, JS was never on the equal-capability promise line
(its four languages are C/C++/Rust/Python - docs/stage-4-plan.md:17-18), so this volume
runs on product-surface grounds, not on debt-repayment grounds.

## 1. Why this volume, in one paragraph

The pitch is "any language plugs in through the C boundary". Two now prove it (C++,
Python). JS is the third named consumer in the planned-crate map ("modukit-js # JS/Node
bindings", docs/whitepaper.md:43) and the only remaining binding whose CONTENT decisions
are already closed rather than open: the direction ruling says the sole sanctioned use of
the napi toolchain is BINDING - external JS calling our C ABI - and never EMBEDDING a JS
engine into the host (.agents/memorys/decisions.md:380, D54). The external precedent
audited for this project spells the two product faces out: face 2, "a JavaScript process
embedding the Rust kernel", has a proven sibling (32 napi exports in the audesys bridge,
docs/reference/audesys.md:123-127, :154); face 1, ModuKit hosting JS-authored plugins, has
no precedent and belongs to the script-host separate case. This hearing is about opening
face 2 only.

## 2. Fact base (every claim anchored or measured)

Contracts - the door this volume consumes, untouched:
- 24 embed verbs = 16 rung-0 + 8 sup (contracts/modukit_embed.h:128, :139-235, vtable
  :246-282 = 2 guards + 15 slots, sup free functions :308-358); all 24 exported from the
  kernel (16 + 8 `extern "C"`, verified by nm on the built cdylib in the Python volume).
- A dlopen-able libmodukit_core.so EXISTS since D100 M0 (crates/modukit-core/Cargo.toml:16
  crate-type carries cdylib) - the JS bridge consumes the same artifact Python's ctypes does.
- The ambient kernel is process-global, first-call-initialized
  (crates/modukit-core/src/embed.rs:92-96) - nothing to invent here either.

Toolchain reality (live commands, 2026-10-23):
- node v22.23.3 on PATH (mise-managed; npm 10.9.9 alongside); process.versions.napi=10.
- NO builtin FFI: builtinModules contains zero ffi-named modules (measured `0`).
- `node:test` and `node:assert` are PRESENT in the standard library (measured import) -
  the JS mirror of the Python volume's "no pytest, stdlib instruments" fact resolves the
  OTHER way: a real test harness ships with node.
- The apt fallback is dead: system node 10.19.0 (ABI-incompatible), node-gyp broken
  (MODULE_NOT_FOUND nopt, measured) - irrelevant below, but it closes the hand-built door.
- The offline crates cache HOLDS the napi stack: napi, napi-derive, napi-derive-backend,
  napi-build, napi-sys + libloading (7 real entries today under ~/.cargo/registry - an
  8th grep hit is winapi-util, a substring collision). NO napi-family crate is referenced
  by any project Cargo.toml (measured); libloading is ALREADY a core dependency
  (crates/modukit-core/Cargo.toml:19, the dl executor since Stage-1) - in-tree proof of
  that crate's availability, one less unknown.
- NOT CHECKED (must be step-0 measured before M1 builds on it): that a napi-bridged cdylib
  compiles from the cached crates with `cargo build --offline` and that the resulting
  `.node` loads into the measured node via process.dlopen. If either fails, R2's A option
  is environment-blocked exactly like cffi was for Python - the hearing decides on the
  probe, not on hope.

Doctrine anchors (verbatim, they bind the technique item):
- "C ABI is the single cross-language exit: every external export goes through a C ABI;
  polyglot bindings are generated, never hand-written per language." (docs/whitepaper.md:64)
- D54, the direction law: "napi-rs's sole sanctioned use is BINDING direction (external JS
  calling our C ABI, modukit-js) - never EMBEDDING a JS engine into the host"
  (.agents/memorys/decisions.md:380). For a bridge this means: the generated body may ONLY
  talk to the exported C symbols of libmodukit_core.so; linking the Rust core directly
  would be the forbidden side door in a new costume.
- Generated floor (D63): base bindings are codegen outputs; hand-writing a replacement
  floor is forbidden. A hand-written bridge body is the drift disease D63 names.
- The whitepaper's own JS row pre-dates the direction ruling in vocabulary but agrees in
  direction ("JS/Node bindings | napi-rs", docs/whitepaper.md:146) - no contradiction to
  adjudicate this time, unlike PyO3 (that delta is already recorded, D100).
- The parity line's four languages are "C/C++/Rust/Py" (docs/stage-4-plan.md:17-18) - JS
  is NOT among them. Scope pressure is therefore product judgment, not contract.

Gate machinery the volume can reuse (all shipped, all live):
- Codegen doors: main.rs --embed_header seam, py.rs face emitter pattern, golden/pin/
  determinism tests (crates/modukit-codegen/tests/py_emitter.rs is the freshest template).
- Demo-leg grammar with sabotage levers and the PASS/SKIP/FAIL triple (nine legs today;
  scripts/check.sh:92-127 plus the python pair appended after it).
- Capability ledger: header is `| Capability | C | C++ | Rust | Python |`
  (docs/capability-ledger.md:63, :98, :126, :157) - there is NO JS column yet. The
  checker leg deliberately checks ROW EXISTENCE only, never cell values
  (scripts/check-capability-ledger.sh:9-10) - so a JS column amends the ledger's four
  header rows only; the checker stays untouched.
- The pinned coverage-boundary recipe (scripts/coverage-boundary.sh) - a Rust bridge crate
  enters the core-adjacent denominator and will need the same honest treatment the cpp band
  got (fixture/library classification at its acceptance).

## 3. Not decided here (named, so nobody re-litigates mid-volume)

- Face 1 - JS-AUTHORED plugins (a script engine hosting plugin code): the script-host
  separate case, same door that holds Python-authored plugins out (docs/stage-4-plan.md:
  134 footnote; D33 runner layers). A JS volume that embeds QuickJS/Boa into the host is
  EXCLUDED by D54's own words.
- npm distribution of the skins (no registry here offline; consumers vendor the built
  .node + .js facade).
- TS type emission (D48 "TS types come from proto generation" is the parked lane; adding
  it here is scope creep - named, not silent).
- ESM/CJS packaging taste: the facade form follows whatever the measured step-0 probe
  requires; not a doctrine matter.

## 4. The four items

### R1 - SCOPE: which face, how much of the 24?

- A. **Face 2 only, all 24 verbs, two milestones** (M1 = 16 rung-0 walk + gates;
  M2 = sup rung + fidelity + orphan census) - the exact Python shape.
  (+) one door, proven form, parity with the two shipped bands; the exe-rung supervision
  is where JS hosts actually shine (a Node app managing plugin processes is the audesys
  precedent's own use case).
  (-) roughly the same effort the Python volume cost.
- B. Face 2, rung-0 only (16 verbs); sup later.
  (+) smaller first cut. (-) the known no-alarm deferral pattern again; JS hosts would
  embed only half the product.
- C. Face 2 + face 1 together.
  (+) "full JS story". (-) face 1 is the script-host separate case - would reopen D33/D54
  territory inside a bindings volume; excluded on §3 grounds, listed for completeness.

Facts lean A.

### R2 - TECHNIQUE: how does JS speak through the C door?

- A. **Generated N-API bridge**: codegen emits a small Rust bridge (body: dlopen
  libmodukit_core.so via libloading, one forwarder per verb calling its exported C symbol,
  napi types only on the JS edge) + a JS facade module; the bridge is built by cargo
  (no node-gyp - that tool is dead here anyway), loaded by node's own addon mechanism.
  Generated floor: the forwarders come from the same header the Python skin came from.
  (+) keeps D54's direction literal (JS -> C ABI); reuses the cached crates (measured
  present) AND the existing codegen parsers; no node headers needed at napi's layer
  (UNVERIFIED - the step-0 probe in §2 is the gate).
  (-) two artifacts instead of one (bridge .node + facade .js); the bridge is a real
  Rust crate, not a script.
- B. koffi/ffi-napi-style runtime loaders - not installable offline (no npm), nothing
  cached (measured): environment-blocked before design, exactly like cffi for Python.
- C. Hand-written bridge body - forbidden by D63's floor sentence on its face; listed to
  be killed, not debated.
- D. Child-process/pipe shape (node spawns modukit-host, speaks TL frames) - not the
  embed face at all (same edge-of-definition rejection Python's R4-C got); it is the
  script-runner territory in disguise.

Facts lean A, CONDITIONAL on the step-0 probe (bridge builds offline from cache + loads).
If the probe fails: the volume stands down honestly, the door stays armed - that outcome
is a valid hearing result, not a defeat.

### R3 - GATES: what makes the JS band falsifiable?

- A. **Mirror the established discipline + the ledger grammar amendment**: golden
  byte-compare for both emitted artifacts; structural pin tests (bridge forwarder list ==
  EMBED_VERBS; facade list == bridge list); a node:test-driven unit harness (measured
  present - the stdlib answer Python never had) pinning the facade against the real .node;
  demo leg tenth + eleventh (face-2 verb walk PY-style `JS-DEMO PASS`, sup rung with
  FIDELITY against the SAME pinned bytes the cpp and python stanzas assert, orphan census
  again); one sabotage lever per leg minimum; capability-ledger GAINS the JS column
  (four header rows :63/:98/:126/:157 - the checker is existence-only by design, so the
  column rides prose and cells, not gate code) - the map must not lie about a shipped band.
  (+) every piece has a shipped template; the JS column is where the ledger's whole point
  (planned -> generated as skins ship) stays honest.
  (-) four header rows plus every data row gains a cell to fill honestly per landing.
- B. Mirror without the ledger column (JS capability recorded only in prose).
  (-) knowingly keeps a lying map; contradicts the tighten-as-skins-ship rule the ledger
  states about itself.
Facts lean A.

### R4 - BUILD HOME: where does the Rust bridge live?

- A. **Workspace member `crates/modukit-js`** (the name is already pre-approved: it is the
  whitepaper's own planned-crate letter, docs/whitepaper.md:43; the naming authority line
  docs/stage-4-plan.md:87 runs against that register, and the letter is ON it).
  (+) single lockfile, pixi lanes, test/coverage machinery all apply to it unchanged; the
  crate is a real product artifact, not a shelf. (-) one more workspace member (the honest
  kind - unlike Python, this volume genuinely has Rust code to own).
- B. Standalone package under `bindings/js/` outside the workspace (mirrors the cpp shelf
  layout). (+) crates/ untouched. (-) a second build graph with its own lockfile drift,
  and the cpp precedent exists because C++ had no cargo lane to join - JS does.
- C. Fold into modukit-codegen. (-) domain violation: codegen emits faces, it is not a
  shipped consumer surface.
Facts lean A.

## 5. Walkthrough protocol

One item per message, plain language, house seven-part form; results into §7 the same
sitting; this hearing would mint D102 as a whole (numbering gate already spans D1-D999 in
pattern; the status.md line text widens per the standing probe-first ritual). A ruling
that grows beyond skins (e.g. anything touching the frozen tables) reopens rather than
smuggles.

## 6. Review trail

- Full grill (fresh session): REJECT with TWO blockers - the ledger checker was
  misdescribed (it is existence-only by design, lines 9-10, not a per-column parser) and
  "no project Cargo.toml references any of them" was false for libloading (a core
  dependency since Stage-1, Cargo.toml:19). Both fixed same-day: claims narrowed, the
  checker stays untouched by a JS column, libloading re-framed as in-tree availability
  proof. Increment recheck (fresh session): OKAY - fixes tree-verified, orphan sweep
  clean, one citation drift (:7-8 -> :9-10) aligned before this banner flip.
- Mid-walk facts worth keeping: Node 22 ships node:test/node:assert (measured); the
  cache's 8th napi grep hit is winapi-util (substring collision, real entries: 7).

## 7. Results table (filled at walkthrough)

| Item | Question | Verdict | What it decides |
|---|---|---|---|
| R1 | scope - face and surface | A | face 2 only, all 24 verbs, two milestones mirroring the cpp/python volumes; face 1 stays the script-host separate case (audesys:123-127, D54) |
| R2 | technique + step-0 probe gate | A | generated napi bridge (body dlopens libmodukit_core.so through the C door) + JS facade, BOTH emitted by the codegen pipeline; CONDITIONAL on the M0 day-one probe (offline build + node load) - probe failure stands the volume down honestly, door stays armed |
| R3 | gates incl the ledger JS column | A | six-gate mirror: dual goldens, dual pins, node:test harness pinning facade against real .node, legs ten + eleven with levers, byte-FIDELITY on the SAME constant as cpp+python (first tri-language byte agreement), ledger gains the JS column - four header rows + honest per-row cells, checker code untouched (existence-only by design) |
| R4 | bridge build home | A | workspace member crates/modukit-js - the name is already on the whitepaper register (:43), so this is settling a planned letter, not a new naming round; single lockfile + all cargo machinery inherited |

## 8. Lane pointer

Lanes (M-splits) get SPEC'D after the walkthrough and the plan document passes its own
review - nothing opens before the go order; same rail as the clock and python volumes.
