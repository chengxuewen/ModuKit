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
| **Fix** | "fix" / "bug" / "error" / "not working" | `/debugging-and-error-recovery` |
| **Refactor** | "refactor" / "optimize" / "simplify" / "clean up" | `/think-before-act` + `/ai-slop-cleanup` |
| **Design** | "design" / "architecture" / "approach" / "how to" | `/think-before-act` + `/api-interface-design` |
| **Test** | "test" / "E2E" / "coverage" | `/test-driven-development` + `/test-harness` |
| **Adjudication** | "裁决逐项过" / "一个个来" / "decide between options" / "walk the options" | `/adjudication-walkthrough` | <!-- c1:allow:trigger -->
| **Audit** | "audit" / "review repo" / "docs drift" / "scan ecosystem" | `/doc-audit` for docs/decisions; `/repo-audit` for whole-repo engineering |
| **Authoring** | "new skill" / "add a skill" / "skill registry" | `/skill-authoring` |
| **Docs** | "docs" / "README" / "documentation" | `/doc-audit` |
| **Security** | "security" / "permissions" / "auth" | `/security-hardening` |
| **Exploration** | "research" / "compare" / "what options" | `/repo-audit` + `/doc-audit` |
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
Recommendation: /debugging-and-error-recovery
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
2. If fixing a bug → /debugging-and-error-recovery
3. If refactoring → /think-before-act
What specifically would you like to do?
```

### Step 4: Execution Suggestion

After recommending, if the user confirms, invoke the skill directly:
```
Confirmed: loading /think-before-act skill...
```

## Relationship with passive matching

- **Passive matching**: each skill's frontmatter description is surfaced by opencode's skill tool and auto-matched to the task scenario. Trigger-phrase OWNERSHIP (which skill owns which phrase, machine-checked, one owner per phrase) lives in `.agents/skills/TRIGGERS.md` — a new skill must add its row there before it is mounted. A PolyOrch-era claim that no routing table existed here was wrong and removed 2026-11-05.
- **skill-router**: explicit invocation (when the user asks "how should I do this" or intent is ambiguous, performs deep analysis and recommendation)

- **context-engineering**: passive trigger — per the root `SKILL.md` registry it assembles the right context before acting (language-rule routing is its description in the generated directory table, not a separate skill); check TRIGGERS.md for phrase ownership.

Complementary: passive matching handles clear scenario hits, skill-router handles ambiguous intent and multi-skill combinations. The full inventory lives in the root `SKILL.md` registry; phrase ownership lives in TRIGGERS.md.

## When Not to Recommend

| Scenario | Don't Recommend | Reason |
|------|--------|------|
| User is urgently fixing something | Any skill | Don't interrupt, fix first then discuss |
| 1-line change | /think-before-act | Overkill |
| User explicitly specified a skill | Other skills | Respect user choice |
| Same skill just recommended | Same skill | Avoid repetition |
