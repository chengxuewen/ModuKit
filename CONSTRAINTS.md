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
4. `gate-i18n`
5. `skill-lint`
6. `regen-fdset`
7. `demo-stage1`
8. `demo-stage2`
9. `demo-stage4-m1`
10. `demo-stage4-m2`
11. `demo-stage5-n2`
12. `demo-wave2-reader`
13. `demo-wave2-clock`
14. `demo-wave2-py`
15. `demo-wave2-py-sup`
16. `demo-wave2-js`
17. `demo-wave2-js-sup`
18. `demo-wave2-plugin-py`
19. `demo-wave2-plugin-js`
20. `cargo-deny`
21. `ctest-stage4`
22. `embed-parity`
23. `gate-abi`
24. `capability-ledger`
25. `gate-agents`
26. `commit-shape`
27. `gate-docs`
28. `check-promises`
29. `numbering`
30. `lane-table`
31. `wire-parity`
32. `gate-mcp`
<!-- END:ci-gates -->
