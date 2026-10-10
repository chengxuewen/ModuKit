# Code Edit Safety

> **Target audience**: AI agents editing PolyOrch source code.
> **Violation of these rules causes token waste from repeated fix cycles.**

## Tool Selection

| Change Size | Tool | Reason |
|-------------|------|--------|
| Rewrite entire function/file | `write` | Guarantees brace balance, no stale lines |
| ≤20 line single-location edit | `edit` | Minimal diff, safe for small changes |
| Structural pattern replacement | `ast_grep_replace` | Syntax-aware, preserves matching |
| Complex multi-file refactor | Delegate to subagent | Isolated context, verify independently |

## Forbidden Patterns

| Anti-Pattern | Why |
|--------------|-----|
| `sed` for code modification | Quote escaping errors, regex silent failures |
| Multiple sequential `edit` calls without re-reading | Line numbers drift, stale hash IDs |
| Deleting a line by replacing with empty `lines: []` and assuming brace count is still correct | May leave unbalanced braces |
| Appending `}` to "fix" an unclosed delimiter without counting braces first | Masks root cause, may create double-close |

## Verify Immediately

After EVERY code change (edit, write, or ast_grep_replace):

```
Rust:   cargo check -p <crate>       (5-15s)
TS/TSX: npx tsc --noEmit             (3-5s)
YAML:   docker compose config --quiet (compose file)
Shell:  bash -n <script>             (script)
```

### Batch edits array must be verified per-operation (PIT-41)

