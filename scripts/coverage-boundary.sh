#!/usr/bin/env bash
# coverage-boundary.sh - the pinned boundary run (D79 hard line, D84a region
# definition, one recipe so numbers are comparable across landings).
#
# What it measures: line coverage per REGION crate - modukit-core,
# modukit-transport, modukit-codegen, modukit-js (four, since D102) - from the
# per-file table of
# one instrumented `cargo llvm-cov --workspace` run: region % =
# (sum(lines) - sum(missed)) / sum(lines) over crates/<region>/src/*.rs.
# Boundary-excluded by D84a: modukit-runtime bins and the test-plugins /
# example fixture crates (they stay in the run for census, not in the metric).
# modukit-js is bin-like-exempt per D103 (generated face; its proofs run node-side):
# the row still PRINTS and the table still needs all four rows - visibility,
# not a vote.
#
# What it asserts (all hard):
#   1. the cov run exits 0 (no failing test can hide behind percentages);
#   2. zero "FAILED" lines and suite census printed for human cross-check;
#   3. each region >= 80.0 line %.
#
# Reproducibility contract (edit-safety #19): this file IS the metric
# definition. Re-measure by running THIS script, never by hand-parsing an
# older log; keep the log path stable (/tmp/cov-boundary-latest.txt).
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.pixi/bin:${PATH}"

LOG=/tmp/cov-boundary-latest.txt
SUM=/tmp/cov-boundary-summary.txt

echo "coverage-boundary: running instrumented workspace cov (long: ~20 min)..."
pixi run -e cov cargo llvm-cov --workspace > "$LOG" 2>&1
cov_exit=$?

failed=$(grep -c 'FAILED' "$LOG" || true)
suites=$(grep -cE '^test result' "$LOG" || true)
tests=$(grep -E '^test result' "$LOG" | awk '{s+=$4} END {print s+0}')

if [ "$cov_exit" -ne 0 ] || [ "$failed" -ne 0 ]; then
  echo "coverage-boundary: FAIL (cov exit=$cov_exit, FAILED lines=$failed - fix tests first, percentages are meaningless)"
  exit 1
fi

# Region metric - THE parser (single source of the aggregation).
grep -E '^crates/modukit-(core|transport|codegen|js)/src/[a-z0-9_]+\.rs[[:space:]]' "$LOG" \
  | awk '{
      if ($1 ~ /modukit-core/)      crate = "modukit-core";
      else if ($1 ~ /modukit-transport/) crate = "modukit-transport";
      else if ($1 ~ /modukit-js/)        crate = "modukit-js";
      else                               crate = "modukit-codegen";
      lines[crate] += $8; missed[crate] += $9;
    } END {
      for (c in lines) printf "%-20s %6.2f%% (%d lines, %d missed)\n", \
        c, 100*(lines[c]-missed[c])/lines[c], lines[c], missed[c];
    }' | sort > "$SUM"

# guard against the empty-table false green (rule #17 family): the parse MUST
# yield exactly the four region rows or the verdict is FAIL, not silence.
if [ "$(wc -l < "$SUM")" -ne 4 ]; then
  echo "coverage-boundary: FAIL (summary table holds $(wc -l < "$SUM") rows, expected 4 - parser or log shape broke)"
  exit 1
fi

cat "$SUM"
echo "census: suites=$suites tests_passed=$tests failed_lines=$failed"

verdict=0
while read -r crate pct rest; do
  val=$(echo "$pct" | tr -d '%')
  if [ "$crate" = "modukit-js" ]; then
    echo "exempt modukit-js ($pct) - bin-like per D103; proofs run node-side"
    continue
  fi
  low=$(awk -v v="$val" 'BEGIN {print (v < 80.0) ? 1 : 0}')
  if [ "$low" = "1" ]; then echo "coverage-boundary: FAIL $crate $pct < 80.0"; verdict=1; fi
done < "$SUM"

[ "$verdict" = "0" ] && echo "coverage-boundary: PASS (all regions >= 80.0)"
exit "$verdict"
