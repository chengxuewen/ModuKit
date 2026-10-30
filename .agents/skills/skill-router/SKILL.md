---
name: skill-router
description: "Analyze user intent and output a recommended skill list. Auto-activates when the user says 'how should I do this', 'what skill should I use', or when intent is ambiguous. Use when the user asks 'what skill should I use', 'how to approach this', or when intent is ambiguous."
---

# skill-router — Skill Routing Analysis

> When user intent is ambiguous or multiple skills compete, analyze and recommend the best skill combination.

## Trigger Conditions

- User says "how should I do this" / "what skill should I use" / "help me choose"
- User intent is ambiguous, needs clarification before recommending
- Multiple skills may apply, needs comparison and selection

## Analysis Workflow

### Step 1: Intent Classification

Read the user message and classify as:

| Category | Keywords | Default Recommendation |
|------|--------|---------|
| **Implementation** | "add" / "implement" / "create" / "modify" | `/think-before-act` + `/test-driven-development` |
| **Fix** | "fix" / "bug" / "error" / "not working" | `/systematic-debugging` |
| **Refactor** | "refactor" / "optimize" / "simplify" / "clean up" | `/think-before-act` + `/remove-ai-slops` |
| **Design** | "design" / "architecture" / "approach" / "how to" | `/brainstorming` + `/openspec-propose` |
| **Test** | "test" / "E2E" / "coverage" | `/test-driven-development` + `/playwright` |
| **Adjudication** | "裁决逐项过" / "一个个来" / "decide between options" / "walk the options" | `/adjudication-walkthrough` |
| **Audit** | "audit" / "review repo" / "docs drift" / "scan ecosystem" | `/doc-audit` for docs/decisions; `/repo-audit` for whole-repo engineering |
| **Authoring** | "new skill" / "add a skill" / "skill registry" | `/skill-authoring` |
| **Docs** | "docs" / "README" / "documentation" | `/doc-audit` |
| **Security** | "security" / "permissions" / "auth" | `/security-review` |
| **Exploration** | "research" / "compare" / "what options" | `/repo-audit` + librarian agent |
| **Ambiguous** | No clear keywords | Propose 2-3 possible directions, let user choose |

### Step 2: Context Check

Before recommending, check current state:

1. **Any in-progress tasks?** — If tasks are already in progress, only recommend skills related to the current task
2. **What skills did the user recently use?** — Avoid recommending what was just used
3. **Current project phase?** — New feature vs fix vs refactor, different phases need different recommendations
4. **Tech stack match?** — Ensure recommended skills apply to the current tech stack (Rust/TS/Docker/Web)

### Step 3: Output Recommendation

**Single skill scenario** (clear intent):
```
Recommendation: /systematic-debugging
Rationale: 2 consecutive fix commits suggest systematic root cause diagnosis is needed first
```

**Multi-skill scenario** (combination needed):
```
Recommended combination:
1. /think-before-act — research options first, avoid blind changes
2. /test-driven-development — write tests before implementation
Rationale: non-trivial multi-module change, test coverage needed
```

**Ambiguous scenario** (clarification needed):
```
Intent unclear, possible directions:
1. If implementing new feature → /think-before-act + /test-driven-development
2. If fixing a bug → /systematic-debugging
3. If refactoring → /think-before-act
What specifically would you like to do?
```

### Step 4: Execution Suggestion

After recommending, if the user confirms, invoke the skill directly:
```
Confirmed: loading /think-before-act skill...
```

## Relationship with passive matching

- **Passive matching**: each skill's frontmatter description is surfaced by opencode's skill tool and auto-matched to the task scenario — no separate routing table exists in this repo (the PolyOrch-era `context-engineering` pointer here was a phantom; fixed 2026-10-14).
- **skill-router**: explicit invocation (when the user asks "how should I do this" or intent is ambiguous, performs deep analysis and recommendation)

- **context-engineering**: passive trigger (auto-recommends when scenarios are detected, via the AGENTS.md directory table)
- **skill-router**: active invocation (when user asks "how should I do this" or intent is ambiguous, performs deep analysis and recommendation)

Complementary: passive matching handles clear scenario hits, skill-router handles ambiguous intent and multi-skill combinations. The full inventory lives in the root `SKILL.md` registry.

## When Not to Recommend

| Scenario | Don't Recommend | Reason |
|------|--------|------|
| User is urgently fixing something | Any skill | Don't interrupt, fix first then discuss |
| 1-line change | /think-before-act | Overkill |
| User explicitly specified a skill | Other skills | Respect user choice |
| Same skill just recommended | Same skill | Avoid repetition |
