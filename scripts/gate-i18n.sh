#!/usr/bin/env bash
# C1 gate: every tracked or untracked (non-ignored) artifact is English.
# Whitelisted classes (extend deliberately, never silently):
#   1) .agents/skills/** -- routing trigger phrases and keyword tables are the
#      activation surface; the whole directory is skipped.
#   2) .opencode/node_modules + vendored deps.
#   3) .po/.pot are outside EXTS (book translations are functional CJK).
# Red/green duty (C0): plant a Chinese comment in README.md -> this gate MUST go red.
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
SKIP_PATH_PREFIX = ('.agents/skills/', '.opencode/node_modules/', 'book/web/vendor/')
EXTS = {
    '.md', '.rst', '.txt', '.cmake', '.rs', '.c', '.cpp', '.h', '.hpp', '.toml',
    '.json', '.jsonc', '.sh', '.bat', '.mjs', '.js', '.py', '.yml', '.yaml',
    '.in', '.wgsl', '.pc', '.jsonl',
}
ALLOW = set()  # add a path only with a recorded reason in the header

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
        if CJK.search(line) and str(p) not in ALLOW:
            bad.append(f"{p}:{i}: {line.strip()[:70]}")

print('\n'.join(bad) if bad else '')
if bad:
    print(f"GATE-I18N ✗ {len(bad)} CJK lines outside the whitelist")
    sys.exit(1)
print("GATE-I18N ✓ (tracked + untracked artifacts English; skills surface and deps skipped)")
PY
