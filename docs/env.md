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
| `default` | `tools`, `rust`, `mcp`, `agents`, `book` | Build, tests, MCP servers, agent layers, docs book. |
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
| `gate-rules` | C/D/PIT citation liveness (local entries + the external allowlist). |
| `gate-memory` | Batch-close memory disposition (`<base>..HEAD`; not in fast ci). |
| `mcp setup/status/index/sync` | Install/verify pins, show versions, rebuild indexes. |
| `docs-gen` | `cargo doc` + regenerate the committed `llms.txt` (book merges `target/doc`). |
| `markdownlint` | Manual Markdown check using `.markdownlint.json` (not in fast ci). |
| `book-build` / `book-check` | Assemble `build/book-src` and build/check `build/book/en`. |
| `book-serve` | Foreground LAN server with a rebuild watcher. |
| `book-graph` | Refresh `graph.json`/`meta.json` only (no mdbook build). |
| `spec-trace` | SDD clause numbers ↔ `spec:` test tags (5 pairs). |
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
  front-door book output path (`build/book/en`), relative links (delegated to
  `check-doc-refs.sh`), book SUMMARY when built.
- `gate-rules`: every C/D/PIT citation in `.agents/rules` resolves to a local
  entry or the external allowlist.
- `spec-trace`: every SDD clause has a matching `spec:` test tag, and every tag
  stays inside the five-prefix whitelist.
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

## Host matrix

| Host | Discovery | Generated layer | Status |
|---|---|---|---|
| CodeBuddy | Root `AGENTS.md` when `CODEBUDDY.md` is absent; `.codebuddy/rules/<name>/RULE.mdc`; `.codebuddy/skills/<name>`; root `.mcp.json` | `rules-links.py` (`mdc`) + `skills-links.py` (link, `--copy` on Windows) | `AGENTS.md` fallback verified in the 2026-10-30 session; 18 rules + 41 skill links generated; host restart pending for the rules/skills/MCP probe |
| Qoder | Native reader of `.agents/rules` and `.agents/skills`; project `.mcp.json` | none | Native reader; no generated layer |
| opencode | Native rules/skills; `.opencode/opencode.json` `mcp{}` merge | `mcp-links.py` | Green |
| project | Root `.mcp.json` (`mcpServers`, launcher) | `mcp-links.py` | Green; the only writer of `.mcp.json` |

Generation rules:

- `.agents/skills` and `.agents/rules` are the only editable sources. Host
  layers are generated by `pixi run agents sync`, never hand-written.
- `codebuddy` is an optional target: an absent `.codebuddy/` layer is a SKIP,
  not a gate failure. `.codebuddy/teams/` is host-owned session state and is
  never generated, checked, or pruned.
- Copy mode (`.codebuddy/skills` on Windows) writes a per-target
  `.codebuddy/skills-link-map.json`; link mode does not need it.
- Forbidden paths (`CODEBUDDY.md`, `.codebuddy/mcp.json`, shadow host skill
  directories) are checked by `gate-agents`; one red observation is recorded in
  the plan review trail.
- Activation policy: full inventory (41 skills) for the pilot. The H3 host
  restart measures the actual context cost; H2b records the final policy (full
  or `skills_allowlist`) here.
- Repository-side check: `python3 scripts/verify-codebuddy-host.py` verifies the
  AGENTS.md fallback, 18 RULE.mdc files, 41 skill links, root `.mcp.json`, and
  `agents-cli.py check`. It cannot see an already-running IDE session.

### Probe log

- 2026-10-30 CLI probe: 18 `.codebuddy/rules/*/RULE.mdc` generated; 41 skill
  links generated; `gate-agents` green; `agents sync --dry-run` writes nothing;
  a planted `CODEBUDDY.md` reds `gate-agents` and removal restores green; a
  tampered copy-mode skill is detected by the copied-tree hash.
- 2026-10-30 H2b allowlist probe: a temporary 3-skill `skills_allowlist`
  (repo-audit, think-before-act, skill-authoring) synced/checked green and
  pruned the other links; removing the key and syncing restored all 41. The
  config was restored to full activation; the final policy waits for the host
  measurement.
- CodeBuddy host restart: PENDING. The current session started before the
  `.codebuddy` layer existed, so the rules/skills/MCP visibility must be
  re-probed in a fresh session. Record the host skill list, one rule, one MCP
  call, and whether `provider` is required in `RULE.mdc`.

## Book

Book sources are English-only (VE D46, 2026-10-30). Other languages are covered by
reader-side browser translation (Edge/Chrome built-in, Immersive Translate): there
are no PO catalogs, no `BOOK_LANG` builds, no gettext preprocessor, and no
per-release translation maintenance. `book-build` writes `build/book/en` only;
`book-serve` is a foreground LAN server with a rebuild watcher; `book-graph`
refreshes `graph.json`/`meta.json` without rebuilding the site. The SDD clauses
under `docs/sdd/` are index entries; `scripts/spec-trace.sh` proves each one has a
matching `spec:` tag on an existing test.

## CI

`.github/workflows/ci.yml` is a standby mirror (origin is gitee; it activates on the
GitHub-mirror day). It checks out submodules recursively, installs the frozen
default and `msrv` envs, runs `agents sync`, `pixi run ci`, the explicit MSRV check,
and `spec-trace`. Pin both actions to a commit SHA before enabling it for real.
The local equivalent is `pixi run ci`; the workflow adds no second gate definition.

## Output territory

| Path | Owner |
|---|---|
| `out/build/` | CMake/Ninja IDE build. |
| `build/book-src/` + `build/book/en/` | mdBook assembly and English site output (ignored). |
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
