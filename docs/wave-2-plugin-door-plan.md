# Wave-2 Plugin-Door Volume Plan - language plugins via the wire

> Date: 2026-10-23. Contract basis: D106 (docs/wave-2-plugin-door-hearing.md,
> rulings C/A/A/A, maintainer-adjudicated one by one). Status: PENDING REVIEW.
> Thesis: Python and Node become plugin AUTHORING languages with ZERO kernel
> change and ZERO new dependencies - the exe door's runner hook is already wired
> (executor_proc.rs:221-251: manifest `runner` names the executable, artifact
> rides as argv, control fd via `--fd=N`; no runner = the stand-in-executable
> path, artifacts exec'd directly - M1 precedent).

## 0. Facts verified before writing

- The whole wire surface lives in ONE Rust module (proc_wire.rs): 11 message
  constants (HELLO=1 .. DELIVER=11) + 3 FWD sub-codes + 2 HELLO capability bits,
  FRAME_MAX 64 KiB, WIRE_VERSION=1 stamped by the writer, per-kind
  parse_/payload_ function pairs, pstr (u32-LE-length-prefixed string) convention.
- Payload codecs for CALL/RESULT are opaque bytes - a plugin in any language may
  pick its own service encoding (JSON included); the wire itself only frames.
- Child control fd rides as fd 0 (stdio swap is the design, executor_proc.rs
  header + modules/01 §4); `--fd=N` argv carries the number convention.
- Skin-relevant placements: the exe arm only; dl stays native (honest limit
  already documented, modules/03 §dl row).
- Offline inventory: python 3.8 stdlib (socket.fromfd, struct) and node 22
  (net.Socket({fd})) can both frame these bytes with zero deps.
- Leg grammar to mirror: the frame-parity fast leg copies the regen-diff shape
  (checked-in artifact, regenerate, byte-compare, loud FAIL); the two demo legs
  copy the wave-2 sup-leg shape (PASS marker + RUNNER line + sabotage lever +
  embedded orphan census).

## 1. Shape (diagram)

```
  BATCH 1 - THE LEDGER              BATCH 2 - PYTHON DOOR        BATCH 3 - NODE DOOR
  (contract before consumers)       (first speaker)              (second speaker)
 +--------------------------+     +-------------------------+   +------------------------+
 | contracts/modukit_wire.h |     | bindings/py/            |   | bindings/js/           |
 |  frame vocabulary ledger |     |   plugin_runtime.py     |   |   plugin_runtime.mjs   |
 |  (11 kinds + FWD subs +  |     |   (framing, HELLO self- |   |   (net.Socket fd 0,    |
 |   2 cap bits + pstr +    |     |    report, register/    |   |    same lifecycle,    |
 |   TL header spec)        |     |    call/bus/clock sugar)|   |    ESM + no deps)     |
 | codegen bin `wire-golden`|     | test-plugins/py/        |   | test-plugins/js/       |
 |  exports golden frame    |     |  echo plugin (.py,      |   |  echo plugin (.mjs,     |
 |  vectors per kind        |     |  runner=python3) +      |   |  runner=node) +       |
 | check.sh NEW FAST LEG:   |     |  liar fixture           |   | demo-wave2-plugin-js.sh |
 |  wire-parity: ledger<->  |     | demo-wave2-plugin-py.sh |   |  (conformance stage -> |
 |  Rust consts -> golden   |     |  (conformance stage ->  |   |   lifecycle -> orphan  |
 |  vectors byte-compare    |     |   lifecycle w/ fidelity |   |   census -> sabotage)  |
 |  (+--self-test hook)     |     |   constant -> orphan    |   | ledger plugin table    |
 | ledger leg extension:    |     |   census -> sabotage)   |   |  gains JS rows         |
 |  frame-rows coverage     |     | RUNNER line + pixi-free |   | RUNNER line ...        |
 +--------------------------+     +-------------------------+   +------------------------+
 Everything runs offline. egress/Windows debts of PARENT volumes untouched.
 DECLINED (armed, with reasons): embedded-interpreter skins (dl-position
  fast-lane; blocked by Python.h + engine crates), plugin-skin codegen (ledger
  not yet hammered by real users), res:// cross-bundle (D30 trigger), CLI day.
```

## 2. Batch 1 - the ledger (contract before any consumer)

- **T1.1 `contracts/modukit_wire.h`** - frame vocabulary ORIGINAL in C-header
  grammar (the house contract style): TL header layout (len u32-LE? - NO: copy
  the shipped truth: header = [u8 len? or u16?] EXACT bytes as write_frame
  emits - the implementer MUST read write_frame/read_frame and transcribe, then
  pin with golden vectors), WIRE_VERSION, all 11 MSG_ constants, 3 FWD subs,
  2 HELLO cap bits, pstr convention, per-kind payload grammar lines
  (field:width:endianness), FRAME_MAX. Header is documentation + machine table;
  nothing compiles from it (stated in the header comment, like fdsets are
  decoder inputs not code).
- **T1.2 `wire-golden` bin** (crates/modukit-codegen, second bin): for every
  message kind, encode one canonical example payload via proc_wire's own
  payload_ functions and dump `contracts/golden/wire-v1/<kind>.frame` raw
  bytes + an index .txt (kind, constant value, payload description). Uses
  proc_wire ONLY (no parser duplicates - drift would poison the vectors).
- **T1.3 check.sh new fast leg `wire-parity`**: re-run the bin into a temp dir,
  byte-compare against checked-in goldens (regen-diff grammar, drift = FAIL
  naming the file), AND cross-check `contracts/modukit_wire.h` numeric values
  against proc_wire.rs `pub const` values (embedded-parity grammar; the ledger
  lying is as fatal as the goldens drifting); ships a `--self-test` mode
  (plants a bumped constant in a scratch copy; must be caught).
- **T1.4 capability-ledger extension**: a new "W. Plugin wire v1" table (11
  kinds + subs + caps as rows); the ledger leg's coverage rule gains the W
  section (its --self-test updated to plant a missing W row too).
- **Acceptance:** full check.sh green; goldens present for all 11 kinds;
  leg observed RED once via --self-test AND once via a hand-bumped constant in
  the scratch copy (never in the repo), restored green.
- Commits (each C3-shaped + tree-verified): `feat(contracts): wire v1 ledger +
  golden vectors` / `feat(scripts): wire-parity leg with self-test` /
  `feat(ledger): plugin-wire W section + coverage rows` + docs(memory) close.

## 3. Batch 2 - the python door

- **T2.1 `bindings/py/plugin_runtime.py`** (hand-written, stdlib-only):
  frame read/write over a socket made from the inherited fd
  (socket.fromfd + duplicate so fd 0 stays intact), HELLO self-report
  (parse_hello_caps mirror - abi u32 + 3 pstrs + caps u32), ACK wait, then
  a dispatcher loop: MSG_START -> user start hook; MSG_REGISTER push -> answer
  with local callable registry; MSG_CALL -> decode (pstr method + bytes), call,
  RESULT with seq echo; STOP -> bye; unknown kinds -> ignore (wire rule) +
  count. User-facing API ~6 functions:
  `serve(plugin_id, version, name, services={...}, on_start=..., on_call=...)`
  and a `log()` passthrough to stderr for debug; DELIVER/DUE rides in a later
  milestone only if legs need it - batch 2 scope = HELLO/ACK/START/STOP/
  REGISTER/CALL/RESULT + DIAL refused-cleanly path.
- **T2.2 fixtures**: `test-plugins/py/py-echo/` (manifest entry kind=exe,
  runner="python3", artifact echo_plugin.py - a service "echo" returning input
  bytes verbatim + one service "caps" returning its HELLO caps; installs as a
  bundle with [[files]] listing the script) and `py-liar` (declares a service
  but answers CALL with garbage - negative path for RESULT error mapping).
  Scripts must run on python 3.8 (no walrus in the runtime, no 3.9+-only APIs;
  state it in the header).
- **T2.3 `scripts/demo-wave2-plugin-py.sh`** (the leg): [0] RUNNER line per
  T3.1 census grammar; [1] conformance stage: encode/decode each of the 11
  golden vectors via the SKIN's own framing (skin reads
  contracts/golden/wire-v1/, byte-roundtrip; drift = RED before anything
  launches); [2] lifecycle stage: a python driver (embed skin, rung-0) installs
  the py-echo bundle as exe placement, start, service_call "echo" with the
  cross-volume fidelity constant (same "sup-fidelity-payload" bytes), expect
  verbatim bytes back; call "caps"; stop; PASS line; [3] orphan census stage:
  SIGKILL the driver mid-flight, poll until zero py processes survive (EOF on
  fd 0 is the runner's self-kill signal - same physics the C++ sup leg proved);
  [4] sabotage: `DEMO_PLUGIN_PY_SABOTAGE=1` flips the skin's ACK step (documented
  named point) - leg must go RED with no PASS line; observed once, in the log.
