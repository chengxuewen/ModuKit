#!/usr/bin/env bash
# D50 local check gate: fast legs + THIRTEEN demo legs (newest leg thirteen:
# wave2-plugin-js, D106 batch 3; leg twelve = wave2-plugin-py, D106 batch 2)
# + ctest per-name census +
# lane-table + wire-parity + loud cargo-deny + commit-shape + doc-liveness (D104/D106 named);
# each leg skips cleanly when its target is absent. CI at the revival trigger runs
# THIS SAME script - zero drift between local and CI.
set -uo pipefail
cd "$(dirname "$0")/.."
status=0
note() { printf '%s\n' "$*"; }

# --list: the single source for CONSTRAINTS.md (one leg per line). Keep this list
# in step with the legs below; the constraints gate compares counts only.
if [ "${1:-}" = "--list" ]; then
  cat <<'EOF'
fmt
clippy
test
gate-i18n
skill-lint
regen-fdset
demo-stage1
demo-stage2
demo-stage4-m1
demo-stage4-m2
demo-stage5-n2
demo-wave2-reader
demo-wave2-clock
demo-wave2-py
demo-wave2-py-sup
demo-wave2-js
demo-wave2-js-sup
demo-wave2-plugin-py
demo-wave2-plugin-js
cargo-deny
ctest-stage4
embed-parity
gate-abi
capability-ledger
gate-agents
commit-shape
gate-docs
check-promises
numbering
lane-table
wire-parity
gate-mcp
EOF
  exit 0
fi

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

# 4. artifact-language gate (C1: tracked + untracked, whitelist in the script)
if [ -f scripts/gate-i18n.sh ]; then
  bash scripts/gate-i18n.sh || { note "FAIL gate-i18n"; status=1; }
else note "skip gate-i18n (no script)"; fi

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
# Wave-2 clock-crossing leg (T2, D99 R5-A): same PASS/SKIP grammar as the
# other six demo legs - SKIP (no build toolchain) is green, RED fails.
if [ -f scripts/demo-wave2-clock.sh ]; then
  w2c=$(bash scripts/demo-wave2-clock.sh 2>/dev/null)
  if echo "$w2c" | grep -q "demo-wave2-clock: PASS"; then note "ok demo-wave2-clock";
  elif echo "$w2c" | grep -q "demo-wave2-clock: SKIP"; then note "skip demo-wave2-clock (no build toolchain)";
  else note "FAIL demo-wave2-clock (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-clock (no script)"; fi
# Wave-2 python embed leg (D100 M1, the eighth demo leg): same PASS/SKIP
# grammar as the other seven demo legs - SKIP (no python3 / toolchain /
# pinned protoc / cpp fixture band) is green, RED fails.
if [ -f scripts/demo-wave2-py.sh ]; then
  w2p=$(bash scripts/demo-wave2-py.sh 2>/dev/null)
  if echo "$w2p" | grep -q "demo-wave2-py: PASS" && echo "$w2p" | grep -q "^RUNNER: "; then note "ok demo-wave2-py";
  elif echo "$w2p" | grep -q "demo-wave2-py: PASS"; then note "FAIL demo-wave2-py (PASS without RUNNER line - T3.1 runner census)"; status=1;
  elif echo "$w2p" | grep -q "demo-wave2-py: SKIP"; then note "skip demo-wave2-py (no python3/toolchain/fixture)";
  else note "FAIL demo-wave2-py (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-py (no script)"; fi
# Wave-2 python sup-rung leg (D100 M2, the ninth demo leg): same PASS/SKIP
# grammar as the other eight demo legs - SKIP (no python3 / toolchain /
# pinned protoc / cpp fixture band) is green, RED fails.
if [ -f scripts/demo-wave2-py-sup.sh ]; then
  w2ps=$(bash scripts/demo-wave2-py-sup.sh 2>/dev/null)
  if echo "$w2ps" | grep -q "demo-wave2-py-sup: PASS" && echo "$w2ps" | grep -q "^RUNNER: "; then note "ok demo-wave2-py-sup";
  elif echo "$w2ps" | grep -q "demo-wave2-py-sup: PASS"; then note "FAIL demo-wave2-py-sup (PASS without RUNNER line - T3.1 runner census)"; status=1;
  elif echo "$w2ps" | grep -q "demo-wave2-py-sup: SKIP"; then note "skip demo-wave2-py-sup (no python3/toolchain/fixture)";
  else note "FAIL demo-wave2-py-sup (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-py-sup (no script)"; fi
