# Agent Governance Borrow Plan (v4)

> STATUS: PLAN OF RECORD, 2026-11-04. This plan turns the MediaServo /
> VisiaEngine / remote_control_host_computer governance review into a bounded
> work queue. The donors are read-only reference checkouts under `.refinfo/`;
> they are not sources of product memory, decisions, or history. English only
> (C1); no circled numerals (C4).
>
> Scope: agent governance only - rules, skills, memory, MCP, pixi tasks,
> scripts, gates, and CLI ergonomics. Functional code, contracts, ABI, wire,
> bindings, and packaging are OUT.
>
> D110 is already closed and the tree is green. This plan does not mint a new
> decision. If GB-2 changes the memory/commit contract, the next free decision
> record is D111 and it opens only on the maintainer's word.
>
> v4 folds the second expert-panel review: GB-1 prerequisites, GB-2 D67-safe
> high-water, GB-8 host-only extras, GB-9 trivial tier, GB-10 filename, GB-11
> submodule correction, and GB-12 provenance hygiene.
>
> EXECUTION STATUS (2026-11-04): P1 GB-8, GB-1, GB-7, and GB-2 phase A are
> implemented. P2 GB-3 report-only, GB-9 expert panel/trigger matrix, and GB-10
> communication rule are implemented. Focused gates, 7 generator tests, 42-skill
> lint, and the 36-leg `ci` are green. GB-2 remains report-only until the first
> real `child:` reference lands; GB-3 remains report-only pending noise review.

## 0. Why this plan

The three donors solve different problems:

- MediaServo is the mature engineering shell: multi-language rule packs,
  parent/child agent generation, memory ownership, and a tested agent toolchain.
- VisiaEngine is the governance mother lode: most project skills, audit loops,
  generated host layers, prose gates, and memory digests.
- remote_control_host_computer is the multi-project parent: tool routing, MCP
  orphan prevention, expert-panel review, and multi-project memory lessons.

ModuKit has already absorbed most of their content. The remaining value is in a
few mechanisms, not in bulk copying. This plan keeps that distinction explicit
so a later session does not re-import 30 skills or 200 KB of donor memory.

## 1. Source snapshot (read-only evidence)

| Surface | MediaServo | VisiaEngine | remote_control | ModuKit current |
|---|---|---|---|---|
| Commits at review | 1136 | 576 | 552 (9+ authors) | D110 closed |
| Skills | 33 | 37 | 29 (7 inoperative) | 41 |
| Rule files | 82 (14 language packs) | 41 (6 language packs) | 84 (14 language packs) | 81 (14 language packs) |
| Memory size | about 580 KB | about 370 KB | about 118 KB | about 100 KB incl. history |
| Agent tooling | 24 files under `scripts/agents/` | per-layer generators under `scripts/` | 27 files under `scripts/` | split across `scripts/` |
| MCP | 3 local servers + lifecycle | 3 local servers + disabled candidates | 3 local servers + full rewrite | 3 local servers + merge |
| pixi | large toolchain, multi-env | 17 KB, env/task mature | 1.9 KB, 5-gate ci | 36 tasks, cov env |
| CLI | `shell_entry.sh`, dual context | thin `visiaengine.sh` | `repo.sh`, no root build | thin `modukit.sh` passthrough |

The review did not find a missing skill category. ModuKit has more skills than
any donor. MediaServo's unique `lsp-code-analysis` and `weaknet-testing` need
adaptation; remote_control's Qt skills and `expert-panel-review` are the other
unique surface. None is a reason to bulk import.

## 2. What is already covered

Do not re-port these:

- `.agents/` single source plus generated host layers;
- `agents sync` / `agents check` / `gate-agents`;
- `rules-links`, `skills-links`, `mcp-links`, MCP single source;
- 14 language rule packs, including the MediaServo common set;
- `repo-audit` with its 2C practice-scan lane and pattern cards;
- `doc-audit`, `skill-authoring`, `skill-router`, `subagent-orchestration`;
- memory digest windows and the AGENTS hub budget;
- book gates, SDD trace, capability ledger, check.sh legs.

