# Developer-onboarding plan - tutorials, examples, bindings, discoverability

> STATUS: PLAN OF RECORD, 2026-11-24 (revision 1). Drawn from a four-lens
> expert panel (correctness / evidence / docs / developer-experience, frozen
> lenses, merged with preserved disagreement) plus a community survey of five
> local-clone donors (CppMicroServices, celix, CTK, dora, and the in-repo
> PolyOrch precedent). Maintainer direction: make it easy for a user to
> quickly learn the usage and to discover ALL features, functions, use cases,
> and patterns across the multi-language bands (C / C++ / Python / JS).

## 0. Discussion anchors

### 0.1 Panel-verified diagnosis (not "we have no examples")

- The real gap is TWO-LAYER: (a) a content gap - no plugin-AUTHOR-view zero-to
  running path (today's examples/ are host-side gate walkers); (b) a
  discoverability gap - README has no quickstart heading and never links the
  book; docs/env.md "three steps" is environment-facing, not pedagogical.
- The ASSETS already exist: 13 standing demo legs (replayed by CI nightly -
  natural anti-drift teaching material), the 84-line clean author-side
  `cpp-example-plugin`, D23's "template standing" fixtures ruling, and the
  sabotage levers (`DEMO_STAGE4_SABOTAGE=1` - a ready-made "watch the gate
  refuse a bad manifest" teaching device).
- Live bugs the panel caught for free (fix regardless of this plan):
  README still says 24 verbs / 16 rung-0 (actual 25 / 17 after the S1
  tail-add); demo-wave2-py's SKIP copy says "under out/" (territory moved to
  build/) and does not point the user at m1; README/bootstrap claim macOS
  while `pixi.toml` platforms are linux-64/win-64; no `pixi demo-*` one-command
  task exists.
- The "same story across bands" acceptance criterion was REWRITTEN by the
  panel: Python plugins ride the exe wall + TL wire, a different world from
  dl + C ABI. The honest framing is "one business domain, two mechanism
  narratives" - never fake a Python plugin by driving the cpp .so from a py
  host.
- Four hidden contracts for script plugins must be taught explicitly: the
  manifest `runner` must be an ABSOLUTE existing path; a plugin subprocess
  must NEVER print (fd 0 is the control channel); `MODUKIT_PY_SKIN` must point
  at bindings/py for out-of-tree plugins; generated skins are DO-NOT-EDIT
  (D63).

### 0.2 Community survey (local-clone-verified)

| Project | Shape | What we borrow / reject |
|---|---|---|
| CppMicroServices | Progressive tutorial: dictionaryservice -> consumers v1/v2/v3 -> spellcheck -> eventlistener; Example1-7 chapters + getting_started.rst; code lives beside chapters | Borrow the PROGRESSION spine (naive consumer -> tracker -> filter maps onto our S1/S2); reject the dictionary domain (we are a plugin HOST; echo/sensor family is ours) |
| celix | documents/ topic pages (framework/components/services/patterns/properties_and_filter) + a separate bundled examples tree | Topic pages ~ our modules 00-11 already; separate examples tree would drift - reject |
| CTK | Documentation/ dir, doxygen | Nothing new for us |
| dora | examples/ with SCENARIO-named dirs (python-dataflow, c-dataflow, rust-dynamic-add-remove, multiple-daemons...) each with a README | Borrow scenario-naming for future example volumes; but our v1 rides existing legs rather than minting parallel scenarios |
| PolyOrch (in-repo submodule, shape precedent only) | docs/tutorials.md single-file learning path: "who this is for" 3-reader routing + numbered chapters, every command proven by the repo's own tests | Borrow the 3-reader routing and the "every command is CI-proven" covenant verbatim in spirit |

Consensus across donors: tutorials live WITH the docs source (so they build
and gate), example code lives where CI compiles it, and no donor maintains a
docs-only example that nothing compiles.

## 1. Goals (three user journeys)

1. **First 15 minutes**: fresh clone -> one command -> watch a full plugin
   lifecycle (install, resolve, start, service call, late-join, uninstall) -
   with copy-paste that works on Linux/WSL and honest platform notes.
2. **First plugin per band**: an author writes and runs their own plugin in
   C++ (dl), Python (script/exe), JS (script/exe), or embeds the host from C
   / C++ / Python / JS - knowing the four hidden contracts up front.
3. **Full capability map**: one page answers "what can this thing do and
   where do I see each capability run" - every shipped feature (25 embed
   verbs, filter/tracker/DM-lite/ConfigAdmin, bus v2 + clock v3 + push,
   supervisor, observability) mapped to its demo leg, module doc, and
   binding face. Counts link the capability ledger, never hardcode.

