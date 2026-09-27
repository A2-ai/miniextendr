#!/usr/bin/env bash
# Tests for the Rosetta rerun helper (tests/webr-smoke-retry.sh) that
# tests/webr-smoke.sh wraps around every phase body. Runs without Docker: a
# fake step replays a plan of attempts, each "<exit status>:<marker|->", where
# "marker" makes that attempt print Rosetta's assertion line.
#
# Covered:
#   - a trap followed by a passing rerun succeeds after exactly one rerun;
#   - a trap that recurs on the rerun fails with the trap's status, no third try;
#   - a trap seen through make (exit 2, marker in the output) is rerun too;
#   - a plain failure and a SIGTRAP (133) without the marker are not rerun;
#   - the marker in the output of a successful attempt changes nothing;
#   - the step's own output reaches the caller (tee) and the per-attempt log
#     is removed afterwards.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

failures=0
pass() { echo "ok - $1"; }
nok() { echo "not ok - $1" >&2; failures=$((failures + 1)); }
check() {
  local name=$1
  shift
  if "$@"; then pass "$name"; else nok "$name"; fi
}
lacks() { ! grep -qF "$1" "$2"; }

# The helper expects the caller's logging functions.
warn() { echo "[ warn] $*" >&2; }
fail() { echo "[FAIL ] $*" >&2; }

# shellcheck source=tests/webr-smoke-retry.sh
source "$root/tests/webr-smoke-retry.sh"

# The helper's per-attempt logs land here; it must be empty after every case.
mkdir -p "$tmp/t"
export TMPDIR="$tmp/t"

# The step runs inside the helper's pipeline (a subshell), so its plan queue
# and call counter live in files, not shell variables.
fake_step() {
  local entry code what
  echo call >> "$tmp/calls"
  entry="$(head -n 1 "$tmp/plan")"
  tail -n +2 "$tmp/plan" > "$tmp/plan.rest"
  mv "$tmp/plan.rest" "$tmp/plan"
  code="${entry%%:*}"
  what="${entry#*:}"
  echo "fake step output, attempt $(wc -l < "$tmp/calls" | tr -d ' ')"
  if [[ "$what" == marker ]]; then
    echo "assertion failed [block != nullptr]: BasicBlock requested for unrecognized address (BuilderBase.h:550 block_for_offset)"
  fi
  return "$code"
}

# run_case <name> <want status> <want calls> <must show|-> <must not show|-> <plan...>
run_case() {
  local name=$1 want_rc=$2 want_calls=$3 shows=$4 hides=$5
  shift 5
  printf '%s\n' "$@" > "$tmp/plan"
  : > "$tmp/calls"
  local rc=0 calls before=$failures
  ( run_with_rosetta_retry "$name" fake_step ) > "$tmp/out" 2>&1 || rc=$?
  calls="$(wc -l < "$tmp/calls" | tr -d ' ')"
  check "$name: exit status $want_rc (got $rc)" test "$rc" -eq "$want_rc"
  check "$name: $want_calls attempt(s) (got $calls)" test "$calls" -eq "$want_calls"
  if [[ "$shows" != - ]]; then
    check "$name: output shows \"$shows\"" grep -qF "$shows" "$tmp/out"
  fi
  if [[ "$hides" != - ]]; then
    check "$name: output does not show \"$hides\"" lacks "$hides" "$tmp/out"
  fi
  check "$name: no helper log left behind" test -z "$(ls -A "$tmp/t")"
  if [[ $failures -gt $before ]]; then
    sed 's/^/    | /' "$tmp/out" >&2
  fi
}

run_case "trap then pass" 0 2 "Rerunning this step once" "recurred" 133:marker 0:-
run_case "trap twice" 133 2 "recurred on the rerun" - 133:marker 133:marker
run_case "trap through make" 0 2 "exit 2 after Rosetta" - 2:marker 0:-
run_case "plain failure" 1 1 - "Rerunning" 1:-
run_case "SIGTRAP without the marker" 133 1 - "Rerunning" 133:-
run_case "marker with success" 0 1 - "Rerunning" 0:marker
run_case "clean pass" 0 1 "fake step output, attempt 1" "Rerunning" 0:-

if [[ $failures -gt 0 ]]; then
  echo "$failures check(s) failed" >&2
  exit 1
fi
echo "all checks passed"
