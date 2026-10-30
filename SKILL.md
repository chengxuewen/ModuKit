# ModuKit Skills Registry

## Superpowers

General-purpose skills loaded through the `superpowers` plugin; available across all projects (brainstorming, writing-plans, test-driven-development, systematic-debugging, requesting/receiving-code-review, verification-before-completion, git-master, playwright, frontend, security-research, and the rest of the built-in set).

## Project Skills

ModuKit-specific skills, located in `.agents/skills/`:

<!-- BEGIN:skill-registry -->

| Skill | Type | Notes |
|-------|------|-------|
| `adjudication-walkthrough` | process | one-at-a-time option adjudication; Chinese trigger keywords are functional literals (C1 exception) |
| `book-to-skill` | tooling | convert books/documents (PDF/EPUB/DOCX/…) into structured skill files |
| `doc-audit` | tooling | documentation/decision consistency audit — **re-scoped to ModuKit canon 2026-10-09** (first full run 2026-10-10) |
| `ecosystem-scan` | tooling | audit `.agents/` and scan the community for adoptable skills/rules/MCP |
| `lesson-review` | memory | batch session review → `.agents/memorys/` |
| `openspec-apply-change` | spec | implement tasks from an OpenSpec change — **inoperative**: openspec CLI not installed (2026-10-09 scan) |
| `openspec-archive-change` | spec | archive a completed change — **inoperative**: openspec CLI not installed |
| `openspec-sync-specs` | spec | sync delta specs into main specs — **inoperative**: openspec CLI not installed |
| `rust-patterns` | vendored | idiomatic Rust ownership/errors/traits/concurrency — resolves the `See skill: rust-patterns` pointer in `rules/rust/coding-style.md`; vendored from affaan-m/ECC @b92f181b3b74 (MIT, fetched 2026-10-14, re-pull manually) |
| `rust-testing` | vendored | unit/integration/property-based/mocking/coverage patterns — vendored from affaan-m/ECC @b92f181b3b74 (MIT, fetched 2026-10-14, re-pull manually) |
| `skill-router` | meta | analyze user intent → recommended skill list |
| `think-before-act` | meta | research → present options → user approval before any non-trivial action |
<!-- END:skill-registry -->

## Usage

Skills auto-activate from task context; invoke explicitly via the `skill` tool (or the matching `/command`).

Not installed here (present in MediaServo's registry): `openspec-propose`, `openspec-explore`, `test-harness`, `review-hardcode` — add only when a real workflow needs them, and no `openspec/` directory exists in this repo yet.
