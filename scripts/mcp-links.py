#!/usr/bin/env python3
"""Check / repair the per-host MCP activation layer.

Single source: <repo>/.agents/mcp.json (+ optional .agents/mcp.local.json for
credentials, gitignored). Targets: scripts/agent-targets.json.

Two host shapes (the command body is identical across hosts -- only the
envelope differs):
  - "mcpServers": write {"mcpServers": <merged>} to the host file
    (.mcp.json at the repo root for the project target).
    The mcpServers key shape is the canonical one, so transfer is a verbatim copy
    of the merged map (only `command` is wrapped in the repo-relative launcher).
  - "named": opencode's opencode.json uses a named-object mcp{} (type/enabled/
    command/environment). Build the FULL expected map from the canonical local
    servers plus `opencode_extra_mcp` (host-only remote/disabled entries), then
    REWRITE the mcp{} section. A merge here leaves removed servers behind
    forever (PIT-248).

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing written).

Per-target flags (scripts/agent-targets.json):
  - `optional`: the layer is gitignored and only exists on machines that ran
    sync for that host; an absent file is a SKIP in `check`, not a finding.
  - `structured_compare`: compare parsed JSON instead of bytes. A host that owns
    its config file re-indents it on save (Trae, D50 R1), so byte comparison
    would redden on formatting alone and rewrite the file on every sync.
"""

import json
import os
import re
import sys

import agent_common as ac

PREFIX = "GATE-MCP-LINKS"


def _merged_servers(cfg):
    src = os.path.join(ac.ROOT, cfg["mcp_source"])
    with open(src) as handle:
        canonical = json.load(handle).get("mcpServers", {})
    local = os.path.join(ac.ROOT, cfg["mcp_local"])
    if os.path.exists(local):
        with open(local) as handle:
            override = json.load(handle).get("mcpServers", {})
        for name, entry in override.items():
            base = dict(canonical.get(name, {}))
            base.update(entry)  # local wins (credentials)
            canonical[name] = base
    return canonical


# POSIX wrapper is referenced RELATIVE to the repo root: `mcp_file` is now the
# commit-tracked `.mcp.json`, which must stay machine-independent, and a
# project-scope MCP host spawns with cwd = workspace root. mcp-launch.sh resolves
# the repo root from BASH_SOURCE, so it does not depend on anything else.
LAUNCHER_SH = os.path.join("scripts", "agents", "mcp-launch.sh")
# Windows keeps the absolute path (escape hatch: a repo-relative .bat is not
# reliably executable through cmd); Windows devs get a local-only diff.
LAUNCHER_BAT = os.path.join(ac.ROOT, "scripts", "agents", "mcp-launch.bat")


def _host_entry(entry, target):
    """Canonical entry (`command` string + `args` list) -> host file entry.

    The MCP `mcpServers` schema requires `command` to be a STRING; a
    `command: ["bash", ...]` array makes CodeBuddy's panel die with a JS
    TypeError (`ir.replace is not a function`). A launcher host replaces the
    `pixi run <task>` prefix with the repo-relative wrapper and keeps the task
    plus any trailing args.
    """
    item = dict(entry)
    if not target.get("mcp_launcher"):
        return item
    command = item.get("command")
    args = list(item.get("args") or [])
    # Forward EVERYTHING after `run` verbatim: the canonical form may carry pixi flags
    # before the task name (`run -e serena mcp-serena`), and the launcher itself already
    # passes its argv straight through (`exec pixi run --manifest-path … "$@"`).
    if command == "pixi" and args[:1] == ["run"] and len(args) >= 2:
        tail = args[1:]
        if os.name == "nt":
            item["command"], item["args"] = LAUNCHER_BAT, tail
        elif target.get("mcp_launcher_absolute"):
            # A gitignored host file may carry a machine-local path; a tracked one
            # (root .mcp.json) may not. Needed because the launcher REFERENCE is
            # cwd-dependent while the launcher itself is not: hosts differ in the
            # spawn cwd (Trae does not start in the repo root, so a relative
            # `scripts/agents/mcp-launch.sh` dies with ENOENT / "no such file").
            item["command"] = "bash"
            item["args"] = [os.path.join(ac.ROOT, LAUNCHER_SH)] + tail
        else:
            item["command"], item["args"] = "bash", [LAUNCHER_SH] + tail
    return item


def _expected_mcp_file(merged, target):
    out = {}
    for name, entry in merged.items():
        out[name] = _host_entry(entry, target)
    return json.dumps({"mcpServers": out}, indent=2, sort_keys=True) + "\n"


