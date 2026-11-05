# Skill Trigger Matrix

This is a boundary map, not a skill. Before adding a skill, add its row here so
trigger overlap is explicit and machine-checked.

| Intent | Trigger phrases | Owning skill | Hand-off / non-goal |
|---|---|---|---|
| Investigate before a non-trivial action | `should we act`, `investigate before acting` | `think-before-act` | Run a review -> `expert-panel-review`; present options -> `adjudication-walkthrough` |
| Run a bounded multi-role review | `expert panel`, `team review`, `multi-role review`, `red-team this plan` | `expert-panel-review` | Whether to act -> `think-before-act`; document corpus -> `doc-audit`; item rulings -> `adjudication-walkthrough` |
| Audit a document corpus | `audit docs`, `document consistency` | `doc-audit` | One plan/conclusion -> `expert-panel-review`; durable ADR -> `documentation-and-adrs` |
| Whole-repo engineering audit | `repo audit`, `whole-repo audit` | `repo-audit` | Route a broad task -> `repo-triage`; security pass -> `security-hardening` |
| Route a broad task to a layer | `which layer`, `where is this implemented` | `repo-triage` | Choose a skill -> `skill-router`; symbol impact -> `lsp-code-analysis` |
| Reproduce and fix a failure | `reproduce failure`, `root cause failure` | `debugging-and-error-recovery` | Whether to act -> `think-before-act`; performance -> `performance-optimization` |
| Review a diff or merge verdict | `review a diff`, `merge verdict` | `code-review-and-quality` | Security deep dive -> `security-hardening`; simplify -> `code-simplification` |
| Present options one at a time | `adjudication`, `one at a time ruling` | `adjudication-walkthrough` | Multi-role review -> `expert-panel-review`; whether to act -> `think-before-act` |
| Analyse impact before editing a symbol | `who implements this`, `what breaks if I change`, `call chain impact` | `lsp-code-analysis` | Layer unknown -> `repo-triage`; contract design -> `api-interface-design`; bulk rewrite -> `ast-grep-rewrites` |

## Rules

- A phrase may have only one owning skill.
- A new skill must add its row before it is mounted.
- `TRIGGERS.md` defines boundaries; it is not a router implementation.
