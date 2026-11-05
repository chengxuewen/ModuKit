#!/usr/bin/env bash
# C1 gate: every tracked or untracked (non-ignored) artifact is English.
# Whitelisted classes (extend deliberately, never silently):
#   1) dependency and build trees (SKIP_PARTS / SKIP_PATH_PREFIX).
#   2) LINE ALLOW -- a line that legitimately carries functional CJK ends with an
#      inline marker: `<!-- c1:allow:trigger -->` (markdown) or
#      `# c1:allow:trigger` / `# c1:allow:vocab` (code, YAML). Exemption is per
#      line, so everything around it stays guarded (adjudication #28: the old
#      whole-directory skip on `.agents/skills/` was a standing blind spot).
#   3) ALLOW -- whole-file exemption for cases where a marker cannot live on the
#      line (YAML frontmatter values, docstring examples); each entry carries a
#      recorded reason below.
#   4) .po/.pot are outside EXTS (book translations are functional CJK).
# Red/green duty (C0): plant an UNMARKED Chinese line in a skill file -> red;
# plant one in README.md -> red; marker removed -> red again.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
python3 - <<'PY'
import pathlib
import re
import subprocess
import sys

CJK = re.compile(r'[\u4e00-\u9fff\u3000-\u303f\uff01-\uff60]')
tracked = subprocess.run(['git', 'ls-files'], capture_output=True, text=True).stdout.split()
untracked = subprocess.run(
    ['git', 'ls-files', '--others', '--exclude-standard'],
    capture_output=True,
    text=True,
).stdout.split()
candidates = sorted(set(tracked + untracked))
SKIP_PARTS = {'.omo', '.pixi', 'node_modules', 'target', 'build', '.git', 'out', '.refinfo', '.qoder', '.serena'}
SKIP_PATH_PREFIX = ('.opencode/node_modules/', 'book/web/vendor/')
LINE_ALLOW = re.compile(r'c1:allow:(?:trigger|vocab)\b')
EXTS = {
    '.md', '.rst', '.txt', '.cmake', '.rs', '.c', '.cpp', '.h', '.hpp', '.toml',
    '.json', '.jsonc', '.sh', '.bat', '.mjs', '.js', '.py', '.yml', '.yaml',
    '.in', '.wgsl', '.pc', '.jsonl',
}
# Whole-file exemptions -- one line each, with the reason inline.
# adjudication-walkthrough: the frontmatter description LEADS with the verbatim
# Chinese activation phrase (hosts match on it); a marker inside the quoted YAML
# value would pollute the description, so the file is exempt and its body lines
# are still marker-scanned by convention.
# book-to-skill utils.py: Chinese numeral/chapter vocabulary is functional parse
# data across 14 lines, two of them inside a docstring where a marker would land
# in user-visible help text.
ALLOW = {
    '.agents/skills/adjudication-walkthrough/SKILL.md',
    '.agents/skills/book-to-skill/book_to_skill/utils.py',
}

bad = []
for f in candidates:
    p = pathlib.Path(f)
    if not p.is_file():
        continue
    if p.suffix not in EXTS:
        continue
    if any(part in SKIP_PARTS for part in p.parts):
        continue
    if f.startswith(SKIP_PATH_PREFIX):
        continue
    try:
        text = p.read_text(errors='ignore')
    except Exception:
        continue
    for i, line in enumerate(text.splitlines(), 1):
        if CJK.search(line) and str(p) not in ALLOW and not LINE_ALLOW.search(line):
            bad.append(f"{p}:{i}: {line.strip()[:70]}")

print('\n'.join(bad) if bad else '')
if bad:
    print(f"GATE-I18N ✗ {len(bad)} CJK lines outside the whitelist")
    sys.exit(1)
print("GATE-I18N ✓ (tracked + untracked artifacts English; per-line c1:allow markers honoured, deps skipped)")
PY
