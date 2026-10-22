# Wave-2 Python Bindings Hearing (D100)

STATUS: DRAWN 2026-10-22 - all four items walked one by one, rulings A,A,A,A (user-
adjudicated). Minted as D100. Volume plan follows on the standard rail.
Volume: Python bindings (the "skins of the C tables" door armed since the C++ volume closed).
Trigger: maintainer word 2026-10-22 opening the Python volume - exactly the "opening on maintainer
word (same adjudication rail)" that the bindings ruling's re-review trigger named.

## 0. How to read this

Four items decide this volume. Each item is one question, walked one at a time, in plain
language. The letters (A/B/C) are the options; the star marks what the facts point at, not
what is decided - nothing here is a decision until the maintainer rules it. Every number in
this file carries a file:line you can check right now.

## 1. Why this volume, in one paragraph

The project's pitch is "any language plugs in through the C boundary". Since the C++ volume
closed, exactly one language has proven that on the host-embedding side. The mixed-development
of that volume - "host AND plugins each freely C/C++/Rust/Python - therefore every public
API must expose equal capability in C/C++/Rust/Py alike" (docs/stage-4-plan.md:17-18) - is
today still a promise kept only halfway. The Python door was explicitly armed as the next
skins candidate (docs/stage-4-plan.md:122, :187; docs/stage-5-plan.md:131-133), and the
cheapest-consumer argument was on the table before C++ won the slot: "cheapest
broad-audience consumer of the C ABI; forces the codegen emitter to prove 'generated, never
hand-written'" (docs/stage-4-plan.md:53-54).

## 2. Fact base (every claim anchored)

Contracts (frozen, this volume touches nothing here):
- 24 embed verbs = 16 rung-0 + 8 sup: declarations contracts/modukit_embed.h:128-358; the
  vtable struct modukit_host_embed_v1 (contracts/modukit_embed.h:246-282) carries 17 members
  = 2 handshake guards + 15 verb slots; embed_abi_version stays a free-function probe
  (contracts/modukit_embed.h:128); the 8 sup verbs are free functions only (:308-358).
- Mirror truth: EMBED_VERBS, 24 entries (crates/modukit-core/src/embed_abi.rs:25-52).
- Plugin-side tables (for reference; not consumed by this volume): PluginAbiV1 7 fields
  (contracts/modukit_plugin.h:130-152), HostAbiV1 3 (:103-112), HostAbiV2 7 (:194-209),
  HostAbiV3 9 (:247-266); Rust mirrors abi.rs:68/:100/:225/:320.
- All 24 verbs are real exported C symbols today: 16 in crates/modukit-core/src/embed.rs,
  8 in crates/modukit-core/src/embed_sup.rs (extern "C" counts verified by `nm` over a
  freshly rebuilt staticlib - stale target/ artifacts predate the doors - 24 symbols).

Kernel shape (decides the link story):
- The embed face binds ONE process-global kernel, built lazily on first use
  (`static KERNEL: OnceLock<Mutex<Ambient>>`, crates/modukit-core/src/embed.rs:92-96;
  "kernel_new/kernel_free are NOT verbs", crates/modukit-codegen/src/embed.rs:19-22). The
  C++ demo just declares the ambient kernel (examples/cpp/host_demo.cc:91) - a ctypes client
  gets identical semantics: first call initializes, no init symbol to invent.
- modukit-core builds `rlib + staticlib` only (crates/modukit-core/Cargo.toml:14). No
  shipped crate under crates/ builds a cdylib; the workspace's cdylib targets are the five
  test-plugins fixtures (each Cargo.toml:10), which export the plugin-side entry symbol -
  not the embed face. ctypes cannot dlopen a
  staticlib - this is the volume's only genuinely new build piece.

Toolchain reality (offline box, verified 2026-10-22):
- System python 3.8.10; `ctypes` import OK (stdlib); `cffi` NOT installed; `pytest` NOT
  installed; no outbound network, so pip cannot add either. The pixi default env ships no
  python binary at all (`pixi run python` unresolvable). Python-side gates must therefore be
  stdlib-only unless a milestone explicitly opens the env problem.

Codegen reuse (the skins machinery already exists):
- Pipeline: protoc-gen-modukit reads fdsets + the two frozen C headers
  (crates/modukit-codegen/src/main.rs:50-63); `generate()` writes .rs/.abi.rs/.h/.hpp per
  contract (crates/modukit-codegen/src/lib.rs:54-93 - the seam where a .py writer hangs).
- The two face-B wrapper generators parse the CHECKED-IN HEADERS as text, with no C++ inside
  the parsing: parse_enums/parse_verbs (crates/modukit-codegen/src/embed.rs:177-316),
  slot/struct parsers (crates/modukit-codegen/src/pluginface.rs:201-288). A python wrapper
  emitter reuses those parses wholesale; only the emission strings are new (~one module).
