#!/usr/bin/env python3
"""Forbid absolute home-path literals in the scripts/agents layer.

Variant transplant of the MediaServo donor's check_no_parent.py @ 9d22cba
(parent-workspace literals, its C15): ModuKit is a top-level repo with no
parent workspace, so the same-family risk here is absolute HOME paths
(`/home/`, `/Users/`) - PIT-34 kin, and a real break under the win-64
platform promise. Scans scripts/ and .agents/; skips __pycache__,
node_modules and memorys (incident notes legitimately quote absolute
paths; the donor skips memorys for the same reason, line 26). A listed
file in abs-path-allowlist.txt (beside this script, per scan root) is
exempt whole-file; everything around it stays guarded.

--self-test plants both literals in a scratch root (MODUKIT_ABS_ROOT
redirects the scan roots), asserts RED, honours the scratch allowlist,
then asserts a clean scratch is green.
"""
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_DEFAULT_ROOT = os.path.dirname(_HERE)
ROOT = os.path.abspath(os.environ["MODUKIT_ABS_ROOT"]) if os.environ.get("MODUKIT_ABS_ROOT") else _DEFAULT_ROOT
ALLOW = os.path.join(ROOT, "scripts", "abs-path-allowlist.txt")
PATTERNS = ("/home/", "/Users/")
ROOTS = (os.path.join(ROOT, "scripts"), os.path.join(ROOT, ".agents"))
SKIP_DIRS = {"__pycache__", "node_modules", "memorys"}
SKIP_EXT = {".pyc", ".png", ".zip", ".gz"}
PREFIX = "ABS-PATH"


def allowed_files():
    if not os.path.isfile(ALLOW):
        return set()
    with open(ALLOW, encoding="utf-8") as fh:
        return {line.strip() for line in fh
                if line.strip() and not line.startswith("#")}


def scan(root, allow):
    fail = 0
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in sorted(files):
            if os.path.splitext(name)[1] in SKIP_EXT:
                continue
            path = os.path.join(base, name)
            rel = os.path.relpath(path, ROOT)
            if rel in allow:
                continue
            try:
                with open(path, encoding="utf-8") as fh:
                    text = fh.read()
            except (OSError, UnicodeDecodeError):
                continue
            for line_no, line in enumerate(text.splitlines(), 1):
                if any(p in line for p in PATTERNS):
                    print("%s ✗ %s:%d %s" % (PREFIX, rel, line_no, line.strip()[:100]))
                    fail = 1
    return fail


def self_test():
    import shutil
    import subprocess
    import tempfile

    self_path = os.path.abspath(__file__)
    s = tempfile.mkdtemp(prefix="abs-path-selftest.")
    try:
        os.makedirs(os.path.join(s, "scripts"))
        os.makedirs(os.path.join(s, ".agents", "memorys"))
        # memorys is skipped by design: a literal planted there must NOT red.
        with open(os.path.join(s, ".agents", "memorys", "notes.md"), "w",
                  encoding="utf-8") as fh:
            fh.write("incident note quoting /home/ubuntu/old - exempt\n")

        def run():
            env = dict(os.environ, MODUKIT_ABS_ROOT=s)
            return subprocess.run([sys.executable, self_path], env=env,
                                  capture_output=True, text=True)

        # plant 1 + 2: both patterns must be reported...
        with open(os.path.join(s, "scripts", "probe-home.md"), "w",
                  encoding="utf-8") as fh:
            fh.write("fixture /home/ubuntu/probe.txt\n")
        with open(os.path.join(s, "scripts", "probe-users.md"), "w",
                  encoding="utf-8") as fh:
            fh.write("fixture /Users/dev/probe.txt\n")
        r = run()
        if r.returncode == 0:
            print("%s ✗ self-test: planted literals NOT caught" % PREFIX)
            print(r.stdout)
            return 1
        for expect in ("probe-home.md", "probe-users.md"):
            if expect not in r.stdout:
                print("%s ✗ self-test: %s not reported" % (PREFIX, expect))
                return 1
        if "notes.md" in r.stdout:
            print("%s ✗ self-test: memorys skip not honoured" % PREFIX)
            return 1
        # ...the scratch allowlist must exempt a listed file...
        with open(os.path.join(s, "scripts", "abs-path-allowlist.txt"), "w",
                  encoding="utf-8") as fh:
            fh.write("# self-test scratch\nscripts/probe-home.md\n")
        r = run()
        if "probe-home.md" in r.stdout:
            print("%s ✗ self-test: allowlist not honoured" % PREFIX)
            return 1
        # ...and the clean root must be green.
        os.remove(os.path.join(s, "scripts", "probe-users.md"))
        r = run()
        if r.returncode != 0:
            print("%s ✗ self-test: clean scratch did not green" % PREFIX)
            print(r.stdout)
            return 1
        print("%s ✓ self-test (both patterns red, memorys skipped, allowlist honoured, clean green)" % PREFIX)
        return 0
    finally:
        shutil.rmtree(s, ignore_errors=True)


def main():
    if "--self-test" in sys.argv:
        return self_test()
    allow = allowed_files()
    fail = 0
    for root in ROOTS:
        if os.path.isdir(root):
            fail |= scan(root, allow)
    if fail:
        print("%s ✗ absolute home-path literals present" % PREFIX)
        return 1
    print("%s ✓ no /home/ or /Users/ literals in scripts/ or .agents/" % PREFIX)
    return 0


if __name__ == "__main__":
    sys.exit(main())
