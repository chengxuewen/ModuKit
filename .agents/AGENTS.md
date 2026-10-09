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
│   └── pitfalls.md             # PIT{n} five-part, on-demand
├── rules/
│   ├── common/                 # 12 files: development-workflow, edit-safety (#1–#24), lesson-memory, …
│   ├── rust|typescript|python|web|golang|cpp/   # MOUNTED via instructions[] (coding-style + hooks)
│   └── csharp|dart|java|kotlin|perl|php|swift/  # unmounted pool — add to instructions[] to activate
└── skills/                     # 10 dirs; book-to-skill carries a scripts/ python package (22 files)
```

## WHERE TO LOOK

| Task | Location |
|---|---|
| Record decision / pitfall / convention | `rules/common/lesson-memory.md` (templates + triggers) |
| Edit-tool corruption history & defenses | `rules/common/edit-safety.md` #8–#24 |
| Why memory files say "ported/reset 2026-10-09" | `memorys/status.md` → History |
| Pre-action research protocol | `skills/think-before-act/SKILL.md` |

## CONVENTIONS (this subtree)

- `instructions[]` files load EVERY TURN — length is latency and cost.
- Mounting a rule pack = add BOTH `coding-style.md` and `hooks.md` paths to `opencode.json`.
- Functional CJK is allowed only as literals (skill trigger keywords, `book-to-skill` Chinese chapter-number parse data); all prose English (C1).
- C/D/PIT numbering is fresh for ModuKit; PolyOrch originals live in `PolyOrch/.agents/memorys/` (reference checkout — read-only, never sync automatically).

## ANTI-PATTERNS

- `memory_create_entities` / knowledge-graph tools — wrong system, memory is these files (`rules/common/agents.md`).
- Appending long markdown to `memorys/*.md` via edit tool — use heredoc or python read→replace(assert count==1)→write (`edit-safety.md` #11, #18, #20).
- Running `doc-audit` without re-scoping — its target doc set (whitepaper strings, C1–C6, D1–D29, `docs/modules/`) is PolyOrch's; port note is in its SKILL.md head.
- Trusting `rules/common/agents.md`'s `~/.claude/agents/` list or `scripts/gate.sh` checks — ported leftovers, inoperative here (see `/AGENTS.md` NOTES).
