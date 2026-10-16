# ModuKit — Pitfalls

> Format per `.agents/rules/common/lesson-memory.md` (five-part):
>
> ```markdown
> ## PIT-{n}: Title (date)
> - **Symptom**: [symptom description]
> - **Root cause**: [root cause analysis]
> - **Solution**: [correct approach]
> - **Verification**: [check command]
> - **Forbidden**: [what must not be done again]
> ```
>
> Numbering starts fresh at `PIT-1` for ModuKit. The PolyOrch-era pitfalls were not ported; the originals live in `PolyOrch/.agents/memorys/pitfalls.md`.


## PIT-1: Treated an exploratory "how about X?" as execution approval (2026-10-09)
- **Symptom**: User asked "how about implementing it inside .opencode/init-mcp-codegraph.mjs?" (an evaluative question; original wording was Chinese). I rewrote the wrapper (.mjs→.sh), edited opencode.json + .gitignore, ran a live install/init test, and left 286MB of downloaded artifacts — all before any approval. User: "I was only discussing with you — why did you execute directly?" (original wording was Chinese)
- **Root cause**: Approval carry-over — the user's earlier question-tool answer "install officially now" (original wording was Chinese) authorized the *old* plan (global official install), and I silently extended it to cover the *new, different* plan this turn (project-local sh wrapper). The existing rule (`.agents/rules/common/edit-safety.md` "User Confirmation Before Edit") already states questions ≠ instructions; I pattern-matched momentum instead of re-checking the gate.
- **Solution**: On being called out: stop, enumerate exactly what changed and what side-effects ran (with sizes/timestamps), present rollback options via one question, wait for an explicit decision (a "keep everything" decision was given; cleanup executed only after that).
- **Verification**: Before any file mutation or install/download, the CURRENT user message must contain a directive verb ("implement / change it / proceed / keep") — evaluative phrasings equivalent to "how about / is it feasible / let us discuss" never count. Test with: quote the exact user sentence authorizing this action, or stop and ask.
- **Forbidden**: Inheriting approval from a previous plan/scheme to a changed scheme; running install/init tests without the same confirmation as edits; treating silence or a partial answer as blanket approval.

## PIT-2: `new-api/fast` model balance kills team members mid-run (2026-10-09)
- **Symptom**: In two consecutive teams (tribunal: yagni-prosecutor, pattern-alchemist; profiles: p-ctk, p-accessbase, p-mediaservo) members on category `unspecified-low`/`artistry` (model `new-api/fast`) entered error state immediately or mid-task; system announcements repeated "Insufficient account balance"; OMO auto-retried on `fast-1`/`fast-2` (some revived, some died again, one wrote a TRUNCATED file before dying).
- **Root cause**: Upstream balance of the `fast` model family — an environment/account condition, not a prompt or task error. Retries share the same depleted pool and may still land; a half-dead member can leave partial artifacts.
- **Solution**: (a) prefer premium-family categories (deep-low / unspecified-high / ultrabrain) for critical deliverables; (b) on member death, spawn a `task()` substitute on a live category with a DUP-GUARD ("if target file exists >=60 lines AND contains a real verdict block, stop"); (c) ALWAYS verify the delivered FILE on disk (section count + `grep -cP '[CJK]'` + tail) — never trust a member's "done" or the task board.
- **Verification**: after any team run, `for f in <expected>; do test -f $f && wc -l $f && grep -c '^## ' $f; done` must show the full section count; a truncated file (like ctk.md at 112 lines/3 sections) must be detected before index updates.
- **Forbidden**: marking a profile "delivered" from a chat report alone; re-running the whole team when only the `fast`-category members failed.

## PIT-3: OMO 5.1.27 team task store cannot reach terminal states (2026-10-09)
- **Symptom**: members: `team_task_update` -> "team path escapes base directory" / "not found"; lead: pending->completed rejected ("no such transition"), claimed->completed also rejected after successful claim. Board stayed pending/claimed for delivered work; closure-contract task check unsatisfiable.
- **Root cause**: task-store bug in oh-my-openagent 5.1.27 (transition guard + path validation), triggered when tasks are pre-created by the lead and later updated across sessions.
- **Solution**: treat the board as advisory only; file system is truth; close teams via full shutdown_request+approve sweep then `team_delete` (succeeded even with non-terminal tasks); record deliverable state in the final report, not the board.
- **Verification**: `team_status` shows all members shutdown_approved and `team_delete` returns deleted:true while tasks still show pending — expected, not a blocker.
- **Forbidden**: burning turns retrying board transitions; waiting on task-board state to decide closure.
- **Update (2026-10-14)**: recurred on omo 5.1.28 - member self-transitions claimed->in_progress->completed SUCCEEDED for 4 of 5 tasks, one task stuck at in_progress (store refused even the owner); lead-side cross-owner updates rejected as second guard (cross-owner not allowed); lead CAN claim-and-complete UNOWNED tasks (TT-gate path); team_delete still errored (members still active) after full request+approve sweep - force:true completed it. Also observed: delivered peer messages did not wake an idle member twice (lead ran the gate inline rather than burn turns).

