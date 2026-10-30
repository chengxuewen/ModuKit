# VE-parity plan - the non-functional surface adopts the VisiaEngine shape

> STATUS: PLAN OF RECORD, 2026-10-29 (revision 7). Twelve adjudications walked one by
> one with the maintainer (ten in section 2: A / A / C / A / A / B / A / A / A / B;
> two follow-ups in section 2.1: C / A); implementation has NOT started. Functional
> code, contracts, binding implementations, demo assertions, and project memory
> entries are out of scope.
> Reference clone: `.refinfo/VisiaEngine` (read-only, git-ignored).

## 1. Ruling and boundary

The default is copy: everything non-functional follows VisiaEngine (VE). Deviations
need a recorded reason, not a preference. The boundary is drawn once here so later
batches cannot widen "functional" to avoid work:

| Class | Examples | Treatment |
|---|---|---|
| Functional code | `crates/**`, `contracts/**`, binding implementations under `bindings/**`, `test-plugins/**`, `examples/**` business content | ModuKit-owned, never copied |
| Functional verification | assertions and sabotage levers inside `scripts/demo-*.sh` | ModuKit-owned, never copied |
| Functional contract text | `docs/sdd/` clause bodies, `docs/capability-ledger.md` rows | authored in ModuKit, shape copied only |
| Project facts | C/D/PIT entries, version literals, test/verb counts | ModuKit-owned, never copied |
| Non-functional | root config, build assembly, pixi, script framework, agent config, docs skeleton, directory layout, CI, hooks, IDE adapters, gitignore | **copy VE** |

Two premises hold: both repositories are `MIT OR Apache-2.0`, and every VE artifact is
English, so copying files creates no compatibility or C1 conflict. Compatibility is not
attribution: MIT requires the copyright and permission notice to travel with substantial
copies, so every copied file is listed in the provenance ledger and the root `NOTICE`
carries VE's copyright line.

## 2. Adjudication record (2026-10-29, user-adjudicated one by one)

| # | Decision | Ruling | Key note |
|---|---|---|---|
| 1 | First batch | **A** | Security + MCP vertical slice; the MCP generator is also the first block of the generation layer |
| 2 | MCP host files | **A** | `.agents/mcp.json` generates root `.mcp.json` and the opencode `mcp{}` merge |
| 3 | MCP server set | **C** | codegraph + playwright + serena |
| 4 | MCP install location | **A** | Root `node_modules`; pins live only in `scripts/mcp-servers.json` |
| 5 | pixi single-track | **A** | Add pixi node and verify the JS RUNNER line first, then delete `mise.toml` / `rust-toolchain.toml` |
| 6 | Output territory | **B** | Keep `out/`; copy the discipline (one tree per preset, root-anchored, no mixing) |
| 7 | Build modules without a lane | **A** | Build SDK / Ffi / Bindings only; no Wasm or Qt; Install / Packaging wait for the milestone |
| 8 | Host switches | **A** | per-surface: opencode rules + MCP; qoder rules / MCP after a token probe; codebuddy / trae / claude / cursor disabled |
| 9 | book and SDD depth | **A** | book over the existing docs; SDD skeleton plus five real clauses |
| 10 | Common-rule absorption | **B** | Merge by diff, keep local numbering; never replace wholesale |

### 2.1 Follow-up adjudications (2026-10-29, revision 2, user-adjudicated one by one)

| # | Decision | Ruling | Key note |
|---|---|---|---|
| 11 | MSRV method | **C** | Start at 1.85, step by 5 (1.90 / 1.95 / 1.99), record the lowest that passes; `rust-version` must equal the msrv pin, and a gate asserts equality |
| 12 | qoder probe scope | **A** | Per-surface: skills already proven (this session lists all 12 project skills); rules and MCP each get a token probe; enable each surface independently |

The MSRV *number* is an output of the probe, not a decision; the qoder *paths* are
outputs of the probe, not assumptions.

### 2.2 P0 review adjudications (2026-10-29, revision 5, user-adjudicated one by one)

| # | Decision | Ruling | Key note |
|---|---|---|---|
| 13 | bootstrap `agent-sync` | **A** | Unconditional advisory; B4 sequenced after B2 |
| 14 | AGENTS single-source section | **A** | ~12 lines in B2, invariants only |
| 15 | `gate-abi` / `cmake-smoke` | **A** | gate-abi in B3, cmake-smoke in B4 |
| 16 | CONSTRAINTS source | **C** | `check.sh` is the only source; `ci` is an alias for `check` |

## 3. Work breakdown

Each batch is independently acceptable. Every gate named here must be observed red
once before it is trusted (C0), and every batch closes with `bash scripts/check.sh`
green plus its own acceptance commands. Commands marked "(target)" do not exist yet
and land with the batch that names them.

### 3.0 Batch order, authority, and provenance

- **Dependency graph**: B0 -> B1 -> B2 -> B3; B3 -> B5; B3 -> B6 (B1 also feeds
  B6's codegraph index); B4 depends on B2 (the `agent-sync` task must exist before
  bootstrap calls it) and may run in parallel with B3; B7 closes after B4/B5/B6. B8
  (host link layer) is deferred until a link-based host is enabled.
- **Authority**: D109 (to be written) is the binding decision record; this file is
  the execution detail. A change here that alters a ruling re-opens the adjudication.
- **Ownership**: every batch names an implementer and an independent verifier; the
  verifier re-runs the acceptance commands from a clean shell and records the output.
  Subagent artifacts are not trusted until re-measured. Plan edits happen between
  batches, never while a batch is running (PIT-14 class).

| Batch | Size | Implementer role | Verifier role |
|---|---|---|---|
| B0 | S | agent-config | orchestrator |
| B1 | M | mcp | orchestrator |
| B2 | M | agent-config | orchestrator |
| B3 | M | docs/ci | orchestrator |
| B4 | M | build | orchestrator |
| B5 | L | rules/skills | orchestrator |
| B6 | L | docs/book | orchestrator |
| B7 | S | ci/ide | orchestrator |

- **Provenance**: every copied VE file is recorded as `source -> destination
  (verbatim|adapt) @ batch` in `docs/reference/visiaengine.md`; the batch commit body
  names `VE <path> @ <commit>`. The root `NOTICE` carries VE's copyright line.
