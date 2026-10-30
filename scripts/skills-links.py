#!/usr/bin/env python3
"""Check / repair the per-host skill activation layer.

Single source: <repo>/.agents/skills/<name>/SKILL.md -- the only editable copy.
Targets: `skills_dir` in scripts/agent-targets.json (one target file; no second
skills-targets.json).

Delivery (`skills_delivery`):
- `link` (default): one relative symlink per canonical skill directory.
- `copy`: a generated copy plus a per-target `skills-link-map.json`; the copied
  tree is hashed, so a tampered copy is visible. `--copy` forces copy mode
  (Windows symlink fallback).
- `native`: the host reads `.agents/skills` itself; no layer is generated.

Absent optional layers are a SKIP, not a finding (D50 B1): `.codebuddy/` is
gitignored, so a fresh clone or CI runner legitimately has no layer.

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing written).
Reuses scripts/agent_common.py.
"""

import hashlib
import json
import os
import shutil
import sys

import agent_common as ac

PREFIX = "GATE-SKILLS"


def _skills_root(cfg):
    return os.path.join(ac.ROOT, cfg["skills_root"])


def _target_dir(target):
    if target.get("skills") is False:
        return None
    skills_dir = target.get("skills_dir")
    if not skills_dir:
        return None
    return os.path.join(ac.ROOT, skills_dir)


def _active(targets):
    return [t for t in targets if t.get("enabled") and _target_dir(t)]


def discover_skills(skills_root):
    found = {}
    for dirpath, _dirnames, filenames in os.walk(skills_root):
        if "SKILL.md" not in filenames:
            continue
        name = os.path.basename(dirpath)
        if name in found:
            raise RuntimeError("duplicate skill %s: %s and %s" % (name, found[name], dirpath))
        found[name] = dirpath
    return found


def _expected_names(target, canonical):
    names = set(canonical)
    allow = target.get("skills_allowlist")
    if not isinstance(allow, list) or not allow:
        return names, []
    allowed = set(allow)
    missing = sorted(allowed - names)
    return names & allowed, missing


def _extra_entries(target_dir, names):
    extras = []
    if not os.path.isdir(target_dir):
        return extras
    for entry in sorted(os.listdir(target_dir)):
        if entry in names or ".bak-" in entry:
            continue
        extras.append(entry)
    return extras


def tree_hash(path):
    digest = hashlib.sha256()
    if not os.path.isdir(path):
        return ""
    for dirpath, _dirnames, filenames in os.walk(path):
        for filename in sorted(filenames):
            full = os.path.join(dirpath, filename)
            rel = os.path.relpath(full, path).replace(os.sep, "/")
            digest.update(rel.encode("utf-8") + b"\0")
            with open(full, "rb") as handle:
                digest.update(handle.read())
    return digest.hexdigest()


def _map_path(target_dir):
    return os.path.join(os.path.dirname(target_dir), "skills-link-map.json")


def _load_map(path):
    if not os.path.isfile(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except ValueError:
        return {}
    return data if isinstance(data, dict) else {}


def _save_map(path, data, dry_run):
    if dry_run:
        return
    ac.ensure_dir(os.path.dirname(path))
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=2, sort_keys=True)
        handle.write("\n")


def check_link(target, canonical, names):
    target_dir = _target_dir(target)
    if not os.path.isdir(target_dir):
        return [("__layer__", ac.MISSING)]
    findings = []
    for name in sorted(names):
        src = canonical[name]
        cls = ac.classify_link(os.path.join(target_dir, name), os.path.realpath(src))
        if cls != ac.OK:
            findings.append((name, cls))
    for entry in _extra_entries(target_dir, names):
        findings.append((entry, "extra"))
    return findings


def sync_link(target, canonical, names, dry_run):
    target_dir = _target_dir(target)
    if not dry_run:
        ac.ensure_dir(target_dir)
    actions = 0
    for name in sorted(names):
        src = canonical[name]
        link = os.path.join(target_dir, name)
        cls = ac.classify_link(link, os.path.realpath(src))
        if cls == ac.OK:
            continue
        actions += 1
        print(" [ LINK ] %-10s %s" % (target["label"], name))
        if dry_run:
            continue
        if cls == ac.EXISTS:
            backup = ac.backup_name(target_dir, name)
            os.rename(link, backup)
            ac.prune_backups(target_dir, name)
        elif os.path.islink(link):
            os.unlink(link)
        try:
            ac.make_link(src, link)
        except RuntimeError:
            print(" [ WARN ] %-10s %s (symlink failed; rerun with --copy)" % (target["label"], name))
            actions -= 1
    for entry in _extra_entries(target_dir, names):
        actions += 1
        print(" [ PRUNE] %-10s %s" % (target["label"], entry))
        if dry_run:
            continue
        full = os.path.join(target_dir, entry)
        if os.path.islink(full):
            os.unlink(full)
        else:
            os.rename(full, ac.backup_name(target_dir, entry))
    return actions


def check_copy(target, canonical, names):
    target_dir = _target_dir(target)
    if not os.path.isdir(target_dir):
        return [("__layer__", ac.MISSING)]
    recorded = _load_map(_map_path(target_dir)).get("skills", {})
    findings = []
    for name in sorted(names):
        src = canonical[name]
        dst = os.path.join(target_dir, name)
        if not os.path.isdir(dst):
            findings.append((name, "missing"))
            continue
        source_hash = tree_hash(src)
        copy_hash = tree_hash(dst)
        entry = recorded.get(name) or {}
        if entry.get("source") != source_hash or entry.get("copy") != copy_hash:
            findings.append((name, "copy-tampered"))
    for entry in _extra_entries(target_dir, names):
        findings.append((entry, "extra"))
    return findings


