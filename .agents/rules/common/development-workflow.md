# Development Workflow

The Feature Implementation Workflow describes the development pipeline: research, planning, TDD, code review, and then committing to git.

## Feature Implementation Workflow

0. **Research & Reuse** _(mandatory before any new implementation)_
   - **GitHub code search first:** Run `gh search repos` and `gh search code` to find existing implementations, templates, and patterns before writing anything new.
     *(host caveat: this box has no egress and no `gh` - the `.refinfo/` local clones are the first-class verification asset per AGENTS NOTES; the web/Exa legs are likewise unavailable.)*
   - **Library docs second:** Use Context7 or primary vendor docs to confirm API behavior, package usage, and version-specific details before implementing.
   - **Exa only when the first two are insufficient:** Use Exa for broader web research or discovery after GitHub search and primary docs.
   - **Check package registries:** Search npm, PyPI, crates.io, and other registries before writing utility code. Prefer battle-tested libraries over hand-rolled solutions.
   - **Search for adaptable implementations:** Look for open-source projects that solve 80%+ of the problem and can be forked, ported, or wrapped.
   - Prefer adopting or porting a proven approach over writing net-new code when it meets the requirement.

1. **Plan First**
   - Use **planner** agent to create implementation plan
   - Generate planning docs before coding: PRD, architecture, system_design, tech_doc, task_list
   - Identify dependencies and risks
   - Break down into phases
   - Present the plan to the user

1.5 **Execution Confirmation Gate** _(mandatory — NEVER skip)_
   - **MUST get explicit user confirmation** before executing ANY plan or todo list:
     - Use the `question` tool with a "Proceed with execution?" prompt (interactive confirm)
  - Action verbs in an instruction ARE its scope: "file the document and audit" orders docs
    updates + review ONLY - running gates, measuring, or fixing requires its own order
    (maintainer correction 2026-10-19, paraphrased per rule 26)
     - Wait for user's affirmative response ("yes", "proceed", "execute")
   - **What requires confirmation**:
     - Creating or deleting crates/workspace members
     - Moving modules between crates
     - Changing directory structures
     - Altering Cargo.toml features (adding/removing features, deps)
     - Modifying CMake build infrastructure
     - Any multi-step todo list execution
   - **What does NOT count as confirmation**:
     - System directives (TODO CONTINUATION, SYSTEM REMINDER)
     - The word "continue" — means "continue discussing" or "continue with trivial/safe work"
     - Silence or timeout
   - **Process**: Present plan → display confirmation prompt → wait for user → execute only after explicit affirmative

2. **TDD Approach**
   - Use **tdd-guide** agent
   - Write tests first (RED)
   - Implement to pass tests (GREEN)
   - Refactor (IMPROVE)
   - Verify coverage against the staged gate (D50)

3. **Code Review**
   - Use **code-reviewer** agent immediately after writing code
   - Address CRITICAL and HIGH issues
   - Fix MEDIUM issues when possible

4. **Commit & Push**
   - Detailed commit messages following conventional commits format (see below)
   - Follow the Pull Request Workflow (see below)

5. **Pre-Review Checks**
   - Verify all automated checks (CI/CD) are passing
   - Resolve any merge conflicts
   - Ensure branch is up to date with target branch
   - Only request review after these checks pass

## Commit Message Format
```
<type>: <description>

<optional body>
```

Types: feat, fix, refactor, docs, test, chore, perf, ci

Note: the binding commit shape for ModuKit is `.agents/memorys/conventions.md` C3
(type(scope): subject <= 60 chars, ` - ` body bullets <= 100 chars); the portable
block above is the general form.

## Pull Request Workflow

When creating PRs:
1. Analyze full commit history (not just latest commit)
2. Use `git diff [base-branch]...HEAD` to see all changes
3. Draft comprehensive PR summary
4. Include test plan with TODOs
5. Push with `-u` flag if new branch

## Execution discipline for improvement batches (shadcn/improve pattern)

Ported from the MediaServo donor (2026-11-23, MediaServo scan @ f589e8f4;
originally adopted there from shadcn/improve). Three pieces, all prose - they
govern HOW a plan is executed, not what the gates check:

1. **Weakest-executor plan standard**: a plan should name the least powerful
   mechanism that can do the job. Prefer an existing gate, then a script, then
   a rule, then a new process - never reach for a heavier institution first.
2. **Reconcile-before-start** (four questions before touching anything): does
   the plan still match the repo's live state? which gates cover the touched
   surface? what did the previous audit round say about this area? is anything
   already landed that the plan does not know about?
3. **Bounded send-back**: a reviewer may (a) approve, or (b) send back at most
   twice with concrete findings, or (c) block and demand the plan be refined.
   Round 3 is not a review round - it is a plan defect, and the plan goes back
   to the drawing stage.

## Ignore-rule locality

Runtime artifacts are ignored **where they are produced**: a subdirectory that
generates lockfiles, caches, or harness output carries its own local
`.gitignore`; the root `.gitignore` keeps only anchored entries and territory
declarations. An unanchored pattern in the root file can swallow same-named
tracked files anywhere in the tree (the MediaServo donor nearly lost its root
`package-lock.json` to exactly this shape, 2026-11-23). When adding ignore
rules, ask: which directory produces this? The `check-leaks.sh` gate (tracked
intersected with ignore rules) is the backstop if the answer goes wrong.