def _same_doc(have, want, structured):
    """Is the file on disk already what the single source says?

    Two comparison modes (`structured_compare` on the target):
    - byte-ish (default): whitespace-insensitive string compare, as before.
    - structured: parse both and compare the objects. Some hosts own their
      config file and re-indent / re-key it on save (Trae, D50 R1); a formatting
      churn would otherwise redden the gate and rewrite the file on every sync.
    """
    if not structured:
        return have.strip() == want.strip()
    try:
        return json.loads(have) == json.loads(want)
    except ValueError:
        return False  # unparseable -> repair


def _report(command, target, file_rel, ok):
    """One line per target/file; absent optional layers are a SKIP, not a finding."""
    label = target["label"]
    if ok:
        print(" [ OK  ] %-10s %s" % (label, file_rel))
    elif command == "check":
        print(" [ FIX ] %-10s %s (mcpServers drift)" % (label, file_rel))
    else:
        print(" [ WRITE] %-10s %s" % (label, file_rel))


_ENV_REF = re.compile(r"^\$\{([A-Z_][A-Z0-9_]*)\}$")


def _opencode_value(value):
    """Canonical `${VAR}` (MCP/codebuddy/claude form) -> opencode `{env:VAR}`."""
    if isinstance(value, str):
        match = _ENV_REF.match(value)
        if match:
            return "{env:%s}" % match.group(1)
    return value


def _extra_guard_reasons(merged, extras):
    """Validate host-only extras before any generation or filesystem write."""
    reasons = []
    for name, entry in (extras or {}).items():
        if name in merged:
            reasons.append(f"opencode_extra_mcp[{name}] duplicates a canonical local server")
        if not isinstance(entry, dict):
            reasons.append(f"opencode_extra_mcp[{name}] must be an object")
            continue
        if entry.get("type") != "remote":
            reasons.append(
                f"opencode_extra_mcp[{name}]: only remote extras are allowed (got {entry.get('type')!r})"
            )
        url = entry.get("url")
        if not isinstance(url, str) or not url:
            reasons.append(f"opencode_extra_mcp[{name}]: remote url must be a non-empty string")
        if "enabled" in entry and not isinstance(entry["enabled"], bool):
            reasons.append(f"opencode_extra_mcp[{name}]: enabled must be boolean")
    return reasons


def _expected_opencode_mcp(merged, extras):
    """Build the complete opencode mcp{} map.

    `merged` is the canonical local set. `extras` is the host-only allowlist in
    agent-targets.json (currently the disabled remote `websearch`). Keep this
    one helper as the only definition of the opencode visible set (PIT-247).
    """
    out = {}
    for name, entry in merged.items():
        command = [entry.get("command", "")] + list(entry.get("args") or [])
        out[name] = {
            "type": "local",
            "enabled": bool(entry.get("enabled", True)),
            "command": command,
        }
        env = entry.get("env")
        if env:
            out[name]["environment"] = {k: _opencode_value(v) for k, v in env.items()}
    for name, entry in (extras or {}).items():
        item = dict(entry)
        if item.get("type") != "remote":
            # The project-local registry stays stdio-only; this allowlist is for
            # host-only remote/disabled entries. Reject anything else loudly.
            raise ValueError("opencode_extra_mcp[%s]: only remote extras are allowed" % name)
        item["enabled"] = bool(item.get("enabled", True))
        out[name] = item
    return out