def sync_copy(target, canonical, names, dry_run):
    target_dir = _target_dir(target)
    map_path = _map_path(target_dir)
    recorded = _load_map(map_path).get("skills", {})
    expected = {}
    actions = 0
    if not dry_run:
        ac.ensure_dir(target_dir)
    for name in sorted(names):
        src = canonical[name]
        dst = os.path.join(target_dir, name)
        source_hash = tree_hash(src)
        copy_hash = tree_hash(dst)
        entry = recorded.get(name) or {}
        if os.path.isdir(dst) and entry.get("source") == source_hash and entry.get("copy") == copy_hash:
            expected[name] = {"source": source_hash, "copy": copy_hash}
            continue
        actions += 1
        print(" [ COPY ] %-10s %s" % (target["label"], name))
        if dry_run:
            continue
        if os.path.isdir(dst) or os.path.islink(dst):
            os.rename(dst, ac.backup_name(target_dir, name))
        ac.copy_tree(src, dst)
        expected[name] = {"source": source_hash, "copy": tree_hash(dst)}
    for entry in _extra_entries(target_dir, names):
        actions += 1
        print(" [ PRUNE] %-10s %s" % (target["label"], entry))
        if dry_run:
            continue
        full = os.path.join(target_dir, entry)
        if os.path.islink(full):
            os.unlink(full)
        else:
            os.rename(full, ac.backup_name(target_dir, entry))
    if not dry_run:
        _save_map(map_path, {"skills": expected}, dry_run)
    return actions


def forbidden_findings(cfg):
    findings = []
    for rel in cfg.get("forbidden_paths") or []:
        full = os.path.join(ac.ROOT, rel)
        if not os.path.exists(full):
            continue
        if os.path.isfile(full):
            findings.append((rel, "forbidden-file"))
            continue
        for dirpath, _dirnames, filenames in os.walk(full):
            if filenames:
                findings.append((rel, "forbidden-dir"))
                break
    return findings


def run(command, targets, cfg, dry_run, copy_mode=False):
    skills_root = _skills_root(cfg)
    if not os.path.isdir(skills_root):
        print("%s x canonical-empty %s" % (PREFIX, skills_root))
        return ac.EXIT_GUARD
    try:
        canonical = discover_skills(skills_root)
    except RuntimeError as exc:
        print("%s x %s" % (PREFIX, exc))
        return ac.EXIT_FINDINGS
    if not canonical:
        print("%s x canonical-empty %s" % (PREFIX, skills_root))
        return ac.EXIT_GUARD
    active = _active(targets)
    if command == "list":
        for target in active:
            print(
                "%-10s skills %s (%s)"
                % (target["label"], target.get("skills_dir"), target.get("skills_delivery", "link"))
            )
        return ac.EXIT_OK
    if command == "check":
        failures = 0
        total = 0
        for rel, cls in forbidden_findings(cfg):
            failures += 1
            print(" [ FIX ] %-10s %-40s %s" % ("forbidden", rel, cls))
        for target in active:
            target_dir = _target_dir(target)
            names, missing = _expected_names(target, canonical)
            if target.get("optional") and not os.path.isdir(target_dir):
                print(" [ SKIP ] %-10s %s (layer absent)" % (target["label"], target.get("skills_dir")))
                continue
            reasons = ac.guard_reasons(target_dir, os.path.realpath(skills_root))
            if reasons:
                ac.die_guard(PREFIX, target.get("skills_dir"), reasons)
            mode = "copy" if copy_mode else target.get("skills_delivery", "link")
            if mode == "copy":
                findings = check_copy(target, canonical, names)
            elif mode == "native":
                findings = []
            else:
                findings = check_link(target, canonical, names)
            findings += [("allowlist-missing", name) for name in missing]
            total += len(names)
            if not findings:
                print(" [ OK  ] %-10s %s" % (target["label"], target.get("skills_dir")))
                continue
            failures += 1
            print(" [ FIX ] %-10s %s" % (target["label"], target.get("skills_dir")))
            for name, cls in findings:
                print("        %-40s %s" % (name, cls))
        if failures:
            print("%s x %d finding(s) (run: agents sync)" % (PREFIX, failures))
            return ac.EXIT_FINDINGS
        print("%s ok (%d skill(s) x %d target(s))" % (PREFIX, total, len(active)))
        return ac.EXIT_OK
    for target in active:
        target_dir = _target_dir(target)
        reasons = ac.guard_reasons(target_dir, os.path.realpath(skills_root))
        if reasons:
            ac.die_guard(PREFIX, target.get("skills_dir"), reasons)
        names, missing = _expected_names(target, canonical)
        for name in missing:
            print(" [ WARN ] %-10s allowlist entry missing: %s" % (target["label"], name))
        mode = "copy" if copy_mode else target.get("skills_delivery", "link")
        print("%s %-10s %s" % (" [DRY] " if dry_run else "        ", target["label"], target.get("skills_dir")))
        if mode == "copy":
            sync_copy(target, canonical, names, dry_run)
        elif mode == "native":
            continue
        else:
            sync_link(target, canonical, names, dry_run)
    print("%s ok sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    argv = sys.argv[1:]
    command = "check"
    if argv and argv[0] in ("check", "sync", "dry-run", "list"):
        command = argv[0]
    copy = "--copy" in argv
    cfg = ac.load_json(os.path.join(ac.ROOT, "scripts", "agent-targets.json"))
    sys.exit(run(command, cfg["targets"], cfg, command == "dry-run", copy))