The review's key negative finding is also part of the record: none of the three
donors has a mechanism worth copying wholesale that ModuKit lacks today. The
remaining candidates are small, bounded, and named below.

## 3. Candidate portfolio

| ID | Candidate | Tier | Source | Why it matters |
|---|---|---|---|---|
| GB-1 | Agent generator self-tests | P1 | MediaServo `scripts/agents/test_*.py` | ModuKit's sync/check tools are untested |
| GB-2 | Cross-repo memory ownership, phase A | P1 | MediaServo `check_memory_ownership.py` + C52 | Root and PolyOrch both carry memory; only prose prevents duplication |
| GB-7 | Tool-routing matrix + rule lock | P1 | remote_control `common/tool-routing.md` | ModuKit has no rule for grep/LSP vs codegraph |
| GB-8 | MCP orphan prevention: full rewrite + host-only extras | P1 | remote_control D20 / PIT-248 | ModuKit `mcp-links.py` still merges and ignores extra keys |
| GB-3 | Prose gate for human-facing docs | P2 | VisiaEngine `gate-prose.py` + prose-banned | Fills the one doc-quality layer ModuKit deferred |
| GB-9 | Expert-panel review engine | P2 | remote_control `expert-panel-review` | Fills the plan/decision review gap, but must not duplicate `common/agents.md` |
| GB-10 | Communication / Grice rule | P2 | remote_control `common/communication.md` | Donor-adjacent new content; must be loaded by at least one host |
| GB-4 | MCP pin gate + environment doctor | P3 | MediaServo `doctor`, remote_control `repo.sh` env guards | `gate-mcp` does not compare versions; pin drift needs a gate |
| GB-5 | Portable agent layout + no-parent guard | P3 trigger | MediaServo `scripts/agents/`, `check_no_parent.py` | Only matters if `.agents` becomes parent-consumed |
| GB-6 | Unique skills and pattern-card persistence | P3 reference | MediaServo `lsp-code-analysis`, `weaknet-testing`; VE 2C; remote_control Qt skills | 2C is already adopted; `lsp-code-analysis` is absorbed into GB-7 |
| GB-11 | Multi-project memory evidence | P3 reference | remote_control root + in-tree + submodule `.agents` trees | Evidence for GB-2; no new implementation |
| GB-12 | Donor-number hygiene in VE-derived scripts | P3 | VisiaEngine script comments already in tree | Bare `D47`/`C17` comments can be mistaken for local decisions |

Tier meaning:

- P1: do next; no product code; one bounded change each.
- P2: do after P1, with a report-only or trial phase first.
- P3: keep as reference until a named trigger fires.
- REJECT: never import as-is; listed in section 5.

## 4. Workstreams

### GB-1: Agent generator self-tests (P1)

> LANDED 2026-11-04: explicit test-mode root override, file-level write guard,
> two-phase MCP writes, 7 behavior tests, and `gate-agents` now runs those
> tests plus the report-only ownership check without adding a new check.sh leg.

**Goal**: make `agents sync` / `agents check` behave like software, not a shell
ritual.

**Source pattern**: MediaServo `test_cli_behavior.py`, `test_dual_context.py`,
`test_common.py`, `tools-check.sh`. Reuse the scenarios, not the donor code.

**Proposed tasks**:

1. Add the test prerequisites in `agent_common` / `agent-links`:
   - a test-only `MODUKIT_AGENT_ROOT` override, gated by
     `MODUKIT_AGENT_TESTS=1`, so an accidentally exported root variable cannot
     redirect normal gates;
   - a shared target write guard that returns 2 and never leaves a partial
     write on a read-only or symlinked target.
2. Add `scripts/tests/test_agent_config.py` using stdlib `unittest`.
3. Cover the drift cycle in a temporary target root:
   - missing layer -> `check` exits 1;
   - `sync` -> `check` exits 0;
   - hand edit a generated file -> `check` exits 1;
   - `sync` repairs it -> `check` exits 0.
4. Cover `--dry-run`: no file is written, and a second sync reports zero writes.
5. Cover guard behavior: writing inside the canonical tree exits 2 and writes
   nothing; a read-only target exits 2 without a partial write.
