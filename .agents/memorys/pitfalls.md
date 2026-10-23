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

## PIT-8: C3 continuation-line bullets shipped twice before the machine gate existed (2026-10-16)
- **Symptom**: two commits (S2-M1 code, S2-M2 F1 fix) shipped body bullets split into
  continuation lines ("   text...") violating the `^ - ` bullet shape; the second one also
  shipped an over-length subject (72 chars).
- **Root cause**: the C3 checks existed as commands to run "right after each commit" - an
  honor system; the habit of wrapping long bullets was re-imported from prose style; the
  subject gate got skipped by printing wc -m without comparing it.
- **Solution**: check.sh gained a commit-shape leg over the pending (origin/master..HEAD) range,
  subject length + body bullet regex; authoring rule: one bullet = one physical line <= 100 chars,
  no wrapping. Wider windows hit five pushed C3-era legacy subjects (immutable by policy) -
  the gate scope is deliberately the unpushed range.
- **Verification**: predicate negative proof observed 2026-10-16 (planted bad-shape ->
  SUBJ-VIOL(79) + BODY-VIOL(shape); good-shape -> silent); check.sh green on the pending range.
- **Third trap (same leg, same day)**: mawk does NOT support {n,m} interval
  expressions - /^ - .{1,99}$/ silently matches NOTHING, every bullet read as
  violation. Gate regex membership rewritten as length+substr (isbullet()).
  Lesson: any awk regex quantifier must get its own planted-red/green proof
  through the exact pipeline, not the mental model of the regex.
- **Forbidden**: shipping a commit whose shape was checked only "from memory of the rule";
- **Verify with exact literals, not substrings (2026-10-21)**: a typo'd marker (double-T) was self-stamped green because the verify grep searched the correct form as a SUBSTRING of the wrong one; two further anchor misses came from anchoring on the text I MEANT to have written rather than disk bytes - this batch's own first attempt demonstrated it live. Anchor = read; verify = full literal with boundary.
- **Fatigue signature (2026-10-20, 4 more catches in one session)**: violations cluster in late-session commits; adopted remedy - run the C3 pair (and C1 on artifact batches) INSIDE the commit command chain, never as post-commit discovery, never as a separate later step; escalation proposed (pending maintainer word): a git pre-commit hook, since the check.sh leg only reports after the fact.
  wrapping a ` - ` bullet across lines.

## PIT-9: Counting cargo test through a busy tty silently drops suites (2026-10-16/18, x2+)
- **Symptom**: `pixi run cargo test --workspace | awk ...` totals came back wrong twice (82 vs 134; 94 vs 133) while suites were actually all green; in one case the count read zero.
- **Root cause**: concurrent cargo interleaves its output on the terminal; awk/grep pipelines over a live tty lose or corrupt lines (the file-lock "Blocking..." chatter interleaves mid-line). The totals were right every time when re-read from a redirected file.
- **Solution**: ALWAYS redirect to a file first, then parse: `pixi run cargo test --workspace >/tmp/x.txt 2>&1; grep -E "^test result" /tmp/opencode/x.txt | awk '{p+=$4;f+=$2}'`. Never parse cargo output through a pipe attached to the live terminal.
- **Verification**: the redirected-file awk vs a suite-by-suite table (stage-2 M3 turn, TOTAL=133 exactly matched the per-suite listing).
- **Forbidden**: quoting a test total that was not parsed from a redirected file; diagnosing "missing suites" from a tty pipe.

## PIT-10: D84-gate full-coverage run re-trips the ETXTBSY family under instrumentation load (2026-10-18)
- **Symptom**: `cargo llvm-cov --workspace` (full-text mode, not summary-only) aborted at process_executor.rs:251 with `Text file busy` on a standin copy; the same suite is green under plain cargo both parallel and serial.
- **Root cause**: llvm-cov instrumentation adds build/load on the same standin-build path; the race window (concurrent test binaries building the same standin dir) widens under it. The atomic-rename hardening (2026-10-18) closed the plain-cargo instance; the cov-run instance remains OPEN and untouched by maintainer order.
- **Solution**: (1) unique per-invocation standin dirs (mk-standin-{pid}-{nanos}-{seq}), landed with the debt lane - the historical shared-path site never recurred (0/13); (2) LLVM_PROFILE_FILE redirected into target/llvm-cov-target via the pixi cov activation env - spawned children inherit it, the writer class is fully accounted for (zero fresh litter after a clean-sweep run, zero ETXTBSY across three post-fix cov runs). CLOSED 2026-10-18.
- **Verification**: a clean full-text `pixi run -e cov cargo llvm-cov --workspace` completing green (the D84 per-crate split still needs exactly this).
- **Forbidden**: writing instrumented-run litter into the source tree (the cov env now redirects it); running the per-crate split through a tty pipe (PIT-9).

