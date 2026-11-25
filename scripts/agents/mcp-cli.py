#!/usr/bin/env python3
"""MCP lifecycle CLI: install / index / sync / status.

Single entry point for the local MCP layer. Run it through pixi so `node`
resolves (the MCP npm packages are launched with the pixi-provided node):

    pixi run mcp -h
    pixi run mcp status

Design notes (see docs/ve-parity-plan.md, B1):
- Subcommands: `setup` (install at pin), `index` (init on first run, full rebuild
  after), `sync` (incremental), `status` (read-only).
- `status` is read-only and always exits 0; missing `node` exits 127 with a hint.
- `setup` is the ONLY explicit installer (`--save-exact`); the launcher still
  installs when a binary is missing (panel first run / clean clone), but the
  launcher no longer reinstalls on pin drift -- it fails with a hint instead.
- Pins live in scripts/agents/mcp-servers.json (single source); package.json /
  package-lock.json / node_modules are gitignored local echoes.
"""

import argparse
import json
import os
import re
import shutil
import tempfile
import subprocess
import sys

try:
    import agent_common as ac
except ModuleNotFoundError:  # direct run from scripts/agents/ (batch one layout)
    sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    import agent_common as ac

PREFIX = "MCP"
SPECS = os.path.join("scripts", "agents", "mcp-servers.json")
NODE_HINT = "node not found on PATH -- run via: pixi run mcp <subcommand>"
NPM_HINT = "npm not found on PATH -- run via: pixi run mcp setup"
BROWSERS_ENV = "PLAYWRIGHT_BROWSERS_PATH"


def _specs():
    with open(os.path.join(ac.ROOT, SPECS)) as handle:
        return json.load(handle)


def _installed_version(pkg):
    """Version of `pkg` currently in ./node_modules, or None when absent."""
    parts = pkg.split("/") if pkg.startswith("@") else [pkg]
    manifest = os.path.join(ac.ROOT, "node_modules", *parts, "package.json")
    if not os.path.exists(manifest):
        return None
    try:
        with open(manifest) as handle:
            return json.load(handle).get("version")
    except (OSError, ValueError):
        return None


def _bin_path(bin_name):
    name = bin_name + ".cmd" if os.name == "nt" else bin_name
    return os.path.join(ac.ROOT, "node_modules", ".bin", name)


def _pkg_dir(pkg):
    parts = pkg.split("/") if pkg.startswith("@") else [pkg]
    return os.path.join(ac.ROOT, "node_modules", *parts)


def _browsers_dir():
    """Where playwright-core puts downloaded browsers (mirrors its platform rules).

    A repo-local override (PLAYWRIGHT_BROWSERS_PATH) is honoured so a CI/cache dir
    can be pinned; the default is the per-user cache, shared across projects.
    """
    override = os.environ.get(BROWSERS_ENV)
    if override and override != "0":
        return override
    home = os.path.expanduser("~")
    if os.name == "nt":
        base = os.environ.get("LOCALAPPDATA") or os.path.join(home, "AppData", "Local")
        return os.path.join(base, "ms-playwright")
    if sys.platform == "darwin":
        return os.path.join(home, "Library", "Caches", "ms-playwright")
    return os.path.join(home, ".cache", "ms-playwright")


def _browser_installed(name):
    prefix = "chromium_headless_shell-" if name == "chromium-headless-shell" else name + "-"
    try:
        return any(entry.startswith(prefix) for entry in os.listdir(_browsers_dir()))
    except OSError:
        return False


def _platform_fallback():
    """The playwright platform tag to impersonate when the host OS is dropped.

    Measured 2026-11-25 on Ubuntu 20.04 / glibc 2.31: Playwright refuses to install
    for `ubuntu20.04-x64`, but its own installer carries a fallback path
    ("BEWARE: your OS is not officially supported by Playwright; downloading
    fallback build for ubuntu22.04-x64") that only fires when
    PLAYWRIGHT_HOST_PLATFORM_OVERRIDE names the newer tag. The downloaded build is
    the SAME version the pin wants (154.0.8037.0 / chromium-1246), and it runs:
    a full MCP `browser_navigate` succeeds on this box. So the fallback is a
    supported knob, not a hack -- it just needs the OS tag we map to.

    Returns None when no override is needed or the host is not linux.
    """
    if not sys.platform.startswith("linux"):
        return None
    return os.environ.get("MODUKIT_MCP_PLATFORM_OVERRIDE", "ubuntu22.04-x64")


