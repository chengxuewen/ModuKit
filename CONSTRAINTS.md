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
7. `selftests`
8. `skill-lint`
9. `abs-paths`
10. `book-check`
11. `regen-fdset`
12. `demo-stage1`
13. `demo-stage2`
14. `demo-stage4-m1`
15. `demo-stage4-m2`
16. `demo-stage5-n2`
17. `demo-rs-examples`
18. `demo-wave2-reader`
19. `demo-wave2-clock`
20. `demo-wave2-py`
21. `demo-wave2-py-sup`
22. `demo-wave2-js`
23. `demo-wave2-js-sup`
24. `demo-wave2-bundle-py`
25. `demo-wave2-bundle-js`
26. `cargo-deny`
27. `ctest-stage4`
28. `cmake-smoke`
29. `embed-parity`
30. `gate-abi`
31. `capability-ledger`
32. `gate-agents`
33. `gate-rules`
34. `spec-trace`
35. `commit-shape`
36. `gate-docs`
37. `check-promises`
38. `numbering`
39. `lane-table`
40. `wire-parity`
41. `gate-mcp`
<!-- END:ci-gates -->