6. Cover two different target roots producing identical MCP content with no
   absolute paths.
7. Cover the opencode branch explicitly: ghost server removal, `websearch`
   disabled/remote preservation, and full-object equality. This is GB-8's
   regression test.
8. Cover copy mode drift, if the Windows `--copy` path is enabled.
9. Run the generator's syntax checks (py_compile or ruff if available) as part
   of the same task, not a separate gate.
10. Add `pixi run agent-tests`, and run the same suite from `gate-agents`.
    The tests invoke `mcp-links.py` directly rather than `agents-cli check`,
    so the command chain does not recurse.
11. Keep the 36-leg count stable unless the maintainer accepts a new leg.

**Acceptance**:

- tests pass under `pixi run agent-tests`;
- `pixi run gate-agents` and `pixi run ci` green;
- one planted drift and one ghost opencode server are observed red before the
  test is trusted;
- no product code, contracts, bindings, or donor memory touched.

**Rollback**: revert the test task and test file; no runtime effect.

### GB-2: Cross-repo memory ownership, phase A (P1)

> LANDED 2026-11-04 as report-only: `pixi run memory-ownership`, runtime
> PolyOrch high water, synthetic self-test, no stored child ledger. Current
> citations: 0.

**Goal**: stop ModuKit and `PolyOrch/.agents/memorys` from recording the same
fact twice or citing a memory id that does not exist.

**Source pattern**: MediaServo C52 and `check_memory_ownership.py`. Adapt the
reference and duplicate-title checks, and derive high water at gate time. Do not
copy the 58 KiB budget or the archive machinery.

**D67 constraint**: the PolyOrch gitlink is the single version authority. Do not
add a `child-refs.md` mirror or any stored high-water table; that is the
secondary ledger D67 forbids. Read the child ledger at gate time and compute the
high water from it.

**Proposed tasks**:

1. Add `scripts/check-memory-ownership.py` with a self-test.
2. Parse root `.agents/memorys` for explicit `child:D<n>`, `child:PIT-<n>`, and
   `child:C<n>` citations; every citation must resolve in the child ledger.
3. Normalize section titles and reject parent/child duplicates.
4. Compute PolyOrch high water at runtime from its `decisions.md`,
   `pitfalls.md`, and `conventions.md`; report it in the gate output. There is no
   stored copy and no refresh step.
5. Treat `PolyOrch/.agents/memorys` as read-only. The gate never writes there.
6. Run phase A report-only until the first real `child:` reference exists. A
   gate that exits green because the tree has no such references is not proof;
   record that state explicitly.
7. Keep `Memory: none (<reason>)` as the current commit convention. Do not
   introduce `parent|child|both` until phase B is explicitly adopted.
8. Add a `pixi run memory-ownership` task and call it from `gate-agents` or a
   single existing gate. Do not add a new check.sh leg.

**Acceptance**:

- self-test covers a missing citation, a duplicate title, and runtime
  high-water derivation;
- a planted `child:D999` citation is observed in report output;
- phase A reports the current "no child references yet" state instead of
  claiming the tree is proven;
- an absent/uninitialized child root is an explicit SKIP, not a red;
- `pixi run ci` green;
- no writes to `PolyOrch` and no donor memory copied.

**Decision gate**: phase B (`Memory: parent|child|both`) is a commit-contract
change. It needs a maintainer word and a D111 record before implementation.

**Rollback**: remove the script and task; no memory data changes.

### GB-3: Prose gate for human-facing docs (P2)

> LANDED 2026-11-04 as report-only: `pixi run prose-report`, polish fields,
> banned-phrase config, type/anchor checks, and reference-page structure
> exemption. Baseline: 8 candidate findings; planted banned phrases observed.

**Goal**: catch prose that sounds like verification without being verifiable,
and keep book chapters doing the job their type claims.

**Source pattern**: VisiaEngine `gate-prose.py`, `docs/prose-banned.toml`,
`.markdownlint.json`. Start with a narrow report-only pass, not a repository-wide
hard gate.

**Proposed tasks**:

1. Copy the rule shape, not the donor text: banned-phrase list, chapter-type
   checks, paragraph length, heading length, real `file:line` anchors.
