#!/usr/bin/env python3
"""Repository-side verification of the CodeBuddy host layer.

This checks the contract the host reads: AGENTS.md fallback, generated
`.codebuddy/rules/*/RULE.mdc`, generated `.codebuddy/skills/*` links, and the
root `.mcp.json`. It cannot verify what an already-running IDE session loaded;
restart CodeBuddy and inspect its skill/rule/MCP panels for that last step.

Usage: python3 scripts/verify-codebuddy-host.py
Exit: 0 all checks pass; 1 findings.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAILS = []
CHECKS = 0


def check(ok, label, detail=""):
    global CHECKS
    CHECKS += 1
    if ok:
        print(" [ OK  ] %s%s" % (label, (" - " + detail) if detail else ""))
    else:
        print(" [ FAIL] %s%s" % (label, (" - " + detail) if detail else ""))
        FAILS.append(label)


def read(path):
    with open(path, "r", encoding="utf-8") as handle:
        return handle.read()


def frontmatter(text):
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    out = {}
    for line in text[4:end].splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            out[key.strip()] = value.strip().strip('"')
    return out


def main():
    targets = json.loads(read(os.path.join(ROOT, "scripts", "agent-targets.json")))
    codebuddy = next((t for t in targets.get("targets", []) if t.get("label") == "codebuddy"), None)
    check(codebuddy is not None, "agent-targets.json has a codebuddy target")
    if codebuddy is None:
        return 1
    check(codebuddy.get("optional") is True, "codebuddy target is optional")
    check(codebuddy.get("mcp") is False and not codebuddy.get("mcp_file"), "codebuddy does not write MCP")
    check(codebuddy.get("rules_mode") == "mdc", "codebuddy rules mode is mdc")
    check(codebuddy.get("skills_delivery") == "link", "codebuddy skills delivery is link")

    agents = os.path.join(ROOT, "AGENTS.md")
    check(os.path.isfile(agents), "AGENTS.md exists")
    check(not os.path.exists(os.path.join(ROOT, "CODEBUDDY.md")), "CODEBUDDY.md absent (AGENTS.md fallback)")
    if os.path.isfile(agents):
        text = read(agents)
        check("D109" in text, "AGENTS.md carries D109")
        rows = re.findall(r"(?m)^\| `[^`]+`(?: \([^)]*\))? \|", text)
        check(len(rows) >= 41, "AGENTS.md skill directory has >= 41 rows", str(len(rows)))

    rules_root = os.path.join(ROOT, codebuddy.get("rules_dir", ".codebuddy/rules"))
    langs = set(codebuddy.get("langs") or [])
    canonical = os.path.join(ROOT, targets["rules_root"])
    expected = set()
    for lang in sorted(langs):
        lang_dir = os.path.join(canonical, lang)
        if not os.path.isdir(lang_dir):
            continue
        for name in sorted(os.listdir(lang_dir)):
            if name.endswith(".md"):
                expected.add("%s-%s" % (lang, name[:-3]))
    actual = set()
    if os.path.isdir(rules_root):
        actual = {name for name in os.listdir(rules_root) if os.path.isdir(os.path.join(rules_root, name))}
    check(os.path.isdir(rules_root), "rules layer exists", codebuddy.get("rules_dir", ""))
    check(actual == expected, "rules set matches common+rust", "%d expected / %d actual" % (len(expected), len(actual)))
    always = set(codebuddy.get("always_on_rules") or [])
    for folder in sorted(expected & actual):
        rule = os.path.join(rules_root, folder, "RULE.mdc")
        if not os.path.isfile(rule):
            check(False, "rule file exists", folder)
            continue
        fm = frontmatter(read(rule))
        missing = [key for key in ("description", "alwaysApply", "enabled", "updatedAt") if not fm.get(key)]
        check(not missing, "rule frontmatter", "%s missing=%s" % (folder, missing))
        want_apply = "true" if folder in always else "false"
        check(fm.get("alwaysApply") == want_apply, "alwaysApply", "%s=%s" % (folder, fm.get("alwaysApply")))
        for target in re.findall(r"\]\(([^)]+)\)", read(rule)):
            if target.startswith(("http://", "https://", "#")):
                continue
            if not os.path.exists(os.path.normpath(os.path.join(os.path.dirname(rule), target))):
                check(False, "rule link resolves", "%s -> %s" % (folder, target))
    check(not os.path.exists(os.path.join(ROOT, ".codebuddy", "mcp.json")), "no .codebuddy/mcp.json")

    skills_root = os.path.join(ROOT, codebuddy.get("skills_dir", ".codebuddy/skills"))
    canonical_skills = os.path.join(ROOT, targets["skills_root"])
    expected_skills = {
        name
        for name in os.listdir(canonical_skills)
        if os.path.isdir(os.path.join(canonical_skills, name))
        and os.path.isfile(os.path.join(canonical_skills, name, "SKILL.md"))
    }
    actual_skills = set(os.listdir(skills_root)) if os.path.isdir(skills_root) else set()
    check(os.path.isdir(skills_root), "skills layer exists", codebuddy.get("skills_dir", ""))
    check(actual_skills == expected_skills, "skills set matches canonical", "%d expected / %d actual" % (len(expected_skills), len(actual_skills)))
    bad_links = []
    for name in sorted(expected_skills & actual_skills):
        link = os.path.join(skills_root, name)
        if not os.path.islink(link):
            bad_links.append(name + " (not a symlink)")
            continue
        want = os.path.realpath(os.path.join(canonical_skills, name))
        if os.path.realpath(link) != want:
            bad_links.append(name + " (wrong target)")
    check(not bad_links, "skills links resolve to canonical", ", ".join(bad_links[:5]))

    mcp_path = os.path.join(ROOT, ".mcp.json")
    check(os.path.isfile(mcp_path), ".mcp.json exists")
    if os.path.isfile(mcp_path):
        data = json.loads(read(mcp_path))
        servers = data.get("mcpServers", {})
        check(len(servers) == 3, ".mcp.json has 3 servers", str(sorted(servers)))
        bad_cmd = [name for name, entry in servers.items() if not isinstance(entry.get("command"), str)]
        check(not bad_cmd, ".mcp.json command is a string", ", ".join(bad_cmd))

    result = subprocess.run(
        [sys.executable, os.path.join(ROOT, "scripts", "agents-cli.py"), "check"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    check(result.returncode == 0, "agents-cli.py check", "exit=%d" % result.returncode)

    print("CODEBUDDY-HOST %s (%d checks, %d findings)" % ("PASS" if not FAILS else "FAIL", CHECKS, len(FAILS)))
    if FAILS:
        print("findings: " + "; ".join(FAILS))
        return 1
    print("note: IDE session visibility still requires a CodeBuddy restart")
    return 0


if __name__ == "__main__":
    sys.exit(main())