- **Rollback**: if a batch's gate stays red after two fix rounds, revert the batch as
  a unit and keep the plan; never disable a gate to land a batch.

### B0 - Housekeeping, decision write-back, and the numbering gate

Deliverables:

- Redact the internal gateway address in `.opencode/agent-model-tiers.md` (VE uses
  `<internal-gateway-host:port>` in the same file), rename the document's title from
  PolyOrch to ModuKit, and align the tier table with `.omo/omo.jsonc` categories.
- Add the ignore lines the port needs from day one: `/package.json`,
  `/package-lock.json`, `.agents/mcp.local.json`.
- Minimal `scripts/agent-targets.json` with two entries: `opencode` (rules + MCP) and
  `project` (no rules/skills; `mcp_file: .mcp.json`, `mcp_shape: mcpServers`). The
  `project` entry gives Adjudication 2's root `.mcp.json` an enabled producer while
  codebuddy stays disabled. B2 extends this file.
- `omo.jsonc` teams: align the dev team categories to VE's `deep`, or keep
  `deep-low` and add it to section 4 deviations. Do not leave the mismatch
  unexplained.
- Write D109 into `.agents/memorys/decisions.md` (full detail:
  `docs/ve-parity-plan.md`); sync the high-water in `docs/architecture.md` section
  11, `AGENTS.md`, and `.agents/memorys/status.md` in one pass.
- Commit this plan, D109, and the B0 changes as the port's first commit; the plan is
  the execution record, not an untracked draft.
- Fix the numbering gate: derive the bound from the actual maximum `D{n}` in
  `decisions.md` and assert the three high-water sites agree. The current pattern
  matches only `D1000+`, so D107/D108 pass vacuously.
- Provenance ledger skeleton in `docs/reference/visiaengine.md` plus the root
  `NOTICE`.
- Record the Appendix D baselines (current `check.sh` leg count and wall time,
  `AGENTS.md` size).

Acceptance:

```bash
grep -rn '192\.168\.' .opencode .agents docs AGENTS.md SKILL.md   # empty
python3 -m json.tool scripts/agent-targets.json >/dev/null        # valid JSON
grep -c '^## D109' .agents/memorys/decisions.md                   # 1
grep -n 'D109' AGENTS.md docs/architecture.md .agents/memorys/status.md  # all three
git log -1 --name-only | grep -qx docs/ve-parity-plan.md          # plan committed
git status --short                                                # no npm/local-MCP noise
```

Red-proof: drop the `/package.json` ignore and touch it -> `git status` shows it;
restore -> clean. Plant a D110 heading -> the numbering gate red; remove -> green.

### B1 - Security + MCP vertical slice (adjudications 1, 2, 3, 4)

Deliverables:

- `.agents/mcp.json` (canonical, `pixi run mcp-<name>` form) and
  `.agents/mcp.local.json.example`.
- `scripts/mcp-servers.json` with pinned entries: codegraph 1.6.0, playwright 0.0.82,
  serena 2.1.0 (verify each pin against the real registry on execution day). Each
  entry carries `probe_mode: start|version` so gate-mcp can start codegraph,
  version-probe playwright, and version-probe serena without launching a browser or
  language servers.
- `scripts/mcp-serve.mjs`, `scripts/mcp-launch.sh`, `scripts/mcp-launch.bat`,
  `scripts/mcp-cli.py`, `scripts/gate-mcp.py`, `scripts/mcp-links.py`,
  `scripts/mcp-file-guard.py`, and the shared `scripts/agent_common.py` (landed once
  here; B2 imports it and never rewrites it).
- Generated root `.mcp.json` and the opencode `mcp{}` merge (non-destructive: other
  opencode keys stay hand-written). One-time migration: delete the arxiv/github
  entries from `.opencode/opencode.json` before the first sync; a non-destructive
  merge would otherwise keep them forever.
- B1 entry point: `python3 scripts/mcp-links.py sync` (the `agents` dispatcher lands
  in B2).
- pixi tasks: `mcp`, `mcp-codegraph`, `mcp-playwright`, `mcp-serena`, `gate-mcp`;
  a serena feature with Python 3.13 + `serena-agent` in its own solve group.
- Keep `.opencode/serena/` as a legacy local install (4,479 files, ignored by its
  nested `.gitignore`); it is not a dead shell. The MCP path is the new pixi
  `serena` env; retire the legacy tree only on an explicit user word.
- `.serena/project.yml` adapted and tracked (only `rust` enabled today: rust-analyzer
  ships in the serena env; the Python/JS/C++ language-server gap is recorded in the
  risk table; not a verbatim VE copy); `.serena/cache/` and `.serena/memories/` stay
  ignored. Serena memories are not a second memory store (VE D41); the single memory
  source stays `.agents/memorys/`.
- `.opencode/init-lsp-wrap.mjs` is conditional: probe opencode on a `.rs` file
  first. If LSP starts, record "wrapper not needed" in the copy ledger and do not
  copy it; if it fails, copy the wrapper and replace `"lsp": true` with explicit
  rust-analyzer / bash / remark commands that use it. `.opencode/with-node.sh` is
  only needed for node bridge MCP scripts, which ModuKit does not have; record that
  omission instead of copying dead code.

Probe matrix (gate-mcp implements exactly this):

| Server | Probe | Absent-tool outcome |
|---|---|---|
| codegraph | start with empty stdin, expect clean EOF | red (pin or binary broken) |
| playwright | `<bin> --version`; never start a browser download | SKIP if browsers are absent, never a red |
| serena | `<env>/bin/serena --version`; never start the server | SKIP-ENV if the env is not installed |

Before B1 is accepted, check `clang-tools` availability on linux-64: VE D29 deferred
serena partly because clangd was absent on its box. If the package cannot be
installed here, record the serena C++ gap and run serena with the Rust/Python
backends only.

Acceptance:

