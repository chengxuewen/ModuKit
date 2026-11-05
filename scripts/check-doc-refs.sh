#!/usr/bin/env bash
# leg: doc-liveness (M3S-1 c1 - the sole PORT_NOW tribunal survivor, D-ruling 2026-10-19).
# Extract backtick-quoted repo-relative paths AND markdown-link targets from the five
# instruction-scope files (root AGENTS.md, .agents/AGENTS.md, conventions.md, status.md,
# + README.md since M3S-2). The link pass closed a blind spot a planted defect proved
# on 2026-11-05 (PIT-37: `[x](dead.md)` sailed through the backtick-only extractor).
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

# ---- self-test (benchmark plan T2.2; link round added 2026-11-05 with the link pass) ----
# Copy the five scanned docs to a scratch dir and plant defects one syntax at a
# time: a backtick dead path must redden, a markdown-link dead target must
# redden, clean copies must green. Path candidates still resolve against the
# repo root (only the scanned-file location is redirected). Repo files untouched.
if [ "${1:-}" = "--self-test" ]; then
  seed_scratch() {
    mkdir -p "$1/.agents/memorys"
    cp AGENTS.md README.md "$1/"
    cp .agents/AGENTS.md "$1/.agents/"
    cp .agents/memorys/conventions.md .agents/memorys/status.md "$1/.agents/memorys/"
  }
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  seed_scratch "$s"
  printf 'Self-test dead pointer: `docs/self-test-defect-dangling-path.md`.\n' >> "$s/AGENTS.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-doc-refs.sh): planted backtick defect NOT caught"
    exit 1
  fi
  rm -rf "$s"; mkdir -p "$s"; seed_scratch "$s"
  printf 'Self-test dead link: [defect](docs/self-test-defect-dangling-link.md).\n' >> "$s/README.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-doc-refs.sh): planted markdown-link defect NOT caught"
    exit 1
  fi
  rm -rf "$s"; mkdir -p "$s"; seed_scratch "$s"
  if ! MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-doc-refs.sh): clean copies went RED"
    exit 1
  fi
  rm -rf "$s"
  echo "self-test OK (check-doc-refs.sh): backtick red, link red, clean green"
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
    s = $0
    while (match(s, /\]\([^) \t]*\)/)) {
      t = substr(s, RSTART + 2, RLENGTH - 3)
      s = substr(s, RSTART + RLENGTH)
      sub(/#.*/, "", t)
      if (t == "") continue
      if (t ~ /^[\/~]/ || t ~ /^\./) continue        # absolute or file-relative - skip
      if (t ~ /^https?:/ || t ~ /^mailto:/) continue
      if (t ~ /:/ || t ~ /[*?\[\]|]/) continue       # schemes and globs - never guess
      print "LINK\t" FILENAME "\t" t                # structured target: bare names count
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
      LINK)
        checked=$((checked + 1))
        if [ ! -e "$cand" ]; then
          printf 'dead doc-link: %s -> (%s) target missing from repo root\n' "$file" "$cand"
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
