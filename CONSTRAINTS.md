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
6. `gate-i18n-selftest`
7. `skill-lint`
8. `book-check`
9. `regen-fdset`
10. `demo-stage1`
11. `demo-stage2`
12. `demo-stage4-m1`
13. `demo-stage4-m2`
14. `demo-stage5-n2`
15. `demo-wave2-reader`
16. `demo-wave2-clock`
17. `demo-wave2-py`
18. `demo-wave2-py-sup`
19. `demo-wave2-js`
20. `demo-wave2-js-sup`
21. `demo-wave2-plugin-py`
22. `demo-wave2-plugin-js`
23. `cargo-deny`
24. `ctest-stage4`
25. `cmake-smoke`
26. `embed-parity`
27. `gate-abi`
28. `capability-ledger`
29. `gate-agents`
30. `gate-rules`
31. `spec-trace`
32. `commit-shape`
33. `gate-docs`
34. `check-promises`
35. `numbering`
36. `lane-table`
37. `wire-parity`
38. `gate-mcp`
<!-- END:ci-gates -->