- Gate precedents to copy, not reinvent: golden byte-compare
  (crates/modukit-codegen/tests/golden.rs) + structural pins against the ledger consts
  (crates/modukit-codegen/tests/cpp_emitter.rs - "verb count moved" assert) + demo legs with
  sabotage hooks (scripts/demo-stage4-m1.sh / -m2.sh; cmake-free precedent:
  scripts/demo-wave2-clock.sh) + check.sh leg grammar (scripts/check.sh:92-127).
- The parity gate pins C-header <-> Rust-mirror only (scripts/check-embed-parity.sh) - a
  python band adds ZERO burden there; its own truth-tests ride the pin/golden pattern.

Doctrine anchors (verbatim, they bind the technique item):
- "C ABI is the single cross-language exit: every external export goes through a C ABI;
  polyglot bindings are generated, never hand-written per language." (docs/whitepaper.md:64)
- Generated floor: "the C ABI and every language's base binding are OUTPUTS of the same
  codegen toolchain. Hand-writing a REPLACEMENT of this floor is forbidden."
  (.agents/memorys/decisions.md D63)
- Direction precedent: for JS, the sanctioned use is BINDING (external language calling our
  C ABI), never EMBEDDING the foreign engine into the host (D54, napi-rs ruling) - the
  reading for Python: consuming the C tables from python is the door; embedding CPython into
  the kernel is the script-host separate case.
- KNOWN TENSION, must be ruled not swallowed: the whitepaper's own tech-stack row names
  "Python bindings | PyO3" (docs/whitepaper.md:144) while its principle row says C-ABI-only
  exit (:64). PyO3 wraps the RUST ABI directly - it would bypass the single door and become
  a hand-maintained parallel surface. This hearing is where the delta gets adjudicated and
  recorded (whitepaper stays verbatim per the standing correction policy; the delta lands in
  the decision ledger / architecture §9 style).
- Planned name of record: `modukit-py # Python bindings` (docs/whitepaper.md:41; planned-crate
  block README.md:60) - naming runs against whitepaper §2.3 authority (docs/stage-4-plan.md:87).

Prior rows on this exact door:
- "Python bindings (skins of the C tables) and the Python script host (separate rulings)"
  (docs/stage-4-plan.md:187; footnote :134).
- "Python bindings or v2 verbs opening on maintainer word (same adjudication rail)" - the
  re-review trigger this hearing answers (decisions.md D93 trigger row).
- Capability ledger: every verb row carries `planned` in the Python column; "`planned`
  Python: wave-2 binding candidate" (docs/capability-ledger.md:49; token semantics :33;
  "tightens as skins ship" :14).

## 3. Not decided here (named, so nobody re-litigates mid-volume)

- Python SCRIPT plugins (a python-authored .so-style plugin, or CPython embedded in the
  host): the script-host separate case (docs/stage-4-plan.md:134). Out of this volume.
- Per-language async sugar (Python asyncio surfaces over the same message shape): parked
  under D6's runner-side clause; a skin decision, not a floor decision. Out.
- Pyodide / wasm-tier python: D33 three-nos (browser engines neither provided nor tested nor
  supported). Out.
- cffi as a floor: not installable on this box offline; the environment fact decides it
  before taste does (§2 toolchain reality). Out unless the env item reopens.
