#!/usr/bin/env bash
# D50 local check gate: six fast checks, each skipping cleanly when its target is
# absent (zero-source repo = green; total runtime seconds). CI at the Stage-1 revival
# trigger runs THIS SAME script - zero drift between local and CI.
# absent (zero-source repo = green; total runtime seconds). CI at the Stage-1 revival
# trigger runs THIS SAME script - zero drift between local and CI.
set -uo pipefail
cd "$(dirname "$0")/.."
status=0
note() { printf '%s\n' "$*"; }

# Toolchain routing per D65: when pixi.toml exists the cargo legs run in the pixi
# build env (mise's cargo shim is refused by design - mise owns host tools only).
HAS_WS=0
CARGO=(cargo)
if [ -d crates ]; then
  HAS_WS=1
  if [ -f pixi.toml ]; then
    export PATH="$HOME/.pixi/bin:$PATH"
    command -v pixi >/dev/null 2>&1 && CARGO=(pixi run cargo)
  fi
fi

# 1. cargo fmt --check
if [ "$HAS_WS" -eq 1 ]; then
  "${CARGO[@]}" fmt --all --check || { note "FAIL fmt"; status=1; }
else note "skip fmt (no crates/ workspace)"; fi

# 2. clippy -D warnings
if [ "$HAS_WS" -eq 1 ]; then
  "${CARGO[@]}" clippy --workspace -- -D warnings || { note "FAIL clippy"; status=1; }
else note "skip clippy (no crates/ workspace)"; fi

# 3. cargo test
if [ "$HAS_WS" -eq 1 ]; then
  "${CARGO[@]}" test --workspace || { note "FAIL test"; status=1; }
else note "skip test (no crates/ workspace)"; fi

# 4. C1 English-artifact gate (canonical pattern+scope verbatim from .agents/memorys/conventions.md;
#    verdict echo intentionally wrapped into note() helpers - semantics identical; audit F14 note)
if grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ contracts/ examples/ \
  docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ \
  $(test -d crates/ && echo crates/); then
  note "FAIL C1 (violations listed above)"; status=1
else note "ok C1"; fi

# 5. skill-tree lint
if command -v python3 >/dev/null 2>&1 && [ -f scripts/skill-lint.py ]; then
  python3 scripts/skill-lint.py || { note "FAIL skill-lint"; status=1; }
else note "skip skill-lint (python3 or script absent)"; fi

