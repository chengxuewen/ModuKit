#!/usr/bin/env bash
# bootstrap.sh — one-time dev-environment setup for ModuKit (user-level, no sudo).
#
#   ./bootstrap.sh
#
# Idempotent. Steps:
#   1. ensure mise        (official installer -> ~/.local/bin if missing)
#   2. ensure shell activation (idempotent append to ~/.bashrc or ~/.zshrc)
#   3. mise install       (toolchain pins from mise.toml; no-op when satisfied)
#   4. ensure pixi        (delegated to PolyOrch/scripts/pixi-init.sh: pinned
#                         auditable installer, already-satisfied no-op, lock-drift retry)
#
# After first run: open a NEW terminal (activation is read at shell start).
# Windows native: use bootstrap.bat - stages mirrored (pixi delegation, git hooks,
#   mirror slot; hooks+slot added 2026-10-23 doc-audit). WINDOWS FIRST-RUN DEBT stands:
#   the bat has never been executed on Windows.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"
ROOT="$PWD"

# 1. locate or install mise ---------------------------------------------------
MISE="$(command -v mise 2>/dev/null || true)"
[ -n "$MISE" ] || [ ! -x "$HOME/.local/bin/mise" ] || MISE="$HOME/.local/bin/mise"
if [ -z "$MISE" ]; then
    echo "[bootstrap] mise not found — installing user-level via https://mise.run"
    curl -fsSL https://mise.run | sh
    MISE="$HOME/.local/bin/mise"
    [ -x "$MISE" ] || { echo "[bootstrap] ERROR: installer finished but $MISE is missing" >&2; exit 1; }
fi
echo "[bootstrap] mise: $("$MISE" --version)"

# 2. ensure activation in the user's shell rc ---------------------------------
case "$(basename "${SHELL:-bash}")" in
    zsh)  RC="$HOME/.zshrc";  SHELL_KIND=zsh ;;
    *)    RC="$HOME/.bashrc"; SHELL_KIND=bash ;;
esac
if ! grep -qF 'mise activate' "$RC" 2>/dev/null; then
    {
        echo 'export PATH="$HOME/.local/bin:$PATH"  # mise bin'
        echo "eval \"\$($MISE activate $SHELL_KIND)\"  # mise toolchain pins — repo pin: mise.toml"
    } >> "$RC"
    echo "[bootstrap] activation appended to $RC"
else
    echo "[bootstrap] activation already present in $RC"
fi

# 3. install the pinned toolchains --------------------------------------------
echo "[bootstrap] applying mise.toml pins (no-op if satisfied) ..."
"$MISE" install

# 4. submodule guard ------------------------------------------------------
#    The pixi bootstrap is supplied by PolyOrch's standard script; initialise
#    the submodule if it is not checked out, fail loudly when the script is
#    still missing (never a silent skip).
if [ ! -f PolyOrch/scripts/pixi-init.sh ]; then
    echo "[bootstrap] PolyOrch/scripts/pixi-init.sh missing - running git submodule update --init --recursive PolyOrch"
    git submodule update --init --recursive PolyOrch \
        || { echo "[bootstrap] ERROR: submodule init failed - check network/git, clone PolyOrch manually and retry" >&2; exit 1; }
    [ -f PolyOrch/scripts/pixi-init.sh ] \
        || { echo "[bootstrap] ERROR: PolyOrch/scripts/pixi-init.sh still missing after submodule init" >&2; exit 1; }
fi

# 5. ensure pixi + install the repo envs - delegate to PolyOrch/scripts/pixi-init.sh
#    (auditable pinned install, PIXI_VERSION default 0.78.0, already-satisfied
#    no-op, lock-drift retry). D65 boundary: mise owns host/agent tools, pixi owns
#    the build envs; pixi.lock stays the committed reproducibility pin.
bash PolyOrch/scripts/pixi-init.sh --manifest-path "$ROOT/pixi.toml" \
    || { echo "[bootstrap] ERROR: pixi bootstrap delegation failed (PolyOrch/scripts/pixi-init.sh)" >&2; exit 1; }

# Reserved host policy slot - China cargo mirror (rsproxy) goes here if the
# maintainer opts in; deliberately not the supplier's job (its scope note
# leaves mirror/registry policy with the host).

# 6. repo-local git hooks: C3 commit-shape pre-flight (commit-msg reads the
#    prepared message; conventions.md C3). Idempotent; --no-verify stays the escape hatch.
if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git config core.hooksPath scripts/hooks
  echo "[bootstrap] core.hooksPath -> scripts/hooks (C3 gate live)"
fi

echo ""
echo "[bootstrap] done. Open a NEW terminal, then verify:"
echo "    node -v        # expect the mise.toml pin (v22.x), not the system node"
echo "    mise doctor"
echo "    pixi --version # build-env manager (D65); repo envs: pixi run <task>"