```bash
pixi run mcp setup                     # installs npm pins; serena env is separate
pixi run gate-mcp                      # green; O5 WARN unless MODUKIT_GATE_MCP_STRICT=1
pixi run mcp status                    # pin vs installed agree
pixi run mcp index                     # codegraph + serena (rust) index
pixi run mcp-codegraph </dev/null      # starts and exits 0 on EOF
python3 scripts/mcp-links.py check     # hosts match the single source
python3 scripts/mcp-file-guard.py check
python3 -m json.tool .opencode/opencode.json >/dev/null
```

Red-proof: bump a pin without installing -> `mcp-serve.mjs` exits 1 with
`pin drift`; restore -> green. Leave a server in `.agents/mcp.json` but drop it from
`mcp-servers.json` -> `gate-mcp` red.

### B2 - Agent configuration generation layer (adjudications 2, 8, 10)

Deliverables:

- `scripts/agent-links.py` (dispatcher; loads a layer only when its script exists and
  its target is enabled), `scripts/agents-cli.py`, `scripts/memory-aggregate.py`,
  `scripts/skills-index.py`, `scripts/mcp-links.py`, `scripts/mcp-file-guard.py`.
- `scripts/agent-targets.json`: extend the B0 file to per-surface status
  (adjudication 12): opencode `rules: true`, `mcp: true`; qoder `rules: true`
  (native `.agents/rules`, observed 2026-10-29), `mcp: probe`; codebuddy / trae /
  claude / cursor all false. Skills are a native
  read from `.agents/skills` for both hosts (observed for opencode and qoder), so
  they are not a target surface here; `skills-targets.json` stays a B8 concern for
  link-based hosts. A single host boolean would let a failed MCP probe block the
  already-proven skills surface, so the schema carries per-surface state.
  `generated_tracked` is exactly `AGENTS.md`, `CONSTRAINTS.md`, `.mcp.json`,
  `.opencode/opencode.json`, `SKILL.md` - the only list the pre-commit hook stages.
  The list is complete from day one; a file whose batch has not landed yet is skipped
  by the hook's `git add -- path 2>/dev/null || true`, not treated as drift.
- `scripts/hooks/pre-commit` (agent sync + gate), coexisting with the existing
  `scripts/hooks/commit-msg` under the current `core.hooksPath`.
- Generated regions: `AGENTS.md` skills-directory and agents-memory, `SKILL.md`
  registry. The `SKILL.md` marker region is created in B2 **before the first sync**;
  a missing marker makes `skills-index.py` WARN and skip, never append or overwrite.
- `AGENTS.md` gains the ~12-line `AGENT CONFIG (single source)` section: `.agents/`
  is the only editable source; generated layers are never hand-edited; the
  pre-commit hook runs sync + gate; a new host enters through `agent-targets.json`
  only after a discovery probe. Invariants only - the tree details stay in
  `.agents/AGENTS.md`.
- `memory-aggregate.py` WINDOW map: `decisions` 25, `pitfalls` 15, `status` 6,
  `history-archive` 0 (pointer line only), `backlog` 10. Baseline: `AGENTS.md` must
  stay <= 30 KB after the first sync.
