#!/usr/bin/env bash
# D50 local check gate: five fast checks, each skipping cleanly when its target is
# absent (zero-source repo = green; total runtime seconds). CI at the Stage-1 revival
# trigger runs THIS SAME script - zero drift between local and CI.
set -uo pipefail
cd "$(dirname "$0")/.."
status=0
note() { printf '%s\n' "$*"; }

HAS_WS=0
[ -d crates ] && command -v cargo >/dev/null 2>&1 && HAS_WS=1

# 1. cargo fmt --check
if [ "$HAS_WS" -eq 1 ]; then
  cargo fmt --all --check || { note "FAIL fmt"; status=1; }
else note "skip fmt (no crates/ workspace)"; fi

# 2. clippy -D warnings
if [ "$HAS_WS" -eq 1 ]; then
  cargo clippy --workspace -- -D warnings || { note "FAIL clippy"; status=1; }
else note "skip clippy (no crates/ workspace)"; fi

# 3. cargo test
if [ "$HAS_WS" -eq 1 ]; then
  cargo test --workspace || { note "FAIL test"; status=1; }
else note "skip test (no crates/ workspace)"; fi

# 4. C1 English-artifact gate (canonical scope, verbatim from .agents/memorys/conventions.md)
if grep -rInP '[\x{4E00}-\x{9FFF}\x{3000}-\x{303F}\x{FF01}-\x{FF60}]' \
  AGENTS.md SKILL.md README.md LICENSE-MIT LICENSE-APACHE mise.toml bootstrap.sh bootstrap.bat \
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