- Touching any frozen surface (contracts/*.h, abi tables, v1 verbs): nothing in this volume
  has reason to; the tables are consumed as-is.

## 4. The four items

### R1 - SCOPE: how much of the 24-verb surface does this volume skin?

- A. **Full embed face, both rungs** - all 24 verbs (16 rung-0 + 8 sup), staged as two
  milestones like the C++ volume did (M1 = 16-verb walk + demo PASS leg; M2 = sup rung +
  byte-fidelity mirror + orphan-proof analogue driven from python).
  (+) literally satisfies the S3 equal-capability line (stage-4-plan:17-18); a python host
  can drive exe-placed plugins, which is half the product story.
  (-) roughly double the demo/gate work of rung-0 alone.
- B. **Rung-0 only (16 verbs)**; sup rung deferred to its own later round.
  (+) smaller first cut; (+) still proves "generated skins" on 16 of 24 rows.
  (-) leaves a python host unable to place exe plugins - a knowingly unequal surface; the
  S3 line says equal capability, and deferral without a named consumer drifts.
- C. A + an extra "consumer scene" demo (bus + clock walk from python as a showcase).
  (+) demo value; (-) the same walk already exercises bus/clock verbs; scene is polish,
  arguably out of scope discipline.

Recommended by facts: A (the S3 parity line is the strongest prior commitment on file).

### R2 - TECHNIQUE: what does the generated floor sit on?

- A. **Generated ctypes floor, stdlib-only**: a new emitter module turns the frozen C
  headers into `<...>_embed.py` (ctypes prototypes + Structure mirrors of HostEmbedV1,
  enum mirrors, Status-style result narrowing per the house pattern), generated by the same
  codegen doors, pinned like the cpp emitter. Targets system python 3.8+.
  (+) zero new installable deps on an offline box; (+) consumes the exact C door (doctrine-
  literal); (+) the header text-parsers already exist and are language-neutral (§2).
  (-) ctypes ergonomics are clunky; sugar above the floor is then a per-language choice,
  which is explicitly allowed (D63 sugar clause) but out of this volume.
- B. cffi - rejected by environment before design: not installed, no pip offline (§3).
- C. PyO3 (whitepaper:144's letter) - the doctrine question: it wraps the Rust ABI,
  bypassing the single C door, and lands as a hand-written parallel surface (forbidden floor
  per D63's letter). Choosing C means a fresh doctrine break, not a skin.
Recommended by facts: A, plus an explicit recorded delta ruling "PyO3 row superseded for the
bindings floor" so the whitepaper tension is closed, not ignored.

### R3 - GATES: what makes this volume falsifiable?

- A. **Full mirror of cpp discipline, stdlib-only instruments**:
  (1) regen goldens gain .py artifacts, byte-compared, regen-diff leg extended;
  (2) structural pin test: emitted python field/verb lists equal EMBED_VERBS/*_ABI_FIELDS
  consts (the cpp_emitter.rs pattern);
  (3) demo leg: scripts/demo-wave2-py.sh walking the scope set with PY-DEMO PASS + one
  sabotage lever + auto-skip only on missing toolchain - inserted as check.sh's eighth leg;
  (4) python self-checks run via stdlib unittest or bare scripts (no pytest, env fact §2);
  (5) capability-ledger flips `planned` -> `generated` on the rows this volume ships, and
  the ledger check already counts rows - wording updated, rows flip honestly.
  (+) same proof standard the box already runs; every lever has a direct precedent file.
  (-) goldens for python add a corpus to maintain.
- B. Lite: pins + demo only, no goldens. (-) diverges from the established golden discipline
  the C++ volume set; "cheaper" here means a weaker falsification surface.
Recommended by facts: A.

### R4 - LINK PIECE: how does ctypes get the 24 symbols?

- A. **Add "cdylib" to modukit-core's crate-type** (crates/modukit-core/Cargo.toml:14, one
  line; the staticlib line was added exactly the same way for the cpp band at Stage-4 M1):
  python loads the produced libmodukit_core.so; the ambient-kernel semantics are identical
  to the C++ demo's (embed.rs:92-96 - first call initializes).
  (+) smallest possible new build piece; (+) direct precedent (staticlib bump); (-) every
  consumer build now also produces an extra artifact (minor).
- B. Separate shim crate (e.g. a tiny cdylib re-export) - (+) core untouched; (-) a NEW
  crate name needs the whitepaper §2.3 naming-authority round (stage-4-plan:87), extra
  workspace member, and re-exports nothing that core cannot export directly - pure overhead.
- C. No .so at all: python talks to a spawned modukit-host via pipes - that is NOT the
  embed face (different product shape); out on definition.
Recommended by facts: A.

## 5. Walkthrough protocol

One item per message, plain language, each in the house seven-part form; ruling letters
record into §7 the same sitting. D100 gets minted for this hearing as a whole (numbering
gate widens D1-D100 with the standing probe-first discipline). If any item's ruling demands
a surface change beyond skins, the item reopens rather than smuggles.

## 6. Review trail

- Full grill (fresh session): REJECT with ONE blocker - the fact-base sentence "no
  cdylib anywhere in the workspace" was false as written (the five test-plugins fixture
  crates are workspace cdylib members); narrowed to shipped crates, conclusion kept.
- Increment recheck (fresh session): OKAY - fixed sentence re-verified workspace-wide
  (examples member swept too), nit fixes carry no new false claim, no edit corruption.
- Mid-walk maintainer query: pybind11 considered and RECORDED NOT ADOPTED (no offline
  source; idiomatic use = hand-written glue against the generated-floor ruling; a
  generated C++ glue lane would double the pipeline for zero capability). Re-entry
  condition: measured ctypes-overhead numbers on a hot path.

## 7. Results table (filled at walkthrough)

| Item | Question | Verdict | What it decides |
|---|---|---|---|
| R1 | scope of the skinned surface | A | all 24 verbs (16 rung-0 + 8 sup), two milestones mirroring the cpp volume; no silent 8-row debt (S3 equal-capability line) |
| R2 | generated floor's technique + PyO3 delta | A | generated ctypes over the frozen header; PyO3 whitepaper row superseded - recorded as delta; pybind11 not adopted (see §6) |
| R3 | gate shape | A | five-gate parity with the cpp volume (goldens, structural pins, eighth demo leg, sabotage lever, ledger flips); instruments stdlib-only - offline box has no pytest |
| R4 | link/build piece | A | one crate-type line adds cdylib to modukit-core (staticlib bump b89f7b9 is the precedent); no shim crate, no naming round |

## 8. Lane pointer

Lanes (the T-splits of this volume) are SPEC'D after the walkthrough and the plan document
passes its own review - nothing opens before the go order, same rail as the clock volume.
