#!/usr/bin/env bash
# check-bundle-vocab.sh - the D111 reintroduction guard (bundle-unification
# plan, rulings A6 + M2): the managed unit is ONE word - bundle. This scanner
# reports every surviving use of the retired word `plugin` OUTSIDE the
# allowlist classes, so the unification waves can be verified complete and
# future reintroductions are caught.
#
# ACTIVATION: INACTIVE until S5 (bundle-unification-plan rev4). Two modes:
#   --report    enumerate hits outside the allowlist (the S1 pre-flight tool;
#               exit 0 always - a census, not a gate)
#   (default)   gate mode: exit 1 on any hit outside the allowlist. INACTIVE
#               until S5: gated behind MODUKIT_VOCAB_ENFORCED=1 so wiring it
#               into check.sh cannot turn the migration waves red.
#
# Allowlist classes (deliberate, each from a ruling):
#   FILE classes:  .agents/memorys/decisions.md (D-history, append-only, A3),
#                  .agents/memorys/history-archive.md (A3),
#                  docs/*-plan.md + docs/*-hearing.md plan-of-record BODIES (A3),
#                  docs/reference/ (M2: external profiles - per-phrase below),
#                  .refinfo/.polyorch/.opencode/serena trees (SKIP, external),
#                  book/web/vendor (SKIP, vendored)
#   PHRASE classes (M2, external vocabulary that names OTHER systems):
#                  "protoc plugin", "protoc-gen-", opencode "plugin": JSON keys,
#                  "claude-code-plugins", "superpowers plugin", "xmake-plugins",
#                  "plugin systems" (generic CS survey phrase in skill bodies)
#   LINE class:    lines ending with `<!-- vocab:plugin:allow <reason> -->`
#                  (the c1:allow sibling marker; per-line, auditable)
set -uo pipefail
SELF=$(realpath "$0")
cd "$(dirname "$0")/.."

REPORT=0
[ "${1:-}" = "--report" ] && REPORT=1

python3 - "$REPORT" <<'PY'
import os, pathlib, re, subprocess, sys

report = sys.argv[1] == "1"
WORD = re.compile(r'\bplugin', re.IGNORECASE)

# file-class allowlist (rulings A3/M2; paths relative to repo root)
FILE_ALLOW_EXACT = {
    ".agents/memorys/decisions.md",
    ".agents/memorys/history-archive.md",
    ".agents/memorys/pitfalls.md",          # historical entries, A3
    ".agents/memorys/conventions.md",       # historical entries, A3
    ".agents/memorys/backlog.md",           # opencode ecosystem refs
    ".agents/memorys/status.md",            # mirrors D111 wording only
    "docs/plugin-management-plan.md",       # plan-of-record body (A3)
    "docs/wave-2-plugin-door-plan.md",     # plan-of-record body (A3)
    "docs/wave-2-plugin-door-hearing.md",  # plan-of-record body (A3)
    "docs/stage-2-plan.md",                # plan-of-record body (A3)
    "docs/wave-3-parent-push-hearing.md",  # plan-of-record body (A3)
    "docs/wave-2-clock-hearing.md",        # plan-of-record body (A3)
    "docs/wave-2-t1-clock-door-plan.md",   # plan-of-record body (A3)
    "docs/wave-2-t2-clock-crossing-plan.md", # plan-of-record body (A3)
    "docs/wave-2-python-hearing.md",       # plan-of-record body (A3)
    "docs/stage-4-plan.md",                # plan-of-record body (A3)
    "docs/stage-5-plan.md",                # plan-of-record body (A3)
    "docs/ve-parity-plan.md",              # plan-of-record body (A3)
    "docs/onboarding-plan.md",              # plan-of-record body (A3)
    "docs/wave-2-bundle-door-plan.md",      # plan-of-record body (A3)
    "docs/wave-2-bundle-door-hearing.md",   # plan-of-record body (A3)
    "docs/stage-1-plan.md",                 # plan-of-record body (A3)
    "docs/bundle-unification-plan.md",      # this plan quotes the retired word
    "docs/reference/k7-v1-stable-note-draft.md",  # external-facing draft, M2
    "AGENTS.md",                            # the D111 line + sync mirrors
    "SKILL.md",                             # superpowers plugin (external ecosystem)
    "scripts/check-bundle-vocab.sh",        # the scanner names the retired word
    "scripts/gate-docs.sh",                 # opencode plugin URL shape check (external)
    "docs/codegraph-bootstrap.md",          # opencode plugin (external)
    "docs/agent-governance-borrow-plan.md", # donor model names (A3 body)
    "crates/modukit-core/src/version.rs",   # CTK Require-Plugin external term
}
FILE_ALLOW_PREFIX = (
    "docs/wave-", "docs/stage-", "docs/ve-parity-plan.md",  # plan-of-record bodies (A3)
    "docs/reference/",      # M2: external profiles; own sentences swept in S4
    ".refinfo/", ".polyorch/", ".opencode/",  # external trees
    "book/web/vendor/",
)
SKIP_PARTS = {".omo", ".pixi", "node_modules", "target", "build", ".git",
              "out", ".qoder", ".serena", ".codebuddy"}

