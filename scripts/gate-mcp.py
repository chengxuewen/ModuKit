#!/usr/bin/env python3
"""Probe every declared MCP server without installing anything.

`probe_mode` comes from scripts/mcp-servers.json:
  - start:   launch through the pinned npm launcher with empty stdin; expect clean EOF.
  - version: run the installed binary with --version; never start a server or browser.
  - pixi:    run the env binary probe; a missing env is SKIP-ENV, never red.

Missing npm package or missing pixi env is a SKIP (yellow): a fresh clone has no
node_modules, and `pixi run mcp setup` is the explicit installer. A present but
broken server, a pin drift, or a registry drift is red.

The PATH probe mirrors the GUI-inherited PATH (O5): ~/.pixi/bin is removed, so a
pixi that only lives there is caught before a host fails to spawn a server. O5 is a
WARN by default (the fix is a machine-level symlink the repo cannot make); set
MODUKIT_GATE_MCP_STRICT=1 to turn it red in CI.
Exit: 0 if no red, 1 if any red.
"""

import json
import os
import re
import shutil
import subprocess
import sys

import agent_common as ac

PREFIX = "GATE-MCP"
CFG = os.path.join(ac.ROOT, ".agents", "mcp.json")
REGISTRY = os.path.join(ac.ROOT, "scripts", "mcp-servers.json")
TOKEN_HINT = re.compile(
    r"environment variable is not set|unauthorized|401|token|GITHUB_PERSONAL_ACCESS_TOKEN",
    re.I,
)
TIMEOUT = 300


def _host_path():
    """PATH as a GUI MCP host would inherit it: no ~/.pixi/bin."""
    pixi_dir = os.path.realpath(os.path.join(os.path.expanduser("~"), ".pixi", "bin"))
    dirs = [
        d
        for d in os.environ.get("PATH", "").split(os.pathsep)
        if d and os.path.realpath(d) != pixi_dir
    ]
    return os.pathsep.join(dirs)


def _bin_path(bin_name):
    name = bin_name + ".cmd" if os.name == "nt" else bin_name
    return os.path.join(ac.ROOT, "node_modules", ".bin", name)


def _env_bin(spec):
    name = spec["probe"][0] + (".exe" if os.name == "nt" else "")
    return os.path.join(ac.ROOT, ".pixi", "envs", spec["pixi_env"], "bin", name)


def _run(cmd, env):
    try:
        proc = subprocess.run(
            cmd,
            cwd=ac.ROOT,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=TIMEOUT,
            env=env,
        )
    except subprocess.TimeoutExpired:
        return None, "timeout after %ds" % TIMEOUT
    return proc, ""


def main():
    with open(CFG) as handle:
        servers = json.load(handle).get("mcpServers", {})
    if not servers:
        print("%s ✓ no servers declared" % PREFIX)
        return 0
    with open(REGISTRY) as handle:
        registry = json.load(handle)

    host_path = _host_path()
    env = dict(os.environ, PATH=host_path)
    red = 0
    yellow = 0
    skip_env = 0

    if shutil.which("pixi", path=host_path) is None:
        message = "pixi not on the GUI-inherited PATH (fix: bootstrap / O5 symlink)"
        if os.environ.get("MODUKIT_GATE_MCP_STRICT") == "1":
            print(" [ RED  ] " + message)
            red += 1
        else:
            print(" [ WARN ] " + message + " (strict: MODUKIT_GATE_MCP_STRICT=1)")

    for name in servers:
        spec = registry.get(name)
        if spec is None:
            print(" [ RED  ] %-14s absent from scripts/mcp-servers.json (registry drift)" % name)
            red += 1
            continue
        mode = spec.get("probe_mode", "start")
        if spec.get("kind") == "pixi":
            binary = _env_bin(spec)
            if not os.path.exists(binary):
                print(
                    " [SKIPENV] %-13s env '%s' not installed (pixi install -e %s)"
                    % (name, spec["pixi_env"], spec["pixi_env"])
                )
                skip_env += 1
                continue
            cmd = [binary] + list(spec["probe"][1:])
        else:
            binary = _bin_path(spec["bin"])
            if not os.path.exists(binary):
                print(
                    " [ SKIP ] %-14s package not installed (run: pixi run mcp setup)" % name
                )
                yellow += 1
                continue
            if mode == "version":
                cmd = [binary, "--version"]
            else:
                cmd = [
                    "node",
                    "scripts/mcp-serve.mjs",
                    "--from",
                    "scripts/mcp-servers.json",
                    "--name",
                    name,
                ]
        print(" [PROBE] %-14s %s" % (name, " ".join(cmd)))
        proc, err = _run(cmd, env)
        if proc is None:
            print(" [ RED  ] %-14s %s" % (name, err))
            red += 1
            continue
        if proc.returncode == 0:
            first = (proc.stdout or proc.stderr).strip().splitlines()
            print(" [ OK   ] %-14s %s" % (name, first[0] if first else "probe ok"))
            continue
        if TOKEN_HINT.search(proc.stderr or "") or TOKEN_HINT.search(proc.stdout or ""):
            print(" [ SKIP ] %-14s missing credential (expected in CI)" % name)
            yellow += 1
            continue
        print(" [ RED  ] %-14s exit=%d" % (name, proc.returncode))
        tail = (proc.stderr or proc.stdout).strip().splitlines()
        if tail:
            print("        %s" % tail[-1])
        red += 1

    if red:
        print("%s ✗ %d red" % (PREFIX, red))
        return 1
    print(
        "%s ✓ (%d green, %d skipped, %d skipped-env)"
        % (PREFIX, len(servers) - yellow - skip_env, yellow, skip_env)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
