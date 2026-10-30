---
name: repo-triage
description: "Layer triage for an ambiguous task in this polyglot framework: map the ask to kernel / transport / codegen-contracts / bindings / examples / CLI-infra / docs, list affected crates and ABI surfaces, then dispatch to the right skill. Use when a task's scope is unclear or spans crates, bindings, and scripts."
type: tooling
notes: "route a task to the right ModuKit layer"
---

# repo-triage — Which Layer, Which Skill?

> Most slow tasks are mis-routed tasks. Find the layer first, then the skill and the gate.

## When to Use

- The request mixes languages or directories and the boundary is unclear.
- You cannot name the affected crate(s)/binding(s) from the ask.
- A change touches more than one of: kernel, transport, codegen/contracts, bindings, plugin door, scripts/CI, docs.

## Layers (this repo's real map)

| Layer | Paths | Ruleset | Usual skills |
|-------|-------|---------|--------------|
| **kernel** | `crates/modukit-core` | `.agents/rules/rust`, `.agents/rules/common` | `rust-engineering`, `performance-optimization`, `debugging-and-error-recovery` |
| **runtime/executor** | `crates/modukit-runtime`, `crates/modukit-core/src/executor*`, `proc_*` | `.agents/rules/rust` | `debugging-and-error-recovery`, `observability-and-instrumentation` |
| **transport** | `crates/modukit-transport`, `seam.rs`, `fdpass.rs`, `vendor_shm.rs` | `.agents/rules/rust` | `rust-engineering`, `security-hardening` |
| **codegen/contracts** | `crates/modukit-codegen`, `contracts/` | `.agents/rules/rust`, `.agents/rules/common` | `constraint-driven-development`, `api-interface-design` |
| **bindings/plugin door** | `bindings/{cpp,py,js}`, `crates/modukit-js`, `test-plugins/` | `.agents/rules/rust`, `.agents/rules/common` | `api-interface-design`, `security-hardening` |
| **examples** | `examples/{rs,cpp,py,js}` | `.agents/rules/rust`, `.agents/rules/common` | `rust-engineering`, `test-harness` |
| **cli-infra** | `scripts/`, `pixi.toml`, `.agents/`, `.github/` | `.agents/rules/common` | `ci-cd-automation`, `ecosystem-scan`, `constraint-driven-development` |
| **docs** | `docs/`, `AGENTS.md`, `CONSTRAINTS.md`, `SKILL.md` | `.agents/rules/common` | `doc-audit`, `documentation-and-adrs` |

## Process (steps + checkpoints + exit)

1. **Restate the ask** in one sentence (what changes, for whom).
2. **Map to layer(s)** using the table above; if two layers, name the boundary (e.g. FFI).
3. **List affected artifacts**: crate(s), exported symbol(s), header/hpp/wasm mirror(s), gate(s).
4. **Checkpoint**: you can state layer + artifacts + the gate that proves it.
5. **Dispatch** to the layer skill(s) from the table, and load the ruleset via `context-engineering`.
6. **State the verification** before editing: e.g. `pixi run cargo clippy -p <crate> -- -D warnings`, `pixi run gate-abi`, `pixi run ci`.
7. **Exit**: layer + artifacts + gate are named; then hand off to the specific skill.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "just start editing" | Without the layer you will load the wrong rules and miss the gate. |
| "it's all one repo, so it's one layer" | Core / FFI / Qt / wasm have different rulesets and gates. |
| "the ask is clear enough" | Clear asks still cross a boundary; name the affected artifacts. |
| "I'll find the gate later" | Naming the gate up front is how you avoid a false-green. |

## Red Flags

- Editing before naming the layer and the gate.
- Treating an FFI change as a pure Rust change (mirror drift).
- Loading every ruleset instead of the layer's.
- Dispatching to no skill at all ("just code").

## Verification (evidence)

- The layer → artifacts → gate mapping is stated in the response.
- The chosen skill(s) exist in the `AGENTS.md` SKILL DIRECTORY.
- The gate output is quoted after the change.

## References

- `context-engineering` — file → language → ruleset routing.
- `skill-router` — task → skill classification when intent is vague.
- `docs/env.md` — toolchain + task reference.