# phrase-class allowlist (M2/B2: external vocabulary naming OTHER systems)
PHRASE_ALLOW = [
    re.compile(r'protoc[- ]plugin', re.I),
    re.compile(r'protoc-gen-', re.I),
    re.compile(r'"plugin"\s*:', re.I),           # opencode JSON schema key
    re.compile(r'claude-code-plugins', re.I),
    re.compile(r'superpowers plugin', re.I),
    re.compile(r'xmake-plugins', re.I),
    re.compile(r'plugin systems\b', re.I),       # generic CS survey phrase
    re.compile(r'\.so\b'),                        # not a phrase: keeps false +
]
LINE_ALLOW = re.compile(r'vocab:plugin:allow\b')

tracked = subprocess.run(["git", "ls-files"], capture_output=True, text=True).stdout.split()
untracked = subprocess.run(["git", "ls-files", "--others", "--exclude-standard"],
                           capture_output=True, text=True).stdout.split()
candidates = sorted(set(tracked + untracked))

EXTS = {".md", ".rst", ".txt", ".cmake", ".rs", ".c", ".cpp", ".h", ".hpp",
        ".toml", ".json", ".jsonc", ".sh", ".bat", ".mjs", ".js", ".py",
        ".yml", ".yaml", ".in", ".jsonl"}

hits = []
for f in candidates:
    rel = pathlib.PurePosixPath(f)
    if rel.suffix not in EXTS:
        continue
    if any(part in SKIP_PARTS for part in rel.parts):
        continue
    if f.startswith(FILE_ALLOW_PREFIX) or f in FILE_ALLOW_EXACT:
        continue
    try:
        text = pathlib.Path(f).read_text(errors="ignore")
    except Exception:
        continue
    for i, line in enumerate(text.splitlines(), 1):
        if not WORD.search(line):
            continue
        if LINE_ALLOW.search(line):
            continue
        if any(p.search(line) for p in PHRASE_ALLOW):
            continue
        hits.append(f"{f}:{i}: {line.strip()[:80]}")

print("\n".join(hits))
if report:
    print(f"VOCAB-REPORT: {len(hits)} surviving plugin-word hits outside the allowlist")
    sys.exit(0)
if os.environ.get("MODUKIT_VOCAB_ENFORCED") == "1":
    if hits:
        print(f"VOCAB ✗ {len(hits)} retired-word hits outside the allowlist (D111)")
        sys.exit(1)
    print("VOCAB ✓ (one word: bundle; zero plugin hits outside the allowlist)")
else:
    print(f"VOCAB (inactive report): {len(hits)} hits would fail once MODUKIT_VOCAB_ENFORCED=1")
PY
