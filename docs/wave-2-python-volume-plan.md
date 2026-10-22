# Wave-2 Python Volume Plan (D100 rail)

STATUS: REVIEWED OKAY 2026-10-22 (trail in §8). Nothing opens before the go order.
Rulings of record: D100 (docs/wave-2-python-hearing.md, drawn 2026-10-22, A,A,A,A).
Deliverable: a generated Python floor over the frozen embed face - 24 verbs, two
milestones, five-gate parity with the C++ volume, stdlib instruments, one-line cdylib.

## 1. Goal and success

A Python 3.8 program on this box, with ZERO installed packages, can: embed the kernel
through the same C door as C++ (all 16 rung-0 verbs), and supervise an exe-placed plugin
host through the 8 sup verbs, byte-for-byte identically to the C++ demos - with every
claim falsifiable by planted red. Success lines:

- `bash scripts/demo-wave2-py.sh` -> `PY-DEMO PASS` (16-verb walk, mirror of the cpp M1 demo)
- `bash scripts/demo-wave2-py-sup.sh` -> `PY-SUP-DEMO PASS` + `FIDELITY PASS` + ORPHAN-PROOF
- both sabotage levers observed RED, then GREEN on restore (C0)
- goldens byte-compare green over the .py corpus; structural pin tests green
- capability-ledger Python column: 24 rows flipped planned -> generated
- `bash scripts/check.sh` all green with EIGHT demo legs

## 2. Fact base (imported from the hearing; anchors re-verified there)

- 24 verbs = 16 rung-0 (contracts/modukit_embed.h:128,139-235,15 in the vtable
  :246-282) + 8 sup (:308-358); all exported by the kernel (crates/modukit-core/src/
  embed.rs 16x / embed_sup.rs 8x `extern "C"`, verified by nm on a fresh staticlib).
- Ambient kernel, first call initializes (crates/modukit-core/src/embed.rs:92-96) - no
  init symbol to invent; the Python demo will behave exactly like examples/cpp/host_demo.cc:91.
- Header-text parsers are language-neutral and already exist
  (crates/modukit-codegen/src/embed.rs:177-316; pluginface.rs:201-288); the generate()
  write-block is the extension seam (crates/modukit-codegen/src/lib.rs:54-93).
- Crate-type truth: core ships rlib+staticlib only (crates/modukit-core/Cargo.toml:14);
  the five workspace cdylibs are plugin fixtures exporting the plugin-side entry only.
- Instruments: system python 3.8.10; ctypes + struct stdlib OK; NO pytest (not installable
  offline); system protobuf-python 3.6.1 exists but the pinned protoc is conda 7.x -
  runtime/gencode compatibility is UNKNOWN until measured (M1 step 0 probe, below).
- Parity gate (scripts/check-embed-parity.sh) pins header <-> Rust mirror only - python
  adds no burden there; its own truths ride pins + goldens (cpp_emitter.rs / golden.rs).

## 3. Milestones

### M0 - link floor (the one-line door)
- crates/modukit-core/Cargo.toml:14 gains "cdylib" (precedent: b89f7b9 CREATED that
  crate-type line with staticlib for the cpp band).
- Acceptance: `pixi run cargo build -p modukit-core` produces libmodukit_core.so; a
  one-liner probe loads it with ctypes and asserts `modukit_embed_abi_version() == 1`.
  The .so lands under the existing target dir; no cmake lane needed (wave-2 demo
  scripts are the cmake-free precedent, scripts/demo-wave2-clock.sh).
- Commit: `feat(core): cdylib lane for python consumers - D100 M0`.

### M1 - the 16-verb skin + its four gates
- Step 0 (measured, not assumed): generate `<demo>.pb2.py` with the pinned protoc,
  import it under system python 3.6.1-runtime. GREEN -> real runtime in demo payloads.
  RED -> hand-rolled minimal fixture codec in the DEMO ONLY (precedent: the cpp demo's
  own hand-parsed codecs, examples/cpp/host_demo.cc:184-218); payload bytes pinned to
  the M1 cpp constant (host_demo.cc:39 kPayload = "embed-demo-payload") - the sup
  FIDELITY constant is the M2 stanza's own anchor, not this one. This branch does not touch the
  floor; it is scaffolding, and the choice ships with its evidence either way.
- Emitter: new module crates/modukit-codegen/src/py.rs - reuses the same header parses;
  emits bindings/py/modukit_embed.py: enum mirrors (values verbatim from the five C enums at embed.h:36,:74,
  :86,:93,:102),
  one ctypes prototype per rung-0 verb (argtypes/restypes from the signatures), the
  i32-channel error dictionary narrowed via the codes already mirrored
  (embed_abi.rs / kernel codes), a thin vtable STRUCTURE mirror for the guards + the
  free-probe. Floor-only: semantics-free, no sugar beyond naming (D63 boundary).
- Wiring: the .py face emits at the main.rs door beside --embed_header (exactly where
  the cpp wrapper face is emitted - golden.rs:34-37 says so); generate()'s fdset corpus
  is NOT the seam. regen-golden.sh cleanup glob gains *.py (scripts/regen-golden.sh:27);
  golden.rs's explicit list stays untouched - the .py face is byte-pinned in
  py_emitter.rs, the cpp_emitter.rs pattern.
- Pins: crates/modukit-codegen/tests/py_emitter.rs - emitted verb list set-equal
  EMBED_VERBS (the cpp_emitter.rs:183 pattern), enum value equality against the header,
  golden byte-compare for the .py face.
