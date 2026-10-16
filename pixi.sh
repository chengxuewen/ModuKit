#!/usr/bin/env bash
# pixi.sh -- activate the pixi workspace that lives in THIS directory.
#
# Installed next to pixi.toml / .pixi/ by polyorch_pixi_init(COPY_SCRIPTS)
# or polyorch_pixi_scripts_install(). The file is managed by PolyOrch; local
# edits are overwritten on the next install.
#
# Usage:   source pixi.sh        (executing it activates a child shell only)
# Requires pixi; if absent, provision it first with
# polyorch_pixi_tool_ensure() (see examples/pixi-bootstrap/).

# No `set -e` on purpose: this file is sourced into an interactive shell and
# must not leak errexit; every failure path exits explicitly.

if [ -n "${BASH_SOURCE[0]:-}" ]; then
    _dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
elif [ -n "${ZSH_VERSION:-}" ]; then
    _dir="$(cd "$(dirname "${(%):-%N}")" && pwd -P)"
else
    _dir="$(cd "$(dirname "$0")" && pwd -P)"
fi

_pixi=""
if command -v pixi >/dev/null 2>&1; then
    _pixi="$(command -v pixi)"
elif [ -x "$HOME/.pixi/bin/pixi" ]; then
    _pixi="$HOME/.pixi/bin/pixi"
fi
if [ -z "$_pixi" ]; then
    echo "pixi not found -- provision it first (polyorch_pixi_tool_ensure, or examples/pixi-bootstrap/bootstrap.cmake)" >&2
    unset _dir
    return 1
fi

_man="$_dir/pixi.toml"
[ -f "$_man" ] || _man="$_dir/pyproject.toml"
if [ ! -f "$_man" ]; then
    echo "no pixi.toml or pyproject.toml beside this script: $_dir" >&2
    unset _dir _pixi
    return 1
fi

# TTY guard: stay silent when the output is piped or consumed by automation.
if [ -t 1 ]; then
    echo "Activating pixi workspace: $_man"
fi

_shell=bash
[ -n "${ZSH_VERSION:-}" ] && _shell=zsh
eval "$("$_pixi" shell-hook --manifest-path "$_man" --shell "$_shell")"

if [ -t 1 ]; then
    echo "Activated. Handy:"
    echo "  pixi task list   -- every task this workspace defines"
    echo "  exit             -- leave the subshell you started to source this"
fi
unset _dir _pixi _man _shell