- **T2.4 wiring**: check.sh eleven-leg zone gains this as leg twelve (RUNNER
  census applies); the "ELEVEN demo legs" prose synced to TWELVE in
  AGENTS/status/README same commit (three-site rule; the sweep row in the
  close note).
- **Acceptance:** leg green + observed RED via sabotage + ledger untouched
  (zero verb growth - the door rides v1 tables); full gate green.
- Commits: `feat(bindings): python plugin runtime skin` / `feat(plugins): py-echo
  + py-liar exe fixtures` / `feat(scripts): plugin-door python leg w/ orphan census`
  / `docs(memory): prose twelve-leg sync + batch 2 close`.

## 4. Batch 3 - the node door

- **T3.1 `bindings/js/plugin_runtime.mjs`** - same lifecycle as the python
  skin, node 22 stdlib only (net.Socket({fd:0}) wrap - read-only usage; Buffer
  framing; no deps, no npm); same golden-vector conformance stage; dispatcher
  as an async loop with an `onCall(method, inputBytes): Promise<Buffer>` map.
- **T3.2 fixtures**: `test-plugins/js/js-echo/` (runner="node", artifact
  echo_plugin.mjs; same two services; byte-exact echo - no toString() -
  fidelity constant must survive as Buffer).
- **T3.3 `scripts/demo-wave2-plugin-js.sh`** - the four-stage mirror of T2.3
  (RUNNER line, conformance, lifecycle w/ fidelity constant, orphan census,
  sabotage lever `DEMO_PLUGIN_JS_SABOTAGE=1`). Leg becomes thirteen.