## PIT-11: Team-audit ack storm - conductor replies manufactured a mail flood (2026-10-19)
- **Symptom**: user saw repeated "you have N new team messages" bursts during a 4-member doc-audit; asked why.
- **Root cause**: conductor acknowledged EVERY member report and broadcast status pings; each ack landed as a new inbox item on a member, whose reply landed as a new item for the lead - a self-sustaining echo loop, multiplied by 4 members.
- **Solution**: team conductors go silent on receipt - collect into the merge table, digest ONCE at the end; no per-message acks, no "stand by" pings after the first call.
- **Verification**: if member-inbox messages after dispatch exceed 1 per member total, the pattern recurred.
- **Forbidden**: ack-per-report orchestration; broadcasting "final call" more than once.

## PIT-12: Multi-language command blocks leave bare code after a heredoc terminator (2026-10-19, x3)
- **Symptom**: bash "unexpected syntax near open('...')" errors and "warning: here-document delimited by end-of-file (wanted MSG)" - commands half-executed, chains short-circuited.
- **Root cause**: pasting python lines after a closed PY/MARKER heredoc (they become bash), or ending a heredoc with the WRONG terminator (EOF vs MSG) which swallows every following line into the message body - twice shipping commit messages that inhaled script tails.
- **Strike 3 (2026-10-21, twice in one session)**: multi-heredoc CHAINS desynchronize when an earlier component fails, and the pattern itself recurred mid-lesson-batch (missing terminator swallowed the git tail). Adopted rule: generators via the write tool as files (rule 20), commit messages via a written /tmp/msg file; never stack three quoted documents in one command.
- **Solution**: one language per invocation; verify each heredoc's opener/terminator pair match before running; after any such error, ALWAYS re-check which steps actually ran (git status / file greps) before re-running the rest.
- **Verification**: grep -c "EOF" inside a MSG-terminated block must be 0; post-error, `git status --short` + `grep -c "<marker string>" target` before retry.
- **Forbidden**: assuming a chained block ran to the end after a syntax error mid-block.

## PIT-13 (trivial): resumed adjudications reverted to summary tables (2026-10-19, x2)
- Once adjudication-walkthrough is the active protocol, EVERY item - including after interruptions/team noise - goes seven-part plain text; the closing summary table is only due AFTER all items are individually ruled.

## PIT-14: Agent delivery is untrustworthy - verdict from tree + own gates, never from the report channel (2026-10-19, x2)
- **Symptom**: two failure shapes the same session-arc: (1) librarian runs returned task=COMPLETED while the final message was only a planning trace (zero tool calls actually executed); (2) HOLDINGS-lane agent completed a fully green 9-minute run but the delivery channel truncated mid-work-stream (background_output from_end + session-file tails both stopped at a mid-message cut).
- **Root cause**: model-side deaths (see PIT-2 balance lineage) and return-channel message truncation both leave the REPORT incomplete while the WORK (shape 2) is real; trusting either report shape - "completed means reported" or "truncated means failed" - misroutes the orchestrator into re-running expensive lanes or accepting undelivered claims.
- **Solution**: on any suspicious delivery (plan-only trace, mid-stream cut, missing report sections): first read the working tree (git status --short, grep the expected surfaces), then run the orchestrator's own gates; the tree+gates verdict decides. Shape 2 resolved exactly this way: 198 tests + both demo gates proved the lane landed while the report never arrived.
- **Verification**: after every background completion, the acceptance sequence is `git status --short` + targeted surface greps + own gate run BEFORE any retry decision; a retry launched purely on a truncated report (with the tree already carrying complete verified work) is a double-work violation.
- **Forbidden**: accepting claims from a dead channel; retrying a whole lane because its MESSAGE died when its OUTPUT lives; the inverse too - never ship from report text without tree verification (cross-refs: PIT-2 model deaths, testing.md Metric Integrity).

