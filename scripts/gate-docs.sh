#!/usr/bin/env bash
# gate-docs - zero-drift checks that have a live claim in this repository.
#   1 root CMake purity (<= 120 lines; no compile rules except ALIAS)
#   2 PolyOrch gitlink three-way (pin == HEAD, branch not detached, pin published)
#   3 CLI task names referenced by ./modukit.sh exist in pixi.toml
#   4 relative links (delegated to check-doc-refs.sh)
#   5 book SUMMARY entries resolve when build/book-src exists
# Owned elsewhere: header-vs-extern (check-embed-parity.sh); package/llms counts
# are skipped because this repo carries no such claim.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
fail=0

# 1 root purity
if [ -f CMakeLists.txt ]; then
  rl=$(wc -l < CMakeLists.txt)
  exempt=$(grep -q '# assembly-growth' CMakeLists.txt && echo 1 || echo 0)
  if [ "$rl" -gt 120 ] && [ "$exempt" -ne 1 ]; then
    echo "GATE-DOCS ✗ root CMakeLists at ${rl} lines (>120; add a '# assembly-growth: <reason>' marker or sink rules into cmake/)"
    fail=1
  fi
  bad=$(grep -nE 'add_executable\(|add_library\(' CMakeLists.txt | grep -vE 'ALIAS' | grep -vE ':[[:space:]]*#' || true)
  if [ -n "$bad" ]; then
    echo "GATE-DOCS ✗ root CMakeLists carries compile rules (assembly layer only; ALIAS excepted):"
    printf '%s\n' "$bad"
    fail=1
  fi
fi

# 2 gitlink
pin=$(git ls-files -s PolyOrch | awk '{print $2}')
if [ -n "$pin" ]; then
  ibranch=$(git -C PolyOrch symbolic-ref --quiet --short HEAD || echo DETACHED)
  ihead=$(git -C PolyOrch rev-parse HEAD 2>/dev/null || true)
  [ "$ibranch" != "DETACHED" ] || { echo "GATE-DOCS ✗ submodule HEAD detached (C2 keeps the pin on dev-modukit)"; fail=1; }
  if [ -n "$ihead" ] && [ "$pin" != "$ihead" ]; then
    echo "GATE-DOCS ✗ gitlink pin ${pin:0:7} != submodule HEAD ${ihead:0:7}"
    fail=1
  fi
  if ! git -C PolyOrch branch -r --contains "$pin" 2>/dev/null | grep -q .; then
    pub=$(GIT_TERMINAL_PROMPT=0 timeout 15 git -C PolyOrch ls-remote origin 2>/dev/null \
      | awk -v s="$pin" '$1==s{print $2; exit}' || true)
    [ -n "$pub" ] || { echo "GATE-DOCS ✗ gitlink pin ${pin:0:7} is on no remote ref"; fail=1; }
  fi
fi

# 3 CLI task names
if [ -f pixi.toml ]; then
  cli_tasks=$(awk '
    /^\[/ { in_tasks = ($0 ~ /^\[(feature\.[a-z0-9_-]+\.)?tasks\]$/); next }
    in_tasks && /^[A-Za-z0-9_-]+[ \t]*=/ { name=$0; sub(/[ \t]*=.*/, "", name); print name }
  ' pixi.toml | sort -u)
  refs=$(grep -ohE '\./modukit\.sh[[:space:]]+[a-z0-9-]+' modukit.sh AGENTS.md CONTRIBUTING.md README.md 2>/dev/null \
    | awk '{print $2}' | sort -u || true)
  cli_exes="cmake cargo node npm python3 ctest rustc rustfmt"
  for name in $refs; do
    grep -qxF "$name" <<< "$cli_tasks" && continue
    grep -qwF "$name" <<< "$cli_exes" && continue
    echo "GATE-DOCS ✗ './modukit.sh $name' is neither a pixi.toml task nor a known env executable"
    fail=1
  done
fi

# 4 relative links (one owner: check-doc-refs.sh)
bash scripts/check-doc-refs.sh || fail=1

# 5 book SUMMARY (B6)
book_note="book: no build yet"
if [ -f build/book-src/SUMMARY.md ]; then
  chapters=0
  while IFS= read -r dest; do
    chapters=$((chapters + 1))
    [ -e "build/book-src/$dest" ] || { echo "GATE-DOCS ✗ SUMMARY points at a missing file: $dest"; fail=1; }
  done < <(grep -oE '^- \[[^]]*\]\([^)]+\)' build/book-src/SUMMARY.md | sed -E 's/.*\]\(([^)]+)\)$/\1/')
  if [ -f build/book-src/web/graph.md ]; then
    grep -q 'src="graph-view.html"' build/book-src/web/graph.md \
      || { echo "GATE-DOCS ✗ graph chapter no longer embeds graph-view.html"; fail=1; }
  fi
  book_note="book: ${chapters} chapters"
fi

if [ "$fail" -eq 0 ]; then
  echo "GATE-DOCS ✓ (root purity / gitlink aligned / cli ${cli_tasks:+$(printf '%s\n' "$cli_tasks" | grep -c .)} tasks / links / ${book_note})"
fi
exit "$fail"
