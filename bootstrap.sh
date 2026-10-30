#!/usr/bin/env bash
# bootstrap.sh — one-time dev setup (pixi only; user-level, no sudo).
#
# Steps:
#   1. submodule guard for the PolyOrch pixi-init vendor
#   2. pixi + repo envs via PolyOrch/scripts/pixi-init.sh
#   3. add ~/.pixi/bin to PATH (idempotent rc line)
#   4. opt-in China cargo mirror (MODUKIT_CN_MIRROR=1)
#   5. repo git hooks (core.hooksPath=scripts/hooks)
#   6. advisory agent-layer sync
#   7. verification hints
#
# mise and rust-toolchain.toml are retired (B4): pixi is the single toolchain.
# Windows native: bootstrap.bat mirrors this; its first-run debt stands.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"
ROOT="$PWD"

# 1. submodule guard ----------------------------------------------------------
if [ ! -f PolyOrch/scripts/pixi-init.sh ]; then
    echo "[bootstrap] PolyOrch/scripts/pixi-init.sh missing - initialising the submodule"
    git submodule update --init --recursive PolyOrch \
        || { echo "[bootstrap] ERROR: submodule init failed - check network/git, clone PolyOrch manually and retry" >&2; exit 1; }
    [ -f PolyOrch/scripts/pixi-init.sh ] \
        || { echo "[bootstrap] ERROR: PolyOrch/scripts/pixi-init.sh still missing after init" >&2; exit 1; }
fi

# 2. pixi + repo envs (auditable pinned install; already-satisfied no-op) ------
bash PolyOrch/scripts/pixi-init.sh --manifest-path "$ROOT/pixi.toml" \
    || { echo "[bootstrap] ERROR: pixi bootstrap delegation failed (PolyOrch/scripts/pixi-init.sh)" >&2; exit 1; }

# 3. PATH for the pixi binary (the envs are activated by pixi itself) ----------
if ! command -v pixi >/dev/null 2>&1; then
    case "$(basename "${SHELL:-bash}")" in
        zsh) RC="$HOME/.zshrc" ;;
        *)   RC="$HOME/.bashrc" ;;
    esac
    if ! grep -qF '# ModuKit pixi PATH' "$RC" 2>/dev/null; then
        echo 'export PATH="$HOME/.pixi/bin:$PATH"  # ModuKit pixi PATH' >> "$RC"
        echo "[bootstrap] pixi PATH line appended to $RC"
    fi
    export PATH="$HOME/.pixi/bin:$PATH"
fi
command -v pixi >/dev/null 2>&1 \
    || { echo "[bootstrap] ERROR: pixi still not resolvable - re-run and check polyorch/scripts/pixi-init.sh output" >&2; exit 1; }
echo "[bootstrap] pixi: $(pixi --version)"

# 4. host policy (opt-in) - China cargo mirror. Default OFF: writing the user's
# cargo config is a network-policy decision, deliberately NOT the supplier's job.
if [ "${MODUKIT_CN_MIRROR:-0}" = "1" ]; then
    CARGO_CONFIG="${CARGO_HOME:-$HOME/.cargo}/config.toml"
    mkdir -p "$(dirname "$CARGO_CONFIG")"
    if grep -q "rsproxy" "$CARGO_CONFIG" 2>/dev/null; then
        echo "[bootstrap] cargo mirror already configured (rsproxy.cn)"
    else
        cat >> "$CARGO_CONFIG" <<'CARGO_EOF'

# ModuKit host policy (opt-in): crates.io mirror for China (rsproxy.cn)
# sparse needs the /index/ path - the git-protocol address 404s under sparse
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
# fallback if rsproxy.cn is down: point replace-with at ustc-sparse
# [source.ustc-sparse]
# registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
CARGO_EOF
        echo "[bootstrap] cargo mirror configured: $CARGO_CONFIG"
    fi
else
    echo "[bootstrap] cargo mirror off (opt in: MODUKIT_CN_MIRROR=1)"
fi

# 5. repo-local git hooks (commit-msg C3 + pre-commit agent sync/gate) ---------
if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git config core.hooksPath scripts/hooks
    echo "[bootstrap] core.hooksPath -> scripts/hooks"
fi

# 6. advisory: regenerate the agent layers (never fatal on a fresh clone) -----
if pixi run agents sync; then
    echo "[bootstrap] agent layers synced"
else
    echo "[bootstrap] WARN: agent sync failed; run: pixi run agents sync"
fi

# 7. verification hints --------------------------------------------------------
echo ""
echo "[bootstrap] done. Open a NEW terminal, then verify:"
echo "    pixi --version"
echo "    pixi run cargo --version"
echo "    pixi run node --version   # v22.x from the mcp feature"
echo "[bootstrap] NOTE: mise is retired; remove old 'mise activate' lines from your rc manually."
