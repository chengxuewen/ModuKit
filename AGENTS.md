# PROJECT KNOWLEDGE BASE

**Generated:** 2026-10-19 (doc-audit A-ruling regen pass, init-deep-equivalent) · **Branch:** master · **Commits:** see `git log` (first committed 2026-10-09; regenerate header via /init-deep)

## OVERVIEW

ModuKit — modular plugin host framework & polyglot extension toolkit (Rust core, C ABI as the only cross-language exit). **Stage-1 (in-process kernel) and Stage-2 (process placement) are CLOSED** (D88); Stage-3 is open as a stability/docs stage — its UI-composition tier stays demoted until a real beachhead demand (D89-K2). Working code: `crates/modukit-core / -runtime / -transport / -codegen` (192 tests; v1 surfaces frozen STABLE per D87). Design baseline whitepaper v1.0 + adjudicated architecture (`docs/architecture.md` + `docs/modules/00-11`); decision ledger `.agents/memorys/decisions.md` at D1-D90 (D53 reserved), index architecture.md §11. Kernel v0 parked outside repo.

## STRUCTURE

```
├── README.md / SKILL.md      # front door / skills registry
├── docs/whitepaper.md        # v1.0 canon - crate names §2.3, roadmap §8 (superseded by §9/§10/§12)
├── docs/architecture.md      # adjudicated baseline (§9 deltas, §11 decision index D1-D90, §12 roadmap)
├── docs/modules/             # 00-11 deep dives (placement, service model, lifecycle, codegen,
│                             #   manifest, bus, transport, walkthrough, record, observability, media base)
├── docs/stage-{1,2,3}-plan.md # PLANS OF RECORD: S1 complete, S2 accepted (D88), S3 active (D89/D90)
├── docs/reference/           # external profiles + audits (roster: its README.md; 00 §4 maintainer register)
├── crates/modukit-core/      # kernel: lifecycle/registry/manifest/bundle/bus/clock/logger/
│   │                         #   executor{dl,exe}+supervisor/proc_wire TL-frames/proxy/observability
│   ├── tests/                # incl. dual_placement + step_replay (v1-frozen proofs)
│   └── ../modukit-runtime/   # bins: modukit-run (shadow-runner), modukit-host (supervisor loop)
├── crates/modukit-transport/ # seam + socketpair/memfd + SCM_RIGHTS fdpass + vendor-shm (iceoryx2 =0.10.0,
│                             #   non-default feature, one-module confinement, D76/D90 lineage)
├── crates/modukit-codegen/   # protoc plugin: Rust skeleton + C ABI shade + fdset decoder
├── contracts/                # .proto ledger + checked-in fdsets (regen-diff gated)
├── examples/ test-plugins/   # demo shelf (rs band) / fixture crates - rulings F+B; plugins/ = reserved, NOT created
├── CMakeLists.txt + cmake/   # PolyOrch-driven graph: pixi translation, cargo lanes, ctest bridges, demo lanes
├── pixi.toml / mise.toml     # build envs (tools/rust/cov; vendor-test task) / host tooling pins
├── scripts/                  # check.sh (D50 local gate incl demo legs + commit-shape), demo-stage{1,2}.sh,
│                             #   regen-fdset/golden.sh, skill-lint.py
├── .agents/                  # memorys/ + rules/ + skills/ governance tree → .agents/AGENTS.md
└── .vscode/ .opencode/ .omo/ # IDE cmake pin (PIT-7) / agent config / OMO state
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| What ModuKit is / naming authority | `docs/whitepaper.md` | §2.3 canonical; §8 roadmap superseded by architecture §12 |
| Current architecture + deltas | `docs/architecture.md` + `docs/modules/` | §11 = decision index D1-D90; §9 = whitepaper deltas; §12 = roadmap |
| What stage are we in, what's next | `.agents/memorys/status.md` | loaded every turn; plans of record in docs/stage-*.md |
| Which skills exist / when they fire | `SKILL.md` | registry incl. inoperative flags |
| Conventions & gates | `.agents/memorys/conventions.md` | C0 executable-checks, C1 English, C2 branch/pin, C3 commit-shape |
| v1 frozen surface (what may NOT break) | architecture §11 D87 row + `docs/stage-2-plan.md` §9 | amendment policy = D83 (tail-add by record, break by hearing) |
| Dev toolchain | `bootstrap.sh`/`.bat` + `mise.toml`/`pixi.toml` | ALL cargo/rustfmt via `pixi run` (mise shims broken, PIT-7 class) |

## CODE MAP

**Live** — codegraph MCP (v1.6.2 launcher) indexes real code since Stage-1; value covers crates/. Entry points: `kernel.rs` (state machine + doors), `executor_proc.rs`+`proc_wire.rs`+`proc_supervisor.rs` (exe placement), `modukit-run`/`modukit-host` (process side), `bundle.rs` (D28 install protocol), `seam.rs`/`fdpass.rs`/`vendor_shm.rs` (transport).

## CONVENTIONS

- **C1**: all artifacts English; chat follows user. (CJK scan is the canonical block below — keep it synced with conventions.md verbatim.)
- **C0**: every constraint carries a runnable check, observed red once before trusted.
- Memory: `C{n}` / `D{n}` / five-part `PIT-{n}`; high-water today D90/C3/PIT-10; numbering gate in status.md guards bounds.
- **C2**: ModuKit lands on master; PolyOrch edits on dev-modukit, tests blessed on main; pin = fresh dev-modukit tip via standalone bumps.
- **C3**: commit subject ≤60 (`type(scope): doing`), body = wrapped ` - ` bullets ≤100, ONE physical line per bullet (machine leg in check.sh; continuation lines are the recurring self-violation this leg caught x5).
- `Cargo.lock` commits with `Cargo.toml`.

## ANTI-PATTERNS (THIS PROJECT)

- Treating `compositor` / unshipped crate names as existing code — D87 froze what exists; v2 verbs are SIGNAGE (D82).
- Editing/citing `PolyOrch/` or `.refinfo/` as project content (submodule = upstream flow per C2; reference clones git-ignored).
- Quoting pre-2026-10-15 "zero source / docs-only repo" claims — stale, caught by doc-audit.
- Workspace-wide `cargo fmt` (edit-safety #12), `pgrep/pkill -f` on own cmdline (#15/#16), tty-pipe test counting (PIT-9), batch doc edits mid-agent-run (PIT-14).
- Registering the D50 fast gate as a ritual: scope the verification to the change (PIT-24) - but demo legs now RUN IN check.sh, don't skip them.

## COMMANDS

```bash
# C1 English-artifact gate (canonical copy lives in .agents/memorys/conventions.md - keep synced)
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ contracts/ examples/ docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"

