# Bundle-unification plan - the one-word terminology refactor (repo-wide)

> STATUS: PLAN OF RECORD, 2026-11-25 (revision 4 - panel-reviewed: a second
> expert panel audited rev3 and its findings + four micro-rulings are folded
> in; execution-ready, S0 first). Origin:
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
| the entry symbol every bundle exports | `modukit_bundle_init` | was modukit_plugin_init; MODUKIT_ENTRY_SYMBOL updates (contracts/modukit_plugin.h:50) |
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
| docs/ | ~1018 | ~78 |
| .agents/ (status/conventions/pitfalls/backlog + 8 skill bodies + mirrors) | 227 | 14 |
| examples/ | 308 | 37 |
| scripts/ | 179 | 19 |
| test-plugins/ | 141 | 26 |
| bindings/ (probe + the two hand-written runtime skins) | 34 | 5 |
| contracts/ | 75 | 4 |
| root misc (CMakeLists, llms.txt, CONSTRAINTS, .gitignore, SKILL.md) | ~12 | 5 |
| README/AGENTS/book | 40 | 3 |
| fixtures exporting modukit_plugin_init | 13 (rs 5 + cpp 3 + c 1 + tutorial 4) | - |
| structural names | test-plugins/, examples/cpp/tutorial/t{1,2,3,7}-*plugin/ dirs, pluginface.rs filename, docs/modules/01-plugin-placement-model.md, tutorials/02-first-plugin.md + 06-script-plugins.md, plugin-management-plan.md + wave-2-plugin-door-*.md (filenames), crate names (example-plugin, bad-plugin, busv2-plugin, timerv3-plugin, liar-plugin), cmake targets, leg names demo-wave2-plugin-* | |

(rev4: the panel found the rev1-3 survey missed .agents/, bindings/, and root
misc entirely - real total ~2900 hits / ~290 files. .agents/ is the layer that
teaches every future agent session the word; leaving it unowned would make the
guard fight a reintroduction engine.)

## 3. Adjudication points (ALL RESOLVED 2026-11-25, walked one by one)

