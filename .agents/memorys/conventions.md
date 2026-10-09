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

Every file this project produces — code, comments, memory files, markdown docs (README, `docs/`, rules) — is written in English. Only AI-session replies follow the user's language (Chinese in, Chinese out); artifacts stay English regardless.

Exception: functional CJK literals that must stay verbatim (trigger keywords in `adjudication-walkthrough`, Chinese chapter-numbering parse data in `book-to-skill`, routing phrases in `skill-router`). `.agents/skills/**` is excluded from the scan; new skills follow the same rule — English prose, verbatim functional literals only.

```bash
# Check: no CJK (incl. CJK/full-width punctuation) in scanned artifacts
grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ $(test -d crates/ && echo crates/) \
  && echo "C1 VIOLATION (lines above)" || echo "C1 OK"
```

## Adding conventions

Add `C1`, `C2`, … here as project decisions harden. Each entry needs: one quoted constraint line + a runnable check block, and must be recorded the moment the decision is made (see `.agents/rules/common/lesson-memory.md`). No project-specific conventions are recorded yet.