# full local gate - fast checks incl BOTH demo gates (D50 + 2026-10-19 audit leg)
bash scripts/check.sh

# stage acceptance gates
bash scripts/demo-stage1.sh      # STAGE1-DEMO PASS
bash scripts/demo-stage2.sh      # STAGE2-DEMO PASS (sabotage hook documented)

# build/test via pixi ONLY (mise cargo/rustfmt shims are broken on this box)
export PATH="$HOME/.pixi/bin:$PATH"
pixi run cargo test --workspace          # totals: parse from redirected file (PIT-9)
pixi run cargo clippy --workspace --all-targets -- -D warnings
pixi run vendor-test                     # vendor-shm lane (17-18s cold, not in fast gate)
pixi run -e cov cargo llvm-cov --workspace   # D79 80-line at stage boundaries

# cmake graph (IDE/ctest surface)
pixi run configure && pixi run build && pixi run ctest --test-dir out/build
```

## NOTES

- `.opencode/node_modules` + npm lock serve opencode plugins, not project deps.
- **Push decision is the maintainer's** (origin=gitee, push only on explicit instruction; C3 history rewrites stay local).
- Dangling-refs family: full list re-hung to Stage-3 M3S-1 via backlog dispositions 2026-10-19; rust-patterns vendored, python/golang-testing pointers still open.
- plugins/ shelf: grammar-reserved, NOT created empty (ruling F+B); first production plugin opens it with its landing.
- License decided (D1: MIT OR Apache-2.0); domain guidance `modukit.dev` / `modukit.rs`.
