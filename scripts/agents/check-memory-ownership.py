#!/usr/bin/env python3
"""Report cross-repo memory ownership between ModuKit and PolyOrch.

Phase A is deliberately REPORT-ONLY:

* explicit ``child:D<n>`` / ``child:PIT-<n>`` / ``child:C<n>`` citations in the
  root memory must resolve in the PolyOrch ledger;
* normalized parent/child section titles must not duplicate;
* PolyOrch high water is derived at run time from its ledger; no secondary
  ``child-refs.md`` mirror is created (D67).

The current tree has no ``child:`` citations. A clean exit therefore means "no
violations found"; it does not mean that cross-repo ownership has been exercised.
The self-test plants synthetic violations so the detector is trusted before the
first real citation lands.

Exit codes: 0 clean/report, 1 findings, 2 guard, 3 internal error.
"""

import argparse
import os
import re
import sys

try:
    import agent_common as ac
except ModuleNotFoundError:  # direct run from scripts/agents/ (batch one layout)
    sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    import agent_common as ac

PREFIX = "MEMORY-OWNERSHIP"
SECTION = re.compile(r"(?m)^##\s+(D\d+|PIT-\d+|C\d+):[ \t]*(.+?)[ \t]*$")
CITATION = re.compile(r"child:(D\d+|PIT-\d+|C\d+)\b")


def _kind(token):
    if token.startswith("PIT-"):
        return "PIT"
    return token[0]


def _number(token):
    return int(re.sub(r"\D", "", token))


def parse_sections(paths):
    """Return {kind: {number: title}} and [(kind, number, normalized title, path)]."""
    ledgers = {"D": {}, "PIT": {}, "C": {}}
    titles = []
    for path in paths:
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8") as handle:
            for line_no, line in enumerate(handle, 1):
                match = SECTION.match(line.rstrip())
                if not match:
                    continue
                token, title = match.groups()
                kind, number = _kind(token), _number(token)
                # Later duplicates are still reported by the normalized title
                # check; the high-water map keeps the maximum number seen.
                ledgers[kind][number] = title
                titles.append((kind, number, normalize_title(title), path, line_no))
    return ledgers, titles


def normalize_title(title):
    """Remove provenance punctuation so equivalent titles compare equal."""
    title = title.replace("`", "").replace("*", "")
    # Titles are captured after the ledger token and colon, so only punctuation
    # and whitespace normalization is needed here.
    title = re.sub(r"[^\w\u4e00-\u9fff]+", "", title, flags=re.UNICODE)
    return title.lower()


def missing_citations(text, child_ledgers):
    missing = []
    for match in CITATION.finditer(text):
        token = match.group(1)
        kind, number = _kind(token), _number(token)
        if number not in child_ledgers.get(kind, {}):
            missing.append("child:%s" % token)
    return sorted(set(missing))


def duplicate_titles(parent_titles, child_titles):
    parent = {title for _kind, _number, title, _path, _line in parent_titles}
    child = {title for _kind, _number, title, _path, _line in child_titles}
    return sorted(parent & child)


def high_water(ledgers):
    return {kind: max(ledgers.get(kind, {}), default=0) for kind in ("D", "PIT", "C")}


def self_test():
    child = {
        "D": {297: "child decision"},
        "PIT": {184: "child pitfall"},
        "C": {51: "child convention"},
    }
    missing = missing_citations("child:D297 child:C51 child:D999", child)
    if missing != ["child:D999"]:
        print("%s self-test ✗ citation miss not detected: %r" % (PREFIX, missing))
        return 1
    parent_titles = [("D", 1, normalize_title("same fact"), "parent.md", 1)]
    child_titles = [("D", 2, normalize_title("Same fact!"), "child.md", 1)]
    if duplicate_titles(parent_titles, child_titles) != ["samefact"]:
        print("%s self-test ✗ duplicate title not detected" % PREFIX)
        return 1
    child_ledgers = {
        "D": {46: "decision"},
        "PIT": {65: "pitfall"},
        "C": {6: "convention"},
    }
    if high_water(child_ledgers) != {"D": 46, "PIT": 65, "C": 6}:
        print("%s self-test ✗ runtime high-water derivation failed" % PREFIX)
        return 1
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", nargs="?", default="check", choices=("check", "self-test"))
    parser.add_argument("--child-root", default=os.path.join("PolyOrch", ".agents", "memorys"))
    args = parser.parse_args(argv)

    if args.command == "self-test":
        return self_test()

    root_memory = os.path.join(ac.ROOT, ".agents", "memorys")
    child_root = os.path.abspath(os.path.join(ac.ROOT, args.child_root))
    if not os.path.isdir(child_root):
        # Phase A is report-only and must not make an uninitialized submodule
        # look like a governance regression.
        print("%s report: child memory root absent (%s); skipped" % (PREFIX, os.path.relpath(child_root, ac.ROOT)))
        return ac.EXIT_OK

    root_paths = [
        os.path.join(root_memory, name)
        for name in ("decisions.md", "pitfalls.md", "conventions.md", "status.md")
    ]
    child_paths = [
        os.path.join(child_root, name)
        for name in ("decisions.md", "pitfalls.md", "conventions.md")
    ]

    root_text = "\n".join(open(path, encoding="utf-8").read() for path in root_paths if os.path.isfile(path))
    _parent_ledgers, parent_titles = parse_sections(root_paths)
    child_ledgers, child_titles = parse_sections(child_paths)

    findings = []
    for token in missing_citations(root_text, child_ledgers):
        findings.append("unresolved citation: %s" % token)
    for title in duplicate_titles(parent_titles, child_titles):
        findings.append("duplicate normalized title: %s" % title)

    water = high_water(child_ledgers)
    citation_count = len(set(CITATION.findall(root_text)))
    print(
        "%s report: citations=%d child-high-water D=%d PIT=%d C=%d"
        % (PREFIX, citation_count, water["D"], water["PIT"], water["C"])
    )
    if not child_titles:
        findings.append("child memory produced no section titles")
    if citation_count == 0:
        print("%s report: phase A has no explicit child: citations yet" % PREFIX)
    if findings:
        for finding in findings:
            print("%s finding: %s" % (PREFIX, finding))
        # Phase A is report-only by design. Return OK until the maintainer
        # decides to make the first real reference a hard gate.
        print("%s report-only: %d finding(s) observed (not gating)" % (PREFIX, len(findings)))
        return ac.EXIT_OK
    print("%s ✓ no ownership findings" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:  # noqa: BLE001 - report internal errors with a distinct exit code
        print("%s internal error: %s" % (PREFIX, exc), file=sys.stderr)
        sys.exit(3)