2. Add a `polish` field to `book/toc.toml` before using a polish-only rule:
   ModuKit currently has `type` but no `polish`, so a polish-only first pass
   would select zero chapters.
3. Define the split explicitly: type/anchor checks cover all chapters; banned
   and paragraph/heading checks cover `polish = true` chapters plus
   `docs/architecture.md`.
4. Run report-only for one batch and record false positives.
5. If the noise is acceptable, fold the check into `gate-docs` or a small
   `gate-prose` task. Do not add a new check.sh leg until it has zero false
   positives.
6. Keep internal plans and memory files out of the prose gate.
7. ModuKit already pins `markdownlint-cli` and defines a `markdownlint` task.
   Decide whether the prose gate calls it; do not add a second dependency. If
   the binary is absent, report an explicit SKIP, not a silent green.

**Acceptance**:

- a banned phrase is observed red in a scratch chapter;
- restoring it is green;
- no false positive on the current book;
- `gate-docs` / `ci` green.

**Rollback**: remove the task or disable the config; no code impact.

### GB-4: MCP pin gate and environment doctor (P3)

**Goal**: close the pin-drift gap and diagnose the local MCP environment before
a host restart fails.

**Source pattern**: MediaServo `doctor`; remote_control `repo.sh` PATH, proxy,
and TMPDIR guards. `mcp-cli.py status` prints pin vs installed but always exits
0; `gate-mcp` currently probes presence and registry names, not versions. Do not
add a second `pins` command.

**Proposed tasks**:

1. Add one shared pin-comparison helper (or reuse the one in `mcp-cli.py`) and
   call it from `gate-mcp.py`; a wrong pin must red the gate.
2. Add `modukit.sh mcp doctor` for pixi availability, launcher paths, config
   shape, the three server names, GUI PATH inheritance, `http_proxy` vs
   `no_proxy` for localhost, and TMPDIR writability.
3. Reuse `.agents/mcp.json` and `scripts/mcp-servers.json`; do not create a
   second server registry.
4. Keep doctor read-only and always exit 0. A broken environment must still
   print its diagnosis; pin drift is the gate's job.

**Acceptance**:

- a deliberately wrong pin is red in `gate-mcp`;
- a missing launcher, a proxy that lacks `127.0.0.1`, and an unwritable TMPDIR
  are each reported by doctor with a readable diagnostic;
- the clean doctor path exits 0;
- no duplicate pin logic is added.

**Rollback**: remove the doctor subcommand and the gate hook; no config change.

### GB-5: Portable agent layout and no-parent guard (P3 trigger)

Do not do this now.

**Trigger**: `.agents/` becomes consumed by a parent workspace, or the agent
scripts outgrow the root `scripts/` directory.

Then borrow the MediaServo shape:

- `scripts/agents/` as the tool home;
- `check_no_parent.py` plus an allowlist;
- a parent/child target root with no absolute paths;
- `install-hooks.sh` using `git rev-parse --git-path hooks`.

The current thin `modukit.sh` passthrough is better than MediaServo's 94 KB
generated shell and stays.

### GB-6: Reference-only candidates (P3)

- `lsp-code-analysis`: absorbed into GB-7's routing matrix; do not add a second
  standalone skill.
- `weaknet-testing`: MediaServo/mediasoup domain-specific; do not port.
- 2C pattern cards: already in ModuKit `repo-audit`; keep using them.
- memory digest windows: already covered; do not import the donor script.
- Qt skills: remote_control-specific; no Qt surface in ModuKit.
- Change-Id gate: Gerrit-specific; borrow only the "hook is best effort, gate
  is enforcement" lesson if a similar gap appears.

### GB-7: Tool-routing matrix and rule lock (P1)

> LANDED 2026-11-04: rule, CodeBuddy always-on registration, opencode
> instruction registration, and the two-way server/rule plus membership lock.

**Goal**: make the agent choose one search path instead of duplicating work, and
make the codegraph server and its routing rule impossible to drift apart.

**Source pattern**: remote_control `common/tool-routing.md` and its `gate-mcp`
rule lock.

**Proposed tasks**:

