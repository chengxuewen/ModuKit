# Stage-3 Plan — Multi-Process UI Composition (Demoted Track, Scoping Draft)

> Status: DRAFT for maintainer adjudication + Momus review. Plan of record once accepted.
> Scope anchor: `docs/architecture.md` §12 item 3 ("multi-process UI composition:
> extension/research track (demoted; revisit with the beachhead's real UI demands)")
> + whitepaper §4/§6 (compositor as extension capability, never core).
> Execution gate: nothing executes until the maintainer says go (standing rule).

## 0. The honesty problem this plan must survive

§12 demoted Stage-3 to an extension/research track **conditional on real UI
demand** ("revisit with the beachhead's real UI demands"). No beachhead product
with UI demands has arrived. What HAS arrived: (a) Stage-2's stable v1 surface,
(b) D84's coverage-debt calendar that fires at the Stage-3 kickoff gate, (c) a
pile of trigger-gated UI-adjacent hooks in the ledger (D15 dmabuf ticket, D44
device-ACL day, media-base D68-D73, Foxglove D61).

Therefore this plan is intentionally structured as **Tier 0 + optional tiers**:
Tier 0 is demanded by existing adjudications regardless of UI demand; the UI
tiers stay gated on demand that has not arrived. The maintainer's first
adjudication (K0) is choosing which tiers this stage actually contains.

## 1. Call sheet — kickoff adjudications

| # | Question | Recommendation |
|---|---|---|
| K1 | D84 calendar: the kickoff gate measures core/codegen against 80. Does the measurement happen BEFORE any Stage-3 milestone starts (blocking per the hook), or in parallel with Tier-0 work? | Measure FIRST, before M3S-1 — the hook is explicit: miss = features blocked until cleared; Tier-0 test-debt work IS the clearing work, so gate-then-build is the only honest order |
| K2 | Does Stage-3 include ANY UI/compositor tier now, or is the whole stage a "stability + debt + tooling" stage with UI deferred to a demand-triggered Stage-3.5? | Deferred (the §12 condition still not met); keep a named re-open trigger: first beachhead UI demand OR D15's GPU lane trigger |
| K3 | Zenoh cross-host carrier (D11's remaining open slot): in or out? | Out (trigger unchanged: first cross-machine topic link); Stage-3 stays single-host |
| K4 | v2 bus/clock verbs (D82): draw signatures this stage or keep PLANNED? | Keep PLANNED unless a Tier-0 deliverable forces shapes (none should); drawing day needs its own hearing per D82 |
| K5 | Does the stage produce a public "v1 stable" artifact set (docs site, binding-stage groundwork, example gallery)? | Minimal: a docs pass + the example gallery from the two demo shelves; no binding work (Stage 5 owns it) |
| K6 | PolyOrch freshness policy stays tip-following? | Yes, unchanged; routine bumps |
| K7 | D73's wiring-hearing trigger ("kernel API stabilization day") plausibly FIRED with D87 (v1 STABLE, 2026-10-18): does the freeze constitute the trigger - open the MediaServo wiring-hearing proposal, or record the reasoned no? (Momus finding 1; §6's "none fire today" is otherwise false) | Adjudicate honestly one by one at kickoff; do NOT decide by omission |

### Verdicts (2026-10-18, user-adjudicated one by one)

- **K1 = A**: gate-then-build, full serial; D84 honored to the letter on first due date.
- **K2 = A**: UI tier OUT; demotion fence verbatim; trigger named (first beachhead UI demand).
- **K3 = A**: Zenoh stays gated (first cross-machine link); ready template argues against pre-pulling.
- **K4 = A**: v2 signatures stay PLANNED; drawing day is its own hearing (D82).
- **K5 = A**: minimal truth-set (README as-built + card notes + gallery); no site, no hand-copied reference.
- **K6 = A**: PolyOrch freshness unchanged (tip-following, routine bumps).
- **K7 = C**: the D73 trigger - v1-STABLE FACT communicated to the MediaServo maintainer, NO wiring-hearing proposal yet; proposal trigger stays ARMED (open item: maintainer sends the factual note - external comms).

## 2. Milestone outline

| # | Milestone | Deliverable | Anchors | Gate / evidence |
|---|---|---|---|---|
| M3S-0 | Kickoff | K1-K6 walked one by one; D-entry per verdict; numbering gate widen; pin freshness check; push decision OPEN at kickoff | §0-1, D84 | every K-item recorded before work opens |
| M3S-G | **D84 kickoff gate (fires here)** | MEASURED 2026-10-18 (filed, not acted on - orchestrator gate-then-build per K1): `pixi run -e cov cargo llvm-cov --workspace --summary-only` -> TOTAL 71.70% (matches the M5a 71.76 within run-to-run jitter). Per-crate split from the full text run ABORTED by a test failure (see observation below) - the two adjudicated debt crates re-quoted from the M5a table until a clean split run: core 78.42 (need >=80, gap 1.6), codegen 73.19 (need >=80, gap 6.8). VERDICT PER D84: BOTH DEBT CRATES MISS -> the anti-second-delay hook is ARMED (new features blocked until cleared); gate NOT cleared. OBSERVATION (not acted on): the full-text cov run tripped a known-family flake (process_executor.rs:251, Text file busy on standin copy - ETXTBSY class, same family as the 2026-10-18 hardened builds; appeared under llvm-cov instrumentation load). | D84, D50 | coverage table 71.70 total; two debt crates below 80; hook armed; observation logged, untouched. DEBT LANE (2026-10-18, two agent runs, orchestrator re-verified): codegen 73.19 -> 85.24 (33 behavior tests); core 78.52 -> 85.52 (25 error-contract tests pinning the D85/D27/D86 contracts; error.rs+bundle_error.rs at 100). BOTH DEBT CRATES NOW CLEAR 80 - hook ready to lift on maintainer word. Residues: PIT-10 cov signature OPEN (2 instrumented sightings, none since the unique-dir fix); demos RED pre-existing (D85 remap vs stale asserts) with the check.sh gate-coverage gap noted |
| M3S-1 | Stability + tooling lanes | (a) recorder/replay re-evaluation IF real traffic now exists (D78 triggers check — honest yes/no with evidence); (b) `rules/common` dangling-refs sweep (Stage-2 tooling now exists); (c) dangling `~/.claude` refs inline-or-trim; (d) Friction-log items 1-2 (throwaway-load, Manifest ctor) paid unless the lane evidence note shows the pain was one-off (the D75 round was real use) — recorded disposition either way | D78/D75/§9 | each lane: evidence note + landing or a recorded no-op with reason |
| M3S-2 | v1-stable documentation pass | README "Development status" un-staled (whitepaper §6 staleness pass separately); README roadmap boxes tick (Stage 1/2 done); module cards 01/07/06 gain the Stage-2 as-built notes (fd-0 stdio, TL frames, supervision domain); friction-log learnings folded into SDK-facing notes | C1, D87 | doc-audit clean run; C1 green; no canon contradiction; any canon line change lands with a delta row (stage-2 mechanics precedent) |
| M3S-3 | Example gallery | examples/ gains stage3 band? NO - gallery = curated README linking stage1/stage2 demos + their transcripts; plugins/ shelf stays reserved | rulings F+B | README gallery section; both demos runnable from clean checkout |
| M3S-4 | Optional UI tier (ONLY if K2 says in) | …reserved, populated by K2's verdict; pre-scoped sketch: PAL device lanes (D72 triggers), dmabuf ticket growth (D15), compositor crate skeleton | §12-3, D15/D72/D44 | would carry its own rows |
| M3S-5 | Stage-3 acceptance | DoD written at M3S-0 close from the tier set chosen; gates: all existing + any new; coverage re-measured (post-debt); push + pin checks | D88 precedent | demo-style falsifiable gate if any new surface ships |

## 3. Sequence

M3S-0 → M3S-G (gate first, blocking) → M3S-1/2/3 (parallel lanes after the gate)
→ M3S-4 (only if K2) → M3S-5. No UI code before the gate clears — that is the
point of the hook.

## 4. What exists at entry / what does NOT

Exists: v1 frozen surface (D87), dual placement + ORPHAN-PROOF, vendor-shm lane,
host loop, wire dial, typed deferrals, 134 default + 19 vendor tests, both demo
gates. NOT (this stage): compositor code, Wayland/X11 anything, dmabuf lanes,
v2 verb shapes, Zenoh carrier, bindings.

## 5. Risks and rollback

| # | Risk | Mitigation |
|---|---|---|
| 1 | Coverage gate misses and the stage stalls | the hook makes debt-work the FIRST work, not a blocker at the end; Tier-0 lanes are test-natural |
| 2 | Scope creep toward UI despite K2 | §12-3's own condition quoted at every milestone close; the demoted-track language is the fence |
| 3 | "Stability stage" becomes busywork | every lane is evidence-first (D78 trigger check, real friction re-verified) or it is dropped |
| 4 | Docs drift vs as-built | doc-audit run as M3S-2's gate, not a formality |

## 6. Non-goals (one breath)

Compositor implementation (unless K2 opens it), Wayland/X11 protocol code,
dmabuf lanes, v2 signatures, Zenoh, bindings, wasm — all have named triggers;
none fire today.

## 7. Definition of Done (draft, finalized at M3S-0)

1. D84 gate measured and cleared (or the hook visibly armed and debt paid).
2. Every Tier-0 lane closed with evidence or a recorded no-op.
3. Docs as-built pass + gallery, doc-audit clean.
4. All memories synced; numbering gate widened per verdicts; pin fresh; push
   decision explicit.
