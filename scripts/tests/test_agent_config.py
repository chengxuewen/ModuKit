"""Behavior tests for the agent single-source generators.

These tests run the real `mcp-links.py` in a subprocess against a temporary
`MODUKIT_AGENT_ROOT`. They do not call `gate-agents`, so there is no
gate -> tests -> gate recursion.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
MCP_SCRIPT = os.path.join(REPO, "scripts", "mcp-links.py")
LOCAL_SERVERS = ("local-codegraph", "local-playwright", "local-serena")


def run_mcp_links(command, root):
    env = os.environ.copy()
    env["MODUKIT_AGENT_TESTS"] = "1"
    env["MODUKIT_AGENT_ROOT"] = root
    return subprocess.run(
        [sys.executable, MCP_SCRIPT, command],
        env=env,
        cwd=root,
        capture_output=True,
        text=True,
    )


def make_targets(root):
    scripts = os.path.join(root, "scripts")
    os.makedirs(scripts, exist_ok=True)
    cfg = {
        "mcp_source": ".agents/mcp.json",
        "mcp_local": ".agents/mcp.local.json",
        "mcp_file_guard": ".mcp.json",
        "opencode_extra_mcp": {
            "websearch": {
                "type": "remote",
                "url": "https://websearch.opencode.ai",
                "enabled": False,
            }
        },
        "targets": [
            {
                "label": "opencode",
                "enabled": True,
                "mcp": True,
                "mcp_shape": "named",
                "opencode_file": ".opencode/opencode.json",
            },
            {
                "label": "project",
                "enabled": True,
                "mcp": True,
                "mcp_shape": "mcpServers",
                "mcp_file": ".mcp.json",
                "mcp_launcher": True,
            },
        ],
    }
    with open(os.path.join(scripts, "agent-targets.json"), "w", encoding="utf-8") as handle:
        json.dump(cfg, handle, indent=2)
        handle.write("\n")


def make_root():
    root = tempfile.mkdtemp(prefix="modukit-agent-test-")
    os.makedirs(os.path.join(root, ".agents"), exist_ok=True)
    shutil.copy2(
        os.path.join(REPO, ".agents", "mcp.json"),
        os.path.join(root, ".agents", "mcp.json"),
    )
    make_targets(root)
    return root


def make_opencode(root, ghost=False):
    path = os.path.join(root, ".opencode", "opencode.json")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    doc = {"mcp": {}}
    if ghost:
        doc["mcp"]["ghost-server"] = {
            "type": "local",
            "enabled": True,
            "command": ["echo", "ghost"],
        }
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(doc, handle, indent=2)
        handle.write("\n")
    return path


class GateMcpRoutingLockRegression(unittest.TestCase):
    """A routing-lock failure must be a clean red, never a crash.

    gate-mcp.py once incremented `red` before initializing it, so any
    `_routing_lock` problem raised UnboundLocalError and the gate died with a
    traceback instead of printing the lock findings (repo-audit Full
    2026-11-04; fixed 2026-11-05). This test keeps that fail path observable.
    """

    def test_lock_failure_counts_red_without_crash(self):
        import contextlib
        import importlib.util
        import io

        scripts_dir = os.path.join(REPO, "scripts")
        sys.path.insert(0, scripts_dir)
        try:
            spec = importlib.util.spec_from_file_location(
                "gate_mcp_under_test", os.path.join(scripts_dir, "gate-mcp.py")
            )
            mod = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(mod)
        finally:
            sys.path.remove(scripts_dir)
        mod._routing_lock = lambda servers: ["synthetic lock problem"]
        mod._run = lambda cmd, env: (None, "probe disabled in test")

        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = mod.main()

        self.assertEqual(code, 1)
        self.assertIn("tool-routing lock: synthetic lock problem", buffer.getvalue())
        self.assertIn("GATE-MCP", buffer.getvalue())


class AgentConfigBehavior(unittest.TestCase):
    def setUp(self):
        self.roots = []

    def tearDown(self):
        for root in self.roots:
            for dirpath, dirnames, _files in os.walk(root):
                for name in dirnames:
                    try:
                        os.chmod(os.path.join(dirpath, name), 0o755)
                    except OSError:
                        pass
            shutil.rmtree(root, ignore_errors=True)

    def temp_root(self, ghost=False):
        root = make_root()
        self.roots.append(root)
        make_opencode(root, ghost=ghost)
        return root

    def test_missing_sync_check_cycle_repairs_both_targets(self):
        root = self.temp_root()
        check = run_mcp_links("check", root)
        self.assertEqual(check.returncode, 1, check.stdout + check.stderr)

        sync = run_mcp_links("sync", root)
        self.assertEqual(sync.returncode, 0, sync.stdout + sync.stderr)

        check = run_mcp_links("check", root)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

        with open(os.path.join(root, ".mcp.json"), encoding="utf-8") as handle:
            project = json.load(handle)["mcpServers"]
        self.assertEqual(sorted(project), sorted(LOCAL_SERVERS))

        with open(os.path.join(root, ".opencode", "opencode.json"), encoding="utf-8") as handle:
            opencode = json.load(handle)["mcp"]
        self.assertEqual(sorted(opencode), sorted(LOCAL_SERVERS + ("websearch",)))
        self.assertFalse(opencode["websearch"]["enabled"])

    def test_generated_drift_is_repaired(self):
        root = self.temp_root()
        self.assertEqual(run_mcp_links("sync", root).returncode, 0)
        path = os.path.join(root, ".mcp.json")
        with open(path, encoding="utf-8") as handle:
            doc = json.load(handle)
        doc["mcpServers"]["local-codegraph"]["command"] = ["pixi", "run", "broken"]
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(doc, handle, indent=2)
            handle.write("\n")

        self.assertEqual(run_mcp_links("check", root).returncode, 1)
        self.assertEqual(run_mcp_links("sync", root).returncode, 0)
        self.assertEqual(run_mcp_links("check", root).returncode, 0)

        sync = run_mcp_links("sync", root)
        self.assertEqual(sync.returncode, 0, sync.stdout + sync.stderr)
        self.assertNotIn("[ WRITE]", sync.stdout)
        self.assertNotIn("[ REWRITE]", sync.stdout)

    def test_dry_run_writes_nothing(self):
        root = self.temp_root(ghost=True)
        with open(os.path.join(root, ".opencode", "opencode.json"), encoding="utf-8") as handle:
            before = handle.read()
        result = run_mcp_links("dry-run", root)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(os.path.exists(os.path.join(root, ".mcp.json")))
        with open(os.path.join(root, ".opencode", "opencode.json"), encoding="utf-8") as handle:
            after = handle.read()
        self.assertEqual(before, after)
        self.assertIn("ghost-server", after)

    def test_full_rewrite_removes_ghost_and_preserves_remote_extra(self):
        root = self.temp_root(ghost=True)
        self.assertEqual(run_mcp_links("sync", root).returncode, 0)
        with open(os.path.join(root, ".opencode", "opencode.json"), encoding="utf-8") as handle:
            mcp = json.load(handle)["mcp"]
        self.assertNotIn("ghost-server", mcp)
        self.assertEqual(
            mcp["websearch"],
            {
                "type": "remote",
                "url": "https://websearch.opencode.ai",
                "enabled": False,
            },
        )
        with open(os.path.join(root, ".mcp.json"), encoding="utf-8") as handle:
            project = json.load(handle)["mcpServers"]
        self.assertNotIn("websearch", project)
        self.assertEqual(sorted(project), sorted(LOCAL_SERVERS))

    def test_two_target_roots_are_identical_and_machine_independent(self):
        left = self.temp_root()
        right = self.temp_root()
        for root in (left, right):
            self.assertEqual(run_mcp_links("sync", root).returncode, 0)
        with open(os.path.join(left, ".mcp.json"), encoding="utf-8") as handle:
            left_project = handle.read()
        with open(os.path.join(right, ".mcp.json"), encoding="utf-8") as handle:
            right_project = handle.read()
        self.assertEqual(left_project, right_project)
        for value in (left_project, right_project):
            self.assertNotIn(left, value)
            self.assertNotIn(right, value)

    def test_symlinked_parent_is_guarded_without_a_partial_write(self):
        root = make_root()
        self.roots.append(root)
        canonical = os.path.join(root, ".agents")
        os.symlink(canonical, os.path.join(root, ".opencode"))
        self.assertEqual(run_mcp_links("sync", root).returncode, 2)
        self.assertFalse(os.path.exists(os.path.join(canonical, "opencode.json")))
        self.assertFalse(os.path.exists(os.path.join(root, ".mcp.json")))

    @unittest.skipIf(
        hasattr(os, "geteuid") and os.geteuid() == 0,
        "root ignores ordinary read-only permission bits",
    )
    def test_readonly_target_is_guarded(self):
        root = self.temp_root()
        self.assertEqual(run_mcp_links("sync", root).returncode, 0)
        path = os.path.join(root, ".mcp.json")
        with open(path, encoding="utf-8") as handle:
            doc = json.load(handle)
        doc["mcpServers"]["local-codegraph"]["command"] = "broken"
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(doc, handle, indent=2)
            handle.write("\n")
        os.chmod(path, 0o444)
        try:
            result = run_mcp_links("sync", root)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("not writable", result.stdout)
        finally:
            os.chmod(path, 0o644)


if __name__ == "__main__":
    unittest.main()