def _record_path():
    return os.path.join(ac.ROOT, "out", "mcp-browser.json")


def _write_record(record):
    """Atomic write (tmp + os.replace) so a concurrent launcher read never tears."""
    path = _record_path()
    os.makedirs(os.path.dirname(path), exist_ok=True)
    handle, tmp = tempfile.mkstemp(prefix=".mcp-browser-", dir=os.path.dirname(path))
    try:
        with os.fdopen(handle, "w") as stream:
            json.dump(record, stream, indent=2, sort_keys=True)
            stream.write("\n")
        os.replace(tmp, path)
    except OSError:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _read_record():
    """The resolution record as written by `setup`, or None when absent/unreadable.

    Shape: {"record_version": 1, "resolutions": {"<server>": {mode, browser, ...}}}.
    An entry without a mode is UNRESOLVED (W1's skeleton) and is filtered by
    _resolution_for rather than treated as a resolution.
    """
    try:
        with open(_record_path()) as handle:
            record = json.load(handle)
    except (OSError, ValueError):
        return None
    return record if isinstance(record, dict) else None


def _resolution_for(record, server):
    """The resolved entry for `server`, or None when missing or mode-less."""
    if not record:
        return None
    entry = (record.get("resolutions") or {}).get(server)
    return entry if isinstance(entry, dict) and entry.get("mode") else None


def _install_browser(server, spec, node):
    """Resolve a browser for `server` through the three-rung ladder.

    Rung 1: Playwright's managed build via its own installer (this also covers the
    dropped-host fallback through PLAYWRIGHT_HOST_PLATFORM_OVERRIDE).
    Rung 2: system browser on PATH (absolute path only, live `--version` proof).
    Rung 3: nothing usable -> warn, `MODUKIT_MCP_STRICT=1` reds it.

    Returns a resolution record dict, or None when no rung resolved.
    """
    name = spec.get("browser")
    if not name:
        return None
    if _browser_installed(name):
        print("%s %-16s browser ok (%s)" % (PREFIX, server, name))
        return {"mode": "playwright", "browser": name, "version": _playwright_browser_version(spec),
                "executable": None, "server": server}

    cli = os.path.join(_pkg_dir(spec["pkg"]), "cli.js")
    if not os.path.exists(cli):
        print("%s %-16s browser %s not installed and %s absent -- install the package first" % (
            PREFIX, server, name, os.path.relpath(cli, ac.ROOT)))
        return None
    print("%s %-16s installing browser %s (playwright-managed) ..." % (PREFIX, server, name))
    env = dict(os.environ)
    override = _platform_fallback()
    if override and not env.get("PLAYWRIGHT_HOST_PLATFORM_OVERRIDE"):
        env["PLAYWRIGHT_HOST_PLATFORM_OVERRIDE"] = override
        print("%s %-16s host OS not served by the pin -- using Playwright's fallback build (%s)"
              % (PREFIX, server, override))
    proc = subprocess.run(
        [node, cli, "install-browser", name, "--no-progress"],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )
    if proc.stdout:
        sys.stderr.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    if proc.returncode == 0 and _browser_installed(name):
        return {"mode": "playwright", "browser": name, "version": _playwright_browser_version(spec),
                "executable": None, "server": server}

    head = (proc.stderr or "") + (proc.stdout or "")
    if "does not support" in head:
        print("%s %-16s ⚠ the pinned Playwright has no build for this host" % (PREFIX, server))
    else:
        print("%s %-16s ⚠ managed browser install failed (offline?)" % (PREFIX, server))

    system = _system_browser()
    if system:
        print("%s %-16s falling back to the system browser (%s)" % (PREFIX, server, system))
        print("        NOTE: system browsers auto-update; Playwright only guarantees its own builds")
        return {"mode": "system", "browser": name, "version": None, "executable": system, "server": server}
    print("%s %-16s no usable browser: managed build unavailable and no system browser found" % (PREFIX, server))
    return None


