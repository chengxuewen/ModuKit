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
# Red/green duty (C0) is machine-replayable (adjudication #35 / backlog P6):
# `--self-test` plants an unmarked CJK line (must RED) and a `c1:allow:trigger`
# line (must stay green) in a scratch scan root, then asserts a marked-only
# root is green. The script source itself carries zero CJK bytes (plants are
# \u escapes), so this gate never needs to exempt its own file.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "$0")/.."

# ---- self-test (adjudication #35, backlog P6) ----
# Scratch scan root via MODUKIT_GATE_INPUT_DIR, the check-capability-ledger
# sibling pattern. One run asserts three things: the planted UNMARKED CJK line
# is reported (red), the marked line is NOT (marker honoured), and after
# dropping the unmarked file the root is fully green. Repo files never touched.
if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  printf 'plain line\nunmarked \u4e2d\u6587 line\n' > "$s/doc.md"
  printf 'marked \u4e2d\u6587 line <!-- c1:allow:trigger -->\n' > "$s/marked.md"
  out=$(MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" 2>&1); rc=$?
  [ "$rc" -ne 0 ] || { echo "self-test FAIL (gate-i18n.sh): planted unmarked CJK NOT caught"; exit 1; }
  case "$out" in *"doc.md:2"*) ;; *) echo "self-test FAIL (gate-i18n.sh): unmarked line not reported at doc.md:2"; exit 1;; esac
  case "$out" in *"marked.md"*) echo "self-test FAIL (gate-i18n.sh): c1:allow marker not honoured"; exit 1;; esac
  rm -f "$s/doc.md"
  MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" >/dev/null 2>&1 \
    || { echo "self-test FAIL (gate-i18n.sh): marked-only root should be green"; exit 1; }
  echo "self-test OK (gate-i18n.sh): planted CJK caught, marker honoured, clean root green"
  exit 0
fi

python3 - <<'PY'
import os
import pathlib
import re
import subprocess
import sys

CJK = re.compile(r'[\u4e00-\u9fff\u3000-\u303f\uff01-\uff60]')
IN = os.environ.get('MODUKIT_GATE_INPUT_DIR', '')
if IN:
    _root = pathlib.Path(IN)
    candidates = sorted(
        str(p.relative_to(_root)) for p in _root.rglob('*') if p.is_file()
    )
else:
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
    rel = pathlib.PurePosixPath(f)
    if rel.suffix not in EXTS:
        continue
    if any(part in SKIP_PARTS for part in rel.parts):
        continue
    if f.startswith(SKIP_PATH_PREFIX):
        continue
    p = (_root / f) if IN else pathlib.Path(f)
    try:
        text = p.read_text(errors='ignore')
    except Exception:
        continue
    for i, line in enumerate(text.splitlines(), 1):
        if CJK.search(line) and f not in ALLOW and not LINE_ALLOW.search(line):
            bad.append(f"{f}:{i}: {line.strip()[:70]}")

print('\n'.join(bad) if bad else '')
if bad:
    print(f"GATE-I18N ✗ {len(bad)} CJK lines outside the whitelist")
    sys.exit(1)
print("GATE-I18N ✓ (tracked + untracked artifacts English; per-line c1:allow markers honoured, deps skipped)")
PY