# 6. contract ledger regen-diff (D12, modules/04 §2): re-run protoc over
#    contracts/*.proto and require every checked-in .fdset to match byte for
#    byte - empty diff or red. Skip-when-absent means "no ledger in this
#    repo"; a ledger with an unresolvable protoc is a FAIL, not a silent
#    skip (an unfalsifiable gate is decoration per C0).
if [ -d contracts ] && [ -f scripts/regen-fdset.sh ]; then
  fd_tmp=$(mktemp -d)
  if bash scripts/regen-fdset.sh "$fd_tmp" >/dev/null 2>&1; then
    drift=0
    for fresh in "$fd_tmp"/*.proto.fdset; do
      base=$(basename "$fresh")
      cmp -s "$fresh" "contracts/$base" || { note "FAIL regen-diff ($base drifted or is missing from contracts/)"; drift=1; }
    done
    for checked in contracts/*.proto.fdset; do
      base=$(basename "$checked")
      [ -f "$fd_tmp/$base" ] || { note "FAIL regen-diff ($base checked in but no contracts proto regenerates it)"; drift=1; }
    done
    if [ "$drift" -eq 0 ]; then
      note "ok regen-diff ($(ls "$fd_tmp"/*.proto.fdset | wc -l) fdsets match protoc)"
    else
      status=1
    fi
  else
    note "FAIL regen-diff (protoc unresolvable: no pinned-env pixi, no system protoc)"; status=1
  fi
  rm -rf "$fd_tmp"
else note "skip regen-diff (no contracts/ ledger or regen script)"; fi

# leg: demo gates (D85-drift lesson 2026-10-18 - the drift hid for two
# days because demos were not in any gate). Skip-when-absent per D50;
# each demo is a PASS-line gate with its own falsifiability probe.
if [ -f scripts/demo-stage1.sh ]; then
  if bash scripts/demo-stage1.sh 2>/dev/null | grep -q "demo-stage1: PASS"; then note "ok demo-stage1"; else note "FAIL demo-stage1 (gate did not print PASS)"; status=1; fi
else note "skip demo-stage1 (no script)"; fi
if [ -f scripts/demo-stage2.sh ]; then
  if bash scripts/demo-stage2.sh 2>/dev/null | grep -q "demo-stage2: PASS"; then note "ok demo-stage2"; else note "FAIL demo-stage2 (gate did not print PASS)"; status=1; fi
else note "skip demo-stage2 (no script)"; fi
# Stage-4 M1 embed demo leg: SKIP (cpp band auto-skipped) counts as green
# like the D50 skip-when-absent legs; RED (build/sabotage) fails the gate.
if [ -f scripts/demo-stage4-m1.sh ]; then
  s4=$(bash scripts/demo-stage4-m1.sh 2>/dev/null)
  if echo "$s4" | grep -q "demo-stage4-m1: PASS"; then note "ok demo-stage4-m1";
  elif echo "$s4" | grep -q "demo-stage4-m1: SKIP"; then note "skip demo-stage4-m1 (cpp band auto-skipped)";
  else note "FAIL demo-stage4-m1 (gate did not print PASS)"; status=1; fi
else note "skip demo-stage4-m1 (no script)"; fi
if [ -f scripts/demo-stage4-m2.sh ]; then
  s4b=$(bash scripts/demo-stage4-m2.sh 2>/dev/null)
  if echo "$s4b" | grep -q "demo-stage4-m2: PASS"; then note "ok demo-stage4-m2";
  elif echo "$s4b" | grep -q "demo-stage4-m2: SKIP"; then note "skip demo-stage4-m2 (cpp band auto-skipped)";
  else note "FAIL demo-stage4-m2 (gate did not print PASS)"; status=1; fi
else note "skip demo-stage4-m2 (no script)"; fi
# Stage-5 N2 cross-wall demo leg (D96 ruling 6): same PASS/SKIP grammar as
# the stage4 legs - SKIP (no build toolchain at all) is green, RED fails.
if [ -f scripts/demo-stage5-n2.sh ]; then
  s5=$(bash scripts/demo-stage5-n2.sh 2>/dev/null)
  if echo "$s5" | grep -q "demo-stage5-n2: PASS"; then note "ok demo-stage5-n2";
  elif echo "$s5" | grep -q "demo-stage5-n2: SKIP"; then note "skip demo-stage5-n2 (no build toolchain)";
  else note "FAIL demo-stage5-n2 (gate did not print PASS)"; status=1; fi
else note "skip demo-stage5-n2 (no script)"; fi
# Wave-2 reader leg (closes the D97 re-review trigger): same PASS/SKIP grammar
# as the other demo legs - SKIP (no build toolchain) is green, RED fails.
if [ -f scripts/demo-wave2-reader.sh ]; then
  w2=$(bash scripts/demo-wave2-reader.sh 2>/dev/null)
  if echo "$w2" | grep -q "demo-wave2-reader: PASS"; then note "ok demo-wave2-reader";
  elif echo "$w2" | grep -q "demo-wave2-reader: SKIP"; then note "skip demo-wave2-reader (no build toolchain)";
  else note "FAIL demo-wave2-reader (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-reader (no script)"; fi

# leg: ctest bridge over the dedicated Stage-4 tree (gate escalation 2026-10-21;
# PIT-17 class - a warm stale tree once fabricated a false 9/9). Incremental rebuild
# THEN full ctest every run, so changed sources cannot hide behind old binaries.
# Skip-when-absent like the D50 legs: no tree (fresh clone) = green, no cost.
if [ -d out/build-stage4 ] && command -v pixi >/dev/null 2>&1; then
  if pixi run cmake --build out/build-stage4 > /tmp/ctest-build.log 2>&1 \
     && pixi run ctest --test-dir out/build-stage4 > /tmp/ctest-run.log 2>&1; then
    note "ok ctest-stage4 ($(grep -E 'tests passed' /tmp/ctest-run.log | tail -1))"
  else
    note "FAIL ctest-stage4 (build or tests red; see /tmp/ctest-build.log + /tmp/ctest-run.log)"; status=1
  fi
else note "skip ctest-stage4 (no out/build-stage4 tree or no pixi)"; fi

# leg: embed-contract parity (Stage-4 M1 lane #2, C++-3): the header IS the
# contract; the Rust mirror must list its verbs exactly. Skip-when-absent
# per D50; present, it is always-on like doc-liveness.
bash scripts/check-embed-parity.sh || { note "FAIL embed-parity"; status=1; }

# leg: capability ledger (Stage-4 M1 lane #7, plan §4 gate adoption note):
# every EMBED_VERBS verb has a row in docs/capability-ledger.md; unshipped
# languages ride the planned column, never a red condition. Always-on like
# embed-parity; row-grammar contract documented in both files.
bash scripts/check-capability-ledger.sh || { note "FAIL capability-ledger"; status=1; }

# leg: C3 commit-shape over the pending (unpushed) range (machine gate, PIT-8)
# scope note: the 56 pushed legacy subjects (48 pre-C3 + 8 Oct-16 gap; recounted 2026-10-19) subjects stay RED in any wider window by design
# - pushed history is immutable (reword covered the pre-push 24 only); the gate owns new work
c3_shape=$(git log origin/master..HEAD --format='Subject: %s%nBody: %b%n_END_' 2>/dev/null | awk '
  function isbullet(l) { return substr(l,1,3)==" - " && length(l)<=102 }
  /^Subject: /{ subj=substr($0,10); if (length(subj)>60) print "SUBJ-VIOL(" length(subj) "): " substr(subj,1,40); inbody=0; next }
  /^Body: /{ inbody=1; b=substr($0,7); if (b=="") next; if (!isbullet(b)) print "BODY-VIOL: " substr(b,1,40); next }
  /^_END_$/{ inbody=0; next }
  inbody==1 { if ($0!="" && !isbullet($0)) print "BODY-VIOL: " substr($0,1,40) }
')
if [ -n "$c3_shape" ]; then printf '%s\n' "$c3_shape" | while IFS= read -r l; do note "FAIL commit-shape: $l"; done; status=1
else note "ok commit-shape (pending range)"; fi

# leg: doc-liveness (M3S-1 c1 - PORT_NOW survivor, D-ruling 2026-10-19). Always-on once
# landed: it is a gate of this repo, not an optional tool - no skip-when-absent branch.
bash scripts/check-doc-refs.sh || { note "FAIL doc-liveness"; status=1; }

if [ "$status" -eq 0 ]; then note "check.sh: all green"; else note "check.sh: RED"; fi
exit "$status"
