---
name: doc-audit
description: "Audit this repository's documentation and agent system for self-consistency: decision liveness, cross-document contradictions, gap coverage, and phase accuracy. Checks the whitepaper, the docs/reference series, the root documents, and the C/D/PIT memory numbering, and runs as a five-dimension parallel audit with interactive per-finding confirmation. Use when the user says 'audit the docs', 'doc-audit', 'check documentation consistency', 'are the decisions still live', 'find documentation gaps', or before a phase transition."
---

# Document & Architecture Audit

> **Re-scoped for ModuKit (2026-10-09)**: audit targets are the ModuKit canon —
> `docs/whitepaper.md` (verbatim canon), root `README.md` / `AGENTS.md` / `SKILL.md`,
> `docs/reference/` (20 files: index, `00-overview.md`, profiles, audits),
> `docs/codegraph-bootstrap.md`, `.agents/memorys/` (C0-C1 / D1 / PIT-1..5),
> `.opencode/opencode.json`, `mise.toml` / `bootstrap.sh` / `bootstrap.bat`.
> PolyOrch-era targets (architecture.md, docs/modules/, C2-C6, D2-D29, vendored
> xmake set) do not exist here and are NOT audit targets.

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
[2] Decision check  -- is each D{n} in decisions.md still live and reflected?
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

Check that every `D{n}` in `decisions.md` is actually reflected in the whitepaper, the
root README, the docs/reference series where applicable, and the status record.

**Core questions**:

- Is each decision's conclusion reflected in `docs/whitepaper.md`, the root `README.md`, and the `docs/reference/` series where applicable?
- Is there anywhere that "the decision says A but the document says B"?
- Do any decisions carry references that have gone stale (a moved file, a renumbered
  section, a renamed skill)?
- **Decision freshness**: has this decision been superseded by a later one without saying so?
  Is a stated version, date, or external fact still true? (Precedent: the 2026-10-09 I420/Visia smuggled-residue check — a suspicion that proved
  false but had to be run before the term was confirmed as own design vocabulary.)

### 2. Document consistency

Check cross-consistency across `docs/whitepaper.md` (canon), the root `README.md`,
`AGENTS.md`, `SKILL.md`, `docs/reference/` (its `README.md` index and `00-overview.md`
decision matrix), and `docs/codegraph-bootstrap.md`.

**Core questions**:

- Is the same concept described the same way everywhere? (file counts, the phase label, the
  chosen stack, the count of skills)
- Does a derived document restate whitepaper facts differently or introduce new facts?
  New facts must be externally sourced and marked (the `docs/reference/` series is the
  sanctioned carrier of whitepaper-external content).
- **Is every file under `docs/` classified?** derived (whitepaper echo), design
  (codegraph-bootstrap), or external-sourced (reference series). A new unclassified file
  is a HIGH finding.
- Do the root documents (`README.md`, `SKILL.md`) agree with `docs/reference/README.md` (the series index) rather than
  restating it differently?
- Are the two READMEs still doing different jobs — the root one a repository front door, the
  `docs/` one the documentation hub?
- Is every local link resolving? (Mechanical probe: the dead-link scan from the 2026-10-09
  Full-scan run; a formal `scripts/gate.sh` link check is a Stage-1 trigger item.)

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
- **Mechanical first pass — run it before manual review:**
  `python3 scripts/skill-lint.py` must exit 0 (frontmatter, name==dir, size, @path refs,
  registry sync; fixture-negative-proved 2026-10-09).
- Do inoperative skills carry explicit registry flags in `SKILL.md`? (openspec-* x3 as of
  2026-10-09; re-verify any skill whose prerequisites appeared or vanished since.)

### 4. Gap scan

Scan for documents or design sections that should exist and do not.

**Core questions**:

- Do the open-decision registers — `docs/reference/00-overview.md` §4, `.agents/memorys/backlog.md`
  Pending, `status.md` Open Items — list anything already decided but not closed, or items
  with no owner/next step? Are the three registers mutually consistent (no item open in one,
  done in another)?
- Are there claims in `docs/whitepaper.md` with no supporting evidence, still unresolved?
- Is anything marked `TBD` or "not yet" that has since been decided? (Precedents found
  2026-10-09: `AGENTS.md` NOTES "License TBD" after D1 landed, and its generated header
  "Commit: none yet" after real commits existed — both fixed same session.)
- `scripts/` now holds `skill-lint.py` only; the C1 gate and env probes still live ad hoc in
  conventions/AGENTS text. Collecting them into `scripts/gate.sh` is a recorded Stage-1
  trigger (backlog), not a current-phase gap.
- Pin hygiene (ModuKit analogue of the build-reproducibility invariant): `mise.toml` pins
  node; plugin versions pinned in `opencode.json` (2026-10-09; superpowers git-ref floats
  by design). `Cargo.lock` rule (constraints.md) activates with the Stage-1 workspace.
- Is the phase's stated deliverable actually complete?

### 5. Phase audit

Check the declared phase against the real state of the repository.

**Core questions**:

- Does the `Phase` line in `status.md` match what the repository actually contains?
- Are the dated entries in the `status.md` Verification table still true when re-run today?
- Are the `Open Items` accurate — is anything still listed as open that has been resolved, or
  marked resolved that is not?
- Does the memory's description of the git state match reality? Run `git log --oneline -3` and
  `git status --short` and compare against what `AGENTS.md` and `status.md` claim. This repository has
  already carried stale generated-header claims (an `AGENTS.md` "Commit: none yet" line that
  survived the first real commits), so treat the git state as something to verify against
  `git log` / `git status`, never to infer from memory.
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
