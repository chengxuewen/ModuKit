# Bundle-unification plan - the one-word terminology refactor (repo-wide)

> STATUS: PLAN OF RECORD, 2026-11-25 (revision 3 - adjudications closed,
> execution schedule S0-S5 queued; S1 first). Origin:
> the maintainer ruled (2026-11-25, two expert panels, sunk costs explicitly
> excluded): the managed unit is named **bundle** - ONE word, everywhere the
> user looks. The prior vocabulary split (plugin = code unit, bundle = D28
> install unit) is retired: two near-synonyms for one referent was the
> confusion source. `plugin` leaves the user-facing vocabulary; `service`
> stays (distinct job: the capability unit); `module` stays quarantined to
> documentation deep-dives and language import units (never a bundle synonym).

## 0. The target vocabulary (the whole user-facing grammar)

| Concept | Word | Notes |
|---|---|---|
| the managed unit (manifest + artifact + lifecycle + services) | **bundle** | OSGi lineage; the word signals the managed model |
| the capability exchanged between units | **service** | unchanged |
| the code contract a bundle implements | `contracts/modukit_bundle.h` | was modukit_plugin.h |
| the entry symbol every bundle exports | `modukit_bundle_init` | was modukit_plugin_init; MODUKIT_ENTRY_MACRO updates |
| the descriptor / state / id types | `modukit_bundle_abi_v1`, `BundleState`, `BundleId` | type-family rename, values unchanged |
| script bundles (py/js children on the wire) | script bundle / bundle door | was "script plugin" / "plugin door" |
| the runtime skins | `bundle_runtime.py` / `bundle_runtime.mjs` | file renames + doc comments |
| the fixture tree | `test-bundles/` | was test-plugins/ |
| demo leg names | `demo-wave2-bundle-py/-js` | router table + census follow |
| per-unit topics | `org.modukit.timer.<bundle-id>`, `config/<bundle-id>` | DOC WORDING ONLY - the string carries the id VALUE; format unchanged |
| build output directory | `<binary>/bundles/<name>/` | the morning ruling folded in, renamed to the one word; D28 deploy tree stays `<bundles>/<id>/<version>/` - built bundles and installed bundles share the word |

Explicitly NOT renamed: `service`, manifest field names (id/version/entry/
requires/provides/config_live - already neutral), embed verb names (all 25 are
neutral), D51 dictionary names (verify in W1), wire frame names (verify in W1),
staging/objects/data paths, `dm` (DependencyMonitor), `bus`/`clock`, rung
terms, band names (C/C++/py/js "bands"), placement kinds (dl/exe).

## 1. Decision provenance

- Panel 1 (terminology census): plugin/bundle near-synonym pair is the
  confusion source; service is clean; module is quarantined.
- Panel 2 (two-tier plugin/bundle proposal, first-principles): capability is a
  lattice, not a total order - two unit KINDS cannot express per-feature
  opt-in; enforcement lives in context construction, not in names; precedent
  (OSGi/CppMS/celix/CTK) is unanimous one-unit-word.