def run(command, targets, cfg, dry_run):
    merged = _merged_servers(cfg)
    extras = cfg.get("opencode_extra_mcp", {})
    extra_reasons = _extra_guard_reasons(merged, extras)
    if extra_reasons:
        print("%s ✗ guard opencode_extra_mcp: %s" % (PREFIX, "; ".join(extra_reasons)))
        return ac.EXIT_GUARD
    if command == "list":
        for t in targets:
            if t.get("mcp_file"):
                print("%-10s mcp %s (%s)" % (t["label"], t.get("mcp_file"), t.get("mcp_shape")))
            elif t.get("opencode_file"):
                print("%-10s mcp %s (named)" % (t["label"], t.get("opencode_file")))
        return ac.EXIT_OK

    failures = 0
    if command == "check":
        for t in targets:
            if not t.get("enabled"):
                continue
            if t.get("mcp_file"):
                path = os.path.join(ac.ROOT, t["mcp_file"])
                if t.get("optional") and not os.path.exists(path):
                    print(" [ SKIP ] %-10s %s (layer absent)" % (t["label"], t["mcp_file"]))
                    continue
                want = _expected_mcp_file(merged, t)
                have = open(path).read() if os.path.exists(path) else ""
                ok = _same_doc(have, want, t.get("structured_compare"))
                if not ok:
                    failures += 1
                _report("check", t, t["mcp_file"], ok)
            elif t.get("opencode_file"):
                path = os.path.join(ac.ROOT, t["opencode_file"])
                with open(path) as handle:
                    cur = json.load(handle).get("mcp", {})
                want = _expected_opencode_mcp(merged, extras)
                if cur != want:
                    failures += 1
                    missing = sorted(set(want) - set(cur))
                    orphan = sorted(set(cur) - set(want))
                    changed = sorted(name for name in set(want) & set(cur) if cur[name] != want[name])
                    detail = []
                    if missing:
                        detail.append("missing=%s" % missing)
                    if orphan:
                        detail.append("orphan=%s" % orphan)
                    if changed:
                        detail.append("changed=%s" % changed)
                    print(" [ FIX ] %-10s %s (named mcp drift: %s)"
                          % (t["label"], t["opencode_file"], ", ".join(detail) or "unknown"))
                else:
                    print(" [ OK  ] %-10s %s" % (t["label"], t["opencode_file"]))
        if failures:
            print("%s ✗ %d target(s) need repair (run: agent-links.py sync)" % (PREFIX, failures))
            return ac.EXIT_FINDINGS
        print("%s ✓ (%d server(s) × %d target(s))" % (PREFIX, len(merged), sum(1 for t in targets if t.get("enabled"))))
        return ac.EXIT_OK

    # sync / dry-run: preflight every write so one bad target cannot leave a
    # partial generation behind when a later target hits a guard.
    canonical_dir = os.path.dirname(os.path.join(ac.ROOT, cfg["mcp_source"]))
    writes = []
    for t in targets:
        if not t.get("enabled"):
            continue
        if t.get("mcp_file"):
            path = os.path.join(ac.ROOT, t["mcp_file"])
            reasons = ac.target_write_reasons(path, canonical_dir)
            if reasons:
                print("%s ✗ guard %s: %s" % (PREFIX, t["mcp_file"], "; ".join(reasons)))
                return ac.EXIT_GUARD
            want = _expected_mcp_file(merged, t)
            have = open(path).read() if os.path.exists(path) else ""
            if os.path.exists(path) and _same_doc(have, want, t.get("structured_compare")):
                print(" [ OK  ] %-10s %s" % (t["label"], t["mcp_file"]))
                continue
            reasons = ac.target_write_reasons(path, canonical_dir)
            if reasons:
                print("%s ✗ guard %s: %s" % (PREFIX, t["mcp_file"], "; ".join(reasons)))
                return ac.EXIT_GUARD
            print(" [ WRITE] %-10s %s" % (t["label"], t["mcp_file"]))
            writes.append(("file", t, path, want))
        elif t.get("opencode_file"):
            path = os.path.join(ac.ROOT, t["opencode_file"])
            reasons = ac.target_write_reasons(path, canonical_dir)
            if reasons:
                print("%s ✗ guard %s: %s" % (PREFIX, t["opencode_file"], "; ".join(reasons)))
                return ac.EXIT_GUARD
            with open(path) as handle:
                doc = json.load(handle)
            want = _expected_opencode_mcp(merged, extras)
            cur = doc.get("mcp", {})
            if cur == want:
                print(" [ OK  ] %-10s %s" % (t["label"], t["opencode_file"]))
                continue
            reasons = ac.target_write_reasons(path, canonical_dir)
            if reasons:
                print("%s ✗ guard %s: %s" % (PREFIX, t["opencode_file"], "; ".join(reasons)))
                return ac.EXIT_GUARD
            print(" [ REWRITE] %-10s %s" % (t["label"], t["opencode_file"]))
            writes.append(("opencode", t, path, (doc, want)))

    if dry_run:
        return ac.EXIT_OK

    for kind, t, path, payload in writes:
        if kind == "file":
            with open(path, "w") as handle:
                handle.write(payload)
            continue
        doc, want = payload
        backup = ac.backup_name(os.path.dirname(path), os.path.basename(path))
        os.makedirs(os.path.dirname(backup), exist_ok=True)
        import shutil
        shutil.copy2(path, backup)  # never lose the assembly point
        doc["mcp"] = want
        with open(path, "w") as handle:
            json.dump(doc, handle, indent=2, sort_keys=False)
            handle.write("\n")
    print("%s ✓ sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    command = sys.argv[1] if len(sys.argv) > 1 else "check"
    if command not in ("check", "sync", "dry-run", "list"):
        print(__doc__.strip())
        sys.exit(0)
    cfg = ac.load_json(os.path.join(ac.ROOT, "scripts", "agent-targets.json"))
    sys.exit(run(command, cfg["targets"], cfg, command == "dry-run"))
