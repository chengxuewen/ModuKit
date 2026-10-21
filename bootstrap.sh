#!/usr/bin/env bash
# bootstrap.sh — one-time dev-environment setup for ModuKit (user-level, no sudo).
#
#   ./bootstrap.sh
#
# Idempotent. Steps:
#   1. ensure mise        (official installer -> ~/.local/bin if missing)
#   2. ensure shell activation (idempotent append to ~/.bashrc or ~/.zshrc)
#   3. mise install       (toolchain pins from mise.toml; no-op when satisfied)
#   4. ensure pixi        (build-env manager per D65; repo pin: pixi.toml + pixi.lock)
#
# After first run: open a NEW terminal (activation is read at shell start).
# Windows native: use bootstrap.bat (same steps, no WSL required).

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

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

# 4. ensure pixi - build-time environments (D65 boundary: pixi owns build envs,
#    mise owns host/agent tools; pixi.lock is the committed reproducibility pin) ----
PIXI="$(command -v pixi 2>/dev/null || true)"
[ -n "$PIXI" ] || [ ! -x "$HOME/.pixi/bin/pixi" ] || PIXI="$HOME/.pixi/bin/pixi"
if [ -z "$PIXI" ]; then
    echo "[bootstrap] pixi not found - installing user-level via https://pixi.sh"
    curl -fsSL https://pixi.sh/install.sh | sh
    PIXI="$HOME/.pixi/bin/pixi"
    [ -x "$PIXI" ] || { echo "[bootstrap] ERROR: pixi installer finished but $PIXI is missing" >&2; exit 1; }
fi
echo "[bootstrap] pixi: $("$PIXI" --version)"

# 5. repo-local git hooks: C3 commit-shape pre-flight (commit-msg reads the
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
