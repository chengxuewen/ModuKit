#!/usr/bin/env bash
# leg: doc-liveness (M3S-1 c1 - the sole PORT_NOW tribunal survivor, D-ruling 2026-10-19).
# Extract backtick-quoted repo-relative paths from the five instruction-scope files
# (root AGENTS.md, .agents/AGENTS.md, conventions.md, status.md, + README.md since M3S-2).
# Scope is tight on purpose - on-demand docs are not scanned.
# Filter bias: SKIP the candidate when in doubt, never FAIL on a guess. So anything that
# looks like a command (spaces, shell metachars), prose, an absolute/home path, a pure
# range shorthand (docs/modules/00-11), or a bare filename (no slash - ambiguous to
# resolve) is skipped. Glob candidates are informational: match passes, no-match skips.
# Fenced code blocks are skipped entirely (they hold commands and trees, not pointers).
set -uo pipefail
cd "$(dirname "$0")/.."

FILES=(AGENTS.md .agents/AGENTS.md .agents/memorys/conventions.md .agents/memorys/status.md README.md)

missing=0
checked=0

cands=$(awk '
  BEGIN { REJ = "[\\\\=&|<>;:,\"$(){}]" }
  FNR == 1 { fence = 0 }
  /^```/   { fence = !fence; next }
  fence    { next }
  {
    n = split($0, p, "`")
    for (i = 2; i <= n; i += 2) {
      c = p[i]
      if (c == "") continue
      if (c ~ /[ \t]/) continue
      if (c ~ REJ) continue
      if (c ~ /^[\/~]/) continue
      last = c; sub(/.*\//, "", last)
      if (last ~ /^[0-9]+-[0-9]+$/) continue
      if (c ~ /[*?\[\]]/) { print "GLOB\t" FILENAME "\t" c; continue }
      if (index(c, "/") == 0) continue  # bare names are ambiguous prose - skip, never guess
      print "PATH\t" FILENAME "\t" c
    }
  }
' "${FILES[@]}")

if [ -n "$cands" ]; then
  while IFS=$'\t' read -r kind file cand; do
    case "$kind" in
      GLOB)
        compgen -G "$cand" > /dev/null 2>&1 && checked=$((checked + 1))
        ;;
      PATH)
        checked=$((checked + 1))
        if [ ! -e "$cand" ]; then
          printf 'dead doc-ref: %s -> `%s` (no such path from repo root)\n' "$file" "$cand"
          missing=$((missing + 1))
        fi
        ;;
    esac
  done <<< "$cands"
fi

if [ "$missing" -gt 0 ]; then
  echo "check-doc-refs: RED ($missing dead ref(s))"
  exit 1
fi
echo "check-doc-refs: ok ($checked path candidates checked, 0 dead)"