## PIT-15 (trivial): adjudication items drift back to jargon after 4+ in a row (2026-10-19)
- Plain-language rule reasserted mid-walkthrough by the maintainer twice; the drift shape is sources/impact sections reverting to insider shorthand as item count grows - run every section through the "read-aloud to a non-CMake colleague" check before sending.

## PIT-16: replace-without-anchor-rejoin silently ate two history headers (2026-10-19, caught next day)
- **Symptom**: history-archive.md carried three entries fused into one line - two entry headers (D92, M8.6) had vanished; discovered only when a LATER insert assert fired on the missing anchor.
- **Root cause**: insertion pattern `s.replace(anchor, NEW_TEXT)` replaced the anchor instead of prepending before it - the anchor text itself was consumed; asserts only verified the anchor EXISTED pre-replace, never that it SURVIVED post-replace.
- **Solution**: the pattern is `s.replace(anchor, NEW + anchor)` (or use split/insert on line indices); after any insertion-style replace, re-grep the anchor and run an entry-continuity scan.
- **Verification**: `grep -c '^:' <file>` must be 0 (no headerless continuations) and the anchor count unchanged after the edit.
- **Forbidden**: trusting a pre-state assert as proof of post-state integrity for mutating replacements.
- **Recurrence 2026-10-21**: history-archive's M1 header swallowed the same way (the habit landed decisions-side only); extended check now covers merged-header shape: entry-header count (`grep -cE '^- 20[0-9]{2}-' <file>`) before/after any entry edit, and the fused signature `\., [a-z-]+\)?: ` mid-line must not match.