1. Add `.agents/rules/common/tool-routing.md` adapted to ModuKit:
   - first line: this defines a division of labour, not a priority order;
   - host-native search instructions are not overridden;
   - exact text, known symbol, exact reference -> native grep or LSP;
   - semantic exploration, call chain, impact surface, unfamiliar survey ->
     `codegraph_explore`;
   - when both can answer, take one path and finish it; do not open two;
   - codegraph's returned source counts as read; do not re-read the same file;
   - an absent index is a graceful fallback, not a retry loop;
   - carry the donor's degradation notes: an index can miss macro/generated
     items, and a text search is a superset; do not claim "no references" from
     one tool alone.
2. Make this rule **always-on** (the donor's evidence is that an unloaded rule
   is an unused rule). Register it in `scripts/agent-targets.json`
   (`always_on_rules` / `rule_descriptions`) and in the hand-maintained
   `.opencode/opencode.json` `instructions[]` if opencode is a target. Measure
   the 1.6 KB cost against the host instruction budget, not the AGENTS hub
   budget; drop another on-demand rule if that budget cannot absorb it.
3. Add a two-way `gate-mcp` rule lock:
   - server present -> rule file must exist;
   - rule file present -> server must exist (a removed server with a left-behind
     rule is also red);
   - neither present is clean.
4. Add a membership check that the rule is actually in at least one host's
   injection list; a rule that exists but is never loaded is a dead file.
5. If the host does not inject the MCP server, the rule degrades to the native
   path and must not make the host fail.

**Acceptance**:

- removing the rule while the server exists is red;
- removing the server while the rule exists is also red;
- restoring the matched pair is green;
- `agents sync`, `gate-mcp`, and `ci` green;
- the rule appears in a registered host instruction list, not only on disk;
- always-on cost is recorded against the host budget, not assumed.

**Rollback**: remove the rule and the lock; no runtime change.

### GB-8: MCP orphan prevention: full rewrite (P1)

> LANDED 2026-11-04: opencode full-object check/rewrite, host-only remote
> allowlist, leak test, and idempotent second sync.

**Goal**: close the current ModuKit gap where the opencode `mcp{}` section is
merged, so a server removed from the canonical source can survive forever.

**Source pattern**: remote_control D20 and PIT-248. MediaServo is not a pure
merge either: its `cli.py` merges and also applies an explicit `drop_mcp`
manual-removal list. ModuKit has neither the full rewrite nor a removal channel.
`scripts/mcp-links.py` today:

```text
sync:  cur.update(want)
check: only expected keys are compared
```

**Chosen shape**: keep `.agents/mcp.json` local-only and canonical. Add a small
`opencode_extra_mcp` allowlist for host-only remote/disabled entries such as
`websearch`. Do not add remote entries to `scripts/mcp-servers.json`; it remains
the local-server registry. Do not repeat the enabled/visible rule in three
scripts (PIT-247): one helper in `mcp-links.py` builds the opencode expected
set.

**Proposed tasks**:

1. Before the rewrite, teach `_expected_opencode_mcp` the shapes it must
   preserve:
   - `enabled` is passed through, not hardcoded `True`;
   - `type: remote` plus `url` renders as an opencode remote entry, not a
     dead `type: local` with an empty command.
2. Add the `opencode_extra_mcp` allowlist for the current `websearch` entry.
   The local target `.mcp.json` must never receive an allowlisted extra.
3. Verify the fourth consumer, `scripts/mcp-servers.json`: it stays the local
   registry; no remote entry is added. `mcp-file-guard.py` and `gate-mcp.py`
   keep validating the local set, with a test that extras cannot leak into
   `.mcp.json`.
4. Change the opencode branch to **full rewrite**:
   - `check`: `cur != want` (an extra key is red, not ignored);
   - `sync`: `doc["mcp"] = want` after a backup.
5. Add red proofs:
   - ghost server: inject extra entry, `check` red, `sync` removes it, second
     `sync` writes nothing;
   - disabled remote: `websearch` remains disabled after rewrite;
   - leak: an allowlisted extra never appears in `.mcp.json`;
   - shape: an array `command` or `{env:VAR}` mixed with a string command is
     red in `mcp-file-guard`.