| # | Point | Lean |
|---|---|---|
| A1 | ABI break style: clean break vs dual-symbol deprecation period | **RESOLVED A** (2026-11-25): clean break - the repo is unpushed (zero external consumers, the only free window); a fallback would ADD mechanism to the D87-frozen loader surface, deeper than the rename itself; D87 amendment rides D111 |
| A2 | Structural rename scope: full (dirs/crates/targets/legs) vs prose-only | **RESOLVED A** (2026-11-25): full - the maintainer's whole-repo instruction; structural names are the most user-visible layer, residue = the split vocabulary reborn |
| A3 | History surfaces: rewrite vs append-only | **RESOLVED A + bridge notes** (2026-11-25): decisions/history-archive/plan bodies stay as-written (append-only, the value of history is original wording); each historical ENTRY POINT gains a terminology bridge note (mapping + reason + pointer: one global note atop decisions.md, one line per plan-of-record header, one in history-archive) |
| A4 | Build output directory | **RESOLVED A** (2026-11-25): `<binary>/bundles/<name>/` per-bundle directories (the morning per-unit-dir ruling, renamed) - built bundles and installed bundles share the word and the shape (D28 `<bundles>/<id>/<version>/` unchanged) |
| A5 | Identity copy: README tagline | **RESOLVED A** (2026-11-25): the model-signaling tagline - an OSGi-style bundle and service framework (bundle + service co-present in the first sentence; the next line's Rust kernel / C ABI / C/C++/Python/JS counters the Java smell) |
| A6 | Reintroduction guard | **RESOLVED A** (2026-11-25): a plugin-word scan with an explicit allowlist (decisions.md, history-archive, plan-of-record bodies, docs/reference/ external profiles) folded into gate-i18n - zero new legs; written in W1, ACTIVATED in W6 (the migration waves legitimately carry the word) |

### Micro-rulings (panel findings, resolved 2026-11-25, all A)

| # | Point | Resolution |
|---|---|---|
| M1 | kernel strings that NAME the old entry symbol (error.rs Display, kernel.rs symbol_hint, error-contract tests) | ride S1 with the symbol they name - diagnostics never point at a dead symbol |
| M2 | docs/reference/ granularity (mixed ModuKit-own text and external quotes) | per-line: OWN sentences sweep (S4), external quotes preserved via the A6 phrase-allowlist class |
| M3 | the reserved-but-never-created `plugins/` shelf (AGENTS:36, README:134, architecture:160, stage-1-plan:145) | renamed to `bundles/` in all four canon spots (S4; zero code - never created) - three-way word-identity with A4 build dir and D28 deploy tree |
| M4 | regen-golden.sh `--plugin_header` flag | renamed `--bundle_header` with the path and main.rs parsing, in S1 - no half-old command face |

## 4. Slices (rev4 - the panel-corrected execution schedule)

| Slice | Delivers | Key gates |
|---|---|---|
| S0 | status.md offload (7568/8000) | i18n, numbering |
| S1 | THE ATOMIC ABI BREAK - defined as THE FULL CONSUMER CLOSURE (rev4: the rev3 enumeration was incomplete; the closure is enumerated mechanically): **pre-flight** = dry-run the whole rename on a scratch branch + run the A6 scanner in REPORT mode to enumerate every consumer, then land. The closure includes: contracts/modukit_plugin.h -> modukit_bundle.h (types, entry symbol, MODUKIT_ENTRY_SYMBOL; parameter names out_plugin_id -> out_bundle_id); embed.h plugin-named types; abi.rs + embed_abi.rs PLUGIN_ABI_FIELDS (the 3-way parity anchor); codegen: pluginface.rs HAND-EDITS (include literal, MODUKIT_PLUGIN_HPP guard, banners, error strings) + cpp_emitter tests + goldens (incl. the modukit_plugin.hpp filename) - "regen propagates" is true ONLY for the embed face; bindings/cpp/probe.cc (include + dlsym + ctest target); ALL 13 fixtures (entry symbols + includes + rs fixtures' PluginAbiV1 type imports); wave2 driver heredocs (skin.PluginState); demo-stage4-m1/m2 script references; gate-script anchors (check-embed-parity.sh:38,49,122,155; regen-golden.sh:34 + --bundle_header flag + main.rs parsing per M4; gen-llms-txt.sh:55 + the llms.txt regen, and its skip-absent becomes RED for contract paths); error strings + symbol_hint + error-contract tests (M1); the doc-refs-pulled spots (README:111, capability-ledger rows, tutorials/02b:143, llms.txt) + the ~10-line teaching-truth patch (llms.txt link, ledger rows, tutorial 02 code block, module 04 lines); the A6 scanner written but INACTIVE; token-level D51/wire audit (identifier-level criterion - proto/wire.h comments mention the word but no tokens) | regen-fdset, embed-parity, golden, m1/m2/stage/wave2 legs, full check.sh |
| S2 | Rust internals EXCLUDING the ABI mirror (moved into S1 per rev4): PluginId/PluginState/PluginRecord/PluginView and friends -> Bundle*, examples/rs demo sources, modukit-runtime bins, kernel dm_report field, ops output column; committed BY SYMBOL FAMILY (whole-repo per family, never per-crate - half-renamed families leave red) | cargo fmt/clippy/test, workspace |
| S2.5 | the .agents governance layer (rev4 new): status/conventions/backlog live text, 8 skill bodies, TRIGGERS; pitfalls/conventions HISTORICAL entries keep original wording with bridge notes (A3); the agents-sync regen (SKILL.md mirrors) | gate-agents, i18n, numbering |
| S3 | Structural, as FAMILY COMMITS (5-8 commits, each family self-complete - census/lane/lock/CONSTRAINTS gates enforce it): (a) script-faces family FIRST: plugin_runtime.py/.mjs -> bundle_runtime.* + the 4 fixtures' isfile/import consumers + the 2 legs' probes + **.gitignore:148-149 negative line same-commit** (else the skins silently leave version control) + DEMO_PLUGIN_*_SABOTAGE lever names; (b) dir family: test-plugins/ -> test-bundles/ (root CMakeLists, spec-trace SCAN_ROOTS, 9 demo script paths); (c) crate family: 5 Cargo.tomls + root workspace + Cargo.lock (lock-freshness gate) + lane table + check.sh census + the 13 crates test files' -p/libname strings + examples/rs sources; (d) cmake-target family incl. t{1,2,3,7}-plugin tutorial dirs and the tutorials/02b backtick-path coupling; (e) leg-name family (check.sh --list/run/census + leg script names + twins; demo.sh is a glob - auto-follows); (f) build-output family: bundles/<name>/ (A4). PolyOrch NOTE (rev4-corrected): all edits are REPO-LOCAL (cmake/ModuKitPolyOrchCrates.cmake lanes + protoc comment); the submodule itself has ZERO ModuKit hits - audit expectation zero, no submodule edit, no pin push | lane-table, census, lock-freshness, full gallery re-run, check.sh |
| S4 | Docs volume: module 01-bundle-placement-model.md (file rename - toc is a glob; 7 inbound links), tutorials, README (the A5 tagline + M3 shelf rename in 4 canon spots), AGENTS, gen-llms-txt.sh copy edits THEN regen (editing the generator, not just the output - else regen reintroduces), capability-ledger, features, gap matrix; docs/reference per M2 (own sentences swept, external quotes preserved); the A3 bridge notes (decisions top, each plan-of-record header - QUALIFIED labels: this plan's A1-A6/S1-S5 are NOT plugin-management-plan's - history-archive) | doc-refs, book-build, gate-docs |
| S5 | Closure: ACTIVATE the A6 guard (rev4 spec: scan surface = git-tracked files minus explicit skip prefixes - reuse the gate-i18n SKIP machinery so untracked .opencode/serena tool trees never flood; allowlist = 4 file classes + PHRASE patterns: "protoc plugin", opencode `"plugin":` JSON keys, external-project quotes, generic "plugin systems"; activation toggle + a self-test arm + word-boundary/case rules); full-leg rehearsal (E7-style, warm-cache labeled); acceptance: zero hits outside the allowlist AND the surviving list printed by the gate | check.sh all green; the scan, zero outside allowlist |

Ordering rationale: S1's break commit is, by A1's clean-break nature, ONE
large commit (60-90 files) - the single mega-commit the plan explicitly
permits (the exemption below); the pre-flight dry-run + report-mode scan is
its opening step, converting the panel's consumer-closure finding into the
work list. S2 follows by symbol family; S2.5 sweeps the governance layer the
survey missed; S3 is family-commits (each family is forced complete by the
census/lane/lock gates - "Cargo.toml first or scripts first" is a
pseudo-question, every commit must be whole); S4 lands docs last; S5 closes.
Every slice boundary is interruptible.

## 5. Verification ledger

Every wave: `bash scripts/check.sh` green (ok=17), plus its listed gates. W4
re-runs the FULL demo gallery (all 14 legs). W6 adds the rehearsal log as the
acceptance evidence (E7-style, warm-cache honestly labeled).

## 6. Risks

- Mid-migration mixed vocabulary (drift window): accepted, bounded - never
  two slices in flight; an "unification rename in flight - code is truth
  until S4" bridge line rides AGENTS.md top from S1 to S4; S1 carries the
  minimal teaching-truth patch (llms.txt + ledger + tutorial 02) so no
  MACHINE victim (dead links) survives the window.
- The S1 break commit is the ONE permitted mega-commit (60-90 files) - the
  A1 clean-break price. Commit-shape discipline transfers to S2 (per symbol
  family) and S3 (per rename family); every commit in the volume rides the
  C3 shape gate.
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

- 2026-11-25 revision 4: second expert panel (correctness / evidence / CI
  gates / blast radius) folded in: S1 redefined as the mechanically-enumerated
  consumer closure (probe.cc, gate-script anchors, pluginface.rs hand-edits,
  parity 3-way anchor, wave2 heredocs, error strings per M1, the doc-refs-
  pulled spots + teaching-truth patch, pre-flight dry-run as the opening
  step); A6 spec rewritten (git-tracked scan surface + skip prefixes, phrase-
  level allowlist classes, toggle + self-test); .agents/ surveyed and owned
  by a new S2.5; the script-faces family (runtime skins + .gitignore
  negative line + sabotage lever names) named as S3's first commit; PolyOrch
  note corrected to repo-local; S3 rewritten as family commits; the plugins/
  reserved shelf renamed bundles/ (M3); reference per-line sweep (M2);
  --bundle_header flag (M4); survey corrected to ~2900/290 incl. .agents and
  bindings; the S1 mega-commit exemption recorded; MODUKIT_ENTRY_MACRO typo
  fixed to MODUKIT_ENTRY_SYMBOL.
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
