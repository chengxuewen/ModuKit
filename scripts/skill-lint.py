#!/usr/bin/env python3
"""skill-lint: executable check for the skill tree (C0 convention).

Usage:
    python3 scripts/skill-lint.py [--skills .agents/skills] [--registry SKILL.md]

Checks every <skills>/*/SKILL.md:
  1. SKILL.md exists
  2. YAML frontmatter block with non-empty `name` and `description`
  3. name == directory name
  4. description length <= 1024 chars
  5. file size <= 800 lines (repo file-size rule)
  6. every `@path` reference in the body resolves inside the skill dir
Registry check (when --registry given and file exists):
  7. every skill directory is mentioned in the registry

Exit code: 0 = clean, 1 = at least one FAIL.
Negative proof (C0): point --skills at a fixture with a broken skill -> must exit 1.
"""
import argparse
import os
import re
import sys

MAX_DESC = 1024
MAX_LINES = 800
SKILL_REL = "SKILL.md"
REF_RE = re.compile(r"@((?:\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:md|py|sh|json|toml|yaml|yml|cjs|mjs|js|ts|txt))")

findings = []  # (severity, skill, message)


def parse_frontmatter(text):
    """Return dict of simple frontmatter keys, or None if no block."""
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return None
    end = None
    for i in range(1, min(len(lines), 60)):
        if lines[i].strip() == "---":
            end = i
            break
    if end is None:
        return None
    fm = {}
    key = None
    for line in lines[1:end]:
        m = re.match(r"^([A-Za-z_][A-Za-z0-9_-]*):\s*(.*)$", line)
        if m:
            key = m.group(1)
            val = m.group(2).strip()
            if val in (">", "|", ">-", "|-", ">+", "|+"):
                val = ""
            fm[key] = val
        elif key and line.startswith((" ", "\t")):
            fm[key] = (fm.get(key, "") + " " + line.strip()).strip()
    return fm


def check_skill(skill_dir):
    name = os.path.basename(skill_dir.rstrip("/"))
    path = os.path.join(skill_dir, SKILL_REL)
    if not os.path.isfile(path):
        findings.append(("FAIL", name, f"{SKILL_REL} missing"))
        return name
    text = open(path, encoding="utf-8", errors="replace").read()

    fm = parse_frontmatter(text)
    if fm is None:
        findings.append(("FAIL", name, "no YAML frontmatter block"))
    else:
        if not fm.get("name"):
            findings.append(("FAIL", name, "frontmatter: name missing/empty"))
        elif fm.get("name") != name:
            findings.append(("FAIL", name, f"frontmatter name '{fm.get('name')}' != dir name"))
        desc = fm.get("description", "")
        if not desc:
            findings.append(("FAIL", name, "frontmatter: description missing/empty"))
        elif len(desc) > MAX_DESC:
            findings.append(("FAIL", name, f"description {len(desc)} chars > {MAX_DESC}"))

    nlines = text.count("\n") + (0 if text.endswith("\n") else 1)
    if nlines > MAX_LINES:
        findings.append(("FAIL", name, f"{nlines} lines > {MAX_LINES} (split the skill)"))

    for lineno, line in enumerate(text.split("\n"), 1):
        for ref in REF_RE.findall(line):
            if ref.startswith("http"):
                continue
            target = os.path.normpath(os.path.join(skill_dir, ref.lstrip("./")))
            if not os.path.exists(target):
                findings.append(("FAIL", name, f"line {lineno}: @{ref} unresolved"))

    return name


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--skills", default=".agents/skills")
    ap.add_argument("--registry", default="SKILL.md")
    args = ap.parse_args()

    root = args.skills
    if not os.path.isdir(root):
        print(f"FAIL [lint] skills dir not found: {root}")
        return 1

    names = []
    for entry in sorted(os.listdir(root)):
        d = os.path.join(root, entry)
        if os.path.isdir(d) and not entry.startswith("."):
            names.append(check_skill(d))

    if args.registry and os.path.isfile(args.registry):
        reg = open(args.registry, encoding="utf-8", errors="replace").read()
        for n in names:
            if n not in reg:
                findings.append(("FAIL", n, f"not mentioned in registry {args.registry}"))

    fails = [f for f in findings if f[0] == "FAIL"]
    for sev, skill, msg in findings:
        print(f"{sev} [{skill}] {msg}")
    print(f"-- skill-lint: {len(names)} skills, {len(fails)} fail, {len(findings) - len(fails)} warn")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
