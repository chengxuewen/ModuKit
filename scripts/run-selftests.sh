#!/usr/bin/env bash
# selftests runner (hearing #41 / backlog P15; agents-engineering-plan B1).
# Replays every in-family --self-test in ONE leg so each planted-defect red
# is observed on every gate run, not once at landing. Explicit list =
# explicit contract: a listed script missing from the tree FAILS the leg
# (never a silent skip); check-memory.sh stays OFF by design (it lives
# outside check.sh as the C5 batch-close gate). Per-entry args (panel O1):
# fourteen take --self-test, check-memory-ownership.py takes the positional
# self-test (its pixi.toml invocation shape) - the list stores the arg per
# entry, no alias surface added.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
fail=0

run() { # run <python3|bash> <script> [args...]
  local kind="$1" script="$2"; shift 2
  if [ ! -f "$script" ]; then
    echo "FAIL selftest $script (missing from tree)"
    fail=1
    return
  fi
  out=$("$kind" "$script" "$@" 2>&1)
  if [ $? -eq 0 ]; then
    echo "ok selftest $(basename "$script")"
  else
    echo "FAIL selftest $(basename "$script")"
    printf '%s\n' "$out" | head -5
    fail=1
  fi
}

run bash scripts/gate-i18n.sh --self-test
run python3 scripts/agents/skill-lint.py --self-test
run bash scripts/check-msrv.sh --self-test
run python3 scripts/book-check.py --self-test
run python3 scripts/agents/rule-ref-lint.py --self-test
run bash scripts/check-commit-shape.sh --self-test
run bash scripts/check-capability-ledger.sh --self-test
run bash scripts/check-numbering.sh --self-test
run bash scripts/check-doc-refs.sh --self-test
run bash scripts/check-embed-parity.sh --self-test
run bash scripts/check-lane-table.sh --self-test
run bash scripts/check-promises.sh --self-test
run bash scripts/check-wire-parity.sh --self-test
run bash scripts/gate-abi.sh --self-test
run python3 scripts/agents/check-memory-ownership.py self-test
run python3 scripts/check-no-abs-paths.py --self-test

if [ "$fail" -eq 0 ]; then
  echo "SELFTESTS ✓ (16 in-family red replays green)"
fi
exit "$fail"
