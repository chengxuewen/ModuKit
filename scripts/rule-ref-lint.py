#!/usr/bin/env python3
"""Every C/D/PIT citation in `.agents/rules` must resolve.

Resolution sources:
  1. local entries in `.agents/memorys/{conventions,decisions,pitfalls}.md`;
  2. the explicit external-provenance allowlist `scripts/rule-ref-allowlist.txt`.

A bare number with neither is red. `--self-test` plants PIT-999 in a probe tree
and requires a red result (C0).
"""

import os
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
RULES_ROOT = pathlib.Path(os.environ.get("RULES_ROOT", ROOT / ".agents" / "rules"))
if not RULES_ROOT.is_absolute():
    RULES_ROOT = ROOT / RULES_ROOT
ALLOWLIST = pathlib.Path(os.environ.get("RULE_REF_ALLOWLIST", ROOT / "scripts" / "rule-ref-allowlist.txt"))
if not ALLOWLIST.is_absolute():
    ALLOWLIST = ROOT / ALLOWLIST

LOCAL_FILES = {
    "conventions": ROOT / ".agents" / "memorys" / "conventions.md",
    "decisions": ROOT / ".agents" / "memorys" / "decisions.md",
    "pitfalls": ROOT / ".agents" / "memorys" / "pitfalls.md",
}
LOCAL_PATTERNS = {
    "conventions": re.compile(r"^## (C\d+)\b"),
    "decisions": re.compile(r"^## (D\d+)\b"),
    "pitfalls": re.compile(r"^#{2,3} (PIT-\d+)\b"),
}
REF = re.compile(r"\b(C\d+|D\d+|PIT-\d+)\b")


def local_ids():
    ids = set()
    for kind, path in LOCAL_FILES.items():
        if not path.exists():
            continue
        for line in path.read_text().splitlines():
            match = LOCAL_PATTERNS[kind].match(line)
            if match:
                ids.add(match.group(1))
    return ids


def allow_ids():
    ids = set()
    if not ALLOWLIST.exists():
        return ids
    for line in ALLOWLIST.read_text().splitlines():
        token = line.split("#", 1)[0].strip()
        if token:
            ids.add(token.split()[0])
    return ids


def scan(root, local, allow):
    findings = []
    for path in sorted(root.rglob("*.md")):
        for lineno, line in enumerate(path.read_text(errors="ignore").splitlines(), 1):
            for match in REF.finditer(line):
                ref = match.group(1)
                if ref not in local and ref not in allow:
                    findings.append("%s:%d: %s" % (path.relative_to(ROOT), lineno, ref))
    return sorted(set(findings))


def main():
    local = local_ids()
    allow = allow_ids()
    if "--self-test" in sys.argv:
        probe = ROOT / "out" / "rule-ref-probe"
        probe.mkdir(parents=True, exist_ok=True)
        (probe / "probe.md").write_text("- planted PIT-999\n")
        findings = scan(probe, local, allow)
        (probe / "probe.md").unlink(missing_ok=True)
        if findings:
            print("RULE-REF ✓ self-test (planted PIT-999 red)")
            return 0
        print("RULE-REF ✗ self-test did not red")
        return 1
    if not RULES_ROOT.is_dir():
        print("RULE-REF SKIP (no .agents/rules)")
        return 0
    findings = scan(RULES_ROOT, local, allow)
    if findings:
        print("\n".join(findings))
        print("RULE-REF ✗ %d unresolved citation(s)" % len(findings))
        return 1
    print("RULE-REF ✓ citations resolve (%d local, %d allowlisted)" % (len(local), len(allow)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