- `skills-index.py` keeps the existing `Skill | Type | Notes` table schema;
  inoperative flags stay in Notes (do not collapse to VE's two-column table).
- pixi tasks: `agents`, `agent-sync`, `gate-agents`.

`rules-links.py` and `skills-links.py` are **deferred to B8**: no enabled host reads
a generated link layer today (opencode reads `.agents/` natively), so shipping them
now would be dead code - the same rule that refused the lane-less CMake modules in
adjudication 7.

Acceptance:

```bash
pixi run agents check                   # (target) exit 0
pixi run agents sync --dry-run          # (target) writes nothing
pixi run gate-agents                    # (target) hub <= 30 KB, markers single-line
git ls-files -ci --exclude-standard     # empty
```

Red-proof: hand-edit a generated region -> `agents check` exits 1; sync -> 0.
Duplicate an END marker -> exit 2 (guard), nothing written (VE PIT-84).
Add a 100-heading file to `.agents/memorys/` without a WINDOW entry -> hub budget
red.

Qoder probe (per-surface, adjudication 12): skills are native (this session lists all
12 project skills), and rules are native too - both `.agents/rules` and
`.qoder/rules` echoed their probe tokens on 2026-10-29; prefer `.agents/rules` as the
single source and generate no qoder rules layer. The only remaining probe is MCP:
after B1 creates root `.mcp.json`, check whether qoder reads the project-level file.
If it is user-level only, record "qoder MCP is user-level only" and leave that surface
disabled.

### B3 - Root documents, CONSTRAINTS, and doc gates

Deliverables:

- `docs/env.md` (single-track boundary after B4: pixi only, mise/rust-toolchain
  retired; pixi tasks, output territory, MCP lifecycle, book workflow, and the gate
  contract: `ci` is an alias for `check`, legs come from `check.sh --list`),
  `CONTRIBUTING.md`, `SECURITY.md` (scope: C ABI, wire decode, plugin load, bundle
  manifest, `unsafe`), `llms.txt` plus `scripts/gen-llms-txt.sh`. The llms source
  list is crates (name + one-line role), contracts (proto + headers), docs/modules
  00-11, the capability ledger, and book/SDD after B6. A missing source is skipped
  loudly, not silently.
- `CONSTRAINTS.md` + `scripts/constraints-index.py`, generated from `check.sh
  --list` alone. The hand-written half carries the seven anti-gaming rules: no
  `--no-verify`, no un-justified `#[allow]`, no `#[ignore]` to skip a failing test,
  never weaken an assertion, never hand-edit a generated layer, dependency changes
  pass `cargo deny` when the tool resolves (an offline skip is recorded loudly,
  never silent), and fix the cause not the gate. Phrase them against ModuKit's
  demo/RUNNER gates, not VE's.
- `scripts/gate-docs.sh` adapted: header-vs-Rust-extern set (merged with
  `check-embed-parity.sh`), relative links (merged with `check-doc-refs.sh`), root
  CMake purity, PolyOrch gitlink three-way, CLI task-name existence, llms numbers.
  Enable a check only when its claim exists: the README has no "workspace has N
  packages" line today, so check 8 stays off until the claim is added intentionally.
- `scripts/gate-i18n.sh` scanning `git ls-files` with VE's exact `EXTS`,
  `SKIP_PATH_PREFIX` (`.agents/skills/`, `.opencode/node_modules/`), and `ALLOW`
  list. `.po` / `.pot` are outside `EXTS`; state explicitly that `book/po/**`
  translations are functional CJK and never scanned.
- `scripts/check-promises.sh` (overclaim lint).
- `scripts/gate-abi.sh`: default-feature build, an `nm` export whitelist derived from
  `contracts/modukit_embed.h` (never hand-typed), and a real C++ demo run; missing
  `nm` or compiler = SKIP, not red. Added as a `gate-abi` pixi task.
- `modukit.sh` / `modukit.bat` pure passthrough (no command table): `-h` exits 0,
  no argument exits 2, missing pixi exits 127; `gate-docs` check 12 asserts every
  task named in the wrapper and docs exists in `pixi.toml`.
- Gate ownership: one gate owns one invariant, and complementary checks stay
  separate. `gate-docs` check 3 CALLS `check-embed-parity.sh` for the verb list and
  adds its own header-prototype-vs-Rust-extern check; check 4 CALLS
  `check-doc-refs.sh` for links. The verb list compares header verbs to
  `embed_abi.rs` constants; the prototype check compares header function names to
  Rust `extern` names. Keep both scripts; `check.sh` calls the merged entry point
  once, so no leg runs twice.
- `check.sh` gains a `--list` mode that prints one leg per line; CONSTRAINTS is
  generated from that list alone. No hand parsing of `note "ok ..."` strings.
- Gate source (one sentence, no ambiguity): `check.sh` is the only gate list; `ci`
  is an alias for `check`. The doc / agent / mcp / skill / abi gates are called as
  check.sh legs, and B4 wires `cmake-smoke` and `msrv` the same way. `ci` has no
  separate depends-on list, so there is no second source to drift.

Note: the PolyOrch pin check is viable here - the pin is contained in
`origin/dev-modukit` and `origin/main`, and the submodule HEAD is on `dev-modukit`
(verified 2026-10-29). Keep the check.

Acceptance:

```bash
pixi run ci                             # (target) green
bash scripts/gate-docs.sh               # (target) green
pixi run gate-abi                       # (target) green; SKIP when nm/cc absent
bash scripts/check.sh --list | wc -l    # (target) >= today's leg count
pixi run agents check                   # CONSTRAINTS matches ci
./modukit.sh -h; echo $?                # 0
./modukit.sh; echo $?                   # 2
```

Red-proof: plant a broken relative link -> `gate-docs` red; plant a CJK comment in
a tracked file -> `gate-i18n` red; plant "will ship" next to a delivered feature ->
`check-promises` red; edit CONSTRAINTS by hand -> `agents check` red.

### B4 - pixi single-track, presets, build modules (adjudications 5, 6, 7)

Sequencing is part of the decision: **pixi node first, deletion second**. Today the
two JS demo legs prefer `.pixi/envs/default/bin/node` and fall back to mise's node;
`check.sh` treats their SKIP as green. Deleting mise first would turn two PASS legs
into a false green.

Deliverables:

1. `nodejs = ">=22,<23"` was added in B1 (the MCP feature); B4 verifies both JS legs
   and confirms the `RUNNER: node ... (.pixi/...)` line, then owns the mise deletion.
2. Delete `mise.toml` and `rust-toolchain.toml`; rewrite `bootstrap.sh` **and**
   `bootstrap.bat` without the mise stages; `uv` retires with mise (arxiv is gone in
   B1); keep the PolyOrch pixi delegation; print the manual `~/.bashrc` / `~/.zshrc`
   cleanup line for the old `mise activate`.
   Decide the tracked `pixi.sh` / `pixi.bat` / `pixi.ps1` trio: adopt VE's
   generated-and-ignored model (recommended) or record why ModuKit keeps them
   tracked. If untracking: `git rm --cached` the three, add `/pixi.sh`,
   `/pixi.bat`, `/pixi.ps1` to `.gitignore`, let the PolyOrch bootstrap regenerate
   them, and prove with `git status --short` that a fresh bootstrap leaves no diff.
   Finish bootstrap with `pixi run agent-sync || echo WARN ...` and end with the
   `pixi run agents status` hint; a sync failure must not sink the environment
   (VE `bootstrap.sh:61-67`).
3. pixi: replace `rust = "*"` with `>=1.99,<2` (the lock resolves rust 1.99.0,
   measured 2026-10-29); add an `msrv` environment in its own solve group. MSRV is
   a promise, not a lock echo (adjudication 11). Probe result 2026-10-29: 1.85 red
   (`libloading 0.9.0` and `napi-build 2.5.0` both require 1.88), 1.88.0 green
   (`cargo check --workspace`, 9.81 s warm). Record `rust-version = "1.88.0"` and
   pin `=1.88.0` in the msrv feature; the wider ladder (1.90 / 1.95 / 1.99) is
   evidence, not a second promise. Between candidates, revert the manifest and lock
   with `git checkout -- pixi.toml pixi.lock`; only the final passing pin is
   committed. Add a check leg: `Cargo.toml`'s `rust-version` must equal the msrv
   pin, mismatch = red. Without that field the msrv leg has nothing to verify.
   Finally wire `msrv` into `check.sh`'s leg list in the same batch.
4. `CMakePresets.json`: add `release`, `debug`, `prebuilt`, `ci-strict`; keep
   `out/` (adjudication 6) and the one-tree-per-preset rule.
5. `cmake/ModuKitSDK.cmake`, `cmake/ModuKitFfi.cmake`,
   `cmake/ModuKitBindings.cmake` (adjudication 7). No Wasm or Qt module. Define the
   composition with the existing `MODUKIT_USE_PIXI` switch: the SDK module resolves
   the toolchain, PolyOrch consumes it, and the boolean is retired or kept as an
   alias - never two truth sources.
6. Root-purity gate: 120-line soft cap, `# assembly-growth` exemption, no
   `add_library` / `add_executable` except ALIAS in the root.
7. Commit `pixi.lock` in the same batch; CI installs with `--frozen`.
8. `scripts/cmake-smoke.sh`: three-state gate (no cmake = SKIP; present = cold
   configure/build/ctest plus two named negative-path asserts: SYSTEM
   anti-pollution and missing prebuilt artifact). Added as a `cmake-smoke` pixi task
   and wired into `check.sh` alongside `msrv`.

Acceptance:

```bash
pixi run node -v                                  # 22.x from .pixi
bash scripts/demo-wave2-js.sh | grep '^RUNNER: node'   # source is .pixi/...
pixi run -e msrv cargo check --workspace          # (target) green
pixi run cmake-smoke                              # (target) green; SKIP without cmake
pixi run cmake --preset release && pixi run cmake --build --preset release
```

Red-proof: remove node from the pixi env -> both JS legs SKIP; the batch is not
done until the RUNNER line is pixi-sourced. Plant an `add_library` in the root ->
root-purity gate red. Declare an msrv env while `rust-version` is absent -> the leg
is documented as vacuous, not green.

### B5 - Rules and skills (adjudications 8, 10)

Deliverables:

- Three-way diff with three named trees: `PolyOrch/.agents/rules` (ancestor,
  read-only per C2), ModuKit (current), VE (donor). Merge net-new sections and
  classify every number into one of three spaces: ModuKit-local C/D/PIT, PolyOrch
  provenance pointers (the existing convention in `edit-safety.md`), or VE
  provenance pointers (new; strip the number or mark it explicitly). Never leave a
  bare VE number that has no local entry.
- `scripts/rule-ref-lint.py` (new, small): every C/D/PIT citation in `.agents/rules`
  resolves to a local entry or the named allowlist in
  `scripts/rule-ref-allowlist.txt`; a bare VE number reds.
- Copy `docker.md` verbatim but keep it out of `instructions[]` (on-demand only); a
  rule with no container workflow costs nothing when unmounted.
- Skills: execute Appendix A's triage (copy 30, adapt 5, keep ModuKit's richer
  `book-to-skill`, keep `review-hardcode` as a deprecated alias, retain
  `rust-patterns` / `rust-testing` as ModuKit-only). The five `openspec-*` skills are
  copied but marked inoperative (CLI absent); the directory stays, the routing note
  says so. `.skill_id` files where VE has them; generated `SKILL.md` registry keeping
  the `Skill | Type | Notes` schema; naming convention in `docs/env.md`. Do not delete
  an inoperative skill silently.
