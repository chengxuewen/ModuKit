# Contributing

## Setup

```bash
bash bootstrap.sh     # first time: mise + pixi envs + git hooks (idempotent)
source pixi.sh        # daily activation; or prefix every command with `pixi run`
pixi run cargo --version   # toolchain smoke
```

## Before you open a change

- `bash scripts/check.sh` (or `pixi run ci`, the same script) must be green.
- Commit discipline is `C3` in `.agents/memorys/conventions.md`: subject <= 60
  characters, body bullets starting ` - ` and <= 100 characters per line.
- Agent layers are generated from `.agents/`: run `pixi run agents sync`; the
  pre-commit hook re-runs it and `pixi run agents check`. Never hand-edit a
  generated layer (`AGENTS.md` marker regions, `CONSTRAINTS.md`, `.mcp.json`,
  the opencode `mcp{}`, `SKILL.md` registry).
- A change touching `unsafe`, the C ABI, serialization, or a public API gets a
  second read (`.agents/rules/common/code-review.md`).

## Where to look

- `AGENTS.md` — project facts and the command surface.
- `CONSTRAINTS.md` — anti-gaming rules plus the generated gate list.
- `docs/architecture.md` + `docs/modules/` — the adjudicated baseline.
- `docs/ve-parity-plan.md` — the non-functional VE-parity port (B0-B8).
