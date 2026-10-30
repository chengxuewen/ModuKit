#!/usr/bin/env bash
# check-promises: conservative overclaim lint for user-facing docs.
# It flags a small set of future-tense phrases in README/whitepaper/architecture;
# deliberately narrow so the gate has no false positives. Extend the pattern only
# when a real overclaim class is observed.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
FILES=(README.md docs/whitepaper.md docs/architecture.md)
PATTERN='\b(will ship|will land|coming soon|not yet implemented|future release)\b'

if [ "${1:-}" = "--self-test" ]; then
  mkdir -p out
  probe=out/overclaim-probe.md
  printf 'This feature will ship in a future release.\n' > "$probe"
  if grep -qiE "$PATTERN" "$probe"; then
    rm -f "$probe"
    echo "CHECK-PROMISES ✓ self-test (planted phrase matched)"
    exit 0
  fi
  rm -f "$probe"
  echo "CHECK-PROMISES ✗ self-test did not match"
  exit 1
fi

hits=$(grep -nEi "$PATTERN" "${FILES[@]}" 2>/dev/null || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  echo "CHECK-PROMISES ✗ future-tense overclaim in a user-facing doc"
  exit 1
fi
echo "CHECK-PROMISES ✓ no overclaim phrases in user-facing docs"
