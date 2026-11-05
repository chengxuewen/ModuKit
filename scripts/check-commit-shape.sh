#!/usr/bin/env bash
# C3 commit-shape gate (PIT-8 lineage; baseline derivation per adjudication #27).
# Rules for every commit in the pending range: subject <=60 chars, body lines are
# ' - ' bullets <=100 chars, one physical line per bullet.
#
# The baseline is DERIVED, never hardcoded: MODUKIT_C3_BASE (explicit ref) wins,
# otherwise the branch's own upstream (@{u}). With no resolvable baseline the leg
# prints a loud WARN and exits 0 -- the silent pass it replaces was a real bug:
# the hardcoded `origin/master` plus `2>/dev/null` made the leg green in any clone
# that lacks that ref (same class MediaServo records for its wire-changelog gate).
#
# --self-test builds a scratch repo: an over-long subject reddens, a clean subject
# greens (C0: the gate is observed failing once before it is trusted).
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
# Target repo for the git queries. The self-test points it at a scratch repo;
# without this the script's own `cd` would aim every query back at the real tree.
TARGET="${COMMIT_SHAPE_REPO:-$ROOT}"

shape_scan() {
  awk '
  function isbullet(l) { return substr(l,1,3)==" - " && length(l)<=100 }
  /^Subject: /{ subj=substr($0,10); if (length(subj)>60) print "SUBJ-VIOL(" length(subj) "): " substr(subj,1,40); inbody=0; next }
  /^Body: /{ inbody=1; b=substr($0,7); if (b=="") next; if (!isbullet(b)) print "BODY-VIOL: " substr(b,1,40); next }
  /^_END_$/{ inbody=0; next }
  inbody==1 { if ($0!="" && !isbullet($0)) print "BODY-VIOL: " substr($0,1,40) }
  '
}

resolve_baseline() {
  local base="${MODUKIT_C3_BASE:-}"
  if [ -n "$base" ]; then
    if ! git -C "$TARGET" rev-parse --verify "$base" >/dev/null 2>&1; then
      echo "COMMIT-SHAPE ✗ MODUKIT_C3_BASE='$base' does not resolve" >&2
      return 2
    fi
    printf '%s\n' "$base"
    return 0
  fi
  base=$(git -C "$TARGET" rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)
  if [ -z "$base" ]; then
    return 3
  fi
  printf '%s\n' "$base"
}

if [ "${1:-}" = "--self-test" ]; then
  tmp=$(mktemp -d "$ROOT/out/commit-shape.XXXXXX") || {
    echo "COMMIT-SHAPE ✗ self-test: cannot create a temp dir under $ROOT/out"
    exit 1
  }
  git init -q "$tmp/repo" || { echo "COMMIT-SHAPE ✗ self-test: git init failed"; rm -rf "$tmp"; exit 1; }
  (
    cd "$tmp/repo"
    git config user.email "gate@example.invalid"
    git config user.name "commit-shape self-test"
    echo seed >seed.txt
    git add seed.txt
    git commit -q -m 'docs: seed a clean subject under the ceiling'
    git commit -q --allow-empty -m 'feat(scope): this subject is deliberately far beyond the sixty character ceiling for C3'
  )
  if (cd "$tmp/repo" && COMMIT_SHAPE_REPO="$tmp/repo" MODUKIT_C3_BASE=HEAD~1 bash "$ROOT/scripts/check-commit-shape.sh") >/dev/null 2>&1; then
    echo "COMMIT-SHAPE ✗ self-test: planted over-long subject did not red"
    rm -rf "$tmp"
    exit 1
  fi
  if ! (cd "$tmp/repo" && git commit -q --amend --allow-empty -m 'docs: replanted subject sits under the ceiling' && COMMIT_SHAPE_REPO="$tmp/repo" MODUKIT_C3_BASE=HEAD~1 bash "$ROOT/scripts/check-commit-shape.sh") >/dev/null 2>&1; then
    echo "COMMIT-SHAPE ✗ self-test: clean range did not green"
    rm -rf "$tmp"
    exit 1
  fi
  rm -rf "$tmp"
  echo "COMMIT-SHAPE ✓ self-test (planted subject red, clean range green)"
  exit 0
fi

baseline=$(resolve_baseline)
rc=$?
if [ "$rc" -eq 3 ]; then
  echo "COMMIT-SHAPE ! WARN: branch '$(git -C "$TARGET" rev-parse --abbrev-ref HEAD 2>/dev/null || echo detached)' has no upstream -- the pending range is NOT checked (set MODUKIT_C3_BASE=<ref> to force a baseline)"
  exit 0
fi
if [ "$rc" -ne 0 ]; then
  exit 1
fi
violations=$(git -C "$TARGET" log "$baseline..HEAD" --format='Subject: %s%nBody: %b%n_END_' | shape_scan)
if [ -n "$violations" ]; then
  printf '%s\n' "$violations" | while IFS= read -r line; do
    echo "COMMIT-SHAPE ✗ $line"
  done
  exit 1
fi
echo "COMMIT-SHAPE ✓ ($(git -C "$TARGET" rev-list --count "$baseline..HEAD") pending commit(s) vs $baseline)"