def _playwright_browser_version(spec):
    """The pin's browser version, read from the installed playwright-core manifest."""
    core = os.path.join(ac.ROOT, "node_modules", "playwright-core", "browsers.json")
    try:
        with open(core) as handle:
            for entry in json.load(handle).get("browsers", []):
                if entry.get("name") == "chromium":
                    return entry.get("browserVersion")
    except (OSError, ValueError):
        pass
    return None


SYSTEM_BROWSER_CANDIDATES = (
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "microsoft-edge",
    "msedge",
)


def _system_browser():
    """Absolute path of the first PATH browser that survives a live `--version`."""
    for candidate in SYSTEM_BROWSER_CANDIDATES:
        found = shutil.which(candidate)
        if not found:
            continue
        try:
            proc = subprocess.run(
                [found, "--version"],
                stdin=subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=20,
            )
        except (OSError, subprocess.SubprocessError):
            continue
        if proc.returncode == 0:
            return os.path.realpath(found)
    return None


def _run_codegraph_status(node):
    """Pass `codegraph status` through; uninitialized is NOT an error (exit 0)."""
    binary = _bin_path("codegraph")
    if not os.path.exists(binary):
        print("%s codegraph binary missing -- run: pixi run mcp setup" % PREFIX)
        return
    proc = subprocess.run(
        [node, binary, "status"],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
    )
    sys.stdout.write(proc.stdout)
    sys.stderr.write(proc.stderr)


def _install(pkg, version, npm):
    """npm install -D --save-exact <pkg>@<version>; npm's stdout goes to stderr.

    Keep the package manager's chatter off stdout: for an MCP server, stdout is
    the JSON-RPC channel, and this code path is shared with the launcher.
    """
    print("%s installing %s@%s ..." % (PREFIX, pkg, version))
    proc = subprocess.run(
        [npm, "install", "-D", "--save-exact", "%s@%s" % (pkg, version)],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=None,  # npm's progress already goes to stderr; inherit it
        text=True,
    )
    if proc.stdout:
        sys.stderr.write(proc.stdout)
    return proc.returncode == 0


def _unblock(pkg):
    """Move stale npm temp dirs aside so npm's rename cannot hit ENOTEMPTY.

    When a version switch is interrupted, npm leaves a non-empty staging dir
    (`.<name>-<hash>`) inside the package scope; the next install tries to rename
    into that exact path and fails with ENOTEMPTY forever. The staging dirs are
    disposable, but bulk-deleting them is both slow and unnecessary -- renaming
    them out of node_modules is enough to unblock the install.
    """
    parts = pkg.split("/") if pkg.startswith("@") else [pkg]
    scope_dir = os.path.join(ac.ROOT, "node_modules", parts[0])
    if not os.path.isdir(scope_dir):
        return 0
    name = parts[-1]
    out_dir = os.path.join(ac.ROOT, "out", "scratch")
    os.makedirs(out_dir, exist_ok=True)
    stale_root = tempfile.mkdtemp(prefix="mcp-cli-stale-", dir=out_dir)
    moved = 0
    for entry in os.listdir(scope_dir):
        if not entry.startswith("." + name + "-"):
            continue
        try:
            os.rename(os.path.join(scope_dir, entry), os.path.join(stale_root, entry))
            moved += 1
        except OSError:
            continue
    return moved


def _is_pixi(spec):
    return spec.get("kind") == "pixi"


def _env_bin(spec):
    """Absolute path of a pixi-kind probe binary inside its (isolated) environment."""
    name = spec["probe"][0]
    if os.name == "nt":
        name += ".exe"
    return os.path.join(ac.ROOT, ".pixi", "envs", spec["pixi_env"], "bin", name)


