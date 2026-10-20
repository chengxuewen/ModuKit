# Stage-4 — INFRA volume: bindings, communication, debugging (plan of record)

> STATUS: ADJUDICATED 2026-10-19 - seven rulings taken one by one (user-adjudicated),
> scope narrowed to the C/C++ line. Milestones M1/M2 below are READY, lanes open on a
> separate go order. Directive of record: infrastructure first (whitepaper §1 CORE
> promises); extension tracks stay exactly where their trigger gates put them. The §12
> amendment (draft in §7) goes live at acceptance per the stage-3 precedent;
> transcript in §8.
>
> CREDITOR OF RECORD (ruled I7=A): whitepaper §1 core definition ("polyglot bindings,
> script hosts" were always core, not extensions - this volume repays an opening debt)
> + maintainer directive with the three-scenario matrix, recorded verbatim:
>   S1 full-C++ stack: C++ launches the main process, hosts the kernel, drives lifecycle
>      ("cpp is C with wrappers" - maintainer wording);
>   S2 full-Rust stack: as shipped today;
>   S3 mixed development: host AND plugins each freely C/C++/Rust/Python - therefore
>      every public API must expose equal capability in C/C++/Rust/Py alike.
> record: infrastructure first - the whitepaper §1 CORE promises (polyglot bindings,
> script hosts) and the unfinished communication/debug floors are the stage; extension
> tracks (remote WebRTC §12-4, UI §12-3, static placement §12-6) stay exactly where
> their trigger gates put them. > Walkthrough transcript: see §4; acceptance writes the §12 amendment per stage-3 precedent.

## 0. Constraints carried into this volume (non-negotiable unless a ruling here says otherwise)

- **D87 v1 STABLE**: no existing verb, frame, dictionary name or manifest grammar changes.
  Everything in this volume must be additive: new doors, new crates, tail-adds per D83.
- **D82 ritual**: v2 plugin-facing verbs are admitted as DIRECTION only - drawing the
  signatures IS this volume's hearing (agenda = I3), and the hearing lands as its own record.
- **K2/K3/K7 gates stand**: UI demoted; Zenoh trigger-gated; MediaServo note remains a
  maintainer send, proposal ARMED unopened.
- **C0 discipline**: every ruling gets a runnable check; every new gate leg is planted red
  once before trusted.
- **Numbering**: ledger at D92 / C3 / PIT-15 - minting D93+ widens the status.md gate first.

## 1. Call sheet - seven items, each adjudicated one by one

### I1 - Scope and sequence
The three infra lines: BINDINGS (codegen -> a second language), COMMUNICATION (v2 plugin-facing
bus/clock verbs + cross-process topics), DEBUGGING (tooling floor on the shipped data plane).
- **A. Serial: bindings -> communication -> debugging** (recommended pre-stance)
- B. Parallel trio (three lanes, three review rounds, one integration mess)
- C. Communication first (v2 is the heaviest shape decision; unblock the hearing early)
- D. Single line only: bindings now, the rest re-scoped after it ships
Evidence: codegen pipeline already proves the generate-not-handwrite doctrine for one
language (cheapest proven path); bus host-side verbs exist (`bus.rs:186/260`) but the
plugin-facing doors are undrawn (D82); the debug DATA plane is complete (introspect trio,
step_replay, ModuleLoaded) - only the TOOLING layer is missing, and its real users are
created by the first two lines.

### I2 - First binding language / script host
- A. **Python** (`modukit-py` line): cheapest broad-audience consumer of the C ABI;
  forces the codegen emitter to prove "generated, never hand-written" (whitepaper doctrine 2)
- B. **Plain C / C++ header set**: zero new runtime deps, but the ABI already IS C - a
  header drop adds little that the fdset ledger does not already pin
- C. **Lua script host** (`modukit-script-lua`): embedding, not binding - a different
  product shape (in-process scripting), bigger kernel surface than the doctrine allows (wp §63)
- D. C# / JS: heavier toolchains, no beachhead demand named
Decision needed from the maintainer regardless of letter: WHO is the first binding consumer?

### I3 - v2 hearing authorization (draws the signatures)
- A. **Authorize the hearing now, agenda = plugin-facing publish/subscribe + cross-process
  topic wire format; schedule-timer deferred to a second agenda** (recommended: the two bus
  verbs are one mechanism, the timer is another)
- B. Full D82 trio in one hearing (publish/subscribe/schedule-timer + topic wire)
- C. Defer the hearing until the bindings line ships (I1-A ordering takes this with it)
Standing on the books since D82: IOU-5 (cross-process bus clause) + friction 8 (no
plugin-facing bus verbs anywhere in ABI v1) - this is the volume's heaviest door, and the
hearing record is where it gets drawn.

### I4 - ABI/wire versioning mechanics for v2 doors
- A. **Versioned door: plugin HELLO negotiates supported ABI major; v1 table stays
  byte-identical, v2 is a separate table+version** (recommended: keeps D87 literal -
  nothing v1 changes; old plugins never see the new door)
- B. Tail-extend HostAbiV1 (D83 tail-add applies to fields; a TABLE swap is not a field
  tail-add - this letter reads closer to a break than the policy intends)
- C. WIRE_VERSION bump of the TL frame (wrong layer: the wire speaks frames, the ABI
  speaks tables - keep them independent, they were separated on purpose)

### I5 - New crate timing
- A. **No crate yet: bindings ship as codegen output + a new examples band; the crate
  name is cut when there is something to version** (recommended pre-stance: five empty
  names in the whitepaper are a map, not a build order)
- B. Register `modukit-py` (or the I2 winner) as a workspace crate on day one
- C. Separate out-of-workspace artifact repo (packaging freedom, pin discipline cost - C2 adjacent)
Whatever the letter, naming runs against whitepaper §2.3 authority before registration.

### I6 - Debug tooling floor
- A. **Reader-only CLI: one bin that consumes the existing artifacts (LedgerDump file,
  step_replay transcript, event feed) - no attach, no ptrace, zero new kernel doors**
  (recommended: the data plane already ships everything; the missing piece is a face)
- B. Reader + live attach (unix socket to the observatory ring - a new door to guard, size 3x)
- C. Defer: let the first binding-language consumer produce the friction report, then build
  the exact tool they ask for (the D75 doctrine: the first non-kernel consumer pays the round)
- I6 is where the maintainer's "debu" most probably lands - the three letters are real range.

### I7 - Creditor registration + roadmap amendment wording
- A. **Creditor of record: whitepaper §1's own core definition + maintainer signature
  (this directive)** - platform-promise debt, not a customer demand; honest form, named
  parties (recommended: the ledger must say who paid for this stage, and this is the truth)
- B. Wait for the first external binding consumer before calling it debt (then it would
  not be this volume, it would be a normal feature lane)
- Amendment text for §12 at acceptance (draft wording here, ruled text at closure):
  Stage-4 becomes this INFRA volume; remote WebRTC renumbers to a later stage number,
  content unchanged, engine-tiering question stays queued inside it; §12-6 candidate untouched.

## 2. Out of scope for this volume (restated, trigger-gated as ruled)

UI composition tier (K2) · Zenoh (K3) · static placement (D92 armed) · remote WebRTC
mechanics (§12-4) · MediaServo wiring (K7 armed) · anything needing a v1 break (D83 hearing, none scheduled).

## 3. Procedure

Walk I1..I7 one by one (adjudication-walkthrough protocol). Closure = summary table +
write-back here + D-entry minting (gate widening first) exactly as the Stage-3 kickoff did.

## 4. Rulings (adjudicated one by one, 2026-10-19)

| # | Ruling | Verdict | One-line key |
|---|---|---|---|
| I2 | first binding language | resolved by I1 | C/C++ is the first consumer (the I1 narrowing answered it); Python bindings/script host = wave-2 per §6 |
| I1 | scope & sequence | A + narrowed | serial order; THIS volume = the C/C++ line; communication (v2 hearing I3/I4) and debugging (I6) move to wave-2 candidates |
| C++-1 | scope boundary | B | plugin SDK AND host embedding; message layer NOT self-built (protoc C++ output used as-is) |
| C++-2 | embed API home | A | NEW table `HostEmbedV1` with its own version discipline; the two frozen v1 tables untouched (D87) |
| C++-3 | contract original | A | the C header IS the contract, checked into contracts/; parity gate leg modeled on the fdset gate; C++ wrappers generated; plugin side = a new codegen emitter |
| C++-4 | ladder coverage | C | two milestones: M1 = same-process rung; M2 = Supervisor verb group tail-added to the same table (D83) |
| C++-5 | artifact homes | A | everything lives in existing shelves (contracts/, codegen emitter, examples/cpp band, test-plugins/, cmake leg); no new crate; whitepaper §2.3 untouched |
| I7 | creditor + re-route | A | creditor dual-named (see header); §12 amendment drafted now, effective at acceptance |

Analysis adoptions recorded with the rulings (accepted, not separately contested):
- **trunk-first doctrine**: every new public API lands in C-header-expressible shape first; rich Rust forms stay syntactic sugar outside the header table.
- **capability ledger gate**: a checked-in matrix artifact + a check.sh leg. PRECISE: one row = one header verb (artifact path fixed at the M1 brief; M1 scope = modukit_embed.h verbs); RED iff a header verb lacks a row; unshipped languages ride a `planned` column, never a red condition (green from day one, tightens as skins ship); the row updates in the SAME change that adds the verb (planted red once before trusted, C0).
- **Python footnote**: Python SCRIPT plugins are a script-host project (modukit-script-*), separate ruling later; Python BINDINGS are ordinary generated skins of the C tables.

## 5. Milestones (lanes open on a separate go order; briefs presented before work)

### M1 - same-process rung (C++ plugin SDK + host embedding) - LANDED 2026-10-20
ALL GREEN (orchestrator-verified per lane): probe byte-equal; embed.h+plugin.h originals with three-way parity; emitters both faces; fixtures+shelf+live probe (descriptor arms); HostEmbedV1 Rust impl + host_demo full word list; ledger + gates. ctest 8/8, 219 tests, three demo legs, check.sh green. Honest gaps recorded: acquire-by-id, bus size-probe, global-kernel semantics, staticlib addition.

- PROBE FIRST (gates the rest of M1): compile-and-link one protoc-generated C++ message under the pinned pixi/cmake toolchain against the checked-in fdset contract - proves the C++-1 "protoc output used as-is" assumption before any emitter work builds on it
- contracts/: `modukit_embed.h` (HostEmbedV1 original) + regen/parity gate leg (fdset precedent, red-planted)
- codegen: C++ emitter for the PLUGIN side (descriptors/services/messages) AND generated embed-side C++ wrappers for HostEmbedV1 (C++-3's two halves; reuses cabi/emit/wire models; messages ride protoc C++)
- examples/cpp: one host embedding the kernel + one C++ plugin, walking the full v1 word list - the demo gate asserts BOTH legs (embedding face + plugin face, per C++-4's two-proof wording); falsifiable, sabotage hook documented
- test-plugins: one C++ dl fixture (+ one bad-ABI C++ fixture for the parity tests)
- cmake: C++ compile leg + ctest bridges
- capability ledger: initial matrix shipped WITH the M1 surfaces; gate leg live after a planted-red observation (C0)
- wiring: `scripts/demo-stage4-m1.sh` (PASS-string grep) becomes a scripts/check.sh leg, red-planted before trusted (C0)
### M2 - child-process rung (D83 tail-add to HostEmbedV1) - LANDED 2026-10-20
(seven sup verbs over a second ambient Supervisor; two-rung wall asserted both
 faces; host_sup_demo walks sup_install->ACTIVE->step->dial-refusal then gets
 SIGKILLed and the runner self-reap census proves ORPHAN-PROOF under a C++ host;
 DEMO_STAGE4_M2_SABOTAGE red observed. M2a built inline after two delegation
 deaths; three cross-language landmines surfaced and fixed by it: RTLD_NODELETE
 (probe-drop+reload re-ran C++ statics -> protobuf pool SIGABRT), wire method
 naming (emitter said Echo, wire said echo - to_snake single-sourced), and the
 runner self-check guard (byte-equality echo was the rs twin’s law in disguise:
 empty probe + pass-on-Ok; fidelity proven parent-side by the proxy round-trip).)
- 
- Supervisor verb group mirrored into the embed table (step/drive_until_idle/dial/restart/watch census)
- fd-ownership and signal discipline carried per S2 lessons (friction 7 class); ORPHAN-PROOF-style demo gate from the C++ host
- small design note for the verb semantics (D83 record, no break hearing)
- wiring: `scripts/demo-stage4-m2.sh` (ORPHAN-PROOF-style from the C++ host) becomes a scripts/check.sh leg, red-planted before trusted (C0)

## 6. Wave-2 candidates (in this volume's shadow, NOT open; trigger = maintainer word or named consumer)
- Python bindings (skins of the C tables) and the Python script host (separate rulings)
- Communication: v2 plugin-facing bus/clock verbs (I3/I4 hearing material preserved in §1)
- Debugging: reader CLI / tooling floor (I6 material preserved in §1)

## 7. §12 amendment DRAFT (effective at acceptance, not before)
Stage-4 becomes the INFRA volume (this document). Post-amendment §12 reads EXACTLY:
items 1-3 unchanged; 4 = INFRA (this volume); 5 = remote WebRTC (renumbered, content
unchanged, engine-tiering question stays queued inside it); item 6 = static placement
candidate (D92) untouched. The old Stage-5 (polyglot ecosystem) dissolves into wave-2 of
the infra track (§6). At acceptance the same amendment syncs README's Roadmap table and
rewrites stale §12-N citations in this document's header.

## 8. Walkthrough transcript (one at a time, plain language)
I1 (A+narrow, cpp chosen over the Python pre-stance) -> C++-1 (B; orchestrator's
"violates the doctrine" stance corrected on the record: embedding mirrors existing core
verbs, does not fatten the core - the honest cost is maintenance surface) -> C++-2 (A;
"do not print your phone number on someone else's badge") -> C++-3 (A; reuse the living
golden-contract regime) -> three-scenario matrix presented by the maintainer mid-walk
(S1/S2/S3), trunk-first + capability ledger adopted -> C++-4 (C; full stack, risk split
across two milestones) -> C++-5 (A; "nothing needs a household register today") ->
I7 (A; dual-named creditor, draft-now/enforce-at-acceptance). Seven of seven ruled, none
deferred.
