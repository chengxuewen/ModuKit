#!/usr/bin/env python3
"""Check / repair the per-host rules activation layer.

Single source: <repo>/.agents/rules/**/*.md -- the only editable copies.
Targets: `rules_dir` in scripts/agent-targets.json (one target file).

Modes, driven by each target's `rules_mode`:
- `mdc`: generate CodeBuddy-style `.codebuddy/rules/<lang>-<topic>/RULE.mdc`
  with deterministic frontmatter (description / alwaysApply / enabled /
  updatedAt), a `langs` filter, and cross-rule links retargeted to the
  generated layout. A differing existing `RULE.mdc` is backed up before it is
  rewritten; extra entries are reported by `check` and pruned by `sync`.
- `links`: transfer canonical `.md` as relative symlinks (or copies on hosts
  that cannot follow links). Kept for future hosts; CodeBuddy uses `mdc`.

Absent optional layers are a SKIP, not a finding (D50 B1): `.codebuddy/` is
gitignored, so a fresh clone or CI runner legitimately has no layer.

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing written).
Reuses scripts/agent_common.py.
"""

import hashlib
import os
import re
import shutil
import sys

import agent_common as ac

PREFIX = "GATE-RULES"

_FM = re.compile(r"^---\n.*?\n---\n", re.DOTALL)
_LINK = re.compile(r"\]\(\.\./([a-z0-9-]+)/([a-z0-9-]+)\.md((?:#[^)]*)?)\)")
_LOCAL_LINK = re.compile(r"\]\(([a-z0-9-]+)\.md((?:#[^)]*)?)\)")


def _langs(target):
    langs = target.get("langs")
    return langs if isinstance(langs, list) and langs else None


def _target_dir(target):
    if target.get("rules") is False:
        return None
    rules_dir = target.get("rules_dir")
    if not rules_dir:
        return None
    return os.path.join(ac.ROOT, rules_dir)


def _active(targets):
    return [t for t in targets if t.get("enabled") and _target_dir(t)]


def _canonical_rels(rules_root, langs):
    out = []
    for rel in ac.iter_rel_files(rules_root, ".md"):
        parts = rel.split("/")
        if len(parts) != 2:
            continue  # README.md at the root, or an unexpected depth
        if langs is not None and parts[0] not in langs:
            continue
        out.append(rel)
    return sorted(out)


def _strip_frontmatter(text):
    match = _FM.match(text)
    return text[match.end():] if match else text


def _first_h1(body):
    for line in body.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return None


def _mdc_folder(rel):
    lang, topic = rel.split("/")
    if topic.endswith(".md"):
        topic = topic[:-3]
    return "%s-%s" % (lang, topic)


def _retarget_links(body, lang):
    body = _LINK.sub(
        lambda m: "](../%s-%s/RULE.mdc%s)" % (m.group(1), m.group(2), m.group(3)),
        body,
    )
    return _LOCAL_LINK.sub(
        lambda m: "](../%s-%s/RULE.mdc%s)" % (lang, m.group(1), m.group(2)),
        body,
    )


def _mdc_content(canonical_path, rel, always_on, descriptions):
    with open(canonical_path, "r", encoding="utf-8") as handle:
        raw = handle.read()
    lang = rel.split("/")[0]
    body = _retarget_links(_strip_frontmatter(raw), lang)
    folder = _mdc_folder(rel)
    h1 = _first_h1(body) or folder
    desc_text = (descriptions or {}).get(folder, h1)
    desc = '"%s"' % desc_text.replace("\\", "\\\\").replace('"', '\\"')
    digest = hashlib.sha256(raw.encode("utf-8")).hexdigest()[:12]
    apply = "true" if folder in (always_on or set()) else "false"
    front = (
        "---\ndescription: %s\nalwaysApply: %s\nenabled: true\nupdatedAt: %s\n---\n"
        % (desc, apply, digest)
    )
    return folder, front + body


