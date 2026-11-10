"""Behavior tests for the agent dispatcher chain (hearing #39, backlog P13).

These tests run the real `agents-cli.py` in a subprocess. Two assertions
(agents-engineering-plan B2):
  1. sync idempotence, measured at REPO level - two consecutive `sync` runs
     leave every generated layer untouched (clean `git diff`; the scratch
     fixture from test_agent_config.py cannot host a full sync because sync
     needs the whole .agents tree plus every generated mirror).
  2. planted drift - corrupting the scratch memorys source must RED the
     `check` command (memory-aggregate detects the hub digest drift).

They do not call `gate-agents`, so there is no gate -> tests -> gate
recursion. Note: assertion 1 performs real `sync` runs; inside gate-agents
the tree is already synced (sync precedes unittest discover), so the runs
are no-ops and the diff stays clean. Run standalone, the first sync may
regenerate drifted layers once - that rewrite is the fixture doing its job.
"""

import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
AGENTS_CLI = os.path.join(REPO, "scripts", "agents", "agents-cli.py")

# Governance-face snapshot copied into each scratch root. scripts/ is needed
# whole: agent-links' loader loads layer modules from ac.ROOT/scripts/. The
# generated mirrors travel along so `check` has something to compare against.
COPY_TREES = ("scripts", ".agents", ".codebuddy")
COPY_FILES = ("AGENTS.md", "SKILL.md", "CONSTRAINTS.md", ".mcp.json",
              ".opencode/opencode.json")
COPY_IGNORE = shutil.ignore_patterns("__pycache__", "node_modules", "*.pyc")
# symlinks=True: .codebuddy is a LINK layer - copying links as real files
# would itself read as drift and red the check for the wrong reason.
COPY_SYMLINKS = True

SYNC_LAYERS = ("AGENTS.md", "SKILL.md", "CONSTRAINTS.md", ".mcp.json",
               ".opencode/opencode.json")


def run_cli(command, root=None):
    """Run agents-cli. root=None -> real repo; else scratch (dual env vars)."""
    env = os.environ.copy()
    if root is not None:
        env["MODUKIT_AGENT_TESTS"] = "1"
        env["MODUKIT_AGENT_ROOT"] = root
    return subprocess.run(
        [sys.executable, AGENTS_CLI, command],
        env=env,
        cwd=root or REPO,
        capture_output=True,
        text=True,
        timeout=300,
    )


def make_scratch():
    """Full governance-face snapshot; too big for per-test setup, so tests
    share one class-level copy and mutate copies of the planted file."""
    root = tempfile.mkdtemp(prefix="agents-cli-selftest.")
    for tree in COPY_TREES:
        shutil.copytree(os.path.join(REPO, tree), os.path.join(root, tree),
                        ignore=COPY_IGNORE, symlinks=COPY_SYMLINKS)
    os.makedirs(os.path.join(root, ".opencode"), exist_ok=True)
    for rel in COPY_FILES:
        shutil.copy2(os.path.join(REPO, rel), os.path.join(root, rel))
    return root


def layer_hashes(root):
    """Hash the generated layers + the git worktree face of the repo."""
    hashes = {}
    for rel in SYNC_LAYERS:
        path = os.path.join(root, rel)
        with open(path, "rb") as fh:
            hashes[rel] = hashlib.sha256(fh.read()).hexdigest()
    git = subprocess.run(["git", "-C", REPO, "diff", "--porcelain"],
                         capture_output=True, text=True)
    hashes["git_diff"] = hashlib.sha256(git.stdout.encode()).hexdigest()
    return hashes


class TestDispatcherChain(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.scratch = make_scratch()

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.scratch, ignore_errors=True)

    def test_sync_is_idempotent_at_repo_level(self):
        first = run_cli("sync")
        self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
        before = layer_hashes(REPO)
        second = run_cli("sync")
        self.assertEqual(second.returncode, 0, second.stdout + second.stderr)
        after = layer_hashes(REPO)
        self.assertEqual(before, after, "second sync changed generated layers")

    def test_planted_memory_drift_reds_check(self):
        # Plant inside the digest WINDOW: the AGENTS.md digest carries D-entry
        # title lines (last 25), so editing the D110 line changes the hub
        # digest. A backlog edit would NOT drift - backlog digest rows are
        # pointer lines only (D47 window shape).
        target = os.path.join(self.scratch, ".agents", "memorys",
                              "decisions.md")
        with open(target, encoding="utf-8") as fh:
            text = fh.read()
        marker = "D110: Parent-to-child bus push accepted"
        self.assertIn(marker, text, "D110 anchor line not found for the plant")
        with open(target, "w", encoding="utf-8") as fh:
            fh.write(text.replace(marker, marker + " XX"))
        planted = run_cli("check", root=self.scratch)
        self.assertNotEqual(planted.returncode, 0,
                            "planted memory drift did NOT red the check")
        self.assertIn("memory region drift", planted.stdout)


if __name__ == "__main__":
    unittest.main()
