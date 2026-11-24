# Framework gap matrix - tutorials and examples vs the three donors

> The archived output of the 2026-11-24 example-volume panel (three frozen
> lenses: topic matrix, feasibility, pedagogy). The maintainer question it
> answers: "is our example inventory thin vs CppMicroServices / celix /
> CTK?" Verdict: quantifiably yes at the time (6 projects / ~441 LOC vs
> CppMS 11 / ~1524 and celix 15 / ~2148), with a qualification - our
> covenant (compiled AND executed with PASS lines) is stronger than both
> donors (CppMS compiles its doc code, celix only compiles). The rulings
> that closed the actionable gaps live in `docs/plugin-management-plan.md`
> rev8 (B1-B6, all A).

## Quantified baseline (panel-measured)

| | CppMS | celix | CTK | ModuKit (at adjudication) |
|---|---|---|---|---|
| example projects | 11 | 15 | 3 app-level | 6 (t-series) |
| LOC | ~1524 | ~2148 | - | ~441 |
| CI relation | compiled + DocCode rebuild | compiled only, zero add_test | - | compiled + executed + PASS lines |

## (a) Capability shipped, example missing - the actionable class

| # | Gap | Donor precedent | Capability evidence | Disposition |
|---|---|---|---|---|
| 1 | DM-lite / on_down | celix has THREE dm examples | `dm.rs:40`, `kernel.rs` dep_lost/dep_restored wiring | rs example on the merged leg (B1) |
| 2 | ServiceTracker + Modified events | CppMS dictionaryclient2; celix track_tracker | `tracker.rs:36-115` | rs example on the merged leg (B1) |
| 3 | S4 ops face | celix shell shape | `host.rs` run_ops (lb/query/dm) | rs example on the merged leg (B1); teaching rides tutorial 01 (B5) |
| 4 | ConfigAdmin hot lane | - (persistence exceeds CppMS) | `config.rs` set/get/declare | rs example on the merged leg (B1) |
| 5 | requires version ranges | CTK Require-Plugin | `version.rs`; resolve check | covered by the DM example's manifest (B1) |
| 6 | multi-service composition + ranking | CppMS spellcheck pair | ranking in tracker.rs | cpp t7 bus-mediated composition (B1) |
| 7 | py/js consumer scenes | - | verb-face only | wave2 leg extensions (B1) |
| 8 | D28 logical bundle (resources/object store) | CppMS frenchdictionary | `bundle.rs:5-7` | recorded; waits for a real resource consumer |

## (b) Capability itself missing - product gaps, examples cannot help

| Gap | Donor | Disposition |
|---|---|---|
| HTTP / embedded web admin face | celix http_example | no trigger; do not build |
| three-tier config (deploy tier) | CTK | deploy-TOML seeder waits for the deploy file type (M7+, recorded in module 05 §6) |
| service factory / prototype scope | CppMS SCR; celix DM | D4 stands (ruling A5); a hearing would be required to overturn |

## (c) Donor topics with no ModuKit analogue - shape differences, deliberately not pursued

| Topic | Why it does not map |
|---|---|
| CTK app scaffolding (ctkExampleHost/HostedApp, PluginBrowser, PluginGenerator) | ModuKit's shape is one cockpit kernel + modukit-host; codegen is an ABI bridge, not a plugin wizard |
| frenchdictionary's localization narrative | the bundle resource mechanism exists; the "localized bundle" USE CASE has no analogue - falls under (a)8 if a consumer appears |

## Structure rules adopted (B2)

- Dual namespace: the progressive t-series admits EVOLUTION steps only
  (poll -> filter -> composition...); non-evolution capability examples
  live in topic directories (kebab-case). CppMS is the progressive
  precedent; celix is the topic precedent; we use both.
- Admission: a t-project is in the series iff a standing leg EXECUTES it
  (stronger than both donors). Honest ceiling ~10-12 executed t-projects
  for a one-maintainer repo.
- Ceilings: past ~12-15 executed scenes on m1, a new domain opens its own
  leg; past ~9 tutorial pages, the toc splits into sub-groups (nested
  precedent exists).
- Provenance: symbol anchors (`T1_DESCRIPTOR in t1.cpp`), not line
  ranges; the rehearsal greps quotes back to their sources (B3).
- Band honesty: the t-series does not teach what no band face exists for
  (events/config are Rust-face today) - the 02b "not yet" block says so
  where the reader would hit the wall (B4).
