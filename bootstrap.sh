#!/usr/bin/env bash
# bootstrap.sh — one-time dev-environment setup for ModuKit (user-level, no sudo).
#
#   ./bootstrap.sh
#
# Idempotent. Steps:
#   1. ensure mise        (official installer -> ~/.local/bin if missing)
#   2. ensure shell activation (idempotent append to ~/.bashrc or ~/.zshrc)
#   3. mise install       (toolchain pins from mise.toml; no-op when satisfied)
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

echo ""
echo "[bootstrap] done. Open a NEW terminal, then verify:"
echo "    node -v      # expect the mise.toml pin (v22.x), not the system node"
echo "    mise doctor"
