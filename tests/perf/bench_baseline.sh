#!/usr/bin/env bash
# Structured Benchmark Baseline Tool (C2)
#
# Saves divan benchmark output as both raw text and machine-readable CSV.
# Supports drift checking between baselines.
#
# Usage:
#   bash tests/perf/bench_baseline.sh save [-- cargo_flags...]
#   bash tests/perf/bench_baseline.sh compare [baseline.csv]
#   bash tests/perf/bench_baseline.sh drift [--threshold=20]
#   bash tests/perf/bench_baseline.sh info
#
# Output directory: miniextendr-bench/baselines/
#
# CSV format: timestamp,bench_target,group,name,args,median_ns,unit

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BASELINE_DIR="$REPO_ROOT/miniextendr-bench/baselines"

mkdir -p "$BASELINE_DIR"

# ---------------------------------------------------------------------------
# Parse divan output into CSV
# ---------------------------------------------------------------------------

# Divan output format (typical):
#   bench_name
#   ├─ arg1  fastest  │ slowest  │ median   │ mean     │ samples │ iters
#   │        1.234 ns │ 5.678 ns │ 2.345 ns │ 2.456 ns │ 100     │ 1000
#   ╰─ arg2  ...
#
# Or without args:
#   bench_name  fastest  │ slowest  │ median   │ mean     │ samples │ iters
#               1.234 ns │ 5.678 ns │ 2.345 ns │ 2.456 ns │ 100     │ 1000

parse_divan_to_csv() {
  local bench_target="$1"
  local timestamp="$2"

  # State machine: track current group/name/args
  python3 - "$bench_target" "$timestamp" <<'PYEOF'
import sys
import re

bench_target = sys.argv[1]
timestamp = sys.argv[2]

current_group = ""
current_name = ""
current_args = ""

# Divan lines with timing data contain "│" separators and units like "ns", "µs", "ms", "s"
timing_re = re.compile(
    r'^\s*'
    r'([\d.]+)\s+(ps|ns|µs|us|ms|s)\s*│\s*'  # fastest
    r'([\d.]+)\s+(ps|ns|µs|us|ms|s)\s*│\s*'  # slowest
    r'([\d.]+)\s+(ps|ns|µs|us|ms|s)\s*│\s*'  # median
    r'([\d.]+)\s+(ps|ns|µs|us|ms|s)'          # mean
)

# Multipliers to nanoseconds
UNIT_NS = {
    'ps': 0.001,
    'ns': 1.0,
    'µs': 1000.0,
    'us': 1000.0,
    'ms': 1_000_000.0,
    's': 1_000_000_000.0,
}

def to_ns(value, unit):
    return float(value) * UNIT_NS.get(unit, 1.0)

# Name line patterns
# Group header: "mod_name" (no │)
# Bench with args: "├─ arg_value  fastest │ ..."
# Bench name: "  bench_name  fastest │ ..."
name_with_args_re = re.compile(r'^[│├╰─\s]+([\w.]+)\s+fastest')
plain_name_re = re.compile(r'^(\w[\w:]*(?:\s*\w+)*)\s*$')

for line in sys.stdin:
    line = line.rstrip()
    if not line.strip():
        continue

    # Check for timing data line
    m = timing_re.search(line)
    if m:
        fastest_val, fastest_unit = m.group(1), m.group(2)
        slowest_val, slowest_unit = m.group(3), m.group(4)
        median_val, median_unit = m.group(5), m.group(6)
        mean_val, mean_unit = m.group(7), m.group(8)

        median_ns = to_ns(median_val, median_unit)
        mean_ns = to_ns(mean_val, mean_unit)

        # Output CSV line
        print(f"{timestamp},{bench_target},{current_group},{current_name},{current_args},{median_ns:.1f},{median_unit},{mean_ns:.1f}")
        continue

    # Check for bench name with args (tree branch characters)
    m = name_with_args_re.match(line)
    if m:
        current_args = m.group(1)
        continue

    # Check for plain name (group or bench name)
    stripped = line.strip()
    # Skip tree-drawing lines
    if stripped.startswith('│') or stripped.startswith('├') or stripped.startswith('╰'):
        continue

    m = plain_name_re.match(stripped)
    if m:
        name = m.group(1).strip()
        if '::' in name:
            # Qualified name like "group::bench"
            parts = name.rsplit('::', 1)
            current_group = parts[0]
            current_name = parts[1]
        else:
            # Could be a group header or a simple bench name
            current_group = name
            current_name = name
        current_args = ""
PYEOF
}

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

cmd_save() {
  local cargo_flags=("$@")
  local timestamp
  timestamp=$(date +%Y%m%d-%H%M%S)
  local commit
  commit=$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

  local txt_file="$BASELINE_DIR/bench-${timestamp}.txt"
  local csv_file="$BASELINE_DIR/bench-${timestamp}.csv"
  local meta_file="$BASELINE_DIR/bench-${timestamp}.meta"

  echo "Running benchmarks..."

  # Save metadata
  {
    echo "timestamp=$timestamp"
    echo "commit=$commit"
    echo "rustc=$(rustc --version 2>/dev/null || echo unknown)"
    echo "os=$(uname -srm)"
    echo "r_version=$(Rscript -e 'cat(R.version.string)' 2>/dev/null || echo unknown)"
    echo "cargo_flags=${cargo_flags[*]:-}"
  } > "$meta_file"

  # Run benchmarks, capture raw output
  cargo bench --manifest-path="$REPO_ROOT/miniextendr-bench/Cargo.toml" "${cargo_flags[@]}" 2>&1 | tee "$txt_file"

  # Parse into CSV
  echo "timestamp,bench_target,group,name,args,median_ns,unit,mean_ns" > "$csv_file"

  # Extract bench target names from the output and parse each
  # Divan prints "bench_target_name" as headers
  parse_divan_to_csv "all" "$timestamp" < "$txt_file" >> "$csv_file"

  local n_rows
  n_rows=$(wc -l < "$csv_file")
  n_rows=$((n_rows - 1))  # subtract header

  echo ""
  echo "Baseline saved:"
  echo "  Raw:  $txt_file"
  echo "  CSV:  $csv_file ($n_rows data points)"
  echo "  Meta: $meta_file"
}

