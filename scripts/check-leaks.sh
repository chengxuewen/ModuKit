#!/usr/bin/env bash
# leg: leak-liveness (repo-audit 2C P1, MediaServo check-leaks.sh pattern, 2026-11-23).
# A tracked file that an ignore rule covers is a leak the status line will never
# report: `git add -f` won it in, a later ignore rule hid it. The invariant is
# that the intersection of `git ls-files` and the active ignore rules stays
# empty. The credential-filename guard checks name shapes; this one checks
# tracking state - the two are complementary, neither covers the other.
# Injected self-test (C0): plant a force-added file under the ignored out/
# territory, expect RED, clean up, expect GREEN. The plant touches the real
# index for the duration of the assertion and is always reverted.
set -uo pipefail
SELF=$(realpath "$0")
cd "$(dirname "$0")/.."

if [ "${1:-}" = "--self-test" ]; then
  planted="out/leak-self-test-fixture.txt"
  cleanup() {
    git rm --cached -q "$planted" 2>/dev/null || true
    rm -f "$planted"
  }
  trap cleanup EXIT
  mkdir -p out
  printf 'leak fixture - name form is irrelevant, tracking state is the point\n' > "$planted"
  git add -f "$planted"
  if "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-leaks.sh): planted tracked-ignored leak NOT caught"
    exit 1
  fi
  cleanup
  trap - EXIT
  if ! "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-leaks.sh): clean tree went RED"
    exit 1
  fi
  echo "self-test OK (check-leaks.sh): planted leak red, clean tree green"
  exit 0
fi

leaks=$(git ls-files -ci --exclude-standard || true)
if [ -n "$leaks" ]; then
  printf '%s\n' "$leaks"
  echo "check-leaks: RED ($(printf '%s\n' "$leaks" | wc -l) tracked file(s) covered by ignore rules)"
  exit 1
fi
echo "check-leaks: ok (tracked-intersect-ignored intersection is empty)"
