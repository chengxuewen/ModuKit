# PROJECT KNOWLEDGE BASE

**Generated:** 2026-10-21 (doc-audit regen pass, init-deep-equivalent; prior 2026-10-19) · **Branch:** master · **Commits:** see `git log` (first committed 2026-10-09; regenerate header via /init-deep)

## OVERVIEW

ModuKit — modular plugin host framework & polyglot extension toolkit (Rust core, C ABI as the only cross-language exit). **Stages 1-3 are CLOSED** (D88/D91; the UI-composition tier stays demoted until a real beachhead demand, D89-K2); **STAGE-4 INFRA volume: C/C++ bindings line shipped M1+M2** 2026-10-20, ruled in as D93/D94 (M3 service-call door landed 2026-10-21), volume ACCEPTED AND CLOSED 2026-10-21 (D95); wave-2: v2 bus LANDED (D96) + debug-reader minimal face LANDED (D97) + closeout LANDED (D98: reader gate leg, cold-census coverage - core debt cleared) + CLOCK HEARING DRAWN (D99, T1+T2 LANDED 2026-10-22) + PYTHON VOLUME ACCEPTED (D100/D101, 2026-10-23; nine-leg gate + pinned coverage boundary); remainder armed in plan §6. Working code: `crates/modukit-core / -runtime / -transport / -codegen` + `bindings/cpp` + contracts C originals (test totals volatile — live in the plans of record; v1 surfaces frozen STABLE per D87). Design baseline whitepaper v1.0 + adjudicated architecture (`docs/architecture.md` + `docs/modules/00-11`); decision ledger `.agents/memorys/decisions.md` at D1-D101 (D53 reserved), index architecture.md §11. Kernel v0 parked outside repo.

## STRUCTURE

```
├── README.md / SKILL.md      # front door / skills registry
├── docs/whitepaper.md        # v1.0 canon - crate names §2.3, roadmap §8 (superseded by §9/§10/§12)
├── docs/architecture.md      # adjudicated baseline (§9 deltas, §11 decision index D1-D99, §12 roadmap (amended at D95 2026-10-21))
├── docs/modules/             # 00-11 deep dives (placement, service model, lifecycle, codegen,
│                             #   manifest, bus, transport, walkthrough, record, observability, media base)
├── docs/stage-{1,2,3,4}-plan.md # PLANS OF RECORD: S1-S3 closed, S4 = INFRA volume (D93/D95: M1-M3 landed, ACCEPTED)
├── docs/stage-5-plan.md        # wave-2 target: D96 hearing + N1 door + N2 crossing LANDED (D97 debug-reader face too)
├── docs/capability-ledger.md # embed/plugin verb capability map (check-capability-ledger leg)
├── docs/codegraph-bootstrap.md # codegraph MCP index bootstrap note
├── docs/reference/           # external profiles + audits (roster: its README.md; 00 §4 maintainer register)
├── crates/modukit-core/      # kernel: lifecycle/registry/manifest/bundle/bus/clock/logger/
│   │                         #   executor{dl,exe}+supervisor/proc_wire TL-frames/proxy/observability
│   ├── tests/                # incl. dual_placement + step_replay (v1-frozen proofs)
│   └── ../modukit-runtime/   # bins: modukit-run (shadow-runner), modukit-host (supervisor loop)
├── crates/modukit-transport/ # seam + socketpair/memfd + SCM_RIGHTS fdpass + vendor-shm (iceoryx2 =0.10.0,
│                             #   non-default feature, one-module confinement, D76/D90 lineage)
├── crates/modukit-codegen/   # protoc plugin: Rust skeleton + C ABI shade + fdset decoder
├── contracts/                # .proto ledger + checked-in fdsets (regen-diff gated) + embed.h/plugin.h C
│                             #   originals, HostEmbedV1 24 verbs (sup service-call door, M3), three-way parity-gated (D93/D94)
├── examples/ test-plugins/   # demo shelf / fixture crates, rs + cpp bands (D93); plugins/ = reserved, NOT created
├── bindings/cpp/             # C++ wrapper shelf: generated faces + live probe (Visia-grammar)
├── CMakeLists.txt + cmake/   # PolyOrch-driven graph: pixi translation, cargo lanes, ctest bridges, demo lanes
├── pixi.toml / mise.toml     # build envs (tools/rust/cov; vendor-test task) / host tooling pins
├── scripts/                  # check.sh (D50 local gate incl ALL NINE demo legs + commit-shape),
│                             #   demo-stage{1,2}.sh + demo-stage4-{m1,m2}.sh + demo-stage5-n2.sh + demo-wave2-{reader,clock,py,py-sup}.sh, regen-fdset/golden.sh,
│                             #   check-embed-parity/capability-ledger.sh, skill-lint.py
├── .agents/                  # memorys/ + rules/ + skills/ governance tree → .agents/AGENTS.md
└── .vscode/ .opencode/ .omo/ # IDE cmake pin (PIT-7) / agent config / OMO state
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| What ModuKit is / naming authority | `docs/whitepaper.md` | §2.3 canonical; §8 roadmap superseded by architecture §12 |
| Current architecture + deltas | `docs/architecture.md` + `docs/modules/` | §11 = decision index D1-D101; §9 = whitepaper deltas; §12 = roadmap |
| What stage are we in, what's next | `.agents/memorys/status.md` | loaded every turn; plans of record in docs/stage-*.md |
| Which embed/plugin verbs exist (capability map) | `docs/capability-ledger.md` + `contracts/modukit_embed.h` | 24 verbs = EMBED_VERBS; parity + ledger legs in check.sh (D93/D94) |
| Which skills exist / when they fire | `SKILL.md` | registry incl. inoperative flags |
| Conventions & gates | `.agents/memorys/conventions.md` | C0 executable-checks, C1 English, C2 branch/pin, C3 commit-shape |
| v1 frozen surface (what may NOT break) | architecture §11 D87 row + `docs/stage-2-plan.md` §9 | amendment policy = D83 (tail-add by record, break by hearing) |
| Dev toolchain | `bootstrap.sh`/`.bat` + `mise.toml`/`pixi.toml` | ALL cargo/rustfmt via `pixi run` (mise shims broken, PIT-7 class) |

## CODE MAP

**Live** — codegraph MCP (v1.6.2 launcher) indexes real code since Stage-1; value covers crates/. Entry points: `kernel.rs` (state machine + doors), `executor_proc.rs`+`proc_wire.rs`+`proc_supervisor.rs` (exe placement), `modukit-run`/`modukit-host` (process side), `bundle.rs` (D28 install protocol), `seam.rs`/`fdpass.rs`/`vendor_shm.rs` (transport).

## CONVENTIONS

- **C1**: all artifacts English; chat follows user. (CJK scan is the canonical block below — keep it synced with conventions.md verbatim.)
- **C0**: every constraint carries a runnable check, observed red once before trusted.
- Memory: `C{n}` / `D{n}` / five-part `PIT-{n}`; high-water today D101/C3/PIT-29; numbering gate in status.md guards bounds.
- **C2**: ModuKit lands on master; PolyOrch edits on dev-modukit, tests blessed on main; pin = fresh dev-modukit tip via standalone bumps.
- **C3**: commit subject ≤60 (`type(scope): doing`), body = wrapped ` - ` bullets ≤100, ONE physical line per bullet (machine leg in check.sh; continuation lines are the recurring self-violation this leg caught x5).
- `Cargo.lock` commits with `Cargo.toml`.

## ANTI-PATTERNS (THIS PROJECT)

- Treating `compositor` / unshipped crate names as existing code — D87 froze what exists; v2 verbs are SIGNAGE (D82).
- Editing/citing `PolyOrch/` or `.refinfo/` as project content (submodule = upstream flow per C2; reference clones git-ignored).
- Quoting pre-2026-10-15 "zero source / docs-only repo" claims — stale, caught by doc-audit.
- Workspace-wide `cargo fmt` (edit-safety #12), `pgrep/pkill -f` on own cmdline (#15/#16), tty-pipe test counting (PIT-9), batch doc edits mid-agent-run (PIT-14).
- Registering the D50 fast gate as a ritual: scope the verification to the change (edit-safety #24) - but demo legs now RUN IN check.sh, don't skip them.

## COMMANDS

```bash
# C1 English-artifact gate (canonical copy lives in .agents/memorys/conventions.md - keep synced)
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ contracts/ examples/ docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ test-plugins/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"