- Demo: examples/py/host_demo.py mirroring host_demo.cc scenes [1]-[11] one-for-one
  through ctypes (11 labelled scenes cover all 16 rung-0 verbs - some scenes merge
  verbs, e.g. resolve+start+state) (install/resolve/start/service_call/bus three-ladder with late-join
  replay/clock/introspect both opaque codecs parsed/stop/uninstall) -> exact
  `PY-DEMO PASS`; lever DEMO_PY_SABOTAGE=1 flips ONE named scene (documented, C0).
- Leg: scripts/demo-wave2-py.sh (auto-skip if no python/.so; PASS-line grammar) enters
  check.sh as the EIGHTH demo leg; README gallery line; capability-ledger 16 rows flip.
- Commit split: `feat(codegen): python embed-face emitter + pins` then
  `feat(examples): python 16-verb demo leg - D100 M1`.

### M2 - sup rung (8 verbs) + fidelity + orphan-proof
- Emitter gains the 8 sup prototypes (same parse sweep, `^int32_t modukit_` regex is
  already the 24-verb source of truth in scripts/check-embed-parity.sh:47).
- Demo: examples/py/host_sup_demo.py mirrors examples/cpp/host_sup_demo.cc: sup install
  -> dial -> READY walk; FIDELITY stanza asserts the SAME expected bytes as the cpp
  stanza (single source constant shared by fixture comment); SIGKILL parent -> /proc
  census (stdlib os walk) finds zero stray runners -> `PY-SUP-DEMO PASS` +
  `FIDELITY PASS`. Levers: DEMO_PY_M2_SABOTAGE=1 (census-blind) + fidelity byte-corrupt
  arm, both observed red.
- Leg: scripts/demo-wave2-py-sup.sh as the NINTH demo leg; ledger's remaining 8 rows
  flip; second golden regen + byte-compare green.
- Commit: `feat(examples): python sup rung + fidelity - D100 M2`.

### M3 - closing sweep
- Ledger/AGENTS/status/architecture rows to "python volume shipped"; stage-5-plan gets
  the PYTHON lane row with evidence links (T1/T2 rows are the template); seven-leg ->
  nine-leg wording sweep in the same places (the D98 six->seven sweep is the pattern).
- Full gate re-run + plan §7 execution notes appended at landing (dangling-pointer rule
  from wave-2-t1 §7 landed the right way - write notes the same commit that cites them).
- Commit: `docs(memory): python volume closed + nine-leg gate sweep`.

## 4. Files touched (inventory)

New: crates/modukit-codegen/src/py.rs; crates/modukit-codegen/tests/py_emitter.rs;
bindings/py/ (generated skin - NOT hand-written, regenerated by the pipeline);
examples/py/host_demo.py; examples/py/host_sup_demo.py; scripts/demo-wave2-py.sh;
scripts/demo-wave2-py-sup.sh; golden .py corpus files.
Edited: crates/modukit-core/Cargo.toml (one line); crates/modukit-codegen/src/main.rs +
lib.rs (wiring); scripts/regen-golden.sh (glob); scripts/check.sh (two legs);
docs/capability-ledger.md (24 row flips); README.md (gallery rows); test payload constant
sharing comment in the cpp fidelity stanza. NOTHING under contracts/, NOTHING under
crates/modukit-core/src/ except zero (v1 frozen - D87; this volume consumes, not amends).

## 5. Risks and named limits

- Protobuf gencode/runtime skew (3.6.1 vs conda 7.x): decided by the M1 step-0 probe,
  both branches pre-drawn above - no mid-flight improvisation.
- ctypes struct layout: the floor mirrors only what the header declares; the demo parses
  event/snapshot bytes via struct.unpack (LE discipline already proven in the cpp demo).
- Python 3.8 ceiling: emitter output must avoid 3.9+ syntax; pin test runs under the
  system interpreter; shortcut: 3.8 floor, upgrade when a newer system python lands.
- No pytest: gates run stdlib unittest / bare scripts; the sabotage levers are shell.
- GIL/threads: irrelevant here - all demos are single-threaded control-plane walks.

## 6. Out of scope (D100 closures)

Python-SCRIPT plugins (own hearing, stage-4-plan.md:134). PyO3 floor (superseded delta,
recorded). pybind11 (recorded not adopted; re-entry = measured overhead). asyncio sugar
(D6). pytest/cffi (offline env fact). Any touch to v1/frozen surfaces (D87).

## 7. Execution notes

(empty - filled at landing, same-commit-cited rule)

## 8. Review trail

- Full grill (fresh session): REJECT - three factual blockers (enum count 6->5;
  host_demo scene labels [1]-[16] -> [1]-[11] with merge note; M3 commit subject
  62 chars -> the repo's own commit-msg hook bound 60) + four nits (emission seam
  is the main.rs --embed_header door per golden.rs:34-37, M1 payload constant is
  host_demo.cc:39 kPayload not the sup stanza's, b89f7b9 CREATED the crate-type
  line, golden.rs list-vs-glob note). Everything structural verified tree-true
  (24=16+8, seven legs, protobuf 3.6.1 vs conda 7.36.2 probes re-run, D100
  faithfulness, D87 untouched).
- Increment recheck (fresh session): OKAY - all five fixes re-verified against
  the tree; zero drift on the five enum lines; distinct wrapper verb calls = 16;
  subject now 56 <= 60; no edit corruption in M0/M1/M3.
- All fixes landed before this banner flip; implementer inherits a plan whose
  every number was read from the tree, not from memory.