- **T3.4 ledger W-section**: no new rows expected (same wire); capability
  ledger gains the two runner fixtures as evidence cells if its rows ask for
  plugin-side implementors (implement-time call, one-line note either way).
- **Acceptance:** full gate green, prose synced to THIRTEEN legs (same
  three-site sweep), both doors keep the shared fidelity constant byte-identical
  across cpp/py/js - now on BOTH sides of the wall.
- Commits: mirror batch 2's four.

## 5. Verification ledger (red before trusted, every row)

| Check | Planted defect (scratch or lever, never repo) | Expected |
|---|---|---|
| wire-parity golden compare | regenerate after touching a payload fn | drift named, RED |
| wire-parity ledger cross-check | bump one ledger number in scratch | RED |
| wire-parity --self-test | its own built-in plant | catches, prints OK |
| ledger W coverage | drop a W row in scratch ledger | RED |
| py leg conformance stage | corrupt one golden vector copy in scratch | RED before launch |
| py leg sabotage lever | env flag | no PASS line, exit 1 |
| py leg orphan census | SIGKILL driver | zero stray python |
| js trio | same three plants | same three verdicts |

Full check.sh at every batch end; C3 + tree-verify per commit (house law now).

## 6. Non-goals (stated so nobody re-litigates)

- dl-position script plugins (embedded interpreters) - armed, named blockers.
- Plugin-skin codegen/emitters - ledger first needs a real-world hammer.
- DELIVER/DUE inside script skins - deferred until a bus roundtrip leg wants it
  (batch 2 may add the DELIVER handler if the lifecycle demo proves cheap).
- res://, CLI day, compression, wasm - each keeps its own trigger (D30/D97-B).
- Windows: nothing in this volume runs on Windows; legs skip (POSIX sockets).

## 7. Deviation + debt ledger (inherits style)

- Skins are hand-written: conformance to golden vectors is the leash; if the
  ledger grows past 2 amendments, revisit the emitter (armed).
- pstr/caps grammar duplicated in 3 languages (Rust truth + 2 skins) - the
  golden-vector leg is the single anti-drift lock; note kept in wire ledger.
- python 3.8 floor (system interpreter) - skins must not use 3.9+ syntax; the
  conformance stage will run under 3.8 in CI-less boxes, so the floor is honest.
- Node-only ESM (.mjs) - no CommonJS dual-path until a consumer asks.

## 8. Review trail

- [ ] Momus review of this plan (fresh session, prompt = file path only)
- [ ] Maintainer approval word
