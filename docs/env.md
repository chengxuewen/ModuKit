# ModuKit environment guide

Operational companion to `AGENTS.md`: how the environments are laid out, which
entry point to use, and how the gates fit together. The authoritative task names
are in `pixi.toml`; this file explains the intended use.

## Three steps on a new machine

```bash
bash bootstrap.sh      # first run only: pixi envs + git hooks (idempotent)
source pixi.sh         # daily shell activation
./modukit.sh -h        # task list; `pixi run <task>` is always equivalent
```

## Entry points

| File | Role |
|---|---|
| `bootstrap.sh` / `.bat` | One-time provisioning; idempotent; installs the git hooks (`core.hooksPath=scripts/hooks`). |
| `pixi.sh` / `.ps1` / `.bat` | Shell activation of the `default` environment. |
| `modukit.sh` / `.bat` | Pure passthrough: `./modukit.sh <task> [args...]` == `pixi run <task> [args...]`. No command table, so it cannot drift from `pixi.toml`. |

## Environments and features

| Environment | Features | Use |
|---|---|---|
| `default` | `tools`, `rust`, `mcp`, `agents` | Build, tests, MCP servers, agent layers. |
| `cov` | `rust`, `cov` | `cargo llvm-cov` boundary runs. |
| `serena` | `serena` | Isolated Python 3.13 + `serena-agent`; never the default env. |
| `msrv` | `msrv` | Rust 1.88 only: the MSRV promise check (not a build env). |

- `tools`: CMake >= 3.25, Ninja, libprotobuf (`protoc`), and the Linux/Windows compiler shim.
- `rust`: the workspace toolchain; cargo/rustfmt/clippy always run through pixi.
- `mcp`: Node 22 + Python 3.11+ for the pinned MCP launchers and CLI.
- `agents`: the Python entry points for the generated layers.
- `serena`: separate solve group; `serena` binary, rust-analyzer, rust-src.

## Tasks

| Task | What it does |
|---|---|
| `ci` | Alias for `bash scripts/check.sh` (the same script CI runs). |
| `agents` / `agent-sync` | Regenerate every agent layer from `.agents/`. |
| `gate-agents` | Verify the generated layers (markers, hub budget, MCP, constraints). |
| `mcp setup/status/index/sync` | Install/verify pins, show versions, rebuild indexes. |
| `docs-gen` | Regenerate the committed `llms.txt`. |
| `cmake-smoke` | Full cold configure/build/ctest + two negative paths (`ci` runs configure-only). |
| `msrv` | `cargo check --workspace` in the 1.88 env (`pixi run -e msrv msrv`). |
| `vendor-test` | Vendor-shm lane (cold, not in the fast gate). |
| `deny` | `cargo deny check licenses bans` when the tool resolves. |
| `configure` / `build` | CMake/IDE surface under `out/build`. |

## Gate contract

`pixi run ci` and `bash scripts/check.sh` are the same gate. `bash scripts/check.sh
--list` is the single source for the gate list; `CONSTRAINTS.md` is generated from
it by `scripts/constraints-index.py` (do not hand-edit that region).

The named gates:

- `gate-docs`: root CMake purity, PolyOrch gitlink three-way, CLI task names,
  relative links (delegated to `check-doc-refs.sh`), book SUMMARY when built.
- `gate-i18n`: every tracked or untracked non-ignored artifact is English; the
  `.agents/skills/` activation surface and dependency trees are skipped.
- `check-promises`: a narrow future-tense overclaim lint on README/whitepaper/
  architecture.
- `gate-abi`: the core cdylib's exported `modukit_*` set equals
  `contracts/modukit_embed.h` (the C++ demo run is `demo-stage4-m1`).
- `gate-mcp`: probes the pinned MCP servers without installing anything.
- `msrv`: `Cargo.toml` `rust-version` equals the pixi msrv pin, then
  `cargo check --workspace` in the 1.88 env (SKIP when the env is absent).
- `cmake-smoke`: cold configure plus two negative paths in `ci`; the full cold
  build/ctest is `pixi run cmake-smoke` at acceptance.
- `gate-agents`: verifies the generated layers.

## Skills

- One directory per skill, lower-kebab name; `SKILL.md` carries YAML frontmatter
  (`name`, `description`, `type`, `notes`, and `status` when inoperative/deprecated).
- `.agents/skills/**` is skipped by `gate-i18n`: routing triggers and keyword tables
  are functional CJK, not prose.
- `pixi run agents sync` regenerates the `SKILL.md` registry from the directories;
  `pixi run gate-skills` verifies the registry and `python3 scripts/skill-lint.py`
  validates frontmatter, size, and `@path` references.

## Output territory

| Path | Owner |
|---|---|
| `out/build/` | CMake/Ninja IDE build. |
| `out/build-<demo>/` | Demo-specific CMake builds. |
| `target/` | Cargo build and coverage. |
| `.pixi/envs/` | Pixi environments. |
| `.serena/cache/` | Serena LSP cache (ignored). |
| `.codegraph/` | CodeGraph index (ignored). |
| `node_modules/` | Root MCP npm echo (ignored). |

Do not add a new root-level build/bin directory; `docs/modules/09-observability-debug.md`
and the `.gitignore` treat these as the only writer classes.

## MCP lifecycle

- Pins live in `scripts/mcp-servers.json`; the canonical server list is
  `.agents/mcp.json`. `scripts/mcp-links.py sync` generates the root `.mcp.json`
  and merges the opencode `mcp{}`.
- `pixi run mcp setup` installs the two npm pins and verifies the serena env;
  `mcp status` is read-only; `mcp index` rebuilds `.codegraph` and the serena
  Rust cache; `mcp sync` is incremental for codegraph and a full serena re-index.
- Serena currently enables only the `rust` language server. Python/JS/C++ language
  servers are a recorded gap in `docs/ve-parity-plan.md` (B1).
- O5: a GUI host inherits the login PATH, where `~/.pixi/bin` may be absent.
  `gate-mcp` warns by default; set `MODUKIT_GATE_MCP_STRICT=1` to red. The fix is a
  user-level symlink (for example `~/.local/bin/pixi`), not a repo change.

## Toolchain boundary

pixi is the single toolchain (B4): `bootstrap.sh` provisions pixi and the default
env; `mise.toml` and `rust-toolchain.toml` are deleted. Node 22 lives in the `mcp`
feature, and cargo/rustfmt/clippy run through `pixi run`. The MSRV floor is 1.88
(B4b); the default env still resolves the newest locked rust.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `cargo: command not found` | Use `pixi run cargo ...`; there is no mise shim. |
| `pixi` missing in a GUI MCP host | Add the O5 symlink; rerun `pixi run gate-mcp`. |
| `pin drift` from `mcp-serve.mjs` | `pixi run mcp setup`; never hand-edit `node_modules`. |
| `agents check` marker error | Run `pixi run agents sync`; if it still reds, fix the duplicate/missing marker by hand once. |
| serena index fails | Check `.pixi/envs/serena` exists; `serena project index .` must run from the repo root. |
