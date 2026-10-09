# ModuKit Skills Registry

## Superpowers

General-purpose skills loaded through the `superpowers` plugin; available across all projects (brainstorming, writing-plans, test-driven-development, systematic-debugging, requesting/receiving-code-review, verification-before-completion, git-master, playwright, frontend, security-research, and the rest of the built-in set).

## Project Skills

ModuKit-specific skills, located in `.agents/skills/`:

| Skill | Type | Notes |
|-------|------|-------|
| `think-before-act` | meta | research → present options → user approval before any non-trivial action |
| `skill-router` | meta | analyze user intent → recommended skill list |
| `adjudication-walkthrough` | process | one-at-a-time option adjudication; Chinese trigger keywords are functional literals (C1 exception) |
| `doc-audit` | tooling | documentation/decision consistency audit — **still scoped to PolyOrch's doc set; re-scope before running** |
| `ecosystem-scan` | tooling | audit `.agents/` and scan the community for adoptable skills/rules/MCP |
| `lesson-review` | memory | batch session review → `.agents/memorys/` |
| `book-to-skill` | tooling | convert books/documents (PDF/EPUB/DOCX/…) into structured skill files |
| `openspec-apply-change` | spec | implement tasks from an OpenSpec change |
| `openspec-archive-change` | spec | archive a completed change |
| `openspec-sync-specs` | spec | sync delta specs into main specs |

## Usage

Skills auto-activate from task context; invoke explicitly via the `skill` tool (or the matching `/command`).

Not installed here (present in MediaServo's registry): `openspec-propose`, `openspec-explore`, `test-harness`, `review-hardcode` — add only when a real workflow needs them, and no `openspec/` directory exists in this repo yet.
