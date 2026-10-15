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

Every file this project produces — code, comments, memory files, markdown docs (README, `docs/`, rules), build configuration (pixi.toml, CMakeLists.txt, cmake/) — is written in English. Only AI-session replies follow the user's language (Chinese in, Chinese out); artifacts stay English regardless.

Exception: functional CJK literals that must stay verbatim (trigger keywords in `adjudication-walkthrough`, Chinese chapter-numbering parse data in `book-to-skill`, routing phrases in `skill-router`). `.agents/skills/**` is excluded from the scan; new skills follow the same rule — English prose, verbatim functional literals only.

```bash
# Check: no CJK (incl. CJK/full-width punctuation) in scanned artifacts
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"
```

## C2: "Branch discipline: master receives milestone merges; work lives on stage1/mN branches; submodule edits go to PolyOrch's own branch, the pin bumps separately"

Milestone work on this repo is committed on `stage1/m<N>-<slug>` branches; `master` only ever receives a milestone merge after that milestone's gates are green. Changes inside the `PolyOrch/` submodule are made on a branch of that repository (never on the detached pin), pushed by the maintainer's own call, and only then does ModuKit record the new gitlink as a standalone `chore(deps): bump PolyOrch <old>..<new>` commit (D67: the gitlink is the single version authority; no branch= field, no floating).

```bash
# Check 1: branch name is master or stage1/mN-slug
git branch --show-current | grep -qE '^(master|stage1/m[0-9]+-[a-z0-9-]+)$' \
  && echo "C2 branch OK" || echo "C2 VIOLATION (branch name outside master|stage1/mN-slug)"
# Check 2: submodule worktree stays clean at the pin (upstream edits must happen on a PolyOrch branch)
test -z "$(git -C PolyOrch status --porcelain)" && echo "C2 submodule clean" || echo "C2 VIOLATION (dirty PolyOrch worktree on the pin)"
```
# negative proof observed 2026-10-15: branch `stage1/BAD_NAME_test` -> RED, valid slug -> GREEN


## Adding conventions

Add `C2`, `C3`, … here as project decisions harden (C0/C1 are live above). Each entry needs: one quoted constraint line + a runnable check block, and must be recorded the moment the decision is made (see `.agents/rules/common/lesson-memory.md`).