- B5 fills the `SKILL.md` registry region created in B2; the markers already exist.
  The Superpowers section and the inoperative notes stay outside the region and are
  never overwritten.
- Update `docs/reference/visiaengine.md` and the `docs/reference/README.md` index
  row: verdict BORROW-PATTERNS -> ADOPTED (non-functional surface, 2026-10-29), plus
  the provenance ledger and three short sections: Adopted surface, Not adopted,
  Provenance. A verdict flip alone does not tell the next reader what changed.

Acceptance:

```bash
python3 scripts/skill-lint.py           # frontmatter, name, size, @refs
python3 scripts/rule-ref-lint.py        # C/D/PIT citations resolve or are allowlisted
pixi run gate-skills                    # (target) registry == directories
grep -rn 'visiaengine\|VisiaEngine' .agents/skills   # only intentional references
```

Red-proof: add a skill directory without a registry row -> `gate-skills` red; plant
`PIT-43` in a merged rule without a provenance marker -> `rule-ref-lint` red. Green
case: the existing PolyOrch provenance pointer `PIT-11` must stay green (it resolves
through the allowlist, not the local ledger).

### B6 - book and SDD skeleton (adjudication 9)

Ordering: run `pixi run mcp index` first (the graph page consumes `.codegraph`); add
`www/` and `.tools/` to `.gitignore` in this batch, not B7, because the build and the
i18n helper create them here.

Deliverables:

- `book/book.toml`, `book/toc.toml` listing README, architecture, env, the
  `docs/modules/00-11` set, a Contracts (SDD) group with the five clauses, and the
  `docs/reference/` profiles; each chapter carries `type` (index / explanation /
  how-to / reference) and `visibility` (public or internal; stage plans are
  internal); `book/web/` graph page; `book/po/messages.pot` +
  `book/po/zh-CN.po` (both generated by `book-i18n pot`; the PO file is tracked and
  may start partially translated); `scripts/book-build.py`, `book-check.py`,
  `book-i18n.py`; pixi `book` feature (mdbook, gettext, markdownlint). The one
  non-pixi install is `mdbook-i18n-helpers` via `cargo-binstall --root .tools`
  (gitignored), exactly as VE does.
- `docs/sdd/` namespaces (`KERNEL`, `EMBED`, `WIRE`, `BUS`, `TRANSPORT`), an
  ownership table, and five real clauses seeded as: `KERNEL-01` lifecycle transition
  table, `EMBED-01` 24-verb parity, `WIRE-01` TL frame prefix, `BUS-01` cross-wall
  publish/subscribe, `TRANSPORT-01` fdpass handoff. The ownership table names the
  owner crate for each. Tags go on existing tests; do not create a second test for a
  clause an existing gate already proves - the gate stays the authority, the clause
  is the index entry.
- Tag convention across languages: `// spec: PREFIX-NN` in Rust/C/C++, `# spec:
  PREFIX-NN` in Python/shell, `// spec: PREFIX-NN` in JS. Scan roots:
  `crates/*/src`, `crates/*/tests`, `test-plugins/`, `examples/`. A clause whose test
  lives in a demo script is out of scope for round 1; name it in the ownership table
  instead.
- `scripts/spec-trace.sh` with a single-line prefix whitelist and the third
  "prefix outside the whitelist" check.
- `scripts/gate-docs.sh` check 13: every `SUMMARY.md` entry under `build/book-src`
  resolves; SKIP when that tree is absent (same convention as VE).

Acceptance:

```bash
pixi run mcp index                      # graph source
pixi run book-build                     # (target) produces www/book/en
pixi run book-check                     # (target) index + graph entries resolve
pixi run book-i18n status               # (target) PO coverage reported
pixi run docs-gen                       # (target) llms.txt includes book + SDD
bash scripts/gate-docs.sh               # (target) check 13 resolves or SKIPs
bash scripts/spec-trace.sh              # (target) 5 pairs aligned
```

