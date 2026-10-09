---
name: doc-audit
description: "Audit this repository's documentation and agent system for self-consistency: decision liveness, cross-document contradictions, gap coverage, and phase accuracy. Checks the whitepaper, the architecture baseline, the modules reference, the root documents, and the C/D/PIT memory numbering, and runs as a five-dimension parallel audit with interactive per-finding confirmation. Use when the user says 'audit the docs', 'doc-audit', 'check documentation consistency', 'are the decisions still live', 'find documentation gaps', or before a phase transition."
---

# Document & Architecture Audit

> **Port note (2026-10-09)**: this skill was copied from PolyOrch and its audit dimensions
> target PolyOrch documents (`whitepaper.md`, C1-C6, D1-D29, `docs/modules/`) that do not
> exist in ModuKit yet. Re-scope before running; see `.agents/memorys/status.md` Open Items.

A full audit of this repository's document system: cross-document consistency, decision
validation, agent infrastructure, gap coverage, and phase accuracy.

**Philosophy**: an audit is not fault-finding, it is debt collection. Documentation debt is
as dangerous as code debt — and in a repository whose only deliverable *is* documentation,
it is the only debt there is.

---

## Entry: audit types

### `/doc-audit` (no arguments)

Present the audit-type menu:

```
[1] Full audit      -- all five dimensions (default)
[2] Decision check  -- are D1-D9 still live and still reflected?
[3] Consistency     -- whitepaper <-> architecture <-> modules <-> root docs
[4] Agent system    -- .agents/ rules, skills, memory, and config self-consistency
[5] Gap scan        -- coverage of unresolved decisions, phases, and missing artifacts
[6] Phase audit     -- status.md against the real state of the repository
```

### `/doc-audit full`

Start the full audit directly, skipping the menu. Equivalent to `[1]`.

### `/doc-audit quick-fix`

Check LOW and MEDIUM findings only, fix them automatically, and skip interactive review.

---

## The five dimensions

### 1. Decision validation

Check that `decisions.md` D1-D9 are actually reflected in the architecture baseline, the
module reference, and the status record.

**Core questions**:

- Is each decision's conclusion reflected in `docs/architecture.md` and `docs/modules/`?
- Is there anywhere that "the decision says A but the document says B"?
- Do any decisions carry references that have gone stale (a moved file, a renumbered
  section, a renamed skill)?
- **Decision freshness**: has this decision been superseded by a later one without saying so?
  Is a stated version, date, or external fact still true? (This repository carries two
  recorded instances of an unsupported claim reaching an authoritative document — PIT-2.)

### 2. Document consistency

Check cross-consistency across `docs/whitepaper.md`, `docs/architecture.md`,
`docs/modules/`, `docs/README.md`, the root `README.md`, and `AGENTS.md`.

**Core questions**:

- Is the same concept described the same way everywhere? (file counts, the phase label, the
  chosen stack, the count of skills)
- Does a module or root document duplicate the whitepaper instead of deriving from it?
  Remember C2: a derived document must not introduce facts the whitepaper does not state.
- **Is the C2 classification still complete?** Every document under `docs/` is derived,
  design, or externally sourced, and the set is enumerated by the C2 check. A new document
  that slipped in unclassified is a HIGH finding.
- Do the root documents (`README.md`, `SKILL.md`) agree with `docs/README.md` rather than
  restating it differently?
- Are the two READMEs still doing different jobs — the root one a repository front door, the
  `docs/` one the documentation hub?
- Is every link relative (`./x.md`) and resolving? (Gate C3.)

### 3. Agent system audit

Check self-consistency across `.agents/rules/`, `.agents/memorys/`, `.agents/skills/`, and
`.opencode/opencode.json`.

**Core questions**:

- `conventions.md` — is the `C{n}` numbering contiguous, and does every `C{n}` cross-reference
  resolve? Does every convention still carry a *runnable* check, as C0 demands?
- `decisions.md` — is `D{n}` contiguous and in order, and does every `D{n}` reference resolve
  to a file and section that still exist?
- `pitfalls.md` — is `PIT-{n}` contiguous, and is every cross-reference complete?
- **Does `SKILL.md` (the skills registry) match the actual inventory?** Count the directories
  under `.agents/skills/` and compare against what the registry claims. A registry that has
  drifted from reality is a HIGH finding.
- Does every `.agents/skills/*/SKILL.md` carry complete frontmatter — a `name` and a
  `description` — and does `name` equal its directory name? (The loader requires both.)
- Does `.opencode/opencode.json` → `instructions[]` still resolve every entry?
- **Does the vendored third-party set still verify?** Gate C5 checks the 58 `xmake-*` /
  `xrepo-*` skills against `XMAKE-MANIFEST.sha256`. A modified vendored file is a HIGH finding:
  it is an unstated modification under Apache-2.0 section 4(b) unless recorded in
  `XMAKE-ATTRIBUTION.md`.
- Is `XMAKE-ATTRIBUTION.md` still accurate as to the pinned commit and the category map?

### 4. Gap scan

Scan for documents or design sections that should exist and do not.

**Core questions**:

- Does `docs/architecture.md` → Open Decision Points contain items that are now decided but
  not closed, or open items with no owner and no next step?
- Are there claims in `docs/whitepaper.md` with no supporting evidence, still unresolved?
- Is anything marked `TBD` that has since been decided? (This repository resolves unknowns by
  marking them `TBD`, never by guessing — so a stale `TBD` is a real finding.)