def _pixi_version(spec):
    """Version reported by the probe binary; None when the env or the binary is absent.

    Deliberately does NOT shell out to `pixi`: nested pixi is not on PATH inside a task
    (review #18), and this helper is also reused by the gate.
    """
    binary = _env_bin(spec)
    if not os.path.exists(binary):
        return None
    try:
        proc = subprocess.run(
            [binary] + list(spec["probe"][1:]),
            cwd=ac.ROOT,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=120,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if proc.returncode != 0:
        return None
    lines = (proc.stdout or proc.stderr).strip().splitlines()
    first = lines[0].strip() if lines else ""
    # Serena prints "Serena 2.1.0-<git describe>-dirty": keep only the semver part so the
    # pin comparison is stable across re-installs (the git suffix moves every build).
    match = re.search(r"\d+\.\d+\.\d+", first)
    return match.group(0) if match else first


def cmd_setup(args):
    specs = _specs()
    npm = shutil.which("npm")
    needs_npm = any(not _is_pixi(spec) for spec in specs.values())
    if needs_npm and npm is None:
        print("%s %s" % (PREFIX, NPM_HINT))
        return 127
    node = shutil.which("node")
    if any(spec.get("browser") for spec in specs.values()) and node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    failures = 0
    warnings = 0
    resolutions = {}
    for name, spec in specs.items():
        version = spec["version"]
        if _is_pixi(spec):
            reported = _pixi_version(spec)
            if reported is None:
                print(
                    "%s %-16s skipped: env '%s' not installed (run: pixi install -e %s)"
                    % (PREFIX, name, spec["pixi_env"], spec["pixi_env"])
                )
                continue
            if not reported.startswith(version):
                print("%s %-16s pinch drift: %s vs pin %s -- run: pixi install -e %s" % (PREFIX, name, reported, version, spec["pixi_env"]))
                failures += 1
            else:
                print("%s %-16s ok (pixi %s, %s)" % (PREFIX, name, spec["pixi_env"], reported))
            continue
        pkg = spec["pkg"]
        have = _installed_version(pkg)
        ready = have == version and os.path.exists(_bin_path(spec["bin"]))
        if ready:
            print("%s %-16s ok (%s@%s)" % (PREFIX, name, pkg, version))
        else:
            if have is not None:
                print("%s %-16s drift: installed %s, pin %s -- reinstalling" % (PREFIX, name, have, version))
            unblocked = _unblock(pkg)
            if unblocked:
                print("%s %-16s moved %d stale staging dir(s) aside" % (PREFIX, name, unblocked))
            if not _install(pkg, version, npm):
                print("%s %-16s ✗ npm install failed" % (PREFIX, name))
                failures += 1
                continue
            after = _installed_version(pkg)
            if after != version or not os.path.exists(_bin_path(spec["bin"])):
                print("%s %-16s ✗ still not at pin (installed %s)" % (PREFIX, name, after or "-"))
                failures += 1
                continue
            print("%s %-16s ✓ ready (%s@%s)" % (PREFIX, name, pkg, version))
        if spec.get("browser"):
            record = _install_browser(name, spec, node)
            if record is None:
                warnings += 1
            else:
                resolutions[name] = record
    if any(spec.get("browser") for spec in specs.values()):
        try:
            _write_record({"record_version": 1, "resolutions": resolutions})
        except OSError as exc:
            print("%s ✗ cannot write %s: %s" % (PREFIX, os.path.relpath(_record_path(), ac.ROOT), exc))
            failures += 1
    if failures:
        print("%s ✗ %d server(s) not ready" % (PREFIX, failures))
        return ac.EXIT_FINDINGS
    if warnings:
        message = "%d browser gap(s) -- servers installed, browser not provisioned" % warnings
        if os.environ.get("MODUKIT_MCP_STRICT") == "1":
            print("%s ✗ %s (strict)" % (PREFIX, message))
            return ac.EXIT_FINDINGS
        print("%s ⚠ %s (strict: MODUKIT_MCP_STRICT=1)" % (PREFIX, message))
        return ac.EXIT_OK
    print("%s ✓ all servers ready" % PREFIX)
    return ac.EXIT_OK


def _codegraph(node, *path_args):
    """Run the pinned codegraph binary with inherited stdio (progress stays visible)."""
    binary = _bin_path("codegraph")
    if not os.path.exists(binary):
        print("%s codegraph binary missing -- run: pixi run mcp setup" % PREFIX)
        return 127
    return subprocess.run(
        [node, binary, *path_args],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
    ).returncode


def _serena_index_all(specs):
    """Rebuild the LSP cache for every pixi-kind server; returns the failure count.

    Serena has no incremental mode: this is always a full `serena project index`.
    """
    failures = 0
    for name, spec in specs.items():
        if not _is_pixi(spec):
            continue
        binary = _env_bin(spec)
        if not os.path.exists(binary):
            print(
                "%s %-16s skipped: env '%s' not installed (run: pixi install -e serena)"
                % (PREFIX, name, spec["pixi_env"])
            )
            continue
        print("%s %-16s rebuilding .serena/cache (serena project index) ..." % (PREFIX, name))
        env = dict(os.environ)
        env["PATH"] = (
            os.path.join(ac.ROOT, ".pixi", "envs", spec["pixi_env"], "bin")
            + os.pathsep
            + env.get("PATH", "")
        )
        env["PIXI_PROJECT_ROOT"] = ac.ROOT
        try:
            proc = subprocess.run(
                [binary, "project", "index", "."],
                cwd=ac.ROOT,
                stdin=subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=900,
                env=env,
            )
        except subprocess.TimeoutExpired:
            print("%s %-16s ✗ serena index timeout after 900s" % (PREFIX, name))
            failures += 1
            continue
        if proc.returncode != 0:
            print("%s %-16s ✗ serena index failed (exit %d)" % (PREFIX, name, proc.returncode))
            tail = (proc.stderr or proc.stdout).strip().splitlines()
            if tail:
                print("        %s" % tail[-1])
            failures += 1
            continue
        langs = sorted(os.listdir(os.path.join(ac.ROOT, ".serena", "cache"))) \
            if os.path.isdir(os.path.join(ac.ROOT, ".serena", "cache")) else []
        print("%s %-16s ✓ indexed (%s)" % (PREFIX, name, ", ".join(langs) or "empty"))
    return failures


def cmd_index(args):
    """First run initializes .codegraph; later runs rebuild it. Serena's LSP cache is rebuilt too."""
    node = shutil.which("node")
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    initialized = os.path.isdir(os.path.join(ac.ROOT, ".codegraph"))
    step = ["index"] if initialized else ["init", "-y"]
    print("%s %s .codegraph (codegraph %s)" % (PREFIX, "rebuilding" if initialized else "initializing", " ".join(step)))
    code = _codegraph(node, *step)
    if code != 0:
        print("%s ✗ codegraph %s failed (exit %d)" % (PREFIX, " ".join(step), code))
        return ac.EXIT_FINDINGS

    specs = _specs()
    failures = _serena_index_all(specs)

    print()
    print("%s index done -- scale check:" % PREFIX)
    _run_codegraph_status(node)
    if failures:
        print("%s ✗ %d non-codegraph index failure(s)" % (PREFIX, failures))
        return ac.EXIT_FINDINGS
    return ac.EXIT_OK


def cmd_sync(args):
    """Codegraph gets an incremental sync; pixi-kind servers get a full LSP-cache re-index."""
    node = shutil.which("node")
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    if not os.path.isdir(os.path.join(ac.ROOT, ".codegraph")):
        print("%s no .codegraph yet -- run: pixi run mcp index" % PREFIX)
        return ac.EXIT_FINDINGS
    code = _codegraph(node, "sync")
    if code != 0:
        print("%s ✗ codegraph sync failed (exit %d)" % (PREFIX, code))
        return ac.EXIT_FINDINGS
    specs = _specs()
    failures = _serena_index_all(specs)
    if failures:
        print("%s ✗ %d serena re-index failure(s)" % (PREFIX, failures))
        return ac.EXIT_FINDINGS
    print("%s ✓ synced" % PREFIX)
    return ac.EXIT_OK


def cmd_status(args):
    specs = _specs()
    node = shutil.which("node")
    print("%s servers declared in %s:" % (PREFIX, SPECS))
    for name, spec in specs.items():
        version = spec["version"]
        if _is_pixi(spec):
            reported = _pixi_version(spec)
            if reported is None:
                state = "missing"
            elif reported.startswith(version):
                state = "ok"
            else:
                state = "DRIFT"
            cache_dir = os.path.join(ac.ROOT, ".serena", "cache")
            langs = ",".join(sorted(os.listdir(cache_dir))) if os.path.isdir(cache_dir) else "empty"
            print(
                "  %-16s %-38s pin=%-8s reported=%-24s %s  cache=[%s]"
                % (name, "pixi:" + spec["pixi_env"], version, reported or "-", state, langs)
            )
            continue
        have = _installed_version(spec["pkg"])
        if have is None:
            state = "missing"
        elif have != version:
            state = "DRIFT"
        else:
            state = "ok"
        print(
            "  %-16s %-38s pin=%-8s installed=%-8s %s"
            % (name, spec["pkg"], version, have or "-", state)
        )
        if spec.get("browser"):
            resolution = _resolution_for(_read_record(), name)
            if resolution:
                detail = resolution.get("mode")
                if resolution.get("executable"):
                    detail += " %s" % resolution["executable"]
                if resolution.get("version"):
                    detail += " v%s" % resolution["version"]
                print("  %-16s browser=%s %s" % ("", spec["browser"], detail))
            elif _browser_installed(spec["browser"]):
                print("  %-16s browser=%s ok (managed build, no record -- run: pixi run mcp setup)"
                      % ("", spec["browser"]))
            else:
                print("  %-16s browser=%s UNRESOLVED (run: pixi run mcp setup)" % ("", spec["browser"]))
    print()
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    _run_codegraph_status(node)
    return ac.EXIT_OK


def cmd_latest(args):
    """Report registry latest vs the pinned versions. Never changes anything.

    The ONLY registry-facing surface in this CLI: `setup` never floats a pin
    (pin drift must fail fast in the launcher, review #4/#16). Offline is a note,
    not a red -- a curiosity query must not break status-like ergonomics.
    """
    specs = _specs()
    named = [name for name, spec in specs.items() if not _is_pixi(spec)]
    if args.server:
        if args.server not in named:
            print("%s ✗ '%s' is not a registry-pinned server (have: %s)"
                  % (PREFIX, args.server, ", ".join(named)))
            return ac.EXIT_FINDINGS
        named = [args.server]
    npm = shutil.which("npm")
    if npm is None:
        print("%s %s" % (PREFIX, "npm not found -- run via: pixi run mcp latest"))
        print("%s (this is an on-demand registry query; nothing to change)" % PREFIX)
        return ac.EXIT_OK
    stale = 0
    for name in named:
        spec = specs[name]
        pkg, pin = spec["pkg"], spec["version"]
        try:
            proc = subprocess.run(
                [npm, "view", pkg, "version"],
                stdin=subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=45,
            )
        except (OSError, subprocess.SubprocessError):
            proc = None
        latest = (proc.stdout or "").strip().splitlines()[0] if proc and proc.returncode == 0 else ""
        if not latest:
            print("%s %-16s %s pin=%s -- registry unreachable or timed out (no action)"
                  % (PREFIX, name, pkg, pin))
            continue
        if latest == pin:
            print("%s %-16s %s pin=%s (latest)" % (PREFIX, name, pkg, pin))
            continue
        stale += 1
        print("%s %-16s %s pin=%s latest=%s" % (PREFIX, name, pkg, pin, latest))
        print("        to bump: edit %s -> %s.version, then run: pixi run mcp setup"
              % (SPECS, name))
    if stale:
        print("%s %d server(s) behind the registry -- pin bumps are a reviewed edit, never automatic"
              % (PREFIX, stale))
    else:
        print("%s ✓ every registry-pinned server is at the latest published version" % PREFIX)
    return ac.EXIT_OK


def build_parser():
    parser = argparse.ArgumentParser(
        prog="mcp",
        description="MCP lifecycle management (local stdio servers, pinned via scripts/agents/mcp-servers.json).",
    )
    sub = parser.add_subparsers(dest="command", metavar="<subcommand>")
    setup = sub.add_parser("setup", help="install/verify every server (npm: pinned pkg; pixi: env present + version)")
    setup.set_defaults(func=cmd_setup)
    index = sub.add_parser("index", help="rebuild every index: .codegraph + Serena LSP cache (.serena/cache)")
    index.set_defaults(func=cmd_index)
    sync = sub.add_parser("sync", help="codegraph incremental sync + Serena LSP-cache re-index")
    sync.set_defaults(func=cmd_sync)
    status = sub.add_parser("status", help="show pin vs installed versions and each server's index/cache state")
    status.set_defaults(func=cmd_status)
    latest = sub.add_parser("latest", help="report registry latest vs the pin (report only; never floats a pin)")
    latest.add_argument("server", nargs="?", help="limit to one registry-pinned server name")
    latest.set_defaults(func=cmd_latest)
    return parser


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    handler = getattr(args, "func", None)
    if handler is None:
        parser.print_help(sys.stderr)
        return 2
    return handler(args)


if __name__ == "__main__":
    sys.exit(main())
