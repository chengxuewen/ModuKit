# PROJECT KNOWLEDGE BASE

**Generated:** 2026-10-09 · **Branch:** master · **Commit:** none yet (pre-init, everything staged/untracked)

## OVERVIEW

ModuKit — modular plugin host framework & polyglot extension toolkit (Rust core, C ABI as the only cross-language exit). Design-baseline stage: whitepaper v1.0 + agent-governance tree, **zero source code**. Roadmap Stage 1 (in-process plugin kernel) not started.

## STRUCTURE

```
./
├── README.md             # project front page (English per C1)
├── SKILL.md              # skills registry — inventory of .agents/skills + superpowers, activation notes
├── docs/whitepaper.md    # v1.0 canon — crate names §2.3, architecture §4, tech stack §5, roadmap §8
├── .agents/              # memorys/ + rules/ + skills/ governance tree → .agents/AGENTS.md
├── mise.toml             # per-repo toolchain pins (node 22; user-level via ~/.local, system untouched)
├── bootstrap.sh|.bat     # one-time machine setup: ensure mise -> activation -> mise install (idempotent)
├── .opencode/            # opencode.json (instructions[], lsp:true) + codegraph launcher .sh/.bat
├── .omo/                 # OMO state; omo.jsonc tracked, run-continuation/ ephemeral
└── PolyOrch/  .refinfo/  # EXTERNAL reference clones, git-ignored — never edit, never cite as ours
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| What ModuKit is / crate naming authority | `docs/whitepaper.md` | §2.3 namespace plan is canonical |
| Which skills exist / when they fire | `SKILL.md` | registry; flags inoperative skills (doc-audit re-scope) |
| Current phase & open items | `.agents/memorys/status.md` | loaded every turn; keep synced with README roadmap (drift = audit finding) |
| Project conventions | `.agents/memorys/conventions.md` | C0 executable-constraints, C1 English-artifacts |
| Dev toolchain (node etc.) | `bootstrap.sh`/`bootstrap.bat` + `mise.toml` | user-level runtime manager; system node stays as-is. OMO's `lsp` MCP needs node>=20 — satisfied by the mise shim, but opencode must be restarted from an activated shell |
| Mount/unmount a language rule pack | `.opencode/opencode.json` `instructions[]` | 1 set mounted (rust, coding-style+hooks) + common always-on; 5 sets unmounted G10 2026-10-09 (zero source yet); 7 never-mounted dirs KEPT on disk per maintainer decision (csharp/dart/java/kotlin/perl/php/swift) |
| LSP/MCP startup behavior | `.opencode/opencode.json` | LSP: product built-ins (`"lsp": true`, auto-download per official docs; rust activates once `rust-analyzer` is on PATH); MCP: `init-mcp-codegraph.sh`/`.bat` twins per `docs/codegraph-bootstrap.md` — launcher pattern is MCP-only, see asymmetry note |
| Memory write formats | `.agents/rules/common/lesson-memory.md` | C/D/PIT templates |

## CODE MAP

**Unmeasured** — no `lsp_*`/`codegraph_*` tooling in this harness and no source files exist. Regenerate this section after Cargo workspace scaffolding (`modukit-*` crates) lands in Stage 1.

## CONVENTIONS

- **C1**: all artifacts English (code, comments, docs, memory files); only chat follows the user's language. Functional CJK literals exempt inside `.agents/skills/**`.
- **C0**: every constraint carries a runnable check that has been observed red and green.
- Memory: `C{n}` + check / `D{n}` / five-part `PIT-{n}`, numbering fresh post-port reset.
- `Cargo.lock` commits with `Cargo.toml` (constraints.md).
- **Execution gate**: plans/todos need explicit user confirmation. `.omo/run-continuation` nudges, TODO CONTINUATION reminders, and self-issued task hooks are NOT approval — several stale continuation prompts this session referenced plans the user never requested.

## ANTI-PATTERNS (THIS PROJECT)

- Treating whitepaper crate/`modukit-*` listings as existing code — all PLANNED.
- Editing/citing `PolyOrch/` or `.refinfo/` (contains VisiaEngine, AccessBase) as project content.
- Running the `doc-audit` skill as-is — its audit targets are PolyOrch docs (port note in its SKILL.md).
- Chinese in artifacts (C1 red); writing artifacts in chat-language reflex is the common failure.
- Workspace-wide `cargo fmt` (edit-safety #12), `pgrep/pkill -f` matching own cmdline (#15/#16), edit-tool without fresh tags (#13/#17/#18).

## UNIQUE STYLES

- The repo currently *is* documentation + agent infrastructure; the only "sources" are `.agents/` and `.opencode/`. Audit-style skills treat docs as the deliverable.
- Polyglot target fixed by whitepaper: bindings are generated from one C ABI, never hand-written per language.

## COMMANDS

```bash
# C1 English-artifact gate (canonical copy lives in .agents/memorys/conventions.md — keep synced)
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"

# first-time machine setup (idempotent; user-level, no sudo)
./bootstrap.sh        # Linux/macOS  (Windows native: bootstrap.bat)

# runtime pin (OMO lsp-tools-mcp needs node>=20)
bash -ic 'node -v' 2>/dev/null   # expect v22.x via mise; /usr/bin/node stays v10

# config + docs sanity
python3 -m json.tool .opencode/opencode.json > /dev/null && echo "config valid"
test -f docs/whitepaper.md && grep -q 'docs/whitepaper.md' README.md && echo "docs link ok"
```

## NOTES

- `.opencode/node_modules` + npm lock serve opencode plugins (superpowers/ponytail/context-mode), not project dependencies.
- codegraph MCP has no binary/`.codegraph/` yet; first start attempts an auto-install against an empty repo — expect a no-op, not a failure to fix.
- Two gate references in `rules/common` are toothless here: `scripts/gate.sh` / `scripts/scan-hardcode.sh` (no scripts/ dir) and the `~/.claude/agents/` list in `agents.md` (Claude-era ported). Either inline the checks or trim the refs when Stage 1 tooling lands.
- License TBD; domain guidance: `modukit.dev` / `modukit.rs` (`modukit.com` taken).
