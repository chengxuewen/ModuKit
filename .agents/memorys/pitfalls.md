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

No pitfalls recorded yet.

## PIT-1: Treated an exploratory "how about X?" as execution approval (2026-10-09)
- **Symptom**: User asked "how about implementing it inside .opencode/init-mcp-codegraph.mjs?" (an evaluative question; original wording was Chinese). I rewrote the wrapper (.mjs→.sh), edited opencode.json + .gitignore, ran a live install/init test, and left 286MB of downloaded artifacts — all before any approval. User: "I was only discussing with you — why did you execute directly?" (original wording was Chinese)
- **Root cause**: Approval carry-over — the user's earlier question-tool answer "install officially now" (original wording was Chinese) authorized the *old* plan (global official install), and I silently extended it to cover the *new, different* plan this turn (project-local sh wrapper). The existing rule (`rules/common/edit-safety.md` "User Confirmation Before Edit") already states questions ≠ instructions; I pattern-matched momentum instead of re-checking the gate.
- **Solution**: On being called out: stop, enumerate exactly what changed and what side-effects ran (with sizes/timestamps), present rollback options via one question, wait for an explicit decision (a "keep everything" decision was given; cleanup executed only after that).
- **Verification**: Before any file mutation or install/download, the CURRENT user message must contain a directive verb ("implement / change it / proceed / keep") — evaluative phrasings equivalent to "how about / is it feasible / let us discuss" never count. Test with: quote the exact user sentence authorizing this action, or stop and ask.
- **Forbidden**: Inheriting approval from a previous plan/scheme to a changed scheme; running install/init tests without the same confirmation as edits; treating silence or a partial answer as blanket approval.
