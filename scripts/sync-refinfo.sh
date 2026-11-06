#!/usr/bin/env bash
# refinfo donor sync (adjudication #31; grammar from MediaServo's
# sync-official-refs.sh). Tracked donors = official upstream with a LIVE
# borrow/dependency claim in docs/reference; each line prints the fresh SHA -
# the PIT-35 evidence to paste into the citing profile or plan row.
# Excluded by ruling: remote_control_host_computer is a DOWNSTREAM port of
# our own governance tree, not upstream (MediaServo's no-third-party-fork
# discipline, applied to ourselves). One-shot donors (CTK sliver, AUDESYS
# prior art, ark - no standing profile claim) stay frozen at their recorded
# pins. Nothing here is silent: HELD/FAIL/SYNC are each printed per donor;
# a dirty or diverged clone FAILs and reddens the run (never auto-stash).
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT="${REFINFO_ROOT:-.refinfo}"
TRACKED=(VisiaEngine MediaServo iceoryx2 zenoh CppMicroServices celix dora AccessBase)

is_tracked() {
  local n
  for n in "${TRACKED[@]}"; do [ "$n" = "$1" ] && return 0; done
  return 1
}

if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d "${PWD}/out/sync-refinfo.XXXXXX") || { echo "REFINFO-SYNC ! self-test: no temp under out/"; exit 1; }
  trap 'rm -rf "$s"' EXIT
  git init -q --bare "$s/up.git"
  git init -q "$s/work"
  ( cd "$s/work" && echo seed > seed.txt && git -c user.email=g@e.invalid -c user.name=t add seed.txt && \
    git -c user.email=g@e.invalid -c user.name=t commit -q -m seed && \
    git remote add origin "$s/up.git" && git push -q origin HEAD )
  git clone -q "$s/up.git" "$s/donor/VisiaEngine" 2>/dev/null
  git clone -q "$s/up.git" "$s/donor/MediaServo" 2>/dev/null
  echo change >> "$s/donor/MediaServo/seed.txt"
  git clone -q "$s/up.git" "$s/donor/CTK" 2>/dev/null
  out=$(REFINFO_ROOT="$s/donor" bash "$0" 2>&1) || true
  echo "$out" | grep -q "SYNC VisiaEngine" || { echo "REFINFO-SYNC ! self-test: clean tracked donor did not SYNC"; exit 1; }
  echo "$out" | grep -q "FAIL MediaServo" || { echo "REFINFO-SYNC ! self-test: dirty tracked donor did not FAIL"; exit 1; }
  echo "$out" | grep -q "HELD CTK" || { echo "REFINFO-SYNC ! self-test: one-shot donor did not HELD"; exit 1; }
  echo "REFINFO-SYNC self-test OK (clean SYNC, dirty FAIL, untracked HELD)"
  exit 0
fi

status=0
for d in "$ROOT"/*/; do
  [ -d "$d" ] || continue
  name=$(basename "$d")
  if ! is_tracked "$name"; then
    echo "refinfo: HELD $name (one-shot snapshot or downstream - not tracked)"
    continue
  fi
  if ! git -C "$d" rev-parse -q --verify HEAD >/dev/null 2>&1; then
    echo "refinfo: FAIL $name (no git HEAD - re-clone it manually)"; status=1; continue
  fi
  if [ -n "$(git -C "$d" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    # tracked changes or gitlink drift only - untracked droppings (?? files)
    # never block --ff-only and are the donor's own business.
    echo "refinfo: FAIL $name (tracked changes or gitlink drift - resolve before syncing)"; status=1; continue
  fi
  if ! git -C "$d" pull --ff-only -q 2>/dev/null; then
    echo "refinfo: FAIL $name (pull --ff-only refused: no remote, offline, or diverged)"; status=1; continue
  fi
  echo "refinfo: SYNC $name $(git -C "$d" log -1 --format='%h %cd' --date=short)"
done
[ "$status" -eq 0 ] && echo "REFINFO-SYNC ✓ (tracked: ${#TRACKED[@]}, see HELD/FAIL lines above)"
exit "$status"
