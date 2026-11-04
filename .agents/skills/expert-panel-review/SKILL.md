---
name: expert-panel-review
description: "Run a bounded multi-role review of one plan, decision, or conclusion. Use for 'expert panel', 'team review', 'multi-role review', or 'red-team this plan'. Two core roles are mandatory; deeper items add up to three lenses. Not for whole-document audits (doc-audit), repo health (repo-audit), whether to act (think-before-act), or item-by-item user adjudication."
---

# expert-panel-review — bounded multi-role review engine

Review one artifact: a plan, decision, design, or material conclusion. The goal
is independent lenses and preserved disagreement, not decorative consensus.

## Boundary

| Need | Use instead |
|---|---|
| Whether to act at all | `think-before-act` |
| Audit a document corpus | `doc-audit` |
| Whole-repo health | `repo-audit` |
| Present options one by one for a ruling | `adjudication-walkthrough` |
| Ordinary diff review | `code-review-and-quality` |

`.agents/skills/TRIGGERS.md` is the authoritative trigger-boundary map.

## Complexity gate

| Item | Panel |
|---|---|
| Trivial / reversible / one-file | core two roles, one pass, or route to `think-before-act` |
| Standard | core two plus one scenario lens |
| Deep / cross-module / irreversible | core two plus two or three lenses |
| Maximum | five roles; record why the cap was reached |

Before opening a panel, write the artifact scope, three to five acceptance
criteria, and the role roster. Let the user add or remove roles.

## Core roles

1. **Correctness reviewer**: technical validity, hidden dependencies, failure
   paths, and compatibility.
2. **Evidence reviewer / devil's advocate**: attack unsupported claims, missing
   commands, absent acceptance criteria, and optimistic assumptions.

Scenario lenses may include security, ABI compatibility, operations, docs, or
performance. Reuse an existing skill as the lens only when it is mounted and
relevant.

## Execution tracks

- **Parallel track**: use team/subagent tools when available. Members are
  read-only and must not write git.
- **Sequential track**: use one context in separate role passes when the host
  has no team tools. Declare that this isolation is weaker than separate
  contexts; do not call the result independently verified.

Isolation rules:

1. A pass sees only the artifact, its lens, and shared acceptance criteria.
2. Do not expose one role's draft or conclusion to the next role before the
   merge pass.
3. Freeze each role's conclusion, evidence, confidence, and uncovered items.
4. After all roles freeze, run one merge pass.

## Output

Report findings with severity, evidence, and confidence. Preserve unresolved
disagreement verbatim enough for the user to adjudicate. End with recommended
next actions and the boundary rows that were used. Do not force consensus.