# Wave-2 JS embed leg (D102 M1, the tenth demo leg): same PASS/SKIP grammar
# as the other nine demo legs - SKIP (no node / toolchain / pinned protoc /
# cpp fixture band) is green, RED fails.
if [ -f scripts/demo-wave2-js.sh ]; then
  w2j=$(bash scripts/demo-wave2-js.sh 2>/dev/null)
  if echo "$w2j" | grep -q "demo-wave2-js: PASS" && echo "$w2j" | grep -q "^RUNNER: "; then note "ok demo-wave2-js";
  elif echo "$w2j" | grep -q "demo-wave2-js: PASS"; then note "FAIL demo-wave2-js (PASS without RUNNER line - T3.1 runner census)"; status=1;
  elif echo "$w2j" | grep -q "demo-wave2-js: SKIP"; then note "skip demo-wave2-js (no node/toolchain/fixture)";
  else note "FAIL demo-wave2-js (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-js (no script)"; fi
# Wave-2 JS sup-rung leg (D102 M2, the eleventh demo leg): same PASS/SKIP
# grammar as the other ten demo legs - SKIP (no node / toolchain / pinned
# protoc / cpp fixture band) is green, RED fails.
if [ -f scripts/demo-wave2-js-sup.sh ]; then
  w2js=$(bash scripts/demo-wave2-js-sup.sh 2>/dev/null)
  if echo "$w2js" | grep -q "demo-wave2-js-sup: PASS" && echo "$w2js" | grep -q "^RUNNER: "; then note "ok demo-wave2-js-sup";
  elif echo "$w2js" | grep -q "demo-wave2-js-sup: PASS"; then note "FAIL demo-wave2-js-sup (PASS without RUNNER line - T3.1 runner census)"; status=1;
  elif echo "$w2js" | grep -q "demo-wave2-js-sup: SKIP"; then note "skip demo-wave2-js-sup (no node/toolchain/fixture)";
  else note "FAIL demo-wave2-js-sup (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-js-sup (no script)"; fi
# Wave-2 plugin-door python leg (D106 batch 2, the twelfth demo leg): the
# hand-written skin walks all 11 wire kinds across the exe wall (conformance
# + lifecycle + FIDELITY + BUS ROUNDTRIP + liar typed error + orphan census,
# sabotage lever DEMO_PLUGIN_PY_SABOTAGE=1). Same PASS/SKIP grammar as the
# other eleven demo legs; the PASS-without-RUNNER FAIL arm below is copied
# from the D104-census template (scripts/check.sh:130-135 pattern - the
# runner-origin line is the census proof, its absence reds the leg).
if [ -f scripts/demo-wave2-plugin-py.sh ]; then
  w2pp=$(bash scripts/demo-wave2-plugin-py.sh 2>/dev/null)
  if echo "$w2pp" | grep -q "demo-wave2-plugin-py: PASS" && echo "$w2pp" | grep -q "^RUNNER: "; then note "ok demo-wave2-plugin-py";
  elif echo "$w2pp" | grep -q "demo-wave2-plugin-py: PASS"; then note "FAIL demo-wave2-plugin-py (PASS without RUNNER line - D104-census)"; status=1;
  elif echo "$w2pp" | grep -q "demo-wave2-plugin-py: SKIP"; then note "skip demo-wave2-plugin-py (no python3/toolchain/fixture)";
  else note "FAIL demo-wave2-plugin-py (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-plugin-py (no script)"; fi