# full local gate - fast checks incl ALL NINE demo legs + ctest bridge (D50 + audit leg + stage4 M1/M2/M3 + wave-2 N2 + D98 reader + T2 clock + D100 python x2)
bash scripts/check.sh

# stage acceptance gates
bash scripts/demo-stage1.sh      # STAGE1-DEMO PASS
bash scripts/demo-stage2.sh      # STAGE2-DEMO PASS (sabotage hook documented)
bash scripts/demo-stage4-m1.sh    # demo-stage4-m1: PASS (embed face; clean SKIP when cpp band absent)
bash scripts/demo-stage4-m2.sh    # demo-stage4-m2: PASS (sup rung + ORPHAN-PROOF; sabotage hook)
bash scripts/demo-stage5-n2.sh     # demo-stage5-n2: PASS (v2 bus across the wall; sabotage hook)
bash scripts/demo-wave2-reader.sh  # demo-wave2-reader: PASS (reader self-check; sabotage lever)
bash scripts/demo-wave2-clock.sh   # demo-wave2-clock: PASS (plugin bell crosses the wall at the dial; sabotage lever)
bash scripts/demo-wave2-py.sh      # demo-wave2-py: PASS (python walks the 16 rung-0 verbs on the generated skin; lever)
bash scripts/demo-wave2-py-sup.sh  # demo-wave2-py-sup: PASS (python sup rung + byte-FIDELITY + orphan census; two levers)
bash scripts/coverage-boundary.sh  # D79 boundary run - all regions >= 80 (long; stage/acceptance edges only)

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
- The agent box has no outbound network (curl and webfetch both dead) - `.refinfo/` local clones are the first-class verification asset for external-project claims; mark anything unpinable as memory-grade explicitly.
- **Push decision is the maintainer's** (origin=gitee, push only on explicit instruction; C3 history rewrites stay local).
- Dangling-refs family: full list re-hung to Stage-3 M3S-1 via backlog dispositions 2026-10-19; rust-patterns vendored, python/golang-testing pointers still open.
- plugins/ shelf: grammar-reserved, NOT created empty (ruling F+B); first production plugin opens it with its landing.
- License decided (D1: MIT OR Apache-2.0); domain guidance `modukit.dev` / `modukit.rs`.