## 2. Scope and OUT

IN: W0 quick fixes; W1 tutorial suite (6 chapters + index); W2 discoverability
triple; W3 examples-as-code hygiene (manifest sample, band contracts doc,
examples/ pointer README that only routes); W4 feature gallery page; W5 book
integration; W6 the C-plugin hearing. OUT: new product features; new CI legs
beyond the doc-refs one-line extension; the video-frame domain (module 08 is
design-only - teaching it would teach something that cannot run); a config
hot-lane chapter (S3-2 has no demo leg yet - canon link only, chapter waits
for its leg); i18n translations (book is English-source by D46; readers use
browser translation).

## 3. Adjudication points (ARMED - walked one by one before W1 starts)

| # | Question | Recommendation |
|---|---|---|
| A1 | C band: mint the first C plugin fixture (`test-plugins/c/`, rides cmake + census = a real CI-face change) vs embed-C-only chapter (zero new surface) | Embed-C-only in v1; the fixture is its own small volume with a hearing |
| A2 | Extend `check-doc-refs` FILES with `docs/tutorials/*.md` (breaks the "on-demand docs not scanned" design on purpose) | Yes - tutorials link canon heavily; dead links there are exactly the rot class this plan fights |
| A3 | `pixi demo-*` tasks: one per band (`demo-cpp`, `demo-py`, ...) or one router (`demo = bash scripts/demo-...`)? | Router task `demo <leg>` is one line and advertises the gallery; per-band tasks multiply parity surface. Router. |
| A4 | Feature gallery home: `docs/tutorials/README.md` section vs own `docs/features.md` | Own page - the gallery serves ALL readers (not just learners) and gets its own book slot |
| A5 | Tutorial code blocks: verbatim from fixtures vs freshly written | Verbatim-from-fixture only (CI compiles what we teach); any code not in a fixture is a link, not a block |
| A6 | A tiny tutorial-lint (every `bash scripts/...` mention resolves to an existing leg) | Fold into the doc-refs extension instead (path liveness already covers it) - no new gate |

## 4. Work streams

### W0 - quick fixes (pre-tutorial, each independently shippable)

- README verb/rung counts 24->25, 16->17 (link `docs/capability-ledger.md`,
  never restate a number without its source).
- demo-wave2-py/js SKIP copy: "under build/" + "run demo-stage4-m1 first"
  (kills the silent-green dead end).
- README + bootstrap honest platform note: supported = Linux (native) and
  Windows (pixi env; demo gallery bash legs under WSL); macOS is not a pixi
  platform today.
- `pixi run demo <leg>` router task + one README line.
- C0: each fix is prose/script-level; full `bash scripts/check.sh` green.

### W1 - tutorial suite (the core volume)