# Wave-2 plugin-door node leg (D106 batch 3, the thirteenth demo leg): the
# hand-written ESM skin walks all 11 wire kinds across the exe wall (same
# conformance + lifecycle + FIDELITY + BUS ROUNDTRIP + liar + census grammar
# as leg twelve, sabotage lever DEMO_PLUGIN_JS_SABOTAGE=1). PASS-without-RUNNER
# arm copied from the D104-census template, like its python twin above.
if [ -f scripts/demo-wave2-plugin-js.sh ]; then
  w2pj=$(bash scripts/demo-wave2-plugin-js.sh 2>/dev/null)
  if echo "$w2pj" | grep -q "demo-wave2-plugin-js: PASS" && echo "$w2pj" | grep -q "^RUNNER: "; then note "ok demo-wave2-plugin-js";
  elif echo "$w2pj" | grep -q "demo-wave2-plugin-js: PASS"; then note "FAIL demo-wave2-plugin-js (PASS without RUNNER line - D104-census)"; status=1;
  elif echo "$w2pj" | grep -q "demo-wave2-plugin-js: SKIP"; then note "skip demo-wave2-plugin-js (no node/toolchain/fixture)";
  else note "FAIL demo-wave2-plugin-js (gate did not print PASS)"; status=1; fi
else note "skip demo-wave2-plugin-js (no script)"; fi
# leg: cargo-deny (wave-2 benchmark plan T3.4): license+bans audit against
# deny.toml. Tool present -> run it; absent (this offline box cannot install
# it) -> loud skip, never a silent one.
if [ "$HAS_WS" -eq 1 ] && [ -f deny.toml ]; then
  if "${CARGO[@]}" deny --version >/dev/null 2>&1; then
    "${CARGO[@]}" deny check licenses bans || { note "FAIL cargo-deny"; status=1; }
  else
    note "skip cargo-deny (tool not installable offline)"
  fi
else note "skip cargo-deny (no deny.toml or no workspace)"; fi

# leg: ctest bridge over the dedicated Stage-4 tree (gate escalation 2026-10-21;
# PIT-17 class - a warm stale tree once fabricated a false 9/9). Incremental rebuild
# THEN full ctest every run, so changed sources cannot hide behind old binaries.
# Skip-when-absent like the D50 legs: no tree (fresh clone) = green, no cost.
if [ -d out/build-stage4 ] && command -v pixi >/dev/null 2>&1; then
  if ! pixi run cmake --build out/build-stage4 > /tmp/ctest-build.log 2>&1; then
    note "FAIL ctest-stage4 (build red; see /tmp/ctest-build.log)"; status=1
  else
    # census (benchmark batch 1 / D104): since the CTEST keyword mints the six
    # rust-side entries, a dropped keyword would SHRINK the list while plain
    # ctest stays green on the survivors - so assert the NAMES, not just runs.
    pixi run ctest --test-dir out/build-stage4 -N > /tmp/ctest-census.log 2>&1 || true
    census_missing=""
    for _t in modukit-core-test modukit-transport-test modukit-codegen-test \
              example-plugin-test bad-plugin-test example_stage1_demo; do
      grep -Eq "Test #[0-9]+: ${_t}( |$)" /tmp/ctest-census.log \
        || census_missing="${census_missing} ${_t}"
    done
    if [ -n "$census_missing" ]; then
      note "FAIL ctest-stage4 census (missing:${census_missing})"; status=1
    elif pixi run ctest --test-dir out/build-stage4 --output-on-failure > /tmp/ctest-run.log 2>&1; then
      note "ok ctest-stage4 ($(grep -E 'tests passed' /tmp/ctest-run.log | tail -1))"
    else
      cp -f out/build-stage4/Testing/Temporary/LastTest.log /tmp/ctest-lasttest.log 2>/dev/null || true
      note "FAIL ctest-stage4 (tests red; see /tmp/ctest-build.log + /tmp/ctest-run.log + /tmp/ctest-lasttest.log)"; status=1
    fi
  fi
