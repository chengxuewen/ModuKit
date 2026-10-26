# ModuKit — Conventions

> Loaded into `instructions[]` on every turn. Keep it short.
> Format per `.agents/rules/common/lesson-memory.md`: `C{n}: "<constraint>" + a runnable check command`.
> A convention without a runnable check is not a convention.
> PolyOrch-era conventions C1–C6 were not ported; the originals live in `PolyOrch/.agents/memorys/conventions.md`.

## C0: "Constraints must be executable"

Every constraint must carry a concrete command and a pass/fail criterion. "Be careful about X" is not a constraint. A check is not trusted until it has been OBSERVED to fail at least once (negative proof: plant a violation, see red; remove, see green) — an unfalsifiable check is decoration.

```bash
# Template: replace the placeholders with a real check
test -f .opencode/opencode.json && python3 -m json.tool .opencode/opencode.json > /dev/null && echo "config valid"
```

## C1: "All artifacts English; chat follows the user"

Every file this project produces — code, comments, memory files, markdown docs (README, `docs/`, rules), build configuration (pixi.toml, CMakeLists.txt, cmake/), and contract originals (`contracts/*.proto` and their `.h`/`.rs` goldens) — is written in English. Only AI-session replies follow the user's language (Chinese in, Chinese out); artifacts stay English regardless.

Exception: functional CJK literals that must stay verbatim (trigger keywords in `adjudication-walkthrough`, Chinese chapter-numbering parse data in `book-to-skill`, routing phrases in `skill-router`). `.agents/skills/**` is excluded from the scan; new skills follow the same rule — English prose, verbatim functional literals only.

```bash
# Check: no CJK (incl. CJK/full-width punctuation) in scanned artifacts
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ contracts/ examples/ bindings/ docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ test-plugins/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"
```
# check scope grew with artifact classes: +M1 build files, +contracts/, +examples/,
# +bindings/ (2026-10-23 doc-audit closed the unguarded Stage-4 shelf; plant-and-
# observe red with a CJK scratch file under bindings/, removed, re-green - C0).

## C2: "Submodule branch discipline: PolyOrch edits happen on its dev-modukit branch; ModuKit works on master and pins the dev-modukit tip, kept fresh"

ModuKit milestone work is committed directly on `master` (no feature-branch ritual - corrected 2026-10-15 after the maintainer rejected the original misreading). Changes inside the `PolyOrch/` submodule follow the upstream flow: first pull latest `main`; develop on the dedicated branch `dev-modukit` (never edit in detached HEAD on the pin); run the relevant tests before pushing dev. When a change set is done, merge into `main` and run PolyOrch's test suites ON `main` (the dev branch is never the blessed state); PIN POLICY (maintainer decision 2026-10-15, superseding the earlier main-blessing-only rule): ModuKit pins the CURRENT TIP of `dev-modukit` and keeps it as fresh as practical - each refresh is a standalone `chore(deps): bump PolyOrch <old>..<new>` gitlink-only commit made by fetching, fast-forwarding the submodule checkout, and staging the gitlink (D67: the gitlink remains the single version authority; no branch= field in .gitmodules, no floating refs - every bump records an exact commit).

```bash
# Check 1: ModuKit work lands on master only
git branch --show-current | grep -qx master \
  && echo "C2 master OK" || echo "C2 VIOLATION (ModuKit work outside master)"
# Check 2: submodule worktree stays clean at the recorded pin (refresh = fetch + ff + standalone bump)
test -z "$(git -C PolyOrch status --porcelain)" && echo "C2 submodule clean" || echo "C2 VIOLATION (dirty PolyOrch worktree at the pin)"
```
# negative proof observed 2026-10-15: HEAD on a tmp branch -> RED, on master -> GREEN
# (superseded text of the first C2 draft - milestone-branch ritual - kept nowhere; the M2 merge commit stands as history)

## C3: "Commit shape: subject under 60 chars, details as wrapped bullets in the body"

Every commit message in ModuKit: `type(scope): short imperative subject` at most 60 characters (no " - " cramming chains into the subject - the bundling habit this rule kills), then a blank line, then body lines each starting ` - ` and at most 100 characters. Rationale lands in the body or not at all. Mainstream grounding: Conventional Commits recommends ~50 for the subject; the kernel checkpatch warns past 75 and git.git's own submission guide wants a short imperative summary with the why in wrapped paragraphs; rust-lang/rust keeps component-prefix subjects short. Our observed failure 2026-10-15: 100-250 char run-on subjects (M2-M4 era) unreadable in `git log --oneline` and on forges.

```bash
# Check (run right after each commit; both parts must hold):
test $(git log -1 --format=%s | wc -m) -le 61 && echo "C3 subject OK" || echo "C3 VIOLATION (subject over 60)"
git log -1 --format=%b | grep -vE '^$|^ - .{1,99}$' | grep -q . \
  && echo "C3 VIOLATION (body lines not ' - ' wrapped or over 100)" || echo "C3 body OK"
```
# negative proof observed 2026-10-15: HEAD at rule-landing time (14654cf, 156-char
# subject) -> RED; the dogfooding commit of this rule -> GREEN. Machine version live since 2026-10-21: scripts/hooks/commit-msg (core.hooksPath, installed by bootstrap.sh). Reword of the 20 older
# local commits deferred until the M5 agent lands (repo-wide history work while a
# worker runs = PIT-14 class; all rewrites stay local - origin untouched).



## C4: "No circled-number glyphs; enumerate as 1) 2) 3) - in replies and artifacts"

Maintainer readability ruling 2026-10-23 (chat is the primary surface; artifacts
carry the same duty so the style never re-invades the repo). Circled numbers
(U+2460..U+2473) are banned; plain numbering "1)" "2)" is the replacement. The
existing corpus was swept the same day (9 files, plain glyph->number replacements).

```bash
# Check (artifacts; the chat half is reply-discipline, enforced at authoring):
grep -rIlP '[\x{2460}-\x{2473}]' AGENTS.md SKILL.md README.md mise.toml bootstrap.sh bootstrap.bat \
  pixi.toml CMakeLists.txt cmake/ contracts/ examples/ bindings/ docs/ \
  .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ test-plugins/ crates/ \
  && echo "C4 VIOLATION (files above)" || echo "C4 OK"
```
# negative proof observed 2026-10-23: planted circled glyph under docs/, RED shown,
# removed, GREEN (C0).

## Adding conventions

Add `C2`, `C3`, … here as project decisions harden (C0/C1 are live above). Each entry needs: one quoted constraint line + a runnable check block, and must be recorded the moment the decision is made (see `.agents/rules/common/lesson-memory.md`).
