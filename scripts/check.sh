#!/usr/bin/env bash
# D50 local check gate: five fast checks, each skipping cleanly when its target is
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
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat pixi.toml CMakeLists.txt cmake/ \
  docs/ .agents/AGENTS.md .agents/memorys/ .agents/rules/ scripts/ \
  $(test -d crates/ && echo crates/); then
  note "FAIL C1 (violations listed above)"; status=1
else note "ok C1"; fi

# 5. skill-tree lint
if command -v python3 >/dev/null 2>&1 && [ -f scripts/skill-lint.py ]; then
  python3 scripts/skill-lint.py || { note "FAIL skill-lint"; status=1; }
else note "skip skill-lint (python3 or script absent)"; fi

if [ "$status" -eq 0 ]; then note "check.sh: all green"; else note "check.sh: RED"; fi
exit "$status"