def _write_with_backup(path, content, dry_run):
    if os.path.isfile(path):
        with open(path, "r", encoding="utf-8") as handle:
            if handle.read() == content:
                return False
        if not dry_run:
            backup = ac.backup_name(os.path.dirname(path), "RULE.mdc")
            os.rename(path, backup)
            ac.prune_backups(os.path.dirname(path), "RULE.mdc")
    if not dry_run:
        ac.ensure_dir(os.path.dirname(path))
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(content)
    return True


def sync_layer_mdc(target, rules_root, dry_run):
    target_dir = _target_dir(target)
    always_on = set(target.get("always_on_rules") or [])
    descriptions = target.get("rule_descriptions") or {}
    rels = _canonical_rels(rules_root, _langs(target))
    expected = set()
    actions = 0
    if not dry_run:
        ac.ensure_dir(target_dir)
    for rel in rels:
        folder, content = _mdc_content(
            os.path.join(rules_root, rel), rel, always_on, descriptions
        )
        expected.add(folder)
        rfile = os.path.join(target_dir, folder, "RULE.mdc")
        if _write_with_backup(rfile, content, dry_run):
            actions += 1
            print(" [ MDCMK] %-10s %s" % (target["label"], folder))
    if not os.path.isdir(target_dir):
        return actions
    for entry in sorted(os.listdir(target_dir)):
        if entry in expected or ".bak-" in entry:
            continue
        actions += 1
        print(" [ MDCDEL] %-9s %s" % (target["label"], entry))
        if dry_run:
            continue
        full = os.path.join(target_dir, entry)
        if os.path.islink(full):
            os.unlink(full)
        elif os.path.isdir(full):
            shutil.rmtree(full, ignore_errors=True)
        else:
            backup = ac.backup_name(target_dir, entry)
            os.rename(full, backup)
            ac.prune_backups(target_dir, entry)
    return actions


def check_layer_mdc(target, rules_root):
    target_dir = _target_dir(target)
    always_on = set(target.get("always_on_rules") or [])
    descriptions = target.get("rule_descriptions") or {}
    findings = []
    if not os.path.isdir(target_dir):
        return [("__layer__", ac.MISSING)]
    expected = set()
    for rel in _canonical_rels(rules_root, _langs(target)):
        folder, content = _mdc_content(
            os.path.join(rules_root, rel), rel, always_on, descriptions
        )
        expected.add(folder)
        rfile = os.path.join(target_dir, folder, "RULE.mdc")
        if not os.path.isfile(rfile):
            findings.append((folder, ac.MISSING))
            continue
        with open(rfile, "r", encoding="utf-8") as handle:
            if handle.read() != content:
                findings.append((folder, "mismatch"))
    for entry in sorted(os.listdir(target_dir)):
        if entry in expected or ".bak-" in entry:
            continue
        findings.append((entry, "extra"))
    return findings


def sync_layer_links(target, rules_root, dry_run):
    target_dir = _target_dir(target)
    canonical_real = os.path.realpath(rules_root)
    expected = set(_canonical_rels(rules_root, _langs(target)))
    actions = 0
    if not dry_run:
        ac.ensure_dir(target_dir)
    for rel in sorted(expected):
        src = os.path.join(rules_root, rel)
        link = os.path.join(target_dir, rel)
        cls = ac.classify_link(link, os.path.realpath(src))
        if cls == ac.OK:
            continue
        actions += 1
        print(" [ LINK ] %-10s %s" % (target["label"], rel))
        if dry_run:
            continue
        ac.ensure_dir(os.path.dirname(link))
        if cls == ac.EXISTS:
            backup = ac.backup_name(target_dir, rel)
            os.rename(link, backup)
            ac.prune_backups(target_dir, rel)
        elif os.path.islink(link):
            os.unlink(link)
        ac.make_link(src, link)
    if os.path.isdir(target_dir):
        for rel in ac.iter_rel_files(target_dir, ".md"):
            if rel in expected:
                continue
            actions += 1
            print(" [ PRUNE] %-10s %s" % (target["label"], rel))
            if not dry_run:
                path = os.path.join(target_dir, rel)
                if os.path.islink(path):
                    os.unlink(path)
                else:
                    os.rename(path, ac.backup_name(target_dir, rel))
    return actions


