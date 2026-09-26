# shellcheck shell=bash
# tests/webr-smoke-retry.sh: sourced by tests/webr-smoke.sh (and by
# tests/webr-smoke-retry-test.sh, which exercises it without Docker).
#
# Docker Desktop runs the amd64 webR image under Rosetta on Apple Silicon.
# Rosetta can fail to translate node's JIT code: node dies with SIGTRAP
# (exit 133) right after printing
#   assertion failed [block != nullptr]: BasicBlock requested for unrecognized address
# at a random point, and the same step passes on a rerun (#1254). Match the
# assertion text, not the exit code: 133 is any SIGTRAP, and a trap inside
# make's npm child surfaces as make's own exit 2. Callers define warn and fail.

ROSETTA_TRAP_MARKER="BasicBlock requested for unrecognized address"
ROSETTA_RERUNS=0

# run_with_rosetta_retry <label> <command...>
# Runs the command with its output teed to the terminal and a log. If it fails
# and the log holds the Rosetta assertion, runs it once more. Returns the last
# attempt's exit status.
run_with_rosetta_retry() {
    local label="$1"
    shift
    local log rc attempt
    log="$(mktemp "${TMPDIR:-/tmp}/webr-smoke.XXXXXX")"
    for attempt in 1 2; do
        set +e
        "$@" 2>&1 | tee "$log"
        rc=${PIPESTATUS[0]}
        set -e
        if [[ $rc -eq 0 ]] || ! grep -qF "$ROSETTA_TRAP_MARKER" "$log"; then
            break
        fi
        if [[ $attempt -eq 1 ]]; then
            ROSETTA_RERUNS=$((ROSETTA_RERUNS + 1))
            warn "${label}: exit ${rc} after Rosetta's \"${ROSETTA_TRAP_MARKER}\" assertion."
            warn "Rosetta failed to translate node's JIT code (amd64 image on Apple Silicon); not a package failure. See docs/WEBR.md, #1254. Rerunning this step once."
        else
            fail "${label}: the Rosetta trap recurred on the rerun. Run the smoke again, or use the arm64-native image (docs/WEBR.md)."
        fi
    done
    rm -f "$log"
    return "$rc"
}