Home: `docs/tutorials/` (English source; book single-source is docs/**).
Chapter budget: SIX + index; every chapter's copy-paste IS a standing demo
leg; every code block is verbatim from a CI-compiled fixture.

| Chapter | Teaches (leg) | Canon links |
|---|---|---|
| README (index) | 3-reader routing: newcomer / plugin author / embedder + band-choice table | modules/00 reading order, env.md |
| 01-first-run | env three steps -> `demo-stage1` (full lifecycle in 60s) | modules/03 |
| 02-first-plugin | manifest + install gates + sabotage lever (red then green) -> `demo-stage4-m1`; fixture cpp-example-plugin | modules/05, 02, 08 §9 |
| 03-the-bus | topics, late-join, clock bell -> `busv2` leg | modules/06 |
| 04-the-exe-wall | supervisor, crash/revive, ORPHAN-PROOF -> `demo-stage2`, `demo-stage4-m2` | modules/01 |
| 05-embed-from-your-language | ONE file, band subsections: C++ (m1) / Python (wave2-py) / JS (wave2-js) / C (embed face, links libmodukit_core.so) | capability-ledger, contracts headers, module 07 |
| 06-script-plugins | py/js plugin door + the FOUR hidden contracts + liar fixtures -> `demo-wave2-plugin-{py,js}` | modules/04, D106/D107 |

Domain thread: the echo/sensor family already in fixtures. Progression
motive (poll -> subscribe -> configure) appears as sentences, not chapters.

### W2 - discoverability triple

- README "Quickstart" section (5 lines: bootstrap -> demo router -> tutorials
  link -> book link -> platform note). README is already in doc-refs scope, so
  the new links are gate-checked.
- `scripts/gen-llms-txt.sh`: add the tutorials index + feature gallery lines;
  regen.
- `docs/modules/00-overview.md` reading order: "run the tutorials first".

### W3 - examples-as-code hygiene

- `examples/README.md`: a ROUTER ONLY (what each band is, which leg runs it,
  link tutorials) - no duplicated teaching prose (single source discipline).
- `docs/tutorials/fixtures/manifest-sample.toml`: one commented sample
  manifest (all v0 fields incl. requires/on_down/config_live) marked as
  documentation copy of the grammar in modules/05.
- The four script-plugin contracts get a canonical table in 06 (chapter) AND
  a one-line pointer from bindings/py/plugin_runtime.py and
  bindings/js/plugin_runtime.mjs headers.

### W4 - feature gallery (`docs/features.md`)

A capability matrix page: rows = shipped capabilities (embed verbs family,
filter/tracker/events, version ranges, DM-lite on_down, ConfigAdmin hot lane,
bus v1/v2, clock v3, parent push, supervisor verbs, observability trio,
ops face), columns = see it run (demo leg) / read the canon (module doc) /
use it per band (binding face or embed verb). Numbers link the capability
ledger. This is the "discover ALL features" deliverable.

### W5 - book integration

`book/toc.toml`: `[[group]] title = "Tutorials" type = "how-to" glob =
"docs/tutorials/*.md"` (insert after Environment, before Modules) + Features
page slot. type=how-to makes check-prose require shell blocks or numbered
steps in every chapter (report-only today, honest).

### W6 - C-plugin hearing (post-v1)

Mint `test-plugins/c/` as its own volume: cmake band mirroring cpp,
census/lane-table accounting, tutorial 02 gains a C variant. Requires A1
ruling + hearing; not in v1's critical path.

## 5. Verification ledger

| Plant / action | Expect |
|---|---|
| fresh-clone rehearsal of chapter 01 commands | install->uninstall walk PASS, no dead command |
| sabotage lever in chapter 02 as written | RED observed, then green on rerun |
| every `bash scripts/...` in tutorials | path exists (doc-refs extension, A2) |
| README quickstart links | doc-refs green (already in scope) |
| llms.txt regen | tutorials + features lines present, committed |
| book-build | Tutorials group renders between Environment and Modules, no Appendix WARN |
| counts anywhere in new prose | link capability-ledger; grep finds no bare "24 verbs"-class restatement |
| full check.sh after each stream | all green, leg count unchanged |

## 6. Diagram

```mermaid
flowchart TD
  A[newcomer: fresh clone] -->|README Quickstart| B[bash bootstrap.sh]
  B --> C[pixi run demo stage1]
  C -->|60s lifecycle aha| D{choose your path}
  D -->|author| E[tutorials 02-04: plugin, bus, exe wall]
  D -->|embedder| F[tutorial 05: embed from C/C++/py/js]
  D -->|script author| G[tutorial 06: py/js plugin door]
  E & F & G --> H[docs/features.md capability map]
  H --> I[modules 00-11 canon + capability ledger]
```

## 7. Risks

- **Drift is the #1 enemy**: the README 24-verb drift happened THIS WEEK.
  Mitigations baked in: chapters quote leg commands (CI replays them), code
  blocks from compiled fixtures, counts link the ledger, doc-refs covers the
  new tree (A2).
- macOS claim vs pixi platforms: honest note in W0; do not promise.
- Scope creep: six chapters is the ceiling (donor parity is 7); the gallery
  is one page; anything more needs a hearing.
- Maintenance owner: tutorials are hand-written by the D-recorded boundary
  (reference-class output is generated; narrative stays hand-written) - each
  chapter names its backing leg so a leg rename reddens the docs grep.

## 8. Gate plan

No new check.sh legs. The only gate-surface change is the one-line doc-refs
FILES extension (A2) plus the toc group (existing book gates pick it up) and
the llms.txt regen (existing generator). Full `bash scripts/check.sh` closes
every stream.

## 9. Deviations and debts

- Config hot-lane chapter deferred until it has a demo leg (S3-2 shipped
  code-first; the leg is the missing proof).
- C plugin fixture deferred to W6 hearing (A1); v1 teaches embed-C only.
- Windows demo gallery = WSL note, not native legs (bash legs are
  Linux-shaped: LD_PRELOAD, /proc).
- dora-style scenario examples (log-aggregator-class) stay a future volume
  once the tutorial layer proves the covenant.

## 10. Change log

- 2026-11-24 revision 1: drawn from the four-lens panel + five-donor survey;
  W0-W6 streams; adjudication points A1-A6 ARMED for the one-by-one walk.