## PIT-4: ctx_batch_execute stringifies commands[]; NODE_OPTIONS prefix kills compound shell commands (2026-10-09)
- **Symptom**: batch runner echoed "Executed 1 commands" with the whole commands-array JSON as one literal shell command (`[{label:: command not found`); independently, `for`/`if`-leading shell commands die as `NODE_OPTIONS='--require …/hook.js' for …` (env-assignment syntax requires a simple command after the prefix). Hit by the orchestrator AND two background librarians separately (100% repro).
- **Root cause**: context-mode wraps each command string with an env-var prefix; shell compound keywords cannot follow `VAR=value` prefixes. In some host paths the commands parameter arrives JSON-stringified instead of parsed.
- **Solution**: for multi-step local work use ONE ctx_execute (javascript or `bash -c '…'`) with self-contained script logic; python read→replace(assert count==1) for file mutations. Never lead a wrapped command with `for`/`if`.
- **Verification**: batch result must report "Executed N commands" with N == len(commands); a literal-JSON echo = fall back to ctx_execute immediately.
- **Forbidden**: retrying the same batch shape after the stringify symptom, or shipping repo scripts that assume `rg`/`openspec`/`docker`/`uvx` present (none installed on this box — 2026-10-09 scan).

## PIT-5: chained command left a stale `git commit -m` in place — wrong message landed (2026-10-09)
- **Symptom**: intended message via second heredoc commit no-op'd; HEAD carried an old license-message string.
- **Root cause**: hand-assembled `cmd1 && commit-A ; commit-B` line where commit-A (a leftover from an earlier draft) was real and consumed the staged index; `;` hid its success/failure from the `&&` chain.
- **Solution**: amend while unpushed (verified via `git log origin/master..HEAD`). Prevention: one commit command per invocation, message inline; never place two commit attempts in one chained call.
- **Verification**: `git log --format='%h %s' -1` after every commit matches the intended subject.
- **Forbidden**: placing two commit attempts in one chained invocation.

## PIT-6: Committed despite red C1 output — gate printed, human loop failed (2026-10-12)
- **Symptom**: batch commit `c9b960a` landed containing CJK in decisions.md (D44 decision line transcribed four chat-language terms verbatim: six-state, resource books, topology, ring); the C1 check in the same command DID print the violation before `git add`/`git commit` ran, and the commit proceeded anyway.
- **Root cause**: gate and action were sequential statements in one script, not conditioned: `[ -z "$out" ] && echo "C1 OK" || echo "$out" | head -2` only REPORTS - nothing consumed the verdict. Rule #26 was known and still the transcription happened under prose-momentum (decision text paraphrasing is where it always leaks).
- **Solution**: fix-forward (this batch + follow-up commit); from now on the canonical pattern is fail-closed: `out=$(grep ...); [ -n "$out" ] && { echo "$out"; exit 1; }` BEFORE any git add/commit in the same script - the gate must be able to abort the chain, not merely speak.
- **Verification**: re-run canonical C1 (conventions.md copy) after the fix - must print C1 OK; `git log -1` shows the fix-forward commit.
- **Forbidden**: running a gate whose output nobody (and no `exit`) consumes before mutating git state; paraphrase discipline must apply to decisions.md text above all - it is where user phrases get recorded.

## PIT-7: IDE ambient cmake ≠ gate-routed cmake - hidden floor lie in an embedded helper surface (2026-10-16)
- **Symptom**: VSCode CMake Tools configure exploded with `Policy "CMP0174" is not known` x many, while every scripted gate (pixi-routed cmake 4.4.4) was green; build landed in repo-root build/ instead of the out/ discipline.
- **Root cause**: two stacked gaps. (1) PolyOrch's rust helpers call `cmake_policy(SET CMP0174 NEW)` un-guarded at six sites, while CMP0174 only exists since cmake 3.31 - their declared 3.25 embed floor was a lie that NO gate in either repo exercised (suites/CI always ran on newer cmake). (2) ModuKit's own gates pin the tool via pixi, but nothing pinned the IDE's tool - ambient tools drift silently when only one path is gated.
- **Solution**: guard with `if(POLICY CMP0174)` upstream on dev-modukit (control-proven: identical 74/5/29 on 4.4.4, +8 cases restored on 3.30.7; pushed/merged by maintainer, then a routine pin bump); pin the IDE to the same authority via .vscode/settings.json (cmake.cmakePath = pixi env cmake, buildDirectory = out/build, Ninja).
- **Verification**: the failing user command `/usr/bin/cmake -S . -B build` (3.30.7) now configures clean; every cmake-family gate stays green on both cmake versions.
- **Forbidden**: trusting "our gates are green" as proof that the ambient/IDE tool path works - any surface a tool can reach must either be pinned (settings) or floor-guarded (if POLICY); and a helper's declared floor must be exercised by its own suite.
