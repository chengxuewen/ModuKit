# CONSTRAINTS — what "done" means here

Single source of truth for the local quality gate. Two parts:

1. **Anti-gaming rules** (hand-written) — behaviours that are never allowed to make a gate green.
2. **The gate list** — GENERATED from `bash scripts/check.sh --list`. Edit `check.sh`, not the marker region.

## Anti-gaming rules (hand-written)

- Never commit with `--no-verify` — it bypasses the pre-commit hook (agent sync + gate).
- Never add `#[allow(...)]` to silence a warning without a justification comment naming the invariant.
- Never use `#[ignore]` to skip a failing test; the only sanctioned ones are env-gated tests run explicitly.
- Never delete or weaken an assertion to make a red gate green.
- Never hand-edit a generated layer: the `AGENTS.md` marker regions, `CONSTRAINTS.md` region, `.mcp.json`, the `.opencode/opencode.json` `mcp{}`, or the `SKILL.md` registry.
- Any dependency change passes `cargo deny` when the tool resolves; an offline skip is recorded loudly, never silent.
- Fix the cause, not the gate. If a gate is red, the gate is right until proven otherwise.

<!-- BEGIN:ci-gates -->

## Gate list (generated from `bash scripts/check.sh --list`; edit check.sh, not this block)

`pixi run ci` is an alias for `bash scripts/check.sh`; it runs:

1. `fmt`
2. `clippy`
3. `test`
4. `msrv`
5. `gate-i18n`
6. `skill-lint`
7. `regen-fdset`
8. `demo-stage1`
9. `demo-stage2`
10. `demo-stage4-m1`
11. `demo-stage4-m2`
12. `demo-stage5-n2`
13. `demo-wave2-reader`
14. `demo-wave2-clock`
15. `demo-wave2-py`
16. `demo-wave2-py-sup`
17. `demo-wave2-js`
18. `demo-wave2-js-sup`
19. `demo-wave2-plugin-py`
20. `demo-wave2-plugin-js`
21. `cargo-deny`
22. `ctest-stage4`
23. `cmake-smoke`
24. `embed-parity`
25. `gate-abi`
26. `capability-ledger`
27. `gate-agents`
28. `commit-shape`
29. `gate-docs`
30. `check-promises`
31. `numbering`
32. `lane-table`
33. `wire-parity`
34. `gate-mcp`
<!-- END:ci-gates -->