**Acceptance**:

- ghost server red observed, then removed and green;
- `websearch` (or its allowlisted replacement) survives the first rewrite;
- allowlisted extras stay out of the project `.mcp.json`;
- `mcp-servers.json` has no remote entry and remains registry-clean;
- `gate-mcp`, `mcp-file-guard`, `gate-agents`, and `ci` green;
- no intentional canonical server is lost;
- `agents sync` is idempotent.

**Rollback**: revert the `mcp-links.py` change and remove the allowlist; the
canonical local source is unchanged.

### GB-9: Expert-panel review engine (P2)

> LANDED 2026-11-04: skill, complexity/isolation protocol, TRIGGERS matrix, and
> skill-lint duplicate-trigger check. Trigger red proof passed.

**Goal**: add a reusable way to review a plan, conclusion, or decision from
independent lenses without producing "the same model copied N times".

**Source pattern**: remote_control `expert-panel-review`. Adapt the execution
model, not the Chinese prose.

**Proposed tasks**:

1. First resolve the existing overlap: `.agents/rules/common/agents.md` already
   has a five-role "Multi-Perspective Analysis" block. Decide explicitly whether
   GB-9 replaces that block, extends it, or points to the new skill. Do not add a
   second parallel review entry point.
2. English adaptation with two fixed core roles (correctness, evidence /
   devil's advocate) plus one to three scenario lenses.
3. Add the complexity gate:
   - trivial item -> core 2 roles one pass, or route to `think-before-act`;
   - standard item -> 3 roles;
   - deep item -> 4-5 roles;
   - hard cap at 5; record why the cap was hit.
4. Support both execution tracks:
   - parallel when the host has team/subagent tools;
   - sequential isolation when it does not, with the weaker isolation declared
     and not described as independent verification.
5. Lock the isolation discipline:
   - each pass sees only artifact + lens + shared acceptance criteria;
   - no drafts or conclusions shared across roles before the merge;
   - each role freezes conclusion, evidence, confidence, and uncovered items;
   - members are read-only and must not write git;
   - declare the roster before the run and let the user add/remove; fix three to
     five acceptance criteria before the run.
6. Add a `.agents/skills/TRIGGERS.md` boundary matrix adapted from MediaServo
   (not remote_control; that repo has no such file). Add one row per review
   entry point and extend `skill-lint.py` with duplicate-trigger detection.
7. Route explicitly against `think-before-act`, `doc-audit`, `repo-audit`,
   `adjudication-walkthrough`, and the existing `common/agents.md` block.
8. Do not edit the existing review skills until a pilot proves the routing.

**Acceptance**:

- a trivial item yields the core-2 one-pass path or routes to
  `think-before-act`, not a full panel;
- pilot on one plan or decision produces independent findings and a preserved
  disagreement section;
- a planted duplicate trigger is observed red in `skill-lint`;
- no collision with existing audit/review triggers;
- `skill-lint` and `ci` green;
- a route-boundary row exists and is checked.

**Rollback**: remove the skill, the trigger row, and the skill-lint addition;
existing skills stay untouched.

### GB-10: Communication / Grice rule (P2, donor-adjacent new content)

> LANDED 2026-11-04: CodeBuddy on-demand rule registered; missing generated
> RULE.mdc observed red and restored.

**Goal**: make the project-level communication contract explicit, not only a
host system-prompt preference.

**Source pattern**: remote_control `.agents/rules/common/communication.md` is
donor-adjacent, not directly portable: it is 359 B, has no Grice reference, and
is absent from the donor's opencode `instructions[]`, so it cannot prove a
loaded rule. The four checks below are new content; the donor's best surviving
bullet is the anti-formalism exception.

**Proposed tasks**:

1. Add a short `.agents/rules/common/communication.md` with four checks:
   - quantity: answer the asked scope, no background dump;
   - quality: verified claims only; mark uncertainty;
   - relation: answer the actual question; separate out-of-scope risks;
   - manner: conclusion first, one term per concept, no ambiguous references.
2. Add four explicit exceptions: safety-critical warning, unverified content,
   relevant adjacent risk, and "allow deliberate human tone; do not turn every
   reply into a table".
3. Register it explicitly in `scripts/agent-targets.json`
   (`always_on_rules` / `rule_descriptions`) and, if opencode should load it,
   the hand-maintained `.opencode/opencode.json` `instructions[]`. The latter
   has no generator; do not call it generated.
4. Make it loaded by at least one host layer. Prefer CodeBuddy on-demand and a
   measured always-on entry only if the host budget allows; a rule no host loads
   is a dead file.
5. Do not add a semantic gate for tone. The rule is a discipline, not a lint.

**Acceptance**:

- rule file exists and is actually loaded by at least one host layer;
- the registration points are updated, including the hand-maintained
  opencode `instructions[]` if used;
- no conflict with the host system prompt;
- `gate-rules` / `ci` green;
- sample plan review uses the four checks without over-brevity or forced tables.

**Rollback**: remove the rule and its registration entries; no code change.

### GB-11: Multi-project memory evidence (P3 reference)

remote_control is a parent workspace with root plus in-tree project `.agents`
trees (`src/TPTouch`, `src/ms_remote_control_station`) and four submodule
`.agents` trees (`QExt`, `OpenCTK`, `CxxKit`, `MediaServo`). The point is
multiple memory trees in one workspace, not one repo. Its history shows the
failure mode:

- numbering can collide across parallel work;
- child commits must not self-reference the parent scope;
- root and child memories can diverge silently.

Use it as evidence for GB-2. Do not add a new implementation here.

### GB-12: Donor-number hygiene in VE-derived scripts (P3)

Some VE-derived scripts still carry donor numbers in comments, for example
`scripts/memory-aggregate.py` uses `D47` for the memory digest window while
ModuKit `D47` is a different decision. This is a comment-level trap, not a
runtime bug.

Tasks:

- scan scripts for bare `D<n>`, `C<n>`, and `PIT-<n>` references copied from VE;
- rewrite them as `donor: <repo> <file>:<line>` provenance or the matching local
  id; do not invent a local `VE-PARITY (D109)` truth;
- do not touch historical memory text, only active script comments;
- add a `scripts/**` comment scan with a self-test, or mark this as a one-time
  checklist if no gate is added.

Acceptance: no bare donor number remains in active scripts; a planted donor
number is observed red; `gate-docs` / `ci` green.

## 5. Explicit non-goals

Do not import:

- MediaServo's 224 KB pitfalls, 183 KB status, or 116 KB decisions;
- VisiaEngine's decisions, pitfalls, or PIT numbers;
- remote_control's memory, Chinese artifacts, or Qt platform rules;
- 33, 37, or 29 donor skills as a batch;
- the 94 KB `mediaservo.sh` monolith;
- `weaknet-testing`, the MediaServo platform rule, or Qt skills;
- the Gerrit Change-Id gate (pattern only, not the code);
- remote_control's no-root-build boundary (ModuKit is a build workspace);
- donor model names, plugin pins, or absolute MCP paths;
- a second `.agents` target registry;
- a new host layer (`.trae`, `.claude`, `.cursor`) just because a donor has one.

## 6. Sequencing

1. Maintainer word for the P1 set: GB-1, GB-2 phase A, GB-7, GB-8.
2. GB-8 first: fix the known MCP orphan gap, including enabled/remote shapes,
   before the ghost-server rewrite.
3. GB-1 tests: cover the generator, including the new full-rewrite behavior.
4. GB-7: add the always-on tool-routing rule and its two-way server/rule lock.
5. GB-2 phase A: add reference, duplicate-title, and runtime-derived high-water
   checks; keep it report-only until a real child reference exists.
6. GB-3 report-only: measure false positives; decide whether it becomes hard.
7. GB-9: resolve the `common/agents.md` overlap, add the trigger matrix, then
   pilot the expert panel.
8. GB-10: add the communication rule only where at least one host loads it.
9. GB-4: add the pin comparison to `gate-mcp` and the doctor only if the MCP
   environment still causes friction.
10. GB-5/GB-6/GB-11/GB-12: wait for their triggers; GB-12 can ride any docs
    touch.
11. If GB-2 phase B is adopted, mint D111 and update the commit convention.

## 7. Gates and evidence

Every item needs:

```text
planted red -> observed -> restored green
pixi run ci                         36 legs green
gate-agents                         clean
git diff --check                    clean
check-memory.sh <range>             MEMORY ✓
```

GB-1 needs a test that runs the real generator against a temporary target root,
not a mock; the root override and recursion short-circuit are prerequisites.
GB-2 uses a synthetic child ledger for its red plants and never writes the real
PolyOrch tree.

GB-3 needs one scratch chapter with a known banned phrase and one real chapter
that stays green.

GB-4 needs one wrong pin observed red in `gate-mcp`, plus a missing launcher, a
proxy without localhost bypass, and an unwritable TMPDIR reported by doctor,
which still exits 0.

GB-7 needs both latch directions: remove the rule while `local-codegraph` stays
in `.mcp.json` -> red; remove the server while the rule stays -> red; matched
pair -> green. It also needs a membership red: rule on disk but absent from the
host instruction list -> red.

GB-8 needs a ghost-server red: inject an extra opencode MCP entry, observe
`check` red, `sync` removes it, second `sync` writes nothing. It also needs a
disabled/remote red: `websearch` survives the rewrite; an allowlisted extra does
not leak into `.mcp.json`; an array `command` or mixed `{env:VAR}` is red in
`mcp-file-guard`.

GB-9 needs a trivial item that takes the core-2 one-pass path or routes to
`think-before-act`, plus a pilot review that preserves at least one
disagreement; it is not a mechanical gate.

GB-10 needs evidence that at least one host loads the rule, plus one real reply
reviewed against the four checks and the anti-formalism exception; it is not a
semantic lint.

GB-12 needs both a grep proof (no bare donor `D/C/PIT` numbers remain in active
VE-derived scripts; historical memory is exempt) and a self-test that plants one
bare donor number and observes red.

## 8. Decisions and triggers

| Question | Current answer | Trigger |
|---|---|---|
| Add agent tests? | Yes, GB-1 | Maintainer word |
| Add memory ownership gate? | Phase A report-only; phase B no | D111 word |
| Fix MCP orphan merge? | Yes, GB-8 | Maintainer word |
| Add tool routing? | Yes, GB-7 always-on | Maintainer word and host budget check |
| Adopt prose gate? | Report-only first | Zero false positives and polish field |
| Add expert panel? | P2 pilot | Resolve common/agents overlap; trivial = core-2 one pass |
| Add communication rule? | P2 loaded by a host | No conflict with host prompt |
| Add MCP pin gate/doctor? | P3, only if friction remains | Pin-drift or launcher/proxy/TMPDIR incident |
| Move agent scripts? | No | `.agents` parent consumption or script growth |
| Port donor skills? | No | A named missing capability, not a donor count |
| Copy donor memory? | Never | No trigger |

## 9. Change log

- 2026-11-04 v4: second expert-panel correction: GB-1 root/guard/recursion
  prerequisites, GB-2 D67-safe runtime high-water and report-only phase,
  GB-4 pin gate + doctor, GB-8 host-only extras and fourth consumer, GB-9
  trivial tier and trigger matrix, GB-10 filename/registration, GB-11 submodule
  correction, GB-12 provenance self-test.
- 2026-11-04 v3: expert-panel review folded in: GB-2 high-water refs, GB-4
  demoted to doctor-only, GB-7 always-on/two-way lock, GB-8 enabled/remote
  shapes and three-consumer alignment, GB-9 isolation/complexity gates, GB-10
  donor-adjacent labeling, GB-11 multi-project re-attribution, GB-12 donor-number
  hygiene.
- 2026-11-04 v2: added remote_control_host_computer evidence, GB-7 tool
  routing, GB-8 MCP full rewrite, GB-9 expert panel, GB-10 communication rule,
  and GB-11 multi-project memory evidence.
- 2026-11-04 v1: plan created from the MediaServo / VisiaEngine governance
  review; P1/P2/P3 split, explicit non-goals, and D111 gate recorded.