Red-proof: add a clause without a test tag -> `spec-trace` red; remove a toc entry's
source -> `book-check` red; remove a PO entry -> `book-i18n status` reports it.

### B7 - CI, IDE, gitignore

Deliverables:

- `.github/workflows/ci.yml` on standby (the remote is gitee; this activates on the
  GitHub-mirror day): `actions/checkout` with `submodules: recursive` (VE's standby
  file omits it - a latent bug, do not copy that line), setup-pixi at the pinned
  version, `pixi install --frozen`, `pixi install -e msrv --frozen`,
  `pixi run agents sync`, `pixi run ci`, MSRV leg.
- `.vscode/tasks.json` and `launch.json` managed regions where PolyOrch provides
  them; `settings.json` keeps the pixi cmake pin and `out/build`.
- `AGENTS.md` STRUCTURE gains the new directories (`book/`, `docs/sdd/`,
  `.serena/`, and the existing `scripts/hooks/`) and the `docs/env.md` pointer; no
  scale numbers. Do not add `.githooks/`: the hook path is `scripts/hooks/`.
- `.gitignore` reorganized by ownership: noise, IDE, build, node, python, pixi,
  secrets, agent host layers, local notes, wrappers, clones. Pre-declare
  `.codebuddy/`, `.trae/`, `.claude/`, `.cursor/`, `.qoder/`, `.serena/cache`,
  `.serena/memories`, `.certs/`, `.tools/`, `www/`, `/package.json`,
  `/package-lock.json`, `CMakeUserPresets.json`, `compile_commands.json`. Keep
  ModuKit's `bindings/py/*` plus the `!bindings/py/plugin_runtime.py` negation and
  the `.refinfo` / `.codegraph` / `out/` blocks.

Acceptance:

```bash
pixi run ci                             # local equivalent of the standby CI
git ls-files -ci --exclude-standard     # empty
git check-ignore -v .pixi .codegraph out target   # expected hits
```

Red-proof: plant a tracked-but-ignored file -> `git ls-files -ci` red; remove
`submodules: recursive` in a scratch workflow -> the configure leg fails (the
PolyOrch gitlink is absent).

### B8 - Host link layer (deferred)

Lands only when a link-based host is enabled (CodeBuddy / Trae / Claude):
`rules-links.py`, `skills-links.py`, `skills-targets.json`, the `.codebuddy` /
`.trae` link layers, Windows `--copy` plus the hashed map, and the forbidden-dir
guard. Not on the current critical path; recorded so the deferral is visible rather
than forgotten.

### 3.9 Done when

B0-B7 are green, B8 is deferred by record, Appendix C is intact, D109 is written and
the high-water sites agree, and a clean clone can run `bash bootstrap.sh` then
`pixi run ci` green.

## 4. Deliberate deviations from VE (recorded, not accidental)

| Deviation | Why |
|---|---|
| `out/` keeps its name; VE uses `cmake-build*/` + `build/` | Adjudication 6: the discipline transfers, the rename costs 15+ references and 5 GB of trees |
| No Wasm / Qt / Install / Packaging modules yet | Adjudication 7: a module must own a lane; VE itself renamed a file for this |
| Host switches follow local verification, not VE's snapshot | Adjudication 8: a wrong path is a dead file (VE D35) |
| Memory keeps 7 files, not VE's 4 | `backlog.md` and `history-archive.md` are ModuKit assets; the digest generator iterates `*.md` |
| `demo-*` names kept, no `smoke-*` rename | 13 gate legs, sabotage levers, and docs reference the current names |
| ModuKit `.opencode` plugin pins kept | VE's plugin list is floating; ModuKit's pins are stronger |
| `.gitattributes` (`*.bat eol=crlf`) kept | VE has none; ModuKit's Windows batch history needs it |
| SDD clause text authored, not copied | VE's clauses are functional contracts; copying them would be a false contract |
| book chapters derive from existing ModuKit docs | VE's book sources are engine-specific; ModuKit's docs already exist |
| No `resources/` directory yet | `test-plugins/` and `examples/` hold the fixtures today; an empty directory is a dead surface |
| ModuKit keeps `deep-low` in the omo dev team (VE uses `deep`) | the `deep`/`deep-low` split is a deliberate ModuKit routing choice (category split 2026-09); models stay premium |

### 4.1 Top risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| 1.85 unsolvable on conda-forge (especially win-64) | medium | record the nearest available; never fake the pin |
| playwright browsers large or absent | medium | probe version only; never download inside a gate |
| serena clangd absent (VE D29) | medium | run Rust/Python backends; record the C++ gap |
| MSRV ladder leaves lock churn | high | throwaway solves; only the final pin commits |
| common-rule merge imports VE numbering | high | `rule-ref-lint.py` + explicit provenance markers |
| book i18n helper not on conda | low | `cargo-binstall --root .tools`, pinned |
| 37 skills inflate host context | medium | route by description; opencode does not load bodies; revisit before CodeBuddy onboarding |
| Windows `bootstrap.bat` never executed | medium | keep the first-run debt explicit; do not claim Windows parity until a green run |

## 5. Open items (grouped by what unblocks them)

Nothing here is a design fork today. Only 5.3 can reopen an adjudication, and only
when its trigger fires.

### 5.1 Probe outputs (the measurement decides the value; no adjudication)

| Item | Probe | Recorded in |
|---|---|---|
| MSRV number | 1.88.0 (probe 2026-10-29: 1.85 red on `libloading 0.9.0` + `napi-build 2.5.0`; 1.88 green in 9.81 s) | B4 result, `Cargo.toml`, msrv pin |
| qoder rules path | native reader: both `.agents/rules` and `.qoder/rules` echoed their tokens (2026-10-29); prefer `.agents/rules`, no generator | B2 result, `agent-targets.json` |
| qoder MCP path | one token entry in project `.mcp.json` / qoder's file | B2 result; user-level only = surface disabled |

### 5.2 Execution tasks (already decided)

- D109 write-back + architecture section 11 + AGENTS + status high-water (B0).
- Provenance/NOTICE fill (B0 skeleton; B1/B5 rows).
- Immediate defects: IP redaction (B0), C1 scope drift (B3), numbering gate bounds
  (B0), legacy `.opencode/serena/` kept (B1).

