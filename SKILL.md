# ModuKit Skills Registry

## Superpowers

General-purpose skills loaded through the `superpowers` plugin; available across all projects (brainstorming, writing-plans, test-driven-development, systematic-debugging, requesting/receiving-code-review, verification-before-completion, git-master, playwright, frontend, security-research, and the rest of the built-in set).

## Project Skills

ModuKit-specific skills, located in `.agents/skills/`:

<!-- BEGIN:skill-registry -->

| Skill | Type | Notes |
|-------|------|-------|
| `adjudication-walkthrough` | process | one-at-a-time option adjudication; Chinese trigger keywords are functional literals (C1 exception) |
| `ai-slop-cleanup` | tooling | remove AI-generated noise from diffs |
| `api-interface-design` | design | API surface and compatibility review |
| `ast-grep-rewrites` | tooling | structural search/rewrite with ast-grep |
| `book-to-skill` | tooling | convert books/documents (PDF/EPUB/DOCX/…) into structured skill files |
| `browser-testing` (inoperative) | browser | inoperative: no web surface; UI tier demoted (D89-K2) |
| `ci-cd-automation` | tooling | CI/CD workflow design and checks |
| `code-review-and-quality` | process | review checklist and quality bar |
| `code-simplification` | process | simplify without changing behavior |
| `constraint-driven-development` | process | turn constraints into executable checks |
| `context-engineering` | meta | assemble the right context before acting |
| `debugging-and-error-recovery` | process | systematic debugging and recovery |
| `deprecation-and-migration` | process | deprecate and migrate without breakage |
| `doc-audit` | tooling | documentation/decision consistency audit — **re-scoped to ModuKit canon 2026-10-09** (first full run 2026-10-10) |
| `documentation-and-adrs` | tooling | docs and ADR discipline |
| `ecosystem-scan` | deprecated | deprecated alias -> repo-audit; grace-period trigger |
| `incremental-implementation` | process | thin vertical slices |
| `lesson-review` | memory | batch session review → `.agents/memorys/` |
| `observability-and-instrumentation` | tooling | kernel event ring, logger, debug-reader |
| `openspec-apply-change` (inoperative) | spec | implement tasks from an OpenSpec change — **inoperative**: openspec CLI not installed (2026-10-09 scan) |
| `openspec-archive-change` (inoperative) | spec | archive a completed change — **inoperative**: openspec CLI not installed |
| `openspec-explore` (inoperative) | spec | inoperative: openspec CLI absent |
| `openspec-propose` (inoperative) | spec | inoperative: openspec CLI absent |
| `openspec-sync-specs` (inoperative) | spec | sync delta specs into main specs — **inoperative**: openspec CLI not installed |
| `performance-optimization` | process | measure before optimizing |
| `planning-and-task-breakdown` | process | plan and split work |
| `repo-audit` | tooling | whole-repo 10-domain audit (skills/rules/MCP/scripts/docs/book/CLI/tests/structure/naming); formerly ecosystem-scan |
| `repo-triage` | tooling | route a task to the right ModuKit layer |
| `review-hardcode` (deprecated alias) | deprecated | deprecated alias to security-hardening |
| `rust-engineering` | rust | workspace gates, unsafe/C ABI review, MSRV 1.88 |
| `rust-patterns` | vendored | idiomatic Rust ownership/errors/traits/concurrency — resolves the `See skill: rust-patterns` pointer in `rules/rust/coding-style.md`; vendored from affaan-m/ECC @b92f181b3b74 (MIT, fetched 2026-10-14, re-pull manually) |
| `rust-testing` | vendored | unit/integration/property-based/mocking/coverage patterns — vendored from affaan-m/ECC @b92f181b3b74 (MIT, fetched 2026-10-14, re-pull manually) |
| `security-hardening` | tooling | security audit and hardening |
| `shipping-and-launch` | process | release and launch checklist |
| `skill-authoring` | process | project skill authoring contract |
| `skill-router` | meta | analyze user intent → recommended skill list |
| `source-driven-development` | process | source-first implementation |
| `subagent-orchestration` | process | split work across subagents |
| `test-driven-development` | process | write the test first |
| `test-harness` | tooling | SDD to tests harness |
| `think-before-act` | meta | research → present options → user approval before any non-trivial action |
<!-- END:skill-registry -->

## Usage

Skills auto-activate from task context; invoke explicitly via the `skill` tool (or the matching `/command`).

All Appendix A skills are present. The five `openspec-*` skills are inoperative until the CLI exists; `review-hardcode` is a deprecated alias to `security-hardening`. Add new skills only when a real workflow needs them.
