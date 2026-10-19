# .agents — Agent Governance Tree

Parent: `/AGENTS.md` (project facts). This file covers only the governance subsystem — do not repeat parent content.

## OVERVIEW

Memory + rule packs + project skills, injected into every AI turn through `.opencode/opencode.json` `instructions[]`.

## STRUCTURE

```
.agents/
├── memorys/                    # live project memory (NOT the knowledge-graph tools)
│   ├── status.md               # ┐ loaded EVERY TURN — hard size discipline
│   ├── conventions.md          # ┘ C0 + C1 so far
│   ├── decisions.md            # D{n}, on-demand
│   ├── backlog.md              # agent-system track only (P-items), on-demand
│   ├── history-archive.md      # full History log (on-demand; status.md keeps only a pointer)
│   └── pitfalls.md             # PIT{n} five-part, on-demand
├── rules/
│   ├── common/                 # 12 files: development-workflow, edit-safety (#1–#27; historical double-#11 renumbered 2026-10-10), lesson-memory, …
│   ├── rust/                       # mounted language pack (coding-style + hooks); ts/py/web/golang/cpp UNMOUNTED G10 — files stay on disk; 7 never-mounted dirs also kept (maintainer ruling 2026-10-09)
│   └── csharp|dart|java|kotlin|perl|php|swift/  # unmounted pool — add to instructions[] to activate
└── skills/                     # 12 dirs (rust-patterns + rust-testing vendored from affaan-m/ECC 2026-10-14, provenance in SKILL.md registry); book-to-skill carries a scripts/ python package (22 files, measured 2026-10-10)
```

## WHERE TO LOOK

| Task | Location |
|---|---|
| Record decision / pitfall / convention | `.agents/rules/common/lesson-memory.md` (templates + triggers) |
| Edit-tool corruption history & defenses | `.agents/rules/common/edit-safety.md` #8–#27 |
| Why memory files say "ported/reset 2026-10-09" | `.agents/memorys/history-archive.md` |
| Pre-action research protocol | `.agents/skills/think-before-act/SKILL.md` |

## CONVENTIONS (this subtree)

- `instructions[]` files load EVERY TURN — length is latency and cost.
- Mounting a rule pack = add BOTH `coding-style.md` and `hooks.md` paths to `opencode.json`.
- Functional CJK is allowed only as literals (skill trigger keywords, `book-to-skill` Chinese chapter-number parse data); all prose English (C1).
- C/D/PIT numbering is fresh for ModuKit; PolyOrch originals live in `PolyOrch/.agents/memorys/` (reference checkout — read-only, never sync automatically).

## ANTI-PATTERNS

- `memory_create_entities` / knowledge-graph tools — wrong system, memory is these files (`.agents/rules/common/agents.md`).
- Appending long markdown to `memorys/*.md` via edit tool — use heredoc or python read→replace(assert count==1)→write (`edit-safety.md` #11, #18, #20).
- Quoting `doc-audit`'s pre-re-scope PolyOrch target claims (C1–C6, D1–D29, `docs/modules/`) — stale; skill re-scoped to ModuKit canon 2026-10-09 (commit 2aa7663).
- Trusting `.agents/rules/common/agents.md`'s `~/.claude/agents/` list or its gate.sh checks — ported leftovers, inoperative here (roster now flagged INOPERATIVE in place; the live local gate is `scripts/check.sh` per D50; see `/AGENTS.md` NOTES).