- Maintainer ruling on the one word, sunk costs excluded: **bundle** - the
  word denotes the self-contained managed module (our architecture's core),
  not the host-socket role. Native OSGi-style precedent 2:1 (CppMS, celix
  chose bundle; CTK chose plugin).
- Consequent design law: the taught word must equal the written symbol - so
  the ABI face renames with the vocabulary.

## 2. Scale survey (2026-11-25, evidence)

| Area | hits | files |
|---|---|---|
| crates/ | 1067 | 90 |
| docs/ | 1018 | 77 |
| examples/ | 308 | 37 |
| scripts/ | 179 | 19 |
| test-plugins/ | 141 | 26 |
| contracts/ | 75 | 4 |
| README/AGENTS/book | 40 | 3 |
| fixtures exporting modukit_plugin_init | 13 | - |
| structural names | test-plugins/, docs/modules/01-plugin-placement-model.md, crate names (example-plugin, bad-plugin, busv2-plugin, timerv3-plugin, liar-plugin), cmake targets, leg names demo-wave2-plugin-* | |

## 3. Adjudication points (ALL RESOLVED 2026-11-25, walked one by one)

| # | Point | Lean |
|---|---|---|
| A1 | ABI break style: clean break vs dual-symbol deprecation period | **RESOLVED A** (2026-11-25): clean break - the repo is unpushed (zero external consumers, the only free window); a fallback would ADD mechanism to the D87-frozen loader surface, deeper than the rename itself; D87 amendment rides D111 |
| A2 | Structural rename scope: full (dirs/crates/targets/legs) vs prose-only | **RESOLVED A** (2026-11-25): full - the maintainer's whole-repo instruction; structural names are the most user-visible layer, residue = the split vocabulary reborn |
| A3 | History surfaces: rewrite vs append-only | **RESOLVED A + bridge notes** (2026-11-25): decisions/history-archive/plan bodies stay as-written (append-only, the value of history is original wording); each historical ENTRY POINT gains a terminology bridge note (mapping + reason + pointer: one global note atop decisions.md, one line per plan-of-record header, one in history-archive) |
| A4 | Build output directory | **RESOLVED A** (2026-11-25): `<binary>/bundles/<name>/` per-bundle directories (the morning per-unit-dir ruling, renamed) - built bundles and installed bundles share the word and the shape (D28 `<bundles>/<id>/<version>/` unchanged) |
| A5 | Identity copy: README tagline | **RESOLVED A** (2026-11-25): the model-signaling tagline - an OSGi-style bundle and service framework (bundle + service co-present in the first sentence; the next line's Rust kernel / C ABI / C/C++/Python/JS counters the Java smell) |
| A6 | Reintroduction guard | **RESOLVED A** (2026-11-25): a plugin-word scan with an explicit allowlist (decisions.md, history-archive, plan-of-record bodies, docs/reference/ external profiles) folded into gate-i18n - zero new legs; written in W1, ACTIVATED in W6 (the migration waves legitimately carry the word) |

## 4. Slices (ordered; each lands green with atomic commits - rev3: W1+W3 merged, S0 added)

| Slice | Delivers | Key gates |
|---|---|---|
| S0 | status.md offload (7568/8000 - memory budget before the volume) | i18n, numbering |
| S1 | THE ATOMIC ABI BREAK (A1 clean-break makes contracts+fixtures ONE slice - a split would leave a red compile window): contracts/modukit_plugin.h -> modukit_bundle.h (types, entry symbol, MODUKIT_ENTRY_SYMBOL); embed.h's plugin-named types; abi.rs layout mirror; codegen regen (all generated faces follow the header, D63); ALL 13 fixtures' entry symbols + includes + the examples' generated-type references; the A6 scan written but INACTIVE; D51/wire token audit (expect zero; any hit is a finding) | regen-fdset, embed-parity, golden, m1/m2/stage/wave2 legs, full check.sh |
| S2 | Rust internals: PluginId/PluginState/PluginRecord/PluginView and friends -> Bundle* (the remaining ~1067 hits, mechanical); kernel dm_report field, ops output column | cargo fmt/clippy/test, workspace |
| S3 | Structural: test-plugins/ -> test-bundles/ (dir), crate names (example-bundle...), cmake targets + FOLDERs, leg names demo-wave2-bundle-*, script paths + router table + ctest names + census/lane accounting; PolyOrch submodule word-audit (edits ride its branch, C2); build outputs -> bundles/<name>/ (A4) | lane-table, census, full gallery re-run, check.sh |
| S4 | Docs volume: 77 files - module 01-bundle-placement-model.md (file rename + toc + links), tutorials, README (the A5 tagline), AGENTS, llms.txt, capability-ledger (plugin-face sections -> bundle face), features, gap matrix; the A3 bridge notes land (decisions top, each plan-of-record header, history-archive) | doc-refs, book-build, gate-docs |
| S5 | Closure: ACTIVATE the A6 guard in gate-i18n; full-leg rehearsal (E7-style, warm-cache labeled); the word `plugin` survives ONLY in the allowlist | check.sh all green; the scan zero outside allowlist |

Ordering rationale: A1's clean break makes the ABI rename ATOMIC - contracts,
codegen, and the 13 fixtures' entry symbols land as ONE green slice (S1); S2
is pure Rust internals (no fixture coupling); S3 re-touches scripts after the
code vocabulary settled; S4 lands docs last so prose quotes final names; S5
closes with the guard activation. Every slice boundary is interruptible.

## 5. Verification ledger

Every wave: `bash scripts/check.sh` green (ok=17), plus its listed gates. W4
re-runs the FULL demo gallery (all 14 legs). W6 adds the rehearsal log as the
acceptance evidence (E7-style, warm-cache honestly labeled).

## 6. Risks

- Mid-migration mixed vocabulary (drift window): accepted, bounded by wave
  atomicity - never two waves in flight.
- Census/lane/parity breakage classes: W1 and W4 are the two gates-touching
  waves; both carry same-commit regen.
- The rename touches 260 files: every wave lands as a small set of atomic
  commits (never one mega-commit), each independently green and bisectable.
- History rewrite temptation: FORBIDDEN (append-only, A3).

## 7. Out of scope

Meaning-preserving refactors beyond the rename; any behavior change; the
two-tier context-narrowing feature (separately trigger-gated); donor clones
(.refinfo untouched); translations.

## 8. Change log

- 2026-11-25 revision 3: execution schedule concretized as slices S0-S5; the
  rev2 wave table's atomicity flaw fixed (W1+W3 merged into S1 - A1's clean
  break forbids a contracts-without-fixtures red window); S0 offload added
  (status.md at 7568/8000); the A6 guard is written in S1 and activated in S5.
- 2026-11-25 revision 2: A1-A6 walked one by one, ALL RESOLVED A (A3 amended
  with terminology bridge notes on every historical entry point). Clean-break
  ABI rename; full structural renames; append-only history + bridges;
  bundles/<name>/ build outputs; the OSGi-style bundle-and-service tagline;
  the guard folded into gate-i18n with a four-class allowlist, activated W6.
  Execution-ready: W1 (contracts + codegen) first.
- 2026-11-25 revision 1: drawn from the terminology panels + the maintainer's
  one-word ruling (bundle); adjudication points A1-A6 ARMED for the walk.