cmd_compare() {
  local target_csv="${1:-}"

  if [[ -z "$target_csv" ]]; then
    # Find most recent baseline
    target_csv=$(ls -t "$BASELINE_DIR"/bench-*.csv 2>/dev/null | head -1)
    if [[ -z "$target_csv" ]]; then
      echo "No baseline CSV found. Run 'bash $0 save' first."
      exit 1
    fi
  fi

  echo "Baseline: $target_csv"
  echo ""
  echo "Top 20 slowest benchmarks (by median_ns):"
  echo ""

  # Sort by median_ns descending, show top 20
  tail -n +2 "$target_csv" | sort -t, -k6 -rn | head -20 | \
    awk -F, '{printf "  %-40s %12.1f %s\n", $3 "::" $4 "(" $5 ")", $6, $7}'
}

cmd_drift() {
  local threshold=20  # percent

  # Parse --threshold=N
  for arg in "$@"; do
    case "$arg" in
      --threshold=*) threshold="${arg#*=}" ;;
    esac
  done

  # Find two most recent baselines
  local files
  files=$(ls -t "$BASELINE_DIR"/bench-*.csv 2>/dev/null | head -2)
  local n_files
  n_files=$(echo "$files" | wc -l)

  if [[ $n_files -lt 2 ]]; then
    echo "Need at least 2 baselines for drift check. Run 'bash $0 save' twice."
    exit 1
  fi

  local current_csv previous_csv
  current_csv=$(echo "$files" | head -1)
  previous_csv=$(echo "$files" | tail -1)

  echo "Drift check (threshold: ${threshold}%)"
  echo "  Current:  $current_csv"
  echo "  Previous: $previous_csv"
  echo ""

  # Join on (group, name, args), compute percent change
  python3 - "$previous_csv" "$current_csv" "$threshold" <<'PYEOF'
import csv
import sys

prev_file = sys.argv[1]
curr_file = sys.argv[2]
threshold = float(sys.argv[3])

def load_csv(path):
    data = {}
    with open(path) as f:
        reader = csv.DictReader(f)
        for row in reader:
            key = (row['group'], row['name'], row['args'])
            data[key] = float(row['median_ns'])
    return data

prev = load_csv(prev_file)
curr = load_csv(curr_file)

regressions = []
improvements = []

for key in sorted(set(prev.keys()) & set(curr.keys())):
    p, c = prev[key], curr[key]
    if p == 0:
        continue
    pct = ((c - p) / p) * 100

    label = f"{key[0]}::{key[1]}({key[2]})"
    if pct > threshold:
        regressions.append((label, p, c, pct))
    elif pct < -threshold:
        improvements.append((label, p, c, pct))

if regressions:
    print(f"REGRESSIONS (>{threshold}% slower):")
    for label, p, c, pct in sorted(regressions, key=lambda x: -x[3]):
        print(f"  {label:<50s} {p:>10.1f} -> {c:>10.1f} ns  ({pct:+.1f}%)")
    print()

if improvements:
    print(f"IMPROVEMENTS (>{threshold}% faster):")
    for label, p, c, pct in sorted(improvements, key=lambda x: x[3]):
        print(f"  {label:<50s} {p:>10.1f} -> {c:>10.1f} ns  ({pct:+.1f}%)")
    print()

if not regressions and not improvements:
    print(f"No significant drift detected (threshold: {threshold}%).")

if regressions:
    sys.exit(1)
PYEOF
}

cmd_info() {
  echo "Baselines in $BASELINE_DIR:"
  echo ""

  local count=0
  for meta in $(ls -t "$BASELINE_DIR"/bench-*.meta 2>/dev/null); do
    count=$((count + 1))
    local base
    base=$(basename "$meta" .meta)
    echo "  $base:"
    while IFS='=' read -r key value; do
      printf "    %-15s %s\n" "$key:" "$value"
    done < "$meta"
    echo ""
    if [[ $count -ge 5 ]]; then
      local total
      total=$(ls "$BASELINE_DIR"/bench-*.meta 2>/dev/null | wc -l)
      if [[ $total -gt 5 ]]; then
        echo "  ... and $((total - 5)) more"
      fi
      break
    fi
  done

  if [[ $count -eq 0 ]]; then
    echo "  (none)"
  fi
}

# ---------------------------------------------------------------------------
# Main dispatch
# ---------------------------------------------------------------------------

case "${1:-save}" in
  save)
    shift || true
    # Strip leading "--" if present
    [[ "${1:-}" == "--" ]] && shift
    cmd_save "$@"
    ;;
  compare)
    shift
    cmd_compare "$@"
    ;;
  drift)
    shift
    cmd_drift "$@"
    ;;
  info)
    cmd_info
    ;;
  *)
    echo "Usage: $0 {save|compare|drift|info} [options]"
    echo ""
    echo "Commands:"
    echo "  save [-- cargo_flags]     Run benchmarks and save structured baseline"
    echo "  compare [baseline.csv]    Show top benchmarks from a baseline"
    echo "  drift [--threshold=20]    Check for regressions between last 2 baselines"
    echo "  info                      List saved baselines with metadata"
    exit 1
    ;;
esac