### 5.3 Trigger-gated forks (adjudicate only when the trigger fires)

| Trigger | Fork |
|---|---|
| MSRV lands at 1.99 only | accept 1.99 vs pay for 1.85 compatibility |
| qoder project-level MCP unreadable | accept user-level MCP out of scope vs build a qoder-specific adapter |
| install-tree milestone opens | tree contents and platform layout |

### 5.4 Already dispositioned by section 4 (not open)

- `demo-*` vs `smoke-*`: keep `demo-*`; aliases only if a VE-shaped task name is required.
- Memory 7 vs 4: keep 7; copy the digest machinery, not the layout.
- `resources/`: not created until a fixture class needs it.
- B8 host link layer: land when a link-based host is enabled.

## 6. Non-goals

- No functional code, contract, binding, fixture, or demo-assertion changes.
- No PolyOrch submodule edits (C2: upstream flow, separate repository).
- No v1 surface changes (D87) and no v2 verb additions.
- No bulk rewrite of SDD clauses or capability-ledger numbers.
- No renumbering of existing C/D/PIT entries.

## 7. References

VE anchors (read-only):

- `AGENTS.md:174-190` - single-source rule and host onboarding discipline.
- `scripts/agent-targets.json` - host target schema, `generated_tracked`, guard flags.
- `scripts/agent_common.py` - shared guard / classify / marker / hub budget.
- `scripts/memory-aggregate.py` - digest windows and marker region.
- `scripts/mcp-links.py` - two host shapes and the non-destructive merge.
- `scripts/mcp-serve.mjs` - pinned npm install, pin-drift failure, stdout discipline.
- `scripts/gate-i18n.sh` - `EXTS` / `SKIP_PATH_PREFIX` / `ALLOW` tables.
- `pixi.toml` - feature split, solve groups, target-scoped dependencies.
- `cmake/VisiaEngine*.cmake` - assembly modules and lane ownership.
- `docs/env.md` - output territory, MCP lifecycle, O5 PATH, book workflow.
- `docs/sdd/` + `scripts/spec-trace.sh` - clause/test bidirectional trace.
- `book/` + `scripts/book-build.py` - mdBook assembly and PO workflow.

ModuKit anchors:

- `PolyOrch/.agents/rules` - the named ancestor for the rules three-way diff (read-only, C2).
- `scripts/check.sh` - the local gate whose `--list` mode feeds CONSTRAINTS.
- `scripts/hooks/commit-msg` - C3 hook that the new pre-commit must coexist with.
- `docs/reference/visiaengine.md` - reference profile that gains the ADOPTED verdict and
  the provenance ledger.

## 8. Change log

- 2026-10-29: created; ten adjudications recorded; B1-B7 scoped; deviations and open
  items listed.
- 2026-10-29 revision 1: P0/P1 review fixes - B0 housekeeping, batch dependency graph,
  `ci` ownership pinned, dead link generators deferred to B8, cross-language SDD tags,
  provenance/NOTICE, per-server probe matrix, MSRV decision point, book ordering and
  PO files, CI submodules, appendices A-D.
- 2026-10-29 revision 2: two follow-up adjudications recorded - MSRV method = C
  (measured-lowest ladder plus the equality gate), qoder probe = A (per-surface,
  skills already proven, rules/MCP token probes).
- 2026-10-29 revision 3: open items grouped into probe outputs / execution /
  triggers; D109 and the numbering gate moved into B0; a `project` target added so
  `.mcp.json` has a producer; qoder schema corrected (skills are native); `ci` msrv
  ordering fixed; wrapper + gate-docs check 12 added; SDD clauses named; top risks
  and the done criterion added.
- 2026-10-29 revision 4: review fixes - serena `project.yml` and the memory-source
  rule, msrv appended to `ci`, skill triage corrected (test-harness adapt, openspec
  inoperative), plan committed in B0, book toc + `docs-gen`, gate ownership
  clarified, pixi wrapper untrack mechanics, rule-ref green case, batch ownership
  table, bootstrap-aware done criterion.
- 2026-10-29 revision 5: B4 sequenced after B2; bootstrap calls `agent-sync`
  (advisory) and ends with the `agents status` hint; dependency graph updated;
  AGENTS single-source section lands in B2; `gate-abi` restored to B3 and
  `cmake-smoke` to B4; CONSTRAINTS/`ci` collapsed to `check.sh` as the only source
  (adjudication 16).
- 2026-10-29 revision 6: P1 review fixes - serena `project.yml` adapt, LSP wrapper
  plus the with-node omission, `probe_mode`, `SKILL.md` markers, toc
  type/visibility, book SUMMARY check 13, CONSTRAINTS anti-gaming half, llms source
  list, reference profile body, omo/tier branding, copy-ledger rows.
- 2026-10-29 revision 7: review fix - SKILL.md marker moved to B2 (before the first
  sync), LSP wrapper made probe-conditional, `cargo deny` offline semantics fixed,
  env gate contract, llms missing-source skip, cmake-smoke negative paths named.
- 2026-10-29 probe results: MSRV 1.88.0 (1.85 red on libloading/napi-build); qoder
  rules are native on both `.agents/rules` and `.qoder/rules`; qoder MCP stays
  pending until B1 creates the root `.mcp.json`.
- 2026-10-30 B4a landed: `mise.toml` and `rust-toolchain.toml` deleted;
  `bootstrap.sh/.bat` are pixi-only (submodule guard, pixi-init, PATH line, hooks,
  advisory agents sync); JS legs report `RUNNER: node v22.23.4 (pixi-env)`;
  CMake presets already mirrored the pixi routes; `cargo metadata --locked` OK;
  full gate green in 86.57 s. B4b owns MSRV + cmake-smoke.
- 2026-10-30 B3 landed: `check.sh --list` = 32 legs (gate-i18n replaces the C1
  hand-list; gate-docs absorbs doc-liveness; gate-abi exact 24-symbol set;
  check-promises narrow phrase lint; gate-mcp/gate-agents wired; ci green 86.59 s).
  Deviations: gate-docs does not duplicate check-embed-parity (that leg owns the
  verb list) and omits the prototype-name/package-count/llms-number checks (no
  claim, or superseded by the exact gate-abi set); gate-abi's C++ demo stays the
  existing demo-stage4-m1; gate-i18n also scans untracked non-ignored files.

