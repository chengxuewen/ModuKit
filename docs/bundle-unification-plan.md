# Bundle-unification plan - the one-word terminology refactor (repo-wide)

> STATUS: PLAN OF RECORD, 2026-11-25 (revision 1, adjudications ARMED). Origin:
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

## 3. Adjudication points (ARMED - walk one by one before W1)

| # | Point | Lean |
|---|---|---|
| A1 | ABI break style: clean break (repo unpushed - zero external consumers) vs dual-symbol deprecation period (executor accepts modukit_bundle_init preferred, modukit_plugin_init fallback) | clean break; D87 amendment via the ruling entry |
| A2 | Structural rename scope: full (dirs/crates/targets/legs) vs prose-only | full (the maintainer's whole-repo instruction) |
| A3 | History surfaces: decisions D-records stay as-written + one new ruling entry appended; plans-of-record get header rev-notes, bodies untouched | append-only |
| A4 | Build output directory: `<binary>/bundles/<name>/` (morning ruling folded in, renamed) | bundles/ |
| A5 | Identity copy: new README tagline framing (bundle-and-service vs module-system wording) | bundle-and-service |
| A6 | Reintroduction guard: a plugin-word scan with an allowlist (decisions, plan bodies, history) folded into an EXISTING gate vs review-only | folded scan |

## 4. Waves (ordered; each wave lands green with atomic commits)

| Wave | Delivers | Key gates |
|---|---|---|
| W0 | This plan + the ruling entry (decisions D111) + vocabulary table in AGENTS.md; the terminology target becomes canon | i18n, doc-refs |
| W1 | Contracts rename: contracts/modukit_plugin.h -> modukit_bundle.h (types + entry symbol + MODUKIT_ENTRY_SYMBOL macro, per A1); embed.h's modukit_plugin_state -> modukit_bundle_state; codegen regen (the generated faces follow the header - single source, D63); D51/wire name audit (expect zero plugin tokens; any hit is a finding) | regen-fdset, embed-parity, golden |
| W2 | Rust internals: PluginId/PluginState/PluginRecord/PluginAbiV1/PluginView and friends -> Bundle* (1067 hits, mechanical); kernel dm_report field, ops output column | cargo fmt/clippy/test, workspace |
| W3 | Fixtures + examples: 13 modukit_bundle_init exports + include updates + local type names (449 hits); PASS lines unchanged | m1/m2/stage legs, wave2 legs |
| W4 | Structural: test-plugins/ -> test-bundles/ (dir), crate names (example-bundle...), cmake targets + FOLDERs, leg names demo-wave2-bundle-*, script paths + router table + ctest names + census/lane accounting; PolyOrch submodule word-audit (edits ride its branch, C2); build outputs -> bundles/<name>/ (A4) | lane-table, census, check.sh full |
| W5 | Docs volume: 77 files - module 01-bundle-placement-model.md (file rename + toc + links), tutorials, README (tagline per A5), AGENTS, llms.txt, capability-ledger (plugin-face sections -> bundle face), features, gap matrix | doc-refs, book-build, gate-docs |
| W6 | Reintroduction scan (A6) + full-leg rehearsal + closure note; the word `plugin` survives ONLY in the allowlist (decisions history, plan bodies, .refinfo) | check.sh all green; the scan |

Ordering rationale: W1 precedes W3 because fixtures compile against the new
header; W4 is last-but-one because structural renames re-touch scripts that
W3 verified; W5 lands after code so docs quote final names.

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

- 2026-11-25 revision 1: drawn from the terminology panels + the maintainer's
  one-word ruling (bundle); adjudication points A1-A6 ARMED for the walk.