def check_layer_links(target, rules_root):
    target_dir = _target_dir(target)
    findings = []
    if not os.path.isdir(target_dir):
        return [("__layer__", ac.MISSING)]
    expected = set(_canonical_rels(rules_root, _langs(target)))
    for rel in sorted(expected):
        src = os.path.join(rules_root, rel)
        cls = ac.classify_link(os.path.join(target_dir, rel), os.path.realpath(src))
        if cls != ac.OK:
            findings.append((rel, cls))
    for rel in ac.iter_rel_files(target_dir, ".md"):
        if rel not in expected:
            findings.append((rel, "extra"))
    return findings


def run(command, targets, cfg, dry_run):
    rules_root = os.path.join(ac.ROOT, cfg["rules_root"])
    canonical_real = os.path.realpath(rules_root)
    if not os.path.isdir(rules_root):
        print("%s x canonical-empty %s" % (PREFIX, rules_root))
        return ac.EXIT_GUARD
    active = _active(targets)
    if command == "list":
        for target in active:
            print(
                "%-10s rules %s (%s)"
                % (target["label"], target.get("rules_dir"), target.get("rules_mode", "links"))
            )
        return ac.EXIT_OK
    if command == "check":
        failures = 0
        total = 0
        for target in active:
            target_dir = _target_dir(target)
            if target.get("optional") and not os.path.isdir(target_dir):
                print(" [ SKIP ] %-10s %s (layer absent)" % (target["label"], target.get("rules_dir")))
                continue
            reasons = ac.guard_reasons(target_dir, canonical_real)
            if reasons:
                ac.die_guard(PREFIX, target.get("rules_dir"), reasons)
            mode = target.get("rules_mode", "links")
            if mode == "mdc":
                findings = check_layer_mdc(target, rules_root)
                total += len(_canonical_rels(rules_root, _langs(target)))
            else:
                findings = check_layer_links(target, rules_root)
                total += len(_canonical_rels(rules_root, _langs(target)))
            if not findings:
                print(" [ OK  ] %-10s %s" % (target["label"], target.get("rules_dir")))
                continue
            failures += 1
            print(" [ FIX ] %-10s %s" % (target["label"], target.get("rules_dir")))
            for rel, cls in findings:
                print("        %-40s %s" % (rel, cls))
        if failures:
            print("%s x %d target(s) need repair (run: agent-links.py sync)" % (PREFIX, failures))
            return ac.EXIT_FINDINGS
        print("%s ok (%d rule files x %d target(s))" % (PREFIX, total, len(active)))
        return ac.EXIT_OK
    for target in active:
        target_dir = _target_dir(target)
        reasons = ac.guard_reasons(target_dir, canonical_real)
        if reasons:
            ac.die_guard(PREFIX, target.get("rules_dir"), reasons)
        print("%s %-10s %s" % (" [DRY] " if dry_run else "        ", target["label"], target.get("rules_dir")))
        if target.get("rules_mode", "links") == "mdc":
            sync_layer_mdc(target, rules_root, dry_run)
        else:
            sync_layer_links(target, rules_root, dry_run)
    print("%s ok sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    argv = sys.argv[1:]
    command = "check"
    if argv and argv[0] in ("check", "sync", "dry-run", "list"):
        command = argv[0]
    cfg = ac.load_json(os.path.join(ac.ROOT, "scripts", "agent-targets.json"))
    sys.exit(run(command, cfg["targets"], cfg, command == "dry-run"))