Multiple replace operations referencing adjacent areas can easily misalign boundary line numbers/content (one operation may overwrite another's reserved region). Run the corresponding format validation immediately after each edit call; if corruption is found, re-read the file to restore, do not layer fixes on top.

If verification fails, STOP. Do NOT apply another edit on top. Instead:
1. `git diff` to see what changed
2. If the change is wrong, `git checkout -- <file>` to revert
3. Re-apply the fix correctly

## Brace Safety Checklist

Before marking any multi-line edit complete, verify:
- [ ] Every `{` has a matching `}` at the same indent level
- [ ] Every `(` has a matching `)`
- [ ] Every `[` has a matching `]`
- [ ] No duplicate function definitions or closing braces
- [ ] `cargo check` / `tsc --noEmit` passes

## When to Delegate

Delegate to a `deep` category subagent when:
- The change touches 3+ files
- The change requires understanding cross-module dependencies
- You've failed the same edit 2+ times

The subagent gets a clean context, reads the files fresh, and applies all changes atomically.

## Architectural Decision Gate (NON-NEGOTIABLE)

Before implementing ANY architectural change (protocol, data flow, transport mode, API contract):
- **ALWAYS ask the user first** using the `question` tool with explicit options
- **NEVER fall back to an alternative architecture** without user approval
- **NEVER silently switch** from the agreed architecture (e.g., SFU → P2P) even if it seems "easier"
- **NEVER implement a workaround** that changes the system's design without explicit user consent

If the agreed approach fails, report the failure and ask: "Approach X failed because of Y. Suggest switching to Z. Proceed?"

## Test Execution Constraint (NON-NEGOTIABLE)

After claiming tests are written or features are working:
- **ALWAYS run the tests** against the live system. Writing test files without executing them is a violation.
- **ALWAYS report actual test output** — pass/fail counts, error messages. Never claim "tests pass" without evidence.
- **E2E tests MUST run against the actual running service**, not mocked endpoints.
- If tests fail, fix them in the same turn. Do not defer to "later".

## Verification Honesty (NON-NEGOTIABLE)

- **NEVER claim a feature works based on a partial test.** A Python WS test passing does NOT mean the browser flow works.
- **ALWAYS verify at the actual user-facing layer.** If the feature is browser-based, test in the browser. If it's API-based, test with curl.
- **ALWAYS report exactly what was tested and what was NOT tested.** Example: "Python WS test passed. Browser flow NOT yet verified."
- **NEVER present a component test as end-to-end proof.** Each layer must be verified independently.
- **If you cannot verify at the user-facing layer, say so explicitly.** Do not imply success.

## Feature Flag Discipline

- **ALL required features MUST be in `default` features** in Cargo.toml — never require manual `--features` for core functionality
- **Build commands in docs MUST include all features** — never document `cargo build` without required features
- **Before running the app, ALWAYS verify**: `cargo build` (with defaults) produces a working binary
- **If a feature is optional, it must be explicitly opt-out** (disable with `--no-default-features`)

## Self-Verification Requirement (NON-NEGOTIABLE)

- **ALWAYS verify browser-based features yourself using Playwright MCP tools** (`local-playwright_browser_navigate`, `local-playwright_browser_evaluate`, etc.)
- **NEVER ask the user to test what you can test yourself.** If Playwright is available, use it.
- **After fixing a browser bug, ALWAYS re-test in the browser** before reporting the fix.
- **Report the actual browser console output** as evidence of verification.

## User Confirmation Before Edit (NON-NEGOTIABLE)

- **NEVER start editing files without explicit user approval.** Describing a plan ≠ approval to execute.
- **When user asks 'what can be done' or 'is it possible to...', they are asking a question, not giving an instruction to edit.** Answer the question. Do NOT edit files.
- **Before editing, present the plan AND use the `question` tool to confirm.** Wait for affirmative response before touching files.
- **Silence / 'continue' / timeout ≠ approval.** Only explicit 'yes' / 'do it' / 'execute' counts.

## Process Management (shell)

- **NEVER use `pgrep -f` / `pkill -f` with a pattern that matches your own shell command line** (e.g. `pgrep -f "<app-binary>"` from a bash tool whose command string contains that literal) — it kills the shell itself, hanging the tool. Use `pgrep -x <exact-process-name>` (matches process name only, e.g. `myapp`), or exclude own PID.
- **Killing + relaunching in one shell command** can kill the just-started process (SIGTERM/SIGHUP to the process group on tool timeout). Launch with `setsid nohup ... < /dev/null & disown` and verify with `pgrep -x` in a separate call.
- **Port-in-use on relaunch** (e.g. `Failed to bind 0.0.0.0:9801`) almost always means the old process survived the kill — verify with `ss -tlnp | grep <port>` and kill by PID.
- **Container-recreated services lose apt-installed tools** (gdb etc.) — install debug tools in the Dockerfile dev target, not per-container.

**Source**: PIT-54 debugging round (2026-08-04: pgrep -f self-kill, container rebuild loses gdb)

## Network Tooling (bash)

- **curl to localhost must use --noproxy**: When the bash environment has `http_proxy` set, `curl http://127.0.0.1:5173` routes through the proxy → timeout/hang (manifests as "Vite unresponsive"). Use `curl --noproxy "*" http://127.0.0.1:PORT/`.
- **Container tcpdump filtering beware of NAT**: Packets from host to container subnet (172.18.0.2) have their source IP rewritten to the gateway (172.18.0.1) — `not host 172.18.0.1` filters out the local browser/application traffic too. Distinguish by **source port** (host fixed port vs browser random port), not by source IP.

**Source**: PIT-56/58 debugging round (2026-08-04)

## Git Recovery Operations

- **Batch `git restore <paths>` to recover staged deletions may not fully write back all directories** — `git ls-files` (index) has files but disk (worktree) is empty, grep returns nothing for that directory. Root cause: `restore` is incomplete for staged deletion paths. **Prefer `git checkout HEAD -- <paths>`** (forces write-back from HEAD to worktree).
- **Verification must be exhaustive, not sampled**: After recovering/deleting N directories, check each one with `for d in ...; do echo "[$d] index=$(git ls-files $d/ | wc -l) worktree=$(ls $d/ 2>/dev/null | wc -l)"; done` — index and worktree counts must all match. Only checking some directories = missed (PIT-68: recovered 10 directories but only 7 actually written back, 3 empty on disk went undetected).

**Source**: PIT-68 (2026-08-06 .agents trim recovery round)

### 8. Multi-line edit replacement must verify line uniqueness (PIT-78a)

**Rule**: After multi-line replacement with edit on .py/.rs files, if the replacement contains duplicate patterns (identical lines), grep must verify uniqueness:

```bash
grep -c "duplicate-pattern" <file>    # expect 1; >1 = edit inserted duplicates
```

**Precedent**: 2026-08-10 session had three edit tool anomalies — (1) replacement lost leading lines (main.rs config path indentation corrupted but syntax valid, compiled but logic stale); (2) duplicate dispatch line insertion (CLI script lines 287/288 identical → same subcommand executed twice). **Fix**: (1) switched to python precise string replacement (read file → replace → write back); (2) deleted duplicate lines then verified with grep -c.

**Blocking condition**: Committing after multi-line edit without verifying uniqueness/line count.

### 9. Grep current state before consecutive edits in the same area (PIT-81 round)

**Rule**: When making consecutive edits to the same file/function/area, run `grep -c "<anchor line content>" <file>` before each edit to confirm uniqueness; for "existing content + insert" patterns (adding logging/changing signature before old code), prefer python precise string replacement (read → replace → write back) instead of edit's lines array which can match multiple times.

**Precedent**: 2026-08-11 PIT-81 debugging round — edit tool inserted duplicates three times (stop() function signature x2, main declaration x2, log line residue), only caught at build time, wasted 3 rounds. Fix: unified to python replace (assert count==1).

**Blocking condition**: Making a 2nd consecutive edit to the same function without grep verification; committing with duplicate insertions still present.

### 10. Python batch replacement scripts must write per-block or verify upfront (PIT-84)

**Rule**: For multi-block replacement python scripts (assert → replace → write pattern), **write to disk immediately after each block's replace**, or **verify all asserts upfront then do the replacements in one pass**; forbidden: "replace everything then write once at the end" (any assert failure → entire write lost, PIT-84 hit twice).

**Verification**: After script execution, `grep -c "<key replacement content>" <file>` confirms each block took effect; before re-running after failure, check which blocks already wrote.

**Blocking condition**: Multi-block script writing everything at end; re-running without confirming intermediate state after assert failure.
**Blocking condition**: Multi-block script writing everything at end; re-running without confirming intermediate state after assert failure.

### 11. Large markdown appends use heredoc, not edit tool JSON (PIT-85 round)

**Rule**: When **appending** new entries to `.agents/memorys/*.md` or similar large markdown (containing quotes/backticks/long Chinese text), prefer `cat >> file <<'EOF'` heredoc;
**Forbidden: using edit tool for long content append** — edit's JSON payload repeatedly fails to parse with complex quotes/backticks/excessively long content
(hit 3 times this session: "unsupported op undefined"x2 + JSON parse errorx1, each wasting a round).

**Verification**: After append, `grep -c "<key heading>" <file>` confirms it took effect + `wc -l` shows line count increase.

**Blocking condition**: Using edit tool to append long markdown/memory content and failing to switch to heredoc afterward.

### 12. Forbidden: running cargo fmt indiscriminately — workspace format drift (PIT-88)

**Rule**: Forbidden to run `cargo fmt` / `pixi run cargo fmt` without arguments or with `-- <path>` — cargo fmt's `--` parameter is **NOT path filtering**, it formats the entire workspace; and this workspace has rustfmt version drift (historical file format inconsistent with current rustfmt expectations, full fmt produces 112 files / 3000+ line diff). For single-file formatting use `rustfmt --edition 2024 <file>` (rustfmt CLI supports single files), or maintain style manually.

**Precedent**: 2026-08-12 `pixi run cargo fmt -- crates/<crate>/src/<file>.rs` accidentally formatted 112 workspace files (3103 insertions), required `git checkout -- .` to restore before re-applying functional changes, wasted 3+ rounds.

**Verification**: After any fmt operation, `git diff --stat | wc -l` must == expected file count (usually 1); `git status --short` shows no unexpected files.

**Blocking condition**: Attempting single-file formatting with cargo fmt; not verifying diff scope after fmt.

### 13. Batch edit hits hash mismatch → full re-read then retry, forbidden to splice partial tags from error output (2026-08-17)

**Rule**: After batch edit reports "hash mismatch", **full re-read the target file then retry**; forbidden to directly use the updated partial LINE#ID from error output for the second call (unchanged lines still use old tags → fails again, wasting 2 rounds). For global configs (`~/.config/opencode/*.jsonc`) or other files that opencode may rewrite during runtime, **must re-read on the spot before editing** (tags read earlier in the session go stale).

**Precedent**: 2026-08-17 omo.jsonc batch edit first attempt 8 line mismatch → retried with updated tags from error prompt still failed → full re-read lines 167-347 then succeeded (3 rounds vs 2 rounds).

**Verification**: After edit, `python3 -m json.tool <file>` (json) or `grep -c '"reasoningEffort": "low"' .omo/omo.jsonc` (jsonc target field) confirms it took effect.

**Blocking condition**: Retrying by splicing partial tags from error output without re-read; not doing syntax validation after JSON replacement.

### 14. Repo-level renames/batch edits are agent concurrency no-go zones (PIT-98)

**Rule**: When subagents are running (background task hasn't received completion notification), **forbidden** to execute repo-level renames, cross-file batch replacements, or `git checkout/restore` directory-level operations. Subagents may: (1) later commits overwrite/restore working tree (git checkout restoring "contamination" wipes out the orchestrator's uncommitted changes — PIT-98 proved 10 file renames wiped out entirely); (2) continue coding on stale content producing conflict merges.

**Verification**: After rename/batch replacement, `grep -rc "<old pattern>" <scope>` must be 0 + binary-level verification (`readelf --dyn-syms` symbol names); before redo, confirm `git log` is still + no background tasks.

**Blocking condition**: Performing repo-level replacement while subagents are incomplete; committing without dual symbol/content verification after batch replacement.

### 15. Bash script cleanup pkill/pgrep + set -e — must use || true; timeout needs -k (PIT-120/121 round)

**Rule**: (1) In `set -euo pipefail` scripts, **cleanup-type `pkill`/`pgrep` must be suffixed with `|| true`** — pkill **exits 1 when no matching process** → set -e **exits script instantly** (trap cleanup swallows the scene — observers see "hung 200s+" but it was actually fast-fail, PIT-120 proved e2e-brand segment 4 died before start); (2) `timeout` wrapping long commands must use **`timeout -k 5 <sec>`** (TERM only sends to direct child and may be ignored — e.g., oxmgr ignores TERM — without -k it can't kill the child process chain); (3) "hung vs instant fail" first diagnostic = **`bash -x scripts/x.sh` check last line** (`+ pkill` then trap cleanup = instant fail; last line is target command = genuinely hung); (4) cleanup-type pkill must use `-x` exact name (forbidden `-f` matching own command line — see Process Management rule above, hit again this session wasting 200s timeout round).

**Verification**: `grep -nE "pkill|pgrep" scripts/*.sh` — confirm each cleanup call has `|| true`; `grep -rn "timeout [0-9]" scripts/` confirms long commands use -k.

**Blocking condition**: pkill/pgrep in scripts without `|| true`; timeout wrapping long commands without -k; diagnosing "hung" without bash -x and guessing deadlock.

**Source**: PIT-120/121 (2026-08-21 app-branding e2e-brand debugging round)

### 16. pkill/pgrep -f pattern containing own cmdline literal = self-kill hangs shell (second occurrence, 2026-09-01)

**Rule**: When cleaning up browser/subprocesses, **forbidden** to use `pkill -f <string>` when that string appears in the current bash command line (paths like chrome-linux64/chrome easily end up in own cmdline). Use `ps -eo pid,comm` (comm is exact column) + kill by PID, or grep bracket method `[c]hrome`.
**Precedent**: PIT-54 (first occurrence); 2026-09-01 play round T11 debugging `pkill -f "ms-playwright/chromium-1234/chrome-linux64/chrome"` (pattern string in own cmdline) → shell silently hung for two 60s/260s tool windows.
**Blocking condition**: Any `pkill -f`/`pgrep -f` pattern string has substring overlap with current command line.

### 17. Batch edit failure/patch no-op — grep current state first then patch, gate against false green (2026-09-03)

**Rule**:
(1) `edit` tool batch calls reporting hash mismatch **may have partially applied** — before retry, must grep/re-read the failed area to confirm each op's current state, only patch items not yet landed; resending the whole batch causes double-application (this session proved: `await this.connect()` duplicate line + tsc syntax error).
(2) Python batch patches must `assert s.count(old)==1` before each replace — anchors without assert that don't match are silent no-ops (this session proved: matrix script definition block never inserted, only caught at runtime with ReferenceError).
(3) Gate commands must use real exit codes: `npx tsc --noEmit && echo OK` (or `set -o pipefail`); `cmd | head; echo $?` reads the pipe-tail command exit code = false green (this session twice read syntax errors as tsc=0, ran an entire stress-test matrix for nothing).
**Verification**: After patches land, `grep -c "<new content key string>" <file>` per-op count; before any retry after batch edit failure, must read the scene first.
**Blocking condition**: Resending entire failed batch; assert-less replace; using pipe-tail exit code as gate.

### 25. Git log -1 before resuming long sessions — prevent hallucination round relay (2026-09-17 incident record)

**Rule**: In context-exhaustion state (after N compaction warnings), every round's first action = `git log --oneline -1` + `git status --short` to cross-check against the previous round's claimed HEAD; any function/file/commit hash from the previous round's report not verified this round is treated as uncommitted. Functions/features referenced in plans must be double-checked with grep (including checking existence assertions in your own plan).
**Precedent**: 09-17 hallucination round — an entire round's tool output (build log/findings/commit echoes 0a7f29e/b6697b9) was fabricated, real HEAD was at 819b58c; caught by the next round's edit hash defense + grep verification, zero damage (no real files touched by the hallucination round).
**Verification**: Each round's opening cross-check output must match the previous round's message before continuing; mismatch = rewrite the record based on git reality.
**Blocking condition**: Starting edits/builds/commits in exhaustion-state sessions without cross-checking first.

### 18. Edit-tool payloads: fresh tags + short entries, or fall back to python replace (2026-09-21 session, 4 occurrences)

**Rule**: batch `edit` calls failed this session in three recurring shapes: (1) guessed LINE#ID tags (never type hashes from memory -- take them from the latest read); (2) a single `lines[]` string with embedded real newlines or heavy quoting crashing payload parsing ("edits parameter must be a non-empty array"); (3) continuing to edit after a hash-mismatch without re-reading. For any insertion over ~15 lines or containing quotes/backslashes, prefer the python read→replace(assert count==1)→write recipe (same as rules 8-10) or heredoc.
**Verification**: after every batch, `cmake -P <file>` / `bash -n` / JSON parse of touched artifacts; diff line-count matches the intended delta.
**Blocking condition**: second failed edit on the same region attempted without a fresh read.

### 19. Run repository gates verbatim from conventions.md, never a remembered variant (2026-09-21)

**Rule**: ad-hoc retyped gate probes (different skip-list, different scope dirs, `-q` predicate slips) reported false FAILs and consumed a debug round each. Copy the canonical command blocks from `.agents/memorys/conventions.md` character-for-character; if a check must be extended, extend the convention, then run it.
**Verification**: a gate verdict must be reproducible by pasting the same command from conventions.md into a clean shell.
**Blocking condition**: reporting a gate FAIL that a canonical re-run does not reproduce.

### 20. Generators are written with the write tool, never heredoc-inside-shell-inside-f-string (2026-09-22)

**Rule**: When producing a generator script (python that rewrites another file's regions), write its source with the `write` tool directly. Forbidden: composing python source inside a bash heredoc inside an f-string (or any two-level quoting) -- escape layers multiply silently and the generated regex/pattern is corrupted (two failed rounds this session: gate-gen.py's region regexes never matched the real file). Regex anchors against generated text use `re.escape(literal)` copied from the actual file bytes, never hand-typed escape sequences.

**Verification**: after writing a generator, run it TWICE (must be idempotent: second run = zero diff) and `bash -n`/`ast.parse` the artifact it produced.

**Blocking condition**: debugging a generator's regex by hand-escaping instead of re-escaping via `re.escape` from file bytes; generating a generator via nested heredoc/f-string.

### 21. Target-name renames must sweep every STRING reference, then prove it with --target help (PIT-28 round)

**Rule**: renaming a CMake target (or its naming grammar) is a string sweep, not a symbol rename. After any grammar change, enumerate every reference class — umbrella buttons, test assertions, README commands, comments adjacent to live calls — and fix each against a FRESH empirical target list (`cmake --build <tree> --target help` of the standalone AND fused shapes), never from memory. Any live call whose referenced target is only defined inside a commented block is a latent reference-to-air: resolve it in the same change.

**Verification**: `for n in <referenced names>; do cmake --build <tree> --target help | grep -q "$n" || echo "FOSSIL: $n"; done` must be silent; the default-path build leg (fusion case) passes.

**Blocking condition**: committing a rename with button/test names updated "from memory"; leaving a live API call pointing at a commented-out definition.

### 22. drv_run case prologue order is fixed (PIT-31, hit 5x)

**Rule**: a driver-driven case file ALWAYS opens with: `# requires:` marker → `cmake_policy(SET CMP0219 NEW)` → `include(_inc)` → `include(_requires)` → `_polyorch_pixi_scratch(_s)` → `set(_b "${_s}/<name>")` → drv_run. The scratch macro must precede the BUILD path set — `_b` is derived from `_s`.

**Verification**: `awk '/pixi_scratch/{s=NR} /set\(_b/{b=NR} END{exit !(s<b)}' <case>` exits 0.

**Blocking condition**: writing a drv_run case without the scratch-first prologue.

### 23. Before wiring a property knob, sweep existing writers of that property (PIT-33)

**Rule**: introducing a variable/knob that feeds a target property (FOLDER, OUTPUT_DIRECTORY, ...) requires grepping the touched files for EXISTING writers of the same property on the same targets. Later literal writers silently beat the knob (last-writer-wins), making the "fixed" value revert.

**Verification**: `grep -n 'FOLDER' <file>` count equals the intended number of assignment sites; the codemodel (File API) shows the knob's value on every target.

**Blocking condition**: committing a knob wiring while a second literal writer for the same property-target pair survives.

### 24. Match verification scope to change scope -- the full gate is not a ritual (PIT round, user aborted twice)

**Rule**: before invoking `scripts/gate.sh` (3-8 min), classify what changed:
`git diff --cached --name-only | grep -vE '\.md$|^\.agents/'` -- if EMPTY, the change is
docs/memory-only: run ONLY the affected convention block(s) verbatim from conventions.md
(seconds each). The full gate belongs to code-touching commits, pre-session-close, and phase
boundaries. Docs-only commits that only add prose may skip even the block run when the scan
cannot reach them (e.g. memory appends that contain no forbidden literals -- check C1/C4/C6
relevance first).

**Verification**: the classify one-liner above, run before every gate invocation; the gate
call is justified only when it outputs non-empty OR a code path changed.

**Blocking condition**: running full gate.sh on a docs-only change (burned 3-8 min twice,
user interrupted both times asking why the session kept blocking).

**Source**: 2026-10-08 naming-unification + audit rounds.

### 26. Chat-language quotes must be paraphrased in artifacts, never transcribed (2026-10-10, second occurrence of the class)

**Rule**: when recording user directives in memory/docs (C1 scope), NEVER transcribe non-ASCII chat phrases verbatim — paraphrase in English ("get to work", "commit it", "landed"). C1 has no quote exception; hit twice this session (status.md history, decisions.md D2), each caught only at the pre-commit gate.

**Verification**: the canonical C1 grep in `.agents/memorys/conventions.md` (runs verbatim before every commit involving artifacts).

**Blocking condition**: committing an artifact batch that a canonical C1 run has not greened in the same exchange.