## PIT-17: stale build trees manufacture green gate claims - counts are only real from a rebuilt tree (2026-10-20, cost one CRITICAL)
- **Symptom**: commit + history asserted "ctest 9/9"; a fresh out-of-tree configure+build returned 8/9 (probe still called the pre-fix wire name "Echo" - the sweep's second family passed only because both probe and fixture in the warm tree were old artifacts).
- **Root cause**: ctest ran against a warm generator tree whose binaries predated the source fix; no repo gate rebuilds-then-counts from clean, so neither the implementer's rerun nor check.sh could see the lie. check.sh also has no ctest leg at all.
- **Solution**: any count written into a commit message, ledger row or history entry must be measured on a freshly configured+built tree (dedicated dir, wipe allowed - the build tree, never the IDE's out/build); string/naming sweeps grep ALL consumer classes (probes, demos, docs, tests) before declaring single-source.
- **Verification**: before quoting N/M, `rm -rf <scratch-build> && configure && build && ctest`; grep both spellings (`"Echo"` and `"echo"`) across bindings/ examples/ crates/ and the result table appears in the report.
- **Forbidden**: warm-tree counts in any permanent record; claiming "single-sourced" from a first-pass grep that only covered emitters.

## PIT-18: the first consumer's dialect becomes the universal law in disguise (2026-10-20, protobuf landmine)
- **Symptom**: runner self-check demanded byte-equality echo of a non-empty literal probe - fine for the raw-bytes rs twin, SIGABRT-impossible for typed protobuf templates (empty request -> non-empty typed reply); cpp x exe placement could not start at all.
- **Root cause**: the guard was written when exactly ONE plugin family existed and codified that family's convention ("shadow-runner-selfcheck" round-trip) as if universal; the probe-drop+reload double-init (protobuf global pool) is the same substitution class: single-family assumptions hiding in shared machinery.
- **Solution**: shared-door rules must test the mechanism (door opens, junk refused), not the exemplar's bytes; fidelity assertions live where the contract actually matters (parent-side proxy round-trip). When a second family falsifies a guard, fix the guard at its source and record what it now does NOT catch.
- **Verification**: for each guard touching plugin-supplied data: "would this pass/fail the same way for a typed (protobuf) template AND a raw-bytes template?" - name both in the guard's comment.
- **Forbidden**: re-relaxing a guard without stating which corruption class moved where; treating "family-1 always passed" as evidence the rule is universal.

## PIT-19 (trivial): probe printed its success line unconditionally over four FAILs (2026-10-20)
- Summary printfs in tests/probes must be result-dependent; green prose beside red counters is a lie with better formatting - and it actively hides the class the probe exists to catch.

## PIT-20 (trivial) - out/build is the IDE's tree; orchestrator scratch builds go to dedicated dirs (2026-10-20)
- Wiping/reconfiguring out/build to "fix" a generator-mismatch destroys the .vscode-pinned tree (PIT-7 lineage); use /tmp or the demo scripts' dedicated out/build-stageN instead.

## PIT-21 (trivial): adjudication walkthrough body in ledger-dense register got a plain-speech demand (2026-10-21)
- The skill's own plain-language rule was violated by file:line citation chains + jargon in the BODY; the seven-part template survives being written in everyday terms - fix: body speaks what changes for a human, ledger anchors go to the Sources section only.
- **Second occurrence same session (v2 hearing item 2)**: the correction did not stick item-side - plain speech must be the DEFAULT from item one, not a per-patch style; hearings walk in human terms or get re-walked.
## PIT-22 (trivial): an assertion whose output the harness never reads is an assertion that never ran (2026-10-21)
- The M2 demo's dial-refusal word sat on a dead branch (parent killed the child at READY, before the post-READY lines could be read); no red was ever observable, so C0 was silently unmet. Fix pattern: make the observer WAIT the marker before ending the observed, and demand one planted red per new stage.

## PIT-23: an instrumented coverage rerun can silently execute fewer suites than the tree has - a false-HIGH percent (2026-10-21)
- **Symptom**: the D95 acceptance recorded core 82.39 with the then-47-suite tree; the honest rerun on a smaller tree (51 suites) measured 66.95. cov1 had executed 25 runnable suites, cov2 27 - twenty-two integration suites never ran under instrumentation (no error surfaced in the redirected log; the binary cache was warm).
- **Root cause**: cargo-llvm-cov's instrumented test phase silently skipped already-built integration binaries; the number looked plausible, and plausible numbers do not get recounted. Same family as PIT-17 (stale trees fabricate green) and D84 (metric definition is part of the claim) - the mirror direction: a missing denominator inflates rather than deflates.
- **Solution**: coverage claims must be accompanied by a suite-execution census (count the "Running unittests/tests" lines in the redirected log; compare against the workspace's binary inventory: test-targets minus lib-bins must equal executed). On mismatch, rerun `clean`-style and re-cite. Current canon after correction: core 66.95 / transport 80.78 / codegen 84.25, line metric D84a; core's real debt is named at the stage-4 §5 acceptance row.
- **Verification**: `grep -c "Running" <covlog>` vs suite inventory from `cargo test --workspace` redirected log; the census line goes next to any quoted percentage.
- **Same-day label correction**: the quoted column ($4 of the summary TOTAL) is the REGION metric, not line% - the series (85.52/85.24/80.78/82.39/66.95) is internally consistent on that column; canon name forward: 'region metric per D84a'. The misnomer stays in same-day D95/plan prose; corrected here and forward.
- **Forbidden**: quoting a coverage % without its suite census; trusting an instrumented run because "the tests are green elsewhere".

## PIT-24: an outbound artifact can be falsified by your own same-day work (2026-10-21)
- **Symptom**: the K7 stable-note draft (armed for days) said "v2 bus/clock verbs are PLANNED, not drawn (D82/K4)" - then the very same session held the hearing (D96), drew the signatures, and shipped N1+N2, making the outbound sentence false before it was ever sent.
- **Root cause**: the as-of-then discipline protects INTERNAL dated rows; an outbound note is never "as of then" - it speaks the moment it is sent. Nothing in the workflow re-checked the outbox when a landing reversed a claim the draft carried.
- **Solution**: when a landing contradicts any claim in an armed outbound artifact, the artifact's register row (status Open Items / register) must gain a "REFRESH REQUIRED" mark in the same commit as the landing; re-verify the whole outbox before send.
- **Verification**: `grep -L "REFRESH" docs/reference/k7*.md` style sweep against the day's D-records before any send; the K7 refresh today (commit 9d5c200) is the precedent execution.
- **Forbidden**: sending an armed draft whose text predates a same-day ruling that moved the facts.


## PIT-25: a malformed replace op "succeeds" and inserts while the consumed region survives (2026-10-21)
- **Symptom**: batch edit ops issued with an `end` anchor but NO `pos` anchor returned plain "Updated" success while behaving as inserts: the Rust bin got a duplicate arg-parse block that unclosed the `fn main` brace (compile error caught it), the plan doc got a duplicate Status paragraph and a dangling sentence (caught by re-read), and status.md grew a leftover continuation line that only a full re-read exposed (third occurrence).
- **Root cause**: the payload builder dropped the required `pos` for range replaces; the tool applied the lines at the wrong anchor instead of rejecting the op, and "Updated" reports insertion, not consumption - a success line proves nothing about the removed side.
- **Solution**: every range replace carries BOTH `pos` and `end`; after every edit that REMOVES or replaces content, verify BOTH directions: `grep -c "<new phrase>" file` (landed) AND `grep -c "<distinctive phrase of the consumed line>" file` (gone, expect 0); if a remnant shows, stop, re-read the whole region, then patch once - never layer more edits over the damage.
- **Verification**: `for p in "new-phrase" "old-line-phrase"; do echo "$p $(grep -c "$p" <file>)"; done` - the old-side count must be 0; diff `git diff --stat` against the intended file count.
- **Forbidden**: trusting "Updated" as evidence a range was consumed; issuing a second blind edit after discovering damage instead of a fresh full read; same family as edit-safety #17 (partial apply) and #8/#9 (duplicates), but this variant ships a SUCCESS verdict.
- **Same-session recurrence note (rule 19 family)**: the register line that triggered this hunt was itself a self-inflicted copy - the session transcribed stage-5's armed list verbatim while stage-4 §6 had already CLOSED its regen item; when propagating a list between ledger files, re-derive each row from its own truth source first (rule 19 generalized from gate commands to registers).
## PIT-26: ledger-voice answers (walkthrough AND status reports) draw "say it plainly" corrections - three occurrences
- **Symptom**: during the wave-2-closeout ruling walk, items 1 and 3 were first presented with ledger-quote lead-ins and file:line enumeration as the body; the maintainer demanded plain-language rewrites both times; the rewrites kept the same facts and passed immediately.
- **Root cause**: the seven-part template was read as a citation ceremony - Sources/Recommendation sections degenerated into "D84 says..., stage-4 §5 L181 says..." chains, burying what was actually being decided; the skill's rule 4 (plain language, survives being read aloud) governs ALL sections, not just Details.
- **Solution**: in any walkthrough, each section opens with the spoken-language substance first (what changes, for whom, what breaks); record anchors (D-numbers, file:line) appear only as short parenthetical pointers AFTER the plain sentence, never as the sentence.
- **Verification**: read-aloud check per paragraph - a colleague outside this repo understands the decision before knowing what any cited record is; no section's first sentence is a quotation or a file:line.
- **Forbidden**: leading an adjudication with a ledger citation; using file:line dumps as body text; needing two same-session corrections to learn a format rule the skill already states.
- **Recurrence 4 (2026-10-22)**: the Python-hearing R1 walkthrough redrew the same class again - correct seven-part shape, but Sources was a file:line stack and the prose still carried builder jargon (gate leg, milestone, byte-fidelity, ledger flip); the correction landed on vocabulary level, not structure. Prevention: before sending any walkthrough message, read it aloud and replace EVERY term a non-builder would stumble on (anchor pointers go in trailing parentheses only); a correctly-shaped brief is still a brief.
- **Recurrence 3 (2026-10-22)**: the class escaped the walkthrough - a design-status analysis + next-volume recommendation was delivered as a D-number/quote table and immediately corrected again; the rule generalizes to ALL maintainer-facing answers (reports, analyses, recommendations): plain sentences carry the substance, record anchors are optional parentheticals at most, and a recommendation must be arguable without knowing any D-number.

- **Recurrence 5 (2026-10-23)**: the MediaServo-benchmark scope adjudication was first shipped as a term-stack (configure/CTEST keyword/version-floor/--self-test/diff as body sentences, file:line dump inside Sources). Same fix as recurrences 3-4 applies with prejudice: before sending ANY walkthrough message, replace every word a non-builder stumbles on, anchors live only in trailing parentheses. The skill states this rule; a third re-learn of a stated rule is the violation itself.

- **Recurrence 6 (2026-10-23)**: adjudication 3 of the benchmark-caliber walk again read as legalese-builder-mix (verdict conditions, gate semantics, class-names as body text); corrected same session. Standing pre-send ritual now mandatory: every walkthrough message must pass a literal read-aloud pass with the test "would a neighbor outside programming understand this?", replacing terms like gate/verdict/lever/class with household words BEFORE sending.

## PIT-27: a resumed critic session stalls silently twice a day; a fresh session completes the same grill in minutes (2026-10-21)
- **Symptom**: two resumed Momus review sessions (increment re-review of a written-back doc) burned the full 30-minute poll window with zero activity ("Poll inactivity timeout reached after 1800000ms"), on different sessions, same day; fresh Momus sessions on the same class of work returned full verdicts in 5-7 minutes (the T2 six-grill review: 5m23s, all anchors verified).
- **Root cause**: the resume path waits on a child that stopped reporting status; whether the review actually ran somewhere unseen is unknowable from the poll side - the deliverable never arrives, so the timeout is a non-event, not a failure to retry-by-resuming.
- **Solution**: dispatch a FRESH critic session with a self-contained prompt for every full re-review (the file path plus the exact verification asks); resume a critic session only for one-line follow-ups on a session that just answered.
- **Verification**: the signature string "Poll inactivity timeout" in the tool result = immediate re-dispatch fresh, never a second resume attempt; count with `grep -c "Poll inactivity timeout" <session-log>` - more than one per day means every resumed review is on notice.
- **Forbidden**: gating implementation on a resumed-critic poll twice; reading a stall as "the review failed with findings" (absence of verdict is absence of verdict).

## PIT-28: an adjudicated hearing text shipped a capacity constant the tree never had (2026-10-21)
- **Symptom**: the D99 clock hearing drew R4-A with "QoS Lossy(16)" in the options, the walkthrough, the §8 results table and the ledger row; the landed bus vocabulary answers wire code 2 with the FIXED slot `LOSSY_RING = 64` (embed_codec.rs:50,110) - the literal 16 existed nowhere; the T2 landing had to carry a named amendment (capacity unified to the vocabulary constant) to keep code and adjudicated prose honest.
- **Root cause**: the option text was drafted before the codec table was read; the walkthrough verified every STRUCTURAL claim against rulings (all survived Momus three passes) but a bare round number inside an option sentence is not structural - adjudicated prose inherits whatever it was written with, and a walked number looks verified because its sentence context was.
- **Solution**: before a hearing/agenda document goes to the walkthrough, grep-anchor every concrete number (capacity, count, bit value, version) to its source file in the same sentence; where the truth is a vocabulary slot, name it by its code identifier (`LOSSY_RING`), never by a rounded guess; unverifiable numbers get written as "the slot named at landing" and the landing row records the pin (the gen-field case, done right).
- **Verification**: `grep -nE "[A-Za-z_]*\((16|32|64|128)\)" docs/wave-2-*.md docs/stage-*.md` - every hit must sit in an amendment note, a quote of the wrong, or beside a file:line anchor; doc-audit carries the same hunt per future hearing doc.
- **Forbidden**: ruling on a literal never read from the code; silently rewriting the adjudicated file (the amendment rides the landing row, per D99 §9's own style).

## PIT-29: a worker session reports "complete" while its transcript reader errors - the tree is the deliverable (2026-10-23)
- **Symptom**: the M2 python worker finished (status completed) but background_output returned "[object object]"-class errors on every fetch variant (its resumed session had also been aborted once before); no report text ever arrived although all expected artifacts sat in the working tree.
- **Root cause**: the child session's message store got corrupted across the abort+resume cycle; task STATUS and task REPORT are independent signals - a "completed" status says nothing about report readability, and waiting on it stalls the lane (same shape as PIT-27's stall, different layer).
- **Solution**: declare the report lost after ONE failed fetch; take over via the tree-audit protocol - treat the dirty tree as a patch from an anonymous author: diff audit (scope boundaries, discipline checks, pin/citation verification) + full live re-measurement of every gate the missing report would have carried; never re-dispatch the same milestone from scratch when the artifacts audit clean.
- **Verification**: before accepting any milestone, the orchestrator's own transcripts exist for: build exit codes, test totals from redirected files, demo green/red/restore states; a worker report missing while these exist locally = fully covered; `grep -c "cannot fetch\|object Object" <session-notes>` - recurrence means the abort-prone resume pattern should be replaced by fresh dispatch with a self-contained brief.
- **Forbidden**: stalling the lane waiting for an unreadable report; trusting "completed" status as delivery evidence; re-running whole milestones whose artifacts pass independent audit (waste, and a second worker racing the same tree = PIT-14 class).

## PIT-30: warm clippy fingerprint serves false green to re-verification (2026-10-23)
- **Symptom**: after the workspace lints landed, the orchestrator's fresh `clippy --workspace --all-targets -D warnings` returned GREEN while crates/modukit-js/src/lib.rs:399 still carried a violating `expect()`; the same command went RED later with the file content unchanged in between except an mtime touch from a probe restore.
- **Root cause**: cargo/clippy cache fingerprints can survive content-adjacent edits (flag matrix already baked, artifact reused); a "green" served from cache is not a recompute - re-verification inherited the delegated lane's stale verdict instead of producing its own.
- **Solution**: orchestrator revalidation after lint-surface edits must force real compiles: touch the suspect files, or `cargo clean -p <crate>` before the leg, or verify with the failing-command replay plus grep of the emitted error lines (0 errors only counts when compilation actually ran - check the log has "Compiling" lines for the crate).
- **Verification**: `grep -c "Compiling modukit-js" <log>` >= 1 alongside exit 0; a green log with zero "Compiling" lines for the touched crate is a cache verdict - rerun with a touch.
- **Forbidden**: accepting a re-verification GREEN whose log never recompiled the touched zone; recording gate evidence from warm-cache runs at batch edges.

## PIT-31: worktree edits can vanish between green verification and commit under parallel agents (2026-10-23)
- **Symptom**: crates/modukit-js/src/lib.rs carried the typed-error kernel fix, clippy legs and the full gate went green at 16:02; at the 16:05 commit the same file (worktree, index, and resulting tree) showed the pre-fix expect line again - no git operation, no cp, no agent owned that pathspec in between.
- **Root cause**: unidentified worktree tampering in the shared-session environment (same contamination class as the task-store cross-talk and injected tool-result noise reported by the three concurrent lanes this volume); the cargo/index plumbing stayed consistent with the reverted file throughout.
- **Solution**: treat every "verified green" file as unverified until its content is confirmed IN THE COMMIT TREE: re-apply, commit, then verify per-commit with git show <sha>:<path> | grep -c "<fix-phrase>" == 1 (and git log -S "<phrase>" finds the commit). Batch landings with concurrent agents get this sweep as the final step before declaring done.
- **Verification**: the post-commit grep returns 1 for each fix-carrying file of the batch; a 0 result means the edit evaporated again - re-land and re-verify immediately.
- **Forbidden**: declaring a batch landed on gate-green evidence alone when concurrent agents share the worktree; trusting git status clean as proof of intended content (clean matches whatever HEAD holds, including regressions).

### PIT-31 CORRECTION (same day - true root cause found, the ledger must not carry a wrong story)
- **What actually happened**: crates/modukit-js/src/lib.rs is a GENERATED file (its own header: "Generated by modukit-codegen ... DO NOT EDIT", D63). The demo-wave2-js legs REGENERATE it every gate run (demo-wave2-js.sh:96 copies the emitted bridge over it). The hand-fix applied twice "evaporated" because the codegen leg legitimately re-emitted the UNFIXED template - the environment was innocent, the edit was in the wrong layer.
- **The real rule**: on a file whose banner says generated, fixes belong in the EMITTER (crates/modukit-codegen/src/js.rs), then scripts/regen-golden.sh, then the skin copies from the golden (leg grammar). Head -5 before any repair edit; if it says Generated, you do not edit it by hand.
- **Verification**: after the emitter fix + regen + skin-copy, full gate ran (legs re-emit) and lib.rs stayed byte-identical to the golden, forced-fresh clippy green (Compiling modukit-js present). That is the root-cured state.
- **Salvaged from the wrong entry**: the post-commit tree-verification discipline stays (it caught the content drift immediately - it just blamed the wrong defendant).
