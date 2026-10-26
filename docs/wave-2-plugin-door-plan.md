# Wave-2 Plugin-Door Volume Plan (v2) - language plugins via the wire

> Date: 2026-10-23. Contract basis: D106 (rulings C,A,A,A) + post-hearing amendment
> round (maintainer-adjudicated one by one: A1 = install-door probe exemption for
> runner-bearing entries, kernel 3 lines + tests; A2 = full bus/clock roundtrip
> per the hearing's acceptance sentence - no silent scope reduction). v1 of this
> plan was team-reviewed (4 lanes, 45 raw -> 2 blockers + 17 corrections, all
> folded here; delta re-review #2 (v2) REJECTED on 2 must-fix - both verified by the
> author and fixed below as v3 (narrowed exemption + gitignore line rewrite), plus its
> F4-F9 minors. Status: FINAL DELTA RE-CHECK RUNNING.

## 0. Corrected fact base (v1 errors fixed at team review)

- pstr = **u8**-length-prefixed string (cap 255, HELLO_STR_MAX family); the
  u32-LE length belongs to the opaque BYTES fields (CALL input, RESULT/DUE/
  FORWARD payloads), not to strings. v1 §0 conflated them.
- Frame header, verbatim from write_frame (read_frame is its mirror):
  **[u32 LE payload-length][u8 WIRE_VERSION=1][u8 kind]**, 6 bytes, length
  covers the payload only. FRAME_MAX 64 KiB is enforced READ-side; the writer
  does not cap -> the ledger declares it a two-way ceiling and skins MUST
  pre-check length before allocating (hosted by the conformance stage, §6).
- Control channel rides the child's **fd 0** (stdin swap; `--fd=0` argv
  convention). stdout AND stderr are nulled by the kernel - scripts get NO
  console; skin `log()` is therefore deleted from the API (diagnostics ride
  the RESULT channel or the reader face, not stdio).
- REGISTER is **pushed by the child** after START (self-report of services) -
  the kernel never sends it; v1's dispatcher description was inverted.
- START/STOP carry empty payloads; ACK carries one i32-LE status but has NO
  encoder pair (the runner hand-rolls `to_le_bytes`). Batch 0 adds the missing
  `ack_payload` twin so goldens cover every kind through proc_wire ONLY.
- `runner` must be a real existing file path (is_file check before spawn; bare
  "python3" is refused) and is deliberately NOT inside the bundle hash envelope
  (it names a HOST interpreter, not a bundle member). Fixtures template the
  absolute interpreter path when the demo builds the bundle.
- Trust posture (stated, not hidden): install-time authenticity = none (anyone
  who can place a bundle + call install runs code as the host user; the hash
  envelope = tamper-after-install integrity for members, not origin auth);
  post-install re-verification today covers >1MiB files only - the script-door
  amendment keeps member hashing at install time; load-time small-file rehash
  stays a named family-wide residual (not this volume's).
- `bindings/py/` is git-ignored WHOLESALE (generated-skin shelf): batch 2 adds
  a tracked carve-out for the hand-written plugin runtime, proven by
  git check-ignore before and after (plant-and-observe, C0).
- Cross-check at the bundle door is unconditional dlopen today (fn at
  bundle.rs:337; kind forced Dl at :347; probe :348-349; its self_report id/
  version comparison IS the install-time identity check). Batch 0 amends it
  NARROWLY: entries with a runner AND a non-".so" artifact skip the probe
  (envelope + grammar stay; identity DEFERS to the spawn-time HELLO cross-check
  at executor_proc.rs:300-303 - its own planted red joins §6); everything else
  - runner-free OR runner-bearing native (.so) bundles, the stage2/m2 leg shape
  - keeps the probe bit-for-bit. The .so suffix is shelf grammar: a script named
  .so gets probed and refused, the author renames it.

## 1. Shape (v2: four batches)

```
 BATCH 0 - THE DOOR AMENDMENT   BATCH 1 - THE LEDGER           BATCH 2 - PYTHON DOOR   BATCH 3 - NODE DOOR
 (kernel, tests first)          (contract before consumers)    (full-11-kind skin)     (mirror)
 +--------------------------+  +---------------------------+  +---------------------+  +--------------------+
 | cross_check: runner =>   |  | contracts/modukit_wire.h  |  | bindings/py/        |  | bindings/js/       |
 |   skip dlopen probe,     |  |  11 kinds + FWD subs +    |  |   plugin_runtime.py |  |  plugin_runtime.mjs|
 |   envelope intact        |  |  caps + TL header bytes + |  | gitignore carve-out |  |  ESM net.Socket    |
 | proc_wire: +ack_payload  |  |  EXTENSIBILITY column     |  | fixtures test-      |  |  js-echo + js-liar |
 | tests: runner-exempt      | |  (which kinds tolerate    |  | plugins/py/:        |  | leg thirteen;      |
 |  install OK; tampered     | |   tail fields / new kinds  |  |  py-echo, py-liar   |  |  ledger E rows     |
 |  script STILL refused;    | |  / unknown-kind posture)  |  | leg twelve:         |  |  gain js cells     |
 |  no-runner bundles        | | wire-golden bin (codegen  |  |  RUNNER -> con-     |  | 5-site sweep 13;   |
 |  byte-for-byte unchanged  | |  src/, counts in region,  |  |  formance(11 golds  |  | bat mirror NOT     |
 | red: flip the exemption    ||  tested via parity leg)   |  |  + hostile frames)  |  |  touched (debt)    |
 |  off, script install       || fast leg wire-parity:     |  | -> lifecycle        |  +--------------------+
 |  must fail loudly; restore ||  goldens byte-diff +      |  |  + FIDELITY         |
 +--------------------------+  |  ledger<->const cross +   |  |  + BUS ROUNDTRIP    |
 Ledger row: exemption lands    |  --self-test              |  |  + liar typed-error |
 in wire.h header comment       | ledger gains section E    |  |  + orphan /proc     |
 (legs 11->12->13 prose sync,   | (next letter; W rejected  |  |  + sabotage; SKIP   |
 5-site sweep incl check.sh     |  - sequence grammar)      |  |  policy stated      |
 header - see batch 2)          +---------------------------+  +---------------------+
```

## 2. Batch 0 - the door amendment (A1 ruling)

- **T0.1** bundle.rs cross_check: `if entry.runner.is_some() && !artifact
  ends ".so"` -> skip the dlopen probe (member hashing + grammar stay; identity
  defers to HELLO); else current behavior bit-for-bit. One branch, comment
  cites D106-amendment. Rationale (delta-review F1, self-verified): a broad
  `runner.is_some()` would silently retire install-time identity for the
  EXISTING runner+.so bundles - stage2_demo (a standing gate leg) installs
  exactly that shape through this door (examples/rs/stage2_demo.rs:109-114).
- **T0.2** proc_wire: add `ack_payload(status i32) -> Vec<u8>` (mirror of
  parse_ack), so goldens never hand-roll framing. Pure addition.
- **Tests:** script-shaped bundle (runner + .py) installs - envelope holds,
  tampered member still refused; lying HELLO for a script entry refused at spawn
  (typed); runner+.so bundle (stage2 shape) STILL probed at install (no
  regression); runner-free bundle untouched; ack round-trip.
- **Reds (C0):** plant: exemption flipped off -> script bundle refused loudly
  (observe, restore); plant: corrupt a [[files]] member post-staging -> install
  fails even WITH the exemption (the envelope is not the casualty).
- Commit: `feat(core): runner-bearing entries skip the dlopen probe`.

## 3. Batch 1 - the ledger (corrected facts, structure as v1)

- **T1.1** `contracts/modukit_wire.h`: 11 kinds with values (HELLO=1 START=2
  STOP=3 ACK=4 REGISTER=5 CALL=6 RESULT=7 DIAL=8 DUE=9 FORWARD=10 DELIVER=11),
  FWD_PUB/SUB/UNSUB=0/1/2, HELLO_CAP bits BUS_V2=1 TIMER_V2=2, header layout
  (§0 verbatim), pstr u8, FRAME_MAX two-way ceiling, RESULT status i32 (D27
  author codes), per-kind payload grammar lines, and the EXTENSIBILITY column
  (HELLO tolerates the caps tail ONLY; all other parses are strict
  trailing-bytes-reject; amendments ride NEW kinds - runner steady loop ignores
  unknown kinds, the START-exchange pump does NOT, documented per direction -
  or new cap bits). Nothing compiles from it (stated in-file).
- **T1.2** wire-golden bin in crates/modukit-codegen **src/** (deliberate: it
  counts inside the codegen coverage region - coverage-boundary.sh:45 - and MUST
  be exercised under instrumentation: a codegen integration test runs the bin
  via CARGO_BIN_EXE and asserts its emitted bytes (the shell parity leg alone
  produces no profraw - delta-review F5)). Emits exactly **14 frames**
  (HELLO_plain, HELLO_caps, START, STOP, ACK, REGISTER, CALL, RESULT, DIAL,
  DUE, FORWARD_PUB, FORWARD_SUB, FORWARD_UNSUB, DELIVER) + index .txt (per-frame
  hex, "implementation-derived regression lock" provenance line, regen command).
  The ledger E-table has 16 rows (11 kinds + 3 subs + 2 caps) - rows are not
  frames, stated to kill the F7 count ambiguity.
- **T1.3** check.sh fast leg `wire-parity`: regenerate into temp, byte-compare
  goldens; ledger numeric columns <-> proc_wire pub consts; --self-test plants
  (golden drift + bumped constant). check.sh header enumeration grows.
- **T1.4** capability ledger: new section **E. Plugin wire v1** - rows = 11
  kinds + 3 subs + 2 caps; evidence cells: Rust(impl) / python(batch 2) /
  node(batch 3) / conformance-leg. Its --self-test gains the missing-E-row plant.
- Commits: 3 (ledger+goldens / parity leg / E-table), C3 + tree-verify.

## 4. Batch 2 - the python door (full hearing scope, A2 ruling)

- **T2.1** bindings/py/plugin_runtime.py, stdlib-only, python 3.8 floor
  (header states it). Scope = ALL 11 kinds: frame loop pre-checks length vs
  FRAME_MAX before ANY alloc; HELLO caps self-report (id/version/name MUST
  equal the manifest - strict cross-check); START -> ack(0) -> push REGISTER
  per declared service; CALL -> on_call -> RESULT(seq,status,bytes); bus
  marshalling FORWARD(SUB/UNSUB/PUB) + DELIVER -> on_bus(topic,payload) +
  publish outbound; DIAL -> DUE(status=REFUSED_REAL) clean refusal (no timer
  claims this volume); STOP -> ack(0) -> exit; unknown kinds ignored + counted;
  EOF anywhere = cooperative exit (residuals named §7). No log() (stdio null).
  API: serve(id, version, name, services={...}, on_call=..., on_bus=...,
  topics=[...]) - fixed here (delta-review F8): non-empty services -> push
  REGISTER per service right after START; non-empty topics -> FORWARD(SUB) per
  topic; a bus-only plugin pushes SUBs and NO REGISTER; both skins identical.
- **T2.2** fixtures test-plugins/py/py-echo (echo + caps services + a bus-topic
  listener) and py-liar (declared service answers RESULT status=-1): THE LEG
  CONSUMES THE LIAR - that CALL must surface a typed error host-side while the
  plugin stays alive (proves the i32 channel is a contract, not decoration).
  Bundle built by the leg with the absolute runner path templated in.
- **T2.3** scripts/demo-wave2-plugin-py.sh stages: [0] RUNNER line; [1]
  conformance BEFORE launching anything: replay all 11 goldens both directions
  through the skin + hostile plants (length > FRAME_MAX refused pre-alloc;
  trailing bytes refused); [2] lifecycle: install -> start -> CALL echo with
  the shared fidelity constant (bytes verbatim) -> caps -> BUS ROUNDTRIP
  (driver publishes, plugin receives + re-publishes, driver receives) -> liar
  CALL expects typed error + plugin still answering -> STOP; [3] orphan census
  via /proc scan (NEVER pgrep -f - house law): SIGKILL driver, poll zero
  strays; [4] sabotage DEMO_PLUGIN_PY_SABOTAGE=1 at one named skin point ->
  leg red. Auto-SKIP without python3/toolchain/fixture (existing grammar).
- **T2.4** wiring: gitignore carve-out = REWRITE the directory line (delta-
  review F3, self-verified: dir-exclusion `bindings/py/` at .gitignore:128
  cannot be re-included by a negation - git never descends): `bindings/py/` ->
  `bindings/py/*` + `!bindings/py/plugin_runtime.py` (`__pycache__/` stays
  ignored); plant-and-observe proves BOTH states via git check-ignore; `git
  add -f` banned as an escape (it would hide the wrong pattern); check.sh leg
  block copied from the census template INCLUDING the PASS-without-RUNNER FAIL
  arm (v1's ambiguous "T3.1" label renamed D104-census - no id collision);
  5-SITE prose sweep at landing (README sentence + gallery rows, AGENTS two
  lines, status verification row, check.sh header - fifth site per house
  precedent, wave-2-js-volume-plan:100) + history-archive entry; status donor trimmed IN THE SAME
  commit (headroom pre-balanced, §7).
- Acceptance: leg green + sabotage red observed + carve-out proven both ways +
  full gate green. Commits: 2 (skin+fixtures / leg+wiring).

## 5. Batch 3 - the node door (mirror)

- **T3.1** bindings/js/plugin_runtime.mjs (node 22 stdlib; net.Socket({fd:0})
  FULL duplex - v1's "read-only use" wording was wrong and is fixed here;
  frames flow both directions on the one socket), async on_call/on_bus, same
  11-kind scope, same conformance + hostile-frame stage, same DIAL clean-
  refusal, ESM only (no CommonJS dual-path until asked).
- **T3.2** fixtures test-plugins/js/js-echo + js-liar (byte-exact Buffer echo -
  no toString() - fidelity constant survives as bytes; liar = status=-1 path).
- **T3.3** scripts/demo-wave2-plugin-js.sh = the five-stage mirror (RUNNER,
  conformance, lifecycle+FIDELITY+BUS ROUNDTRIP+liar, /proc orphan census,
  DEMO_PLUGIN_JS_SABOTAGE=1). Leg thirteen; 5-site sweep again; ledger E rows
  gain node cells; close notes land (history + plan §9 + stage-5 lane row +
  architecture §11 pointer if acceptance moves - mint stays at D107).
- Acceptance: full gate green; both doors keep the fidelity constant
  byte-identical across cpp/py/js on BOTH sides of the wall (embed legs already
  pin it host-side; these two pin it plugin-side).

## 6. Verification ledger (planted-red before trusted, every row)

| Row | Plant | Expect |
|---|---|---|
| exemption off (batch 0) | flip the T0.1 branch off | script bundle refused loudly; restore green |
| envelope holds | corrupt a [[files]] member, exemption ON | install still FAILS (hash) - exemption is narrow |
| no-regression (probe class) | stage2-shape runner+.so bundle via installer | probe still runs, identical verdict (NOT kernel-install suites - they bypass the door, F6) |
| HELLO lie (script entry) | runner+.py bundle self-reports wrong id | refused at spawn, typed error, plugin never starts |
| ack symmetry | mutate ack_payload byte | parity leg red |
| goldens drift | edit one checked-in vector | wire-parity red naming the file |
| ledger lie | bump a wire.h number in scratch | parity red; --self-test catches its own |
| E-table gap | drop one E row in scratch ledger | ledger leg red |
| skin framing | corrupt one scratch golden per skin | conformance red BEFORE any launch |
| hostile frame | len 0xFFFFFFFF / trailing bytes | refused pre-alloc + counted (expected pass of the plant) |
| fidelity | flip one byte in fixture echo | FIDELITY stanza red |
| liar | make liar hang instead of status=-1 | typed-error assertion red (proves the check checks) |
| sabotage duo | each leg env lever | no PASS line, exit 1 |
| carve-out | remove !-line, git add skin | skin untracked (red); restore -> green |

Full check.sh at every batch end; per-commit C3 (hook has caught its own author
6x - single-physical-line bullets only) + post-commit tree-verify.

## 7. Deviations + named residuals (explicit, nothing swallowed)

- A1 makes this volume "3-line kernel amendment", NOT "zero kernel change" -
  D106 carries the amendment; v1's claim archived as superseded here.
- Orphan census claims only well-behaved single-process skins; blocking user
  handlers / grandchildren / node open handles are the cooperative-exit
  residual (same family as the deferred cgroup/rlimit layer - not solved here).
- The runner binary is outside the hash envelope by nature (it names a HOST
  interpreter); authenticity stays a hosting-operator duty (wasm/sandbox tier
  un-started, whitepaper line untouched).
- Small-file post-install rehash remains a family-wide hole (native included);
  armed, not this volume's.
- Skin codegen/emitters stay armed until the wire ledger survives a real
  amendment (rule-of-two before abstraction).
- DIAL refusal means script plugins get NO clock timers this volume (the DUE
  frame is emitted only as the refusal); the claim path arms with the embed
  road or a follow-up round of its own hearing.
- status.md byte discipline: headroom is thin - every batch landing carries its
  own donor trim in the same commit, verification-table rows are the valve.
- Windows: this volume adds no .bat surface (legs are POSIX; the existing bat
  first-run debt unchanged).

## 8. Non-goals (v1 list, none crept in)

dl-position script plugins (embed road armed); res:// cross-bundle; CLI day
(modukit-cli); compression; wasm; C#; UI tier. Each keeps its own trigger.

## 9. Review trail

- [x] Team review of v1 2026-10-23 (4 lanes: facts/completeness/safety/ops;
  45 raw -> blockers + 17 corrections; blockers adjudicated A1/A2 same day).
- [x] Delta re-review of v2 (ultrabrain, fresh): REJECT - 2 must-fix (F1 broad
  exemption retires install-time identity for existing runner+.so gate legs;
  F3 dir-exclusion gitignore cannot be re-included) + 7 minors. Author
  self-verified both must-fixes against the tree (cross_check internals; git
  scratch-repo experiment 0/1) and folded all nine into this v3.
- [x] Momus plan review of v2: OKAY (citation pass; two non-blocking notes).
- [ ] FINAL delta re-check (v3, nine folded fixes only - budget 2/2).
- [ ] Momus plan review (house flow, fresh session, prompt = file path only).
- [ ] Maintainer approval word.