## Appendix A - Skill triage (37 VE directories + 2 ModuKit-only)

- **Copy (30)**: `adjudication-walkthrough`, `ai-slop-cleanup`,
  `api-interface-design`, `ast-grep-rewrites`, `ci-cd-automation`,
  `code-review-and-quality`, `code-simplification`,
  `constraint-driven-development`, `context-engineering`,
  `debugging-and-error-recovery`, `deprecation-and-migration`, `doc-audit`,
  `documentation-and-adrs`, `ecosystem-scan`, `incremental-implementation`,
  `lesson-review`, `openspec-apply-change`, `openspec-archive-change`,
  `openspec-explore`, `openspec-propose`, `openspec-sync-specs`,
  `performance-optimization`, `planning-and-task-breakdown`, `security-hardening`,
  `shipping-and-launch`, `skill-router`, `source-driven-development`,
  `subagent-orchestration`, `test-driven-development`, `think-before-act`.
  The five `openspec-*` skills are copied but marked inoperative (CLI absent).
- **Adapt (5)**: `browser-testing` (no web surface: mark inoperative or rewrite for
  plugin UI), `observability-and-instrumentation` (bus, event ring, debug-reader),
  `repo-triage` (ModuKit layers: core / transport / codegen / contracts / bindings /
  plugin door), `rust-engineering` (crates + contracts + bindings), `test-harness`
  (SDD namespaces and the five-clause skeleton, not VE's clause families).
- **Keep ModuKit's richer version (1)**: `book-to-skill` (634 lines vs VE's 499).
- **Deprecated alias (1)**: `review-hardcode` -> `security-hardening`; keep the
  directory, never route to it.
- **ModuKit-only (2)**: `rust-patterns`, `rust-testing`.
- Result: 39 skill directories (37 VE + 2 ModuKit-only), one of which is a deprecated
  alias.

## Appendix B - Copy ledger template

| Source (VE) | Destination (ModuKit) | Mode | Batch |
|---|---|---|---|
| `scripts/agent_common.py` | `scripts/agent_common.py` | verbatim | B1 |
| `scripts/mcp-links.py` | `scripts/mcp-links.py` | adapt (target schema) | B1 |
| `scripts/mcp-serve.mjs` | `scripts/mcp-serve.mjs` | verbatim | B1 |
| `scripts/mcp-cli.py` | `scripts/mcp-cli.py` | adapt (serena env/index) | B1 |
| `scripts/gate-mcp.py` | `scripts/gate-mcp.py` | adapt (`probe_mode` + O5 WARN) | B1 |
| `scripts/mcp-file-guard.py` | `scripts/mcp-file-guard.py` | verbatim | B1 |
| `scripts/mcp-launch.sh` / `.bat` | same | verbatim | B1 |
| `scripts/memory-aggregate.py` | `scripts/memory-aggregate.py` | adapt (WINDOW map) | B2 |
| `scripts/gate-i18n.sh` | `scripts/gate-i18n.sh` | adapt (EXTS/ALLOW) | B3 |
| `scripts/gate-abi.sh` | `scripts/gate-abi.sh` | adapt (header-derived whitelist) | B3 |
| `scripts/cmake-smoke.sh` | `scripts/cmake-smoke.sh` | adapt (`out/` tree) | B4 |
| `.serena/project.yml` | `.serena/project.yml` | adapt (server set) | B1 |
| `.opencode/init-lsp-wrap.mjs` | `.opencode/init-lsp-wrap.mjs` | conditional (probe; omit if LSP starts) | B1 |
| `scripts/book-build.py` | `scripts/book-build.py` | adapt (paths/toc) | B6 |

A copied file without a row fails review. The batch commit body names
`VE <path> @ <commit>`; the row lands in `docs/reference/visiaengine.md`.

## Appendix C - Do-not-regress list

- Contract gates: `check-embed-parity.sh`, `check-capability-ledger.sh`,
  `check-lane-table.sh`, `check-wire-parity.sh`.
- Thirteen demo legs with their PASS/SKIP grammar, RUNNER census, and sabotage
  levers.
- `scripts/hooks/commit-msg` (C3 commit-shape) and the C0-C4 conventions.
- `bindings/py/*` plus the `!bindings/py/plugin_runtime.py` negation.
- `.refinfo/` and `PolyOrch/` stay read-only; no edits, no `git add`.
- v1 surfaces stay frozen (D87); no v2 verb additions in this effort.

## Appendix D - Measured baselines

Record these at B0 and compare at each batch close:

- `AGENTS.md` size after the first sync: <= 30 KB.
- Current `check.sh` leg count, captured at B0 from the existing script; after B3,
  `bash scripts/check.sh --list | wc -l` must report the same count. No leg may
  disappear.
- Skill directories after B5: 39 (37 VE + 2 ModuKit-only).
- `bash scripts/check.sh` wall time: no more than a 20% regression after B3.
- `pixi run node -v`: 22.x, sourced from `.pixi`, after B4.
- `git ls-files -ci --exclude-standard`: empty after B0 and at every batch close.

### B0 baseline (measured 2026-10-29)

- `AGENTS.md`: 11,535 bytes before the first generated digest (limit 30 KB).
- `scripts/check.sh`: 285 lines; 16 numbered/leg headers (13 demo legs);
  `bash scripts/check.sh` all green in 214.38 s (ctest 9/9).
- `.agents/memorys/status.md`: 7,944 bytes against the 8,000-byte cap (56 bytes of
  headroom; B1 must offload a paragraph to `history-archive.md` before adding lines).
- `git ls-files -ci --exclude-standard`: empty (after the `.omo` negation fix).
- MSRV probe: 1.88.0 (1.85 red: `libloading 0.9.0` and `napi-build 2.5.0` require
  1.88; 1.88 green, `cargo check --workspace` 9.81 s).
- qoder / node baselines are pending the B2/B4 probes.
