---
name: think-before-act
description: "Meta-constraint: investigate before acting, present options for user approval, do not act rashly. Use BEFORE any non-trivial action (debug/test/implement/refactor/fix/config/upgrade). Especially when facing silent failures, runtime errors not caught by compiler, or behaviour contradicting documentation."
---

# think-before-act — Research → Present Options → User Approval → (Team Review) → Execute

> Do not act rashly.

## Trigger Conditions

Any **non-trivial** operation: debug, test, implement, refactor, fix, config change, upgrade.
Especially triggered when failure modes are abnormal: silent failure, runtime errors not caught by compiler, behaviour contradicting documentation.

---

## Decision Gate (ask yourself before every operation)

1. **"What does the official recommendation say?"** → Look it up if you don't know, don't guess.
2. **"Has this project encountered this before?"** → Check the project knowledge base (.agents/memorys/ / git log).
3. **"Has anyone in the community solved this?"** → Search issues / forums / StackOverflow (search the exact error message).
4. **"Does the user approve this approach?"** → Present options and wait for approval, don't change directly.
5. **"Why did the last attempt fail?"** → If you can't explain it, don't try the next one.

---

## Severity Levels

| Level | Criteria | Flow |
|------|------|------|
| **Trivial** | No interface/build/dependency changes, reversible, single file, no side effects | Do it directly + one-line explanation |
| **Standard** | Changes interface/build/dependencies/multi-file/irreversible/framework-level issue | Phase 1→2→3 |
| **Urgent** | User explicitly states "urgent/fix first report later" (production down, data loss) | Do it first + post-hoc report |

---

## Phase 1: Research (standard level, skipping forbidden)

### 5-Tier Knowledge Sources (by priority)

1. **Project history**: .agents/memorys/decisions.md / pitfalls.md / git log
2. **Language/toolchain native docs**: cargo doc / rustdoc / --help / man
3. **Framework/tool official docs**: find recommended practices, not random search
4. **Community experience**: GitHub issues / StackOverflow / forums (search exact error messages)
5. **Similar projects**: how do open-source implementations with the same tech stack do it

### Testing Scenario Additional Checklist

- [ ] Project testing conventions (framework? AAA? naming? coverage?)
- [ ] Spec/contract docs → extract test scenarios
- [ ] Existing test patterns (how is same module tested? fixtures?)

### Output

Research summary: what was found, what the official recommendation says, what the project conventions are.

---

## Phase 2: Present Options (standard level, auto-execution forbidden)

### Format

```
Option A: [description] — pros / cons / impact scope
Option B: [description] — pros / cons / impact scope
Recommendation: [X], rationale: [...]
```

### Code Changes Must List

- Which files will be modified
- Summary of changes (not full diff)
- Potential risks
- Rollback method

### Approval

Wait for user confirmation via the `question` tool.
- Silence / timeout / "continue" ≠ approval
- Must have explicit affirmative response from user
- Partial approval ("do A not B") → execute only the approved parts

### Complex Changes: Team Review

For complex changes (build changes / architecture changes / framework-level issues / affecting >5 files),
after user approval but before execution, add a team review step:

```
1. team_create — create review team (3 perspectives)
2. team_task_create — assign review tasks
3. team_send_message — send draft plan
4. Collect review feedback → revise plan
5. User confirms revision → enter Phase 3
```

CRITICAL items found in review must be fixed. LOW items can be recorded as tech debt.

---

## Phase 3: Execute (user has approved)

- Execute according to the user's chosen option
- Verify each step (don't accumulate unverified changes)
- On failure: roll back + report, **do not auto-try Option B** (go back to Phase 1 for additional research)

---

## Core Iron Rules

1. **No new information = no new attempt.** Retrying with different parameters after failure, without new diagnostic data, is acting rashly.
2. **Test at the right level.** Unit (logic) / Integration (boundary) / E2E (user flow) / Acceptance (business) — different levels catch different faults. If unit tests pass but the feature is broken, you tested at the wrong level.
3. **Trivial code needs no tests.** One line, no branches, no side effects → YAGNI applies to tests.

---

## Test Level Quick Reference

| Level | Catches | Rust Tool |
|------|----------|-----------|
| Unit | Logic errors | `cargo test` |
| Integration | Interface mismatches | `cargo test --test '*'` |
| E2E | Broken user paths | Playwright / Python WS scripts |
| Acceptance | Unmet requirements | Manual verification / pixi run test-sfu |

---

## Forbidden List (verify Y before doing X)

| Want to Do | Verify First |
|------|--------|
| Change config / change build | Checked official recommended config approach |
| Retry after failure | Have new diagnostic info (not just parameter guessing) |
| Edit generated files / build artifacts | Confirmed edits won't be overwritten on next build |
| Delete something you don't understand | Confirmed it's not depended on by other modules |
| Edit files directly | User has approved the option |
| Change dependencies / pixi.toml in production | Passed local verification first |

---

## Relationship with Existing Skills

| Skill | Relationship |
|------|------|
| `lesson-memory` (C9) | think-before-act looks up past lessons; lesson-memory records new lessons |
| `lesson-review` | think-before-act is prevention; lesson-review is retrospective |
| `systematic-debugging` | think-before-act manages "investigate before fixing"; debugging manages "how to fix" |
| `verification-before-completion` | think-before-act manages "before acting"; verification manages "after completion" |

## Multi-Ecosystem Examples

| Ecosystem | Typical "investigate before acting" Scenario |
|------|----------------------|
| Rust | Lifetime error → read the relevant rustbook chapter first, don't blindly add `.clone()` |
| Python | Dependency conflict → check pip/uv resolution rules first, don't repeatedly `pip install` |
| Go | Interface not satisfied → read the godoc interface contract first, don't blindly change signatures |
| JS/TS | Duplicate module → check bundler official recommendations first, don't randomly tweak aliases |
| Java | Version conflict → run `mvn dependency:tree` first, don't manually exclude |
| K8s | Pod crash → check official troubleshooting + `kubectl describe` first, don't randomly change yaml |

---

## Karpathy Four Principles Mapping

This project's think-before-act skill naturally aligns with Andrej Karpathy's four engineering philosophy principles:

| Karpathy Principle | Skill Equivalent | Description |
|--------------|-----------|------|
| **Think Before Coding** | Phase 1: Research + 5-Tier Knowledge Sources | Check project history, official docs, community experience before acting, don't guess |
| **Simplicity First** | Core Iron Rule #3 + Severity Levels (trivial/standard) | Prefer simple solutions, YAGNI, no over-engineering |
| **Surgical Changes** | Phase 2: Present Options + Forbidden List | Precise modifications, list impact scope, don't touch unrelated code |
| **Goal-Driven Execution** | Phase 3: Execute + Decision Gate | Execute according to approved option, verify each step, don't deviate |

> **Community Reference**: [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) — systematizes Karpathy's engineering philosophy into AI-executable skills, consistent with this skill's design philosophy.
