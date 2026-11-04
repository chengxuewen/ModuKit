#!/usr/bin/env python3
"""Report-only prose structure gate for the human-facing book.

GB-3 starts in report mode. The command always exits zero after reporting
findings; promotion to a hard gate is a separate decision after the noise is
measured.

Scope comes from `book/toc.toml`:
  * chapter and group sources are checked;
  * type/anchor checks cover every listed chapter;
  * banned-word, heading, and paragraph checks cover chapters marked
    `polish = true`, plus `docs/architecture.md`;
  * internal plans, memory, SDD, and reference pages are not scanned unless the
    book explicitly lists them.

Checks:
  * how-to/tutorial chapters have a shell fence or numbered step;
  * explanation chapters have a mermaid fence, `file:line` anchor, or external
    link, and do not use step headings;
  * reference chapters avoid reader instructions;
  * `file:line` anchors point to real files and lines;
  * polished pages avoid banned filler phrases, oversized headings, and
    oversized paragraphs.

`markdownlint` is reported as AVAILABLE or SKIP; this script does not invoke it
and never mistakes a missing binary for a clean prose pass.

Exit codes: 0 report complete, 2 configuration/guard error.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import sys
import tomllib
from dataclasses import dataclass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOC = os.path.join(ROOT, "book", "toc.toml")
BANNED = os.path.join(ROOT, "docs", "prose-banned.toml")
PREFIX = "PROSE-REPORT"

FENCE = re.compile(r"^\s*```", re.M)
SHELL_FENCE = re.compile(r"^\s*```(?:bash|sh|shell|console)\b", re.M)
MERMAID = re.compile(r"^\s*```mermaid\b", re.M)
NUMBERED = re.compile(r"^\s*\d+[.)][ \t]+\S", re.M)
STEP_HEADING = re.compile(r"^\s*#{1,6}[ \t]+(?:step[ \t]+\d+|\u6b65\u9aa4[ \t]+\d+)\b", re.I | re.M)
ANCHOR = re.compile(
    r"(?P<path>(?:\./|\.\./)?[\w][\w./-]*\.(?:rs|md|toml|cpp|h|hpp|py|sh|json|yml|yaml|js|mjs)):(?P<line>\d+)"
)
EXTERNAL = re.compile(r"\]\(https?://")
HEADING = re.compile(r"^(#{1,6})[ \t]+(.+)$", re.M)
TABLE_ROW = re.compile(r"^\s*\|")


@dataclass
class Page:
    rel: str
    page_type: str | None
    polish: bool
    structure: bool = True


def load_toc() -> tuple[list[Page], list[str], list[str]]:
    with open(TOC, "rb") as handle:
        doc = tomllib.load(handle)
    pages: dict[str, Page] = {}
    errors: list[str] = []
    skipped: list[str] = []

    def add(rel: str, page_type: str | None = None, polish: bool = False, structure: bool = True):
        rel = os.path.normpath(rel).replace(os.sep, "/")
        if not os.path.isfile(os.path.join(ROOT, rel)):
            # Generated book inputs (for example web/graph.md) may legitimately
            # be absent before their generator runs; report a SKIP, not a gate
            # error, because this command is report-only.
            skipped.append(rel)
            return
        prior = pages.get(rel)
        if prior is None:
            pages[rel] = Page(rel, page_type, polish, structure)
        else:
            # A chapter and a group may name the same file; keep the strongest
            # declared metadata instead of scanning it twice.
            prior.page_type = prior.page_type or page_type
            prior.polish = prior.polish or polish
            prior.structure = prior.structure and structure

    for chapter in doc.get("chapter", []):
        add(
            chapter.get("source", ""),
            chapter.get("type"),
            bool(chapter.get("polish", False)),
            bool(chapter.get("structure", True)),
        )

    for group in doc.get("group", []):
        pattern = os.path.join(ROOT, group.get("glob", ""))
        import glob as globmod

        for match in sorted(globmod.glob(pattern, recursive=True)):
            rel = os.path.relpath(match, ROOT)
            add(
                rel,
                group.get("type"),
                bool(group.get("polish", False)),
                bool(group.get("structure", True)),
            )

    ordered = sorted(pages.values(), key=lambda page: page.rel)
    return ordered, errors, skipped


def load_banned():
    with open(BANNED, "rb") as handle:
        return tomllib.load(handle).get("words", [])


def strip_regions(text: str):
    """Yield (line_no, line, in_fence) while preserving fence state."""
    in_fence = False
    for line_no, line in enumerate(text.splitlines(), 1):
        if FENCE.match(line):
            yielded = not in_fence
            in_fence = not in_fence
            yield line_no, line, yielded
        else:
            yield line_no, line, not in_fence


def paragraph_blobs(text: str):
    """Return (start_line, joined text) for prose blocks outside fences/tables."""
    blocks: list[tuple[int, str]] = []
    current: list[str] = []
    start = 0
    for line_no, line, outside in strip_regions(text):
        stripped = line.strip()
        prose = outside and stripped and not TABLE_ROW.match(line) and not stripped.startswith(("#", ">", "-", "*", "+"))
        if prose:
            if not current:
                start = line_no
            current.append(stripped)
        elif current:
            blocks.append((start, " ".join(current)))
            current = []
    if current:
        blocks.append((start, " ".join(current)))
    return blocks


def resolve_anchor(raw: str, current_dir: str):
    """Resolve anchors against the current page, then repo root and references."""
    candidates = [os.path.normpath(os.path.join(current_dir, raw))]
    candidates.append(os.path.normpath(os.path.join(ROOT, raw)))
    if not raw.startswith(("docs/", "book/", "web/")):
        candidates.append(os.path.normpath(os.path.join(ROOT, "docs", "reference", raw)))
    for candidate in candidates:
        if os.path.isfile(candidate):
            return candidate
    return None


def check_page(page: Page, banned: list[str], paragraph_limit: int, heading_limit: int):
    if not page.structure:
        return []

    path = os.path.join(ROOT, page.rel)
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    lines = text.splitlines()
    findings: list[str] = []

    if page.page_type == "how-to" and not (SHELL_FENCE.search(text) or NUMBERED.search(text)):
        findings.append("how-to lacks a shell fence or numbered procedure")
    if page.page_type == "tutorial" and not (FENCE.search(text) or NUMBERED.search(text)):
        findings.append("tutorial lacks a fence or numbered procedure")
    if page.page_type == "explanation":
        if not (MERMAID.search(text) or ANCHOR.search(text) or EXTERNAL.search(text)):
            findings.append("explanation lacks mermaid, file:line anchor, or external evidence")
        if STEP_HEADING.search(text):
            findings.append("explanation uses step headings; move procedures to how-to/tutorial")

    if page.page_type == "reference":
        if re.search(r"\b(?:you(?:[ \t]+must)|you[ \t]+should)\b", text, re.I):
            findings.append("reference contains reader instructions")

    for match in ANCHOR.finditer(text):
        resolved = resolve_anchor(match.group("path"), os.path.dirname(path))
        if resolved is None:
            findings.append(f"anchor target missing: {match.group(0)}")
            continue
        line_no = int(match.group("line"))
        line_count = sum(1 for _ in open(resolved, encoding="utf-8", errors="ignore"))
        if line_no < 1 or line_no > line_count:
            findings.append(f"anchor line out of range: {match.group(0)}")

    if page.polish:
        lowered = text.lower()
        for phrase in banned:
            if phrase.lower() in lowered:
                findings.append(f"banned phrase: {phrase}")
        for match in HEADING.finditer(text):
            rendered = re.sub(r"[`*_~]", "", match.group(2)).strip()
            if len(rendered) > heading_limit:
                findings.append(f"heading too long at line {text[:match.start()].count(chr(10)) + 1}: {rendered}")
        for start, blob in paragraph_blobs(text):
            if len(blob) > paragraph_limit:
                findings.append(f"paragraph too long starting at line {start} ({len(blob)} chars)")

    return findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--paragraph-limit", type=int, default=600)
    parser.add_argument("--heading-limit", type=int, default=70)
    args = parser.parse_args()

    pages, errors, skipped = load_toc()
    if errors:
        for error in errors:
            print(f"{PREFIX} error: {error}")
        return 2
    if not pages:
        print(f"{PREFIX} error: no present book chapters resolved")
        return 2
    for rel in skipped:
        print(f"{PREFIX} skip: {rel} (source not generated yet)")

    banned = load_banned()
    all_findings: list[tuple[str, str]] = []
    polished = 0
    for page in pages:
        findings = check_page(page, banned, args.paragraph_limit, args.heading_limit)
        if page.polish:
            polished += 1
        for finding in findings:
            all_findings.append((page.rel, finding))

    lint = shutil.which("markdownlint")
    if lint:
        print(f"{PREFIX} markdownlint: AVAILABLE ({lint}); not invoked by report mode")
    else:
        print(f"{PREFIX} markdownlint: SKIP (binary not on PATH)")

    print(
        f"{PREFIX} scope: {len(pages)} listed page(s), {polished} polished, "
        f"{len(all_findings)} candidate finding(s); REPORT-ONLY"
    )
    for rel, finding in all_findings:
        print(f"{PREFIX} finding: {rel}: {finding}")
    print(f"{PREFIX} ✓ report complete (promotion requires a separate maintainer decision)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:  # noqa: BLE001 - report config errors distinctly
        print(f"{PREFIX} internal error: {exc}", file=sys.stderr)
        sys.exit(2)
