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
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "$0")/.."
IN="${MODUKIT_GATE_INPUT_DIR:-}"; [ -n "$IN" ] && IN="${IN%/}/"

# ---- self-test (benchmark plan T2.2) ----
# Copy the five scanned docs to a scratch dir, append one dead repo-relative
# path to the scratch AGENTS.md, re-run against the scratch copy: the dead
# ref MUST go red. Path candidates still resolve against the repo root
# (only the scanned-file location is redirected). Repo files are never touched.
if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  mkdir -p "$s/.agents/memorys"
  cp AGENTS.md README.md "$s/"
  cp .agents/AGENTS.md "$s/.agents/"
  cp .agents/memorys/conventions.md .agents/memorys/status.md "$s/.agents/memorys/"
  printf 'Self-test dead pointer: `docs/self-test-defect-dangling-path.md`.\n' >> "$s/AGENTS.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-doc-refs.sh): planted defect NOT caught"
    exit 1
  fi
  echo "self-test OK (check-doc-refs.sh): planted defect caught"
  exit 0
fi

FILES=("${IN}AGENTS.md" "${IN}.agents/AGENTS.md" "${IN}.agents/memorys/conventions.md" "${IN}.agents/memorys/status.md" "${IN}README.md")

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