- Does `scripts/` still sit empty while `conventions.md` carries runnable gates that would be
  better collected there?
- Does the reproducibility invariant (D4 — anything affecting the build result is pinned in
  the repository) have any remaining violation?
- Is the phase's stated deliverable actually complete?

### 5. Phase audit

Check the declared phase against the real state of the repository.

**Core questions**:

- Does the `Phase` line in `status.md` match what the repository actually contains?
- Are the dated entries in the `status.md` Verification table still true when re-run today?
- Are the `Open Items` accurate — is anything still listed as open that has been resolved, or
  marked resolved that is not?
- Does the memory's description of the git state match reality? Run `git log --oneline -3` and
  `git status --short` and compare against what `AGENTS.md` and `status.md` claim. This repository
  has already carried a false git claim once (`AGENTS.md` said the tree was "everything untracked"
  while 219 of 223 entries were staged, which would have made `git clean -fd` look safe), so treat
  the git state as something to verify, never to infer from memory.
- Does a document claim a completion state that contradicts another document?

---

## Audit modes

### A. Team mode (recommended for a full audit)

Three or more large documents → `team_create` with 4-6 parallel members.

```
team_create(inline_spec={
  name: "doc-audit",
  members: [
    { name: "decision-validator",  category: "deep", prompt: "<dimension 1 core questions>" },
    { name: "consistency-checker", category: "deep", prompt: "<dimension 2 core questions>" },
    { name: "agent-auditor",       category: "deep", prompt: "<dimension 3 core questions>" },
    { name: "gap-scanner",         category: "deep", prompt: "<dimension 4 core questions>" }
  ]
})
```

**Conductor rules**:

- On start, tell the user: "starting N parallel audits, roughly 3-5 minutes".
- Until every member is done, do only non-overlapping work — never re-run a search a member
  is already running.
- When all are done: **deduplicate and merge**. The same problem found by two or more
  dimensions becomes one finding.
- Sort by severity: CRITICAL → HIGH → MEDIUM → LOW.
- Timeout: a member producing nothing after 10 minutes is labelled "timed out", not silently
  dropped.
- Conflict: dimension A says X and dimension B says Y → mark it for human review rather than
  picking a side.
- Close the team when every task is terminal — do not leave members running.

### B. Background-agent mode (light audit)

Few documents → `task(category="deep", run_in_background=true)` × N in parallel.

### C. Single-thread mode

Very small scope → inspect directly with Read and Grep. Do not spawn subagents.

---

## Interactive review: finding format

**Review item by item**, presenting each with the `question()` tool.

```markdown
## [severity] [number]: [title]

### Detail
| Source | Location | Content |
|--------|----------|---------|
| Document A | line X | ... |
| Document B | line Y | ... |

### Options
| Option | Pro | Con |
|--------|-----|-----|
| A. [name] | ... | ... |
| B. [name] | ... | ... |

### Recommendation
[Option X]. [Reasoning.]
```

Options: adopt the recommendation / choose another option / do not act / custom.
Progress: `[item N of M]`.

---

## Workflow

### Phase 1: Start
1. Confirm the audit scope and type.
2. Choose the mode (team / background / single-thread).
3. Report: "starting N parallel audits".

### Phase 2: Merge
1. Deduplicate — the same problem from several sources is merged and annotated.
2. Sort — CRITICAL → HIGH → MEDIUM → LOW.
3. Cross-corroborate — a finding two or more dimensions agree on is raised in priority.

### Phase 3: Interactive review
Item-by-item review, confirmed through `question()`.

### Phase 4: Fix
1. Create a todo list.
2. Work in dependency order: decisions first, then documents, then the status record.
3. Verify after every edit — re-run the affected gate from `conventions.md`.

### Phase 5: Report
```
Audit complete -- [date]
Audit type: [full / decision / consistency / agent / gap / phase]
Findings: N | fixed: M | not actioned: K
Suggested next: [the area with the highest density of findings]
```

---

## Severity standard

| Severity | Trigger | Blocks? |
|--------|---------|:---:|
| CRITICAL | A document contradiction that would cause a wrong implementation, a reversed decision, or a missing core contract | yes |
| HIGH | A stale reference, an ambiguous phase label, a duplicated document, a numbering gap, a drifted registry, or a modified vendored file | warn |
| MEDIUM | A wording difference, a conflicting example, or a missing artifact that does not block the current phase | no |
| LOW | A formatting inconsistency, a missing reference, or a `TBD` awaiting confirmation | no |

---

## Suggested audit frequency

- After any `D{n}` decision changes: `/doc-audit decisions`
- Before a phase transition: `/doc-audit full`
- During a long working stretch: `/doc-audit full` weekly
- After any large documentation change: `/doc-audit consistency`

---

## Community references

| Precedent | Pattern borrowed |
|------|-----------|
| [large-codebase-audit](https://github.com/MJWNA/large-codebase-audit-skill) | 9-surface AI-layer audit, aligned with Anthropic best practice |
| [claude-ecosystem](https://github.com/melodic-software/claude-code-plugins) | Meta-skill architecture, 16 audit agents |
| [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit) | Two-tier design, 13 checks, automatic escalation |

---

## Division of labour with `ecosystem-scan`

| Dimension | ecosystem-scan | doc-audit |
|------|:---:|:---:|
| Agent infrastructure (skills / rules / MCP) | specialist | dimension 3 |
| External community comparison | core | no |
| Internal document consistency | no | specialist |
| Decision validation | no | specialist |
| Security gate | full mode only | no |
