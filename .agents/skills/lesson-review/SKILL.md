---
name: lesson-review
description: "Batch session review: systematically extract lessons learned and write them to project memory. Use after long debugging sessions (>1h or >3 failed attempts), when user corrects multiple errors, after build fixes, or via the /lesson-review command."
---

# lesson-review — Batch Lessons Learned Consolidation

## Trigger Conditions

- User says "summarize lessons" / "update memory" / "record lessons" / "review session"
- Long debugging session ended (>1h or >3 failed attempts)
- After user reports multiple errors
- After resolving a problem that took >30min
- `/lesson-review` command

## Relationship with Rules and Skills

```
think-before-act  →  [Action]  →  lesson-memory  →  doc-audit
   (consult)                       (instant capture)          (periodic audit)
                    lesson-review ← batch gap-filling
```

| | `lesson-memory` Rule (C9) | `lesson-review` Skill |
|---|---|---|
| Timing | Instant (after each error, auto-triggered) | Batch (session end / user-triggered) |
| Method | Reflection | Interactive review |
| Granularity | Single lesson | Full session scan |
| Complement | First line of defense (no omissions) | Second line of defense (no misjudgment + classification) |

- `think-before-act` (consult): checks existing lessons before acting, avoids repeating mistakes
- `lesson-memory` (C9 rule): **instantly auto-writes** after each error, no user request needed
- `lesson-review` (this skill): **batch review** at session end, fills gaps, classifies and archives
- `doc-audit` (audit): periodically checks memory file consistency

## Workflow

### Step 1: Scan Session

Review which moments in this session triggered rules but may have been missed:

- Compilation/build failure
- >1 failed attempt before locating root cause
- User corrected approach/preference
- Unexpected discovery ("oh, I didn't expect that")
- Problem taking >30min
- Syntax damage after editing (unbalanced braces, duplicate lines, etc.)

### Step 2: Extract Each Item (5-Question Checklist)

For each finding, complete each item:

1. **What went wrong?** (symptom description)
2. **Why did it go wrong?** (root cause analysis)
3. **What's the correct approach?** (solution)
4. **How to prevent it?** (check command / constraint)
5. **Where to store it?** (per write targets table)

### Step 3: Write

Write using the target file's template format.

- Important lessons → full template (symptom + root cause + solution, three elements)
- Trivial fix (1 line / no branches / no side effects) → 1-line capture, no full template needed

### Step 4: Verify

- [ ] Every pitfall has a verify field (check command — how to confirm it's fixed)
- [ ] Every convention can be verified via grep/lint
- [ ] No duplicates with existing entries (grep target file to confirm)
- [ ] decisions.md numbering is sequential (no gaps)
- [ ] Cross-references are correct (e.g., pitfalls referencing conventions)
- [ ] High-frequency / high-cost lessons flagged for possible CI gate escalation

## Output Format

```markdown
## Session Lessons Summary — YYYY-MM-DD

### Recorded (N items)
1. [title] → pitfalls.md
2. [title] → conventions.md
...

### Not Recorded (no need)
- [reason: one-time issue / already recorded / environment-specific]

### Suggested Escalation
- [a certain lesson suggests adding CI gate, rationale: repeated 3+ times / took >1h]
```

## Write Targets (per project config)

Works for any project, just modify paths:

```
- Technical pitfalls → {pitfall_log}      # .agents/memorys/pitfalls.md
- Dev constraints → {conventions}       # .agents/memorys/conventions.md
- Architecture decisions → {decisions}         # .agents/memorys/decisions.md
- Runnable checks → {checks}          # .agents/rules/common/edit-safety.md
- Test requirements → {test_rules}        # .agents/rules/common/testing.md
- Security rules → {security_rules}    # .agents/rules/common/security.md
- Rust coding → {rust_style}       # .agents/rules/rust/coding-style.md
- Project status → {status}            # .agents/memorys/status.md
```
