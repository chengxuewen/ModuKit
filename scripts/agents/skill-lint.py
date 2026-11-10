#!/usr/bin/env python3
"""skill-lint: executable check for the skill tree (C0 convention).

Usage:
    python3 scripts/agents/skill-lint.py [--skills .agents/skills] [--registry SKILL.md]

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
Negative proof (C0): `--self-test` builds a scratch tree, plants a name
mismatch, a dangling @ref and an unmounted trigger owner (all three must red
in one run), then asserts a clean scratch is green. Repo files never touched:
the child run is re-exec'd with cwd inside the scratch root, so every
repo-relative default (.agents/skills, TRIGGERS.md, SKILL.md) redirects.
"""
import argparse
import os
import re
import sys

MAX_DESC = 1024
MAX_LINES = 800
SKILL_REL = "SKILL.md"
REF_RE = re.compile(r"@((?:\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:md|py|sh|json|toml|yaml|yml|cjs|mjs|js|ts|txt))")
SKILL_REF_RE = re.compile(r"See skill:\s*`([a-z0-9-]+)`")
CMD_REF_RE = re.compile(r"(?<![A-Za-z0-9_])/([a-z][a-z0-9-]{2,})(?![A-Za-z0-9_])")
EXTERNAL_COMMANDS = {"brainstorming"}
GHOST_NAMES = re.compile(r"(?<![A-Za-z0-9-])(systematic-debugging|remove-ai-slops|security-review)(?![A-Za-z0-9-])")
TRIGGERS_REL = os.path.join(".agents", "skills", "TRIGGERS.md")

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


def check_references(names):
    """Mounted rules and skill-router must only name real skills/commands."""
    for pack in ("common", "rust"):
        base = os.path.join(".agents", "rules", pack)
        if not os.path.isdir(base):
            continue
        for dirpath, _dirnames, filenames in os.walk(base):
            for filename in sorted(filenames):
                if not filename.endswith(".md"):
                    continue
                path = os.path.join(dirpath, filename)
                for lineno, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
                    for ref in SKILL_REF_RE.findall(line):
                        if ref not in names:
                            rel = os.path.relpath(path)
                            findings.append(("FAIL", "refs", f"{rel}:{lineno}: See skill `{ref}` unresolved"))
    router = os.path.join(".agents", "skills", "skill-router", "SKILL.md")
    if os.path.isfile(router):
        for lineno, line in enumerate(open(router, encoding="utf-8", errors="replace").read().split("\n"), 1):
            for name in CMD_REF_RE.findall(line):
                if name not in names and name not in EXTERNAL_COMMANDS:
                    findings.append(("FAIL", "refs", f"{router}:{lineno}: command `/{name}` unresolved"))
    for skill in sorted(names):
        path = os.path.join(".agents", "skills", skill, "SKILL.md")
        if not os.path.isfile(path):
            continue
        for lineno, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
            for ghost in GHOST_NAMES.findall(line):
                findings.append(("FAIL", "refs", f"{path}:{lineno}: ghost skill `{ghost}`"))


def check_trigger_matrix(names):
    """Every phrase in TRIGGERS.md has one owner and that owner exists."""
    if not os.path.isfile(TRIGGERS_REL):
        findings.append(("FAIL", "triggers", f"trigger matrix missing: {TRIGGERS_REL}"))
        return

    rows = []
    for line in open(TRIGGERS_REL, encoding="utf-8", errors="replace"):
        if not line.strip().startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        rows.append(cells)
    if not rows:
        findings.append(("FAIL", "triggers", "trigger matrix has no table rows"))
        return

    header = [cell.lower() for cell in rows[0]]
    try:
        intent_i = next(i for i, cell in enumerate(header) if "intent" in cell)
        phrase_i = next(i for i, cell in enumerate(header) if "trigger phrase" in cell)
        owner_i = next(i for i, cell in enumerate(header) if "owning skill" in cell)
    except StopIteration:
        findings.append(("FAIL", "triggers", "matrix header must name Intent, Trigger phrases, and Owning skill"))
        return

    phrase_owners: dict[str, set[str]] = {}
    for row_no, cells in enumerate(rows[1:], 2):
        if set(cells) <= {"", "---", ":---", ":---:", "---:"}:
            continue
        if len(cells) <= max(intent_i, phrase_i, owner_i):
            findings.append(("FAIL", "triggers", f"row {row_no}: missing cells"))
            continue

        def clean(value):
            return value.replace("`", "").replace("*", "").strip().lower()

        owner = clean(cells[owner_i])
        if owner not in names:
            findings.append(("FAIL", "triggers", f"row {row_no}: owner `{owner}` is not a mounted skill"))
        phrases = [clean(part) for part in cells[phrase_i].split(",") if clean(part)]
        if not phrases:
            findings.append(("FAIL", "triggers", f"row {row_no}: no trigger phrases"))
            continue
        for phrase in phrases:
            phrase_owners.setdefault(phrase, set()).add(owner)

    for phrase, owners in sorted(phrase_owners.items()):
        if len(owners) > 1:
            findings.append(("FAIL", "triggers", f"duplicate trigger `{phrase}` owned by {sorted(owners)}"))


def self_test():
    """C0 replay (#35 rollout, backlog P7): three planted defects must red."""
    import shutil
    import subprocess
    import tempfile

    def write(path, text):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(text)

    def run(cwd):
        return subprocess.run(
            [sys.executable, os.path.abspath(__file__)],
            cwd=cwd, capture_output=True, text=True,
        )

    header = "| Intent | Trigger phrases | Owning skill |\n|---|---|---|\n"
    s = tempfile.mkdtemp(prefix="skill-lint-selftest.")
    try:
        sk = os.path.join(s, ".agents", "skills")
        write(os.path.join(sk, "good-skill", "SKILL.md"),
              "---\nname: good-skill\ndescription: probe skill for the self-test\n---\nbody\n")
        write(os.path.join(sk, "TRIGGERS.md"),
              header + "| probe intent | probe phrase | good-skill |\n")
        write(os.path.join(s, "SKILL.md"), "registry mentions good-skill\n")
        r = run(s)
        if r.returncode != 0:
            print("SKILL-LINT ✗ self-test: clean scratch did not green")
            print(r.stdout)
            return 1
        write(os.path.join(sk, "bad-name", "SKILL.md"),
              "---\nname: wrong-name\ndescription: probe\n---\nbody\n")
        write(os.path.join(sk, "dangling-ref", "SKILL.md"),
              "---\nname: dangling-ref\ndescription: probe\n---\nsee @./nope.md\n")
        write(os.path.join(sk, "TRIGGERS.md"),
              header + "| probe intent | probe phrase | good-skill |\n"
                       "| ghost intent | ghost phrase | ghost-skill |\n")
        r = run(s)
        for expected in ("!= dir name", "unresolved", "ghost-skill` is not a mounted skill"):
            if r.returncode == 0 or expected not in r.stdout:
                print(f"SKILL-LINT ✗ self-test: planted defect not caught ({expected})")
                print(r.stdout)
                return 1
        print("SKILL-LINT ✓ self-test (name mismatch, dangling @ref, ghost owner all red; clean green)")
        return 0
    finally:
        shutil.rmtree(s, ignore_errors=True)


def main():
    if "--self-test" in sys.argv:
        return self_test()
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

    check_references(set(names))
    check_trigger_matrix(set(names))

    fails = [f for f in findings if f[0] == "FAIL"]
    for sev, skill, msg in findings:
        print(f"{sev} [{skill}] {msg}")
    print(f"-- skill-lint: {len(names)} skills, {len(fails)} fail, {len(findings) - len(fails)} warn")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