else note "skip ctest-stage4 (no out/build-stage4 tree or no pixi)"; fi

# leg: embed-contract parity (Stage-4 M1 lane #2, C++-3): the header IS the
# contract; the Rust mirror must list its verbs exactly. Skip-when-absent
# per D50; present, it is always-on like doc-liveness.
bash scripts/check-embed-parity.sh || { note "FAIL embed-parity"; status=1; }

# leg: gate-abi (B3): the core cdylib's exported modukit_* set equals
# contracts/modukit_embed.h (exact set; the C++ demo run is demo-stage4-m1).
if [ -f scripts/gate-abi.sh ]; then
  bash scripts/gate-abi.sh || { note "FAIL gate-abi"; status=1; }
else note "skip gate-abi (no script)"; fi

# leg: capability ledger (Stage-4 M1 lane #7, plan §4 gate adoption note):
# every EMBED_VERBS verb has a row in docs/capability-ledger.md; unshipped
# languages ride the planned column, never a red condition. Always-on like
# embed-parity; row-grammar contract documented in both files.
bash scripts/check-capability-ledger.sh || { note "FAIL capability-ledger"; status=1; }

# leg: gate-agents (B2/B3): generated layers match .agents (markers, hub,
# MCP, CONSTRAINTS). Uses the pixi task when available; plain python3 otherwise.
if [ -f scripts/agents-cli.py ]; then
  if command -v pixi >/dev/null 2>&1; then
    pixi run gate-agents || { note "FAIL gate-agents"; status=1; }
  else
    python3 scripts/agents-cli.py check || { note "FAIL gate-agents"; status=1; }
  fi
else note "skip gate-agents (no script)"; fi

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

# leg: gate-docs (B3): root CMake purity, PolyOrch gitlink, CLI task names,
# relative links (delegated to check-doc-refs.sh), book SUMMARY when built.
if [ -f scripts/gate-docs.sh ]; then
  bash scripts/gate-docs.sh || { note "FAIL gate-docs"; status=1; }
else note "skip gate-docs (no script)"; fi

# leg: check-promises (B3): narrow future-tense overclaim lint on user docs.
if [ -f scripts/check-promises.sh ]; then
  bash scripts/check-promises.sh || { note "FAIL check-promises"; status=1; }
else note "skip check-promises (no script)"; fi

# leg: numbering (B0, D109) - the ledger's max D is the only high-water source;
# every D1-D<n> claim and the architecture section-11 row must match it. The
# script's --self-test plants D<max+1> and expects red (C0).
if [ -f scripts/check-numbering.sh ]; then
  bash scripts/check-numbering.sh || { note "FAIL numbering"; status=1; }
else note "skip numbering (no script)"; fi

# leg: lane-table reconciliation (benchmark plan T2.3): the cmake lane rows and the
# cargo workspace members must agree - ghost lanes and unregistered crates/*
# members are red. Always-on like embed-parity; the gate ships in-repo.
bash scripts/check-lane-table.sh || { note "FAIL lane-table"; status=1; }

# leg: wire parity (wave-2 plugin door batch 1, D106 plan T1.3): goldens regeneration
# byte-diff + wire.h <-> proc_wire.rs constant cross-check; the --self-test ships
# inside the script (both plants, C0). Skips cleanly when the batch-1 shelf or
# pixi (cargo owner, D65) is absent; the script itself never touches repo files.
if [ -f contracts/modukit_wire.h ] && command -v pixi >/dev/null 2>&1; then
  bash scripts/check-wire-parity.sh || { note "FAIL wire-parity"; status=1; }
else note "skip wire-parity (no wire ledger header or no pixi)"; fi

# leg: gate-mcp (B1/B3): probe the pinned servers without installing anything.
if command -v pixi >/dev/null 2>&1; then
  pixi run gate-mcp || { note "FAIL gate-mcp"; status=1; }
else note "skip gate-mcp (no pixi)"; fi

if [ "$status" -eq 0 ]; then note "check.sh: all green"; else note "check.sh: RED"; fi
exit "$status"
