#!/usr/bin/env bash
# tests/webr-smoke.sh
#
# Local smoke test: builds rpkg as a wasm32 side-module inside the
# miniextendr-webr-dev container, loads it in a webR Node.js session,
# and runs the full testthat suite under wasm.
#
# Exit codes:
#   0  — miniextendr loaded (test failures are expected and tolerated)
#   1  — infrastructure failure (docker, build error, library() crash)
#
# Usage:
#   bash tests/webr-smoke.sh [--rebuild-image] [--no-cache] [--keep] [-h|--help]
#
# Options:
#   --rebuild-image   Force re-build of the docker image before running.
#   --no-cache        Pass --no-cache to docker build (implies --rebuild-image).
#   --keep            Don't clean up /tmp/webr-smoke inside the container on exit.
#   -h, --help        Show this help text and exit.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MX_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Constants ───────────────────────────────────────────────────────────────

IMAGE="miniextendr-webr-dev:latest"
R_VERSION="4.5.1"
WEBR_ROOT="/opt/webr"
R_WASM_LIB="${WEBR_ROOT}/wasm/R-${R_VERSION}/lib/R/library"
R_HOST_EXE="${WEBR_ROOT}/host/R-${R_VERSION}/bin/R"
R_NATIVE_EXE="/opt/R/current/bin/R"
WASM_TOOLS="${WEBR_ROOT}/tools"
R_SOURCE="${WEBR_ROOT}/R/build/R-${R_VERSION}"
WEBR_VARS_MK="${WEBR_ROOT}/packages/webr-vars.mk"
SMOKE_TMP="/tmp/webr-smoke"

# ── Argument parsing ─────────────────────────────────────────────────────────

REBUILD_IMAGE=0
NO_CACHE=""
KEEP=0

usage() {
    grep '^#' "$0" | grep -v '^#!/' | sed 's/^# \{0,1\}//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rebuild-image) REBUILD_IMAGE=1 ;;
        --no-cache)      NO_CACHE="--no-cache"; REBUILD_IMAGE=1 ;;
        --keep)          KEEP=1 ;;
        -h|--help)       usage ;;
        *) echo "Unknown option: $1" >&2; echo "Run with --help for usage." >&2; exit 1 ;;
    esac
    shift
done

# ── Logging helpers ──────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    CLR_RESET="\033[0m"
    CLR_BOLD="\033[1m"
    CLR_GREEN="\033[0;32m"
    CLR_YELLOW="\033[0;33m"
    CLR_RED="\033[0;31m"
    CLR_CYAN="\033[0;36m"
else
    CLR_RESET="" CLR_BOLD="" CLR_GREEN="" CLR_YELLOW="" CLR_RED="" CLR_CYAN=""
fi

log()  { printf "${CLR_CYAN}[smoke]${CLR_RESET} %s\n" "$*"; }
ok()   { printf "${CLR_GREEN}[  ok ]${CLR_RESET} %s\n" "$*"; }
warn() { printf "${CLR_YELLOW}[ warn]${CLR_RESET} %s\n" "$*" >&2; }
fail() { printf "${CLR_RED}[FAIL ]${CLR_RESET} %s\n" "$*" >&2; }
step() { printf "\n${CLR_BOLD}==> %s${CLR_RESET}\n" "$*"; }

# ── docker helpers ────────────────────────────────────────────────────────────

# docker_run: run a bash -c script inside the container with the repo bind-mounted.
docker_run() {
    local script="$1"
    docker run --rm \
        -v "${MX_ROOT}:/work" \
        -w /work \
        "${IMAGE}" \
        bash -c "$script"
}

# docker_pipe: pipe stdin as a script to bash inside the container.
docker_pipe() {
    docker run --rm -i \
        -v "${MX_ROOT}:/work" \
        -w /work \
        "${IMAGE}" \
        bash
}

# ── Cleanup trap ─────────────────────────────────────────────────────────────
# Runs native configure inside the container to restore rpkg/src/Makevars
# to host (non-wasm) state. Phase 2 leaves Makevars in wasm-mode; this
# ensures the user's checkout isn't left in a broken state.

_cleanup_done=0
cleanup() {
    if [[ $_cleanup_done -eq 1 ]]; then return; fi
    _cleanup_done=1

    log "Running cleanup: restoring native Makevars..."
    docker_run "
        set -euo pipefail
        cd /work
        bash rpkg/configure 2>/dev/null || true
    " || warn "Cleanup configure failed — rpkg/src/Makevars may still be in wasm-mode. Run: bash rpkg/configure"

    if [[ $KEEP -eq 0 ]]; then
        log "Removing ${SMOKE_TMP} inside container..."
        docker_run "rm -rf ${SMOKE_TMP}" 2>/dev/null || true
    else
        log "--keep set: leaving ${SMOKE_TMP} inside container for inspection."
    fi
}
trap cleanup EXIT

# ── Preflight checks ─────────────────────────────────────────────────────────

preflight() {
    step "Preflight"

    if ! command -v docker &>/dev/null; then
        fail "docker not found on PATH."
        exit 1
    fi

    if ! docker info &>/dev/null; then
        fail "Docker daemon not running or not accessible."
        exit 1
    fi

    if [[ $REBUILD_IMAGE -eq 1 ]]; then
        log "Building docker image ${IMAGE} (--rebuild-image)..."
        # shellcheck disable=SC2086
        docker build $NO_CACHE -f "${MX_ROOT}/Dockerfile.webr" -t "${IMAGE}" "${MX_ROOT}"
        ok "Image built."
    else
        if ! docker image inspect "${IMAGE}" &>/dev/null; then
            log "Image ${IMAGE} not found locally — building..."
            docker build -f "${MX_ROOT}/Dockerfile.webr" -t "${IMAGE}" "${MX_ROOT}"
            ok "Image built."
        else
            ok "Image ${IMAGE} present."
        fi
    fi

    log "Creating ${SMOKE_TMP} in container..."
    docker_run "mkdir -p ${SMOKE_TMP}"
}

# ── Phase 1: Native install (regenerates wasm_registry.rs) ──────────────────
# R CMD INSTALL with the host R triggers the cdylib build path which writes
# wasm_registry.rs with the live slice contents. Without this, the wasm build
# compiles but registers zero R routines.

phase_native_install() {
    step "Phase 1: Native install (regenerating wasm_registry.rs)"

    log "Running native R CMD INSTALL inside container..."
    docker_run "
        set -euo pipefail
        mkdir -p ${SMOKE_TMP}/native-lib
        cd /work
        bash rpkg/configure
        ${R_NATIVE_EXE} CMD INSTALL --no-test-load \
            --library=${SMOKE_TMP}/native-lib \
            rpkg
    "

    # Verify wasm_registry.rs was regenerated (content-hash must not be stub value)
    log "Checking wasm_registry.rs content-hash..."
    local content_hash
    content_hash="$(docker_run "grep 'content-hash:' /work/rpkg/src/rust/wasm_registry.rs | awk '{print \$NF}'")"

    if [[ "$content_hash" == "0000000000000000" ]]; then
        fail "Phase 1 did not regenerate wasm_registry.rs — cdylib build path may be broken."
        fail "content-hash is still the stub value (0000000000000000)."
        fail "Check that native R CMD INSTALL ran the cdylib pass (IS_WASM_INSTALL must be false)."
        exit 1
    fi

    ok "wasm_registry.rs regenerated (content-hash: ${content_hash})"
}

# ── Phase 2: wasm32 side-module build ───────────────────────────────────────
# CC=emcc triggers the IS_WASM_INSTALL=true branch in configure.ac, which
# sets up the wasm32-unknown-emscripten target + build-std flags.

phase_wasm_build() {
    step "Phase 2: wasm32 side-module build"

    log "Running wasm32 R CMD INSTALL inside container..."
    docker_run "
        set -euo pipefail
        cd /work
        CC=emcc bash rpkg/configure
        WASM_TOOLS=${WASM_TOOLS} \
        R_SOURCE=${R_SOURCE} \
        R_MAKEVARS_USER=${WEBR_VARS_MK} \
        ${R_HOST_EXE} CMD INSTALL \
            --library=${R_WASM_LIB} \
            --no-docs --no-test-load --no-staged-install \
            /work/rpkg
    "

    # Verify the wasm library was installed
    log "Verifying wasm library installation..."
    if ! docker_run "test -d '${R_WASM_LIB}/miniextendr'"; then
        fail "wasm library not found at ${R_WASM_LIB}/miniextendr"
        exit 1
    fi

    ok "wasm32 side-module installed to ${R_WASM_LIB}/miniextendr"
}

# ── write_runner_files ────────────────────────────────────────────────────────
# Writes package.json and smoke-runner.mjs into the container's SMOKE_TMP dir
# by piping a heredoc to bash inside the container.
#
# Single-quoted EOF markers prevent the outer shell from interpolating
# ${...} and backticks — those must reach the container verbatim (for the
# .mjs file) and also prevent host-shell expansion inside the heredoc bodies.
# Paths are hardcoded as the literal string /tmp/webr-smoke (not $SMOKE_TMP)
# because single-quoted heredocs do no expansion. The guard below asserts that
# SMOKE_TMP matches that literal so future refactors that change the variable
# fail loud rather than silently writing files to the wrong place.

write_runner_files() {
    # Belt-and-suspenders: single-quoted heredocs hardcode /tmp/webr-smoke.
    # If SMOKE_TMP ever changes, this blows up immediately instead of silently
    # writing runner files to the wrong directory inside the container.
    [[ "${SMOKE_TMP}" == "/tmp/webr-smoke" ]] || {
        fail "SMOKE_TMP must be /tmp/webr-smoke; heredocs hardcode this path — update write_runner_files if you change it."
        exit 1
    }
    # package.json: no shell variables needed in the content; use single-quoted EOF.
    docker_pipe <<'OUTER_EOF'
set -euo pipefail
cat > /tmp/webr-smoke/package.json <<'EOF'
{
  "name": "webr-miniextendr-smoke",
  "version": "1.0.0",
  "type": "module",
  "private": true,
  "dependencies": {
    "webr": "file:///opt/webr/src"
  }
}
EOF
OUTER_EOF

    # smoke-runner.mjs: pure JS — no bash variable expansion needed.
    # All ${...} inside are JavaScript template literals / string interpolation,
    # not bash. Use single-quoted OUTER_EOF to pass them through verbatim.
    docker_pipe <<'OUTER_EOF'
set -euo pipefail
cat > /tmp/webr-smoke/smoke-runner.mjs <<'EOF'
import { WebR } from "webr";

const R_WASM_LIB = "/opt/webr/wasm/R-4.5.1/lib/R/library";
const TESTTHAT_LOG = "/tmp/webr-smoke/testthat.log";

async function main() {
  console.log("[smoke] Initialising webR...");
  const webR = new WebR({
    baseUrl: "file:///opt/webr/dist/",
    interactive: false,
  });
  await webR.init();
  console.log("[smoke] webR initialised.");

  // Mount the wasm R library into the virtual filesystem.
  await webR.FS.mkdir("/wasm-rlib");
  await webR.FS.mount("NODEFS", { root: R_WASM_LIB }, "/wasm-rlib");
  await webR.evalR('.libPaths(c("/wasm-rlib", .libPaths()))');
  console.log("[smoke] NODEFS mount done.");

  // Try to install testthat from repo.r-wasm.org.
  // If the network fetch fails (no HTTPS from container), skip the suite.
  let hasTestthat = false;
  try {
    console.log("[smoke] Installing testthat from repo.r-wasm.org ...");
    await webR.installPackages(["testthat"], { mount: false, quiet: false });
    hasTestthat = true;
    console.log("[smoke] testthat installed.");
  } catch (e) {
    console.warn("[smoke] WARNING: Could not install testthat:", e.message);
    console.warn("[smoke] Skipping testthat suite — will check library() only.");
  }

  // CRITICAL CHECK: library(miniextendr) must succeed.
  console.log("[smoke] Loading miniextendr...");
  const loadResult = await webR.evalR(`
    tryCatch({
      library(miniextendr)
      "OK"
    }, error = function(e) {
      paste0("ERROR: ", conditionMessage(e))
    })
  `);
  const loadStatus = await loadResult.toJs();
  // R returns a character vector; unwrap to string.
  const loadMsg = (loadStatus && loadStatus.values) ? loadStatus.values[0] : String(loadStatus);
  if (typeof loadMsg !== "string" || !loadMsg.startsWith("OK")) {
    console.error("[smoke] FAIL: library(miniextendr) failed:", loadMsg);
    process.exitCode = 1;
    return;
  }
  console.log("[smoke] OK: library(miniextendr) succeeded.");

  if (!hasTestthat) {
    console.log("[smoke] Smoke test PASSED (library load only — testthat unavailable).");
    return;
  }

  // Run testthat. Many tests will fail (worker/fork/threading); that is expected.
  // We capture output to a log file and report only counts.
  console.log("[smoke] Running testthat suite (output -> " + TESTTHAT_LOG + ")...");
  const resultsRaw = await webR.evalR(`
    tryCatch({
      sink(file = "/tmp/webr-smoke/testthat.log", append = FALSE, split = FALSE)
      on.exit(sink(NULL), add = TRUE)

      result <- tryCatch(
        testthat::test_local("/work/rpkg", reporter = "summary", stop_on_failure = FALSE),
        error = function(e) {
          # test_local may not exist in older testthat; fall back to test_dir.
          testthat::test_dir("/work/rpkg/tests/testthat", reporter = "summary",
                             stop_on_failure = FALSE)
        }
      )

      list(
        passed   = sum(as.integer(result[["passed"]])),
        failed   = sum(as.integer(result[["failed"]])),
        skipped  = sum(as.integer(result[["skipped"]])),
        warnings = sum(as.integer(result[["warning"]]))
      )
    }, error = function(e) {
      list(passed = -1L, failed = -1L, skipped = -1L, warnings = -1L,
           suite_error = conditionMessage(e))
    })
  `);

  // Unwrap the R named list.
  const counts = await resultsRaw.toJs();
  const asMap = {};
  if (counts && counts.names && counts.values) {
    for (let i = 0; i < counts.names.length; i++) {
      const v = counts.values[i];
      asMap[counts.names[i]] = (v && v.values !== undefined) ? v.values[0] : v;
    }
  }

  if (asMap.suite_error) {
    console.warn("[smoke] WARNING: testthat runner error:", asMap.suite_error);
    console.log("[smoke] Smoke test PASSED (library loaded; suite runner error above).");
    return;
  }

  const p = (asMap.passed  ?? "?");
  const f = (asMap.failed  ?? "?");
  const s = (asMap.skipped ?? "?");
  const w = (asMap.warnings ?? "?");

  console.log(`[smoke] testthat: passed=${p}  failed=${f}  skipped=${s}  warnings=${w}`);
  console.log("[smoke] (Many failures are expected under wasm — worker/fork/thread assumptions.)");
  console.log("[smoke] Full output -> " + TESTTHAT_LOG);
  console.log("[smoke] Smoke test PASSED.");
}

main().catch((e) => {
  console.error("[smoke] Uncaught runner error:", e);
  process.exitCode = 1;
});
EOF
OUTER_EOF
}

# ── Phase 3: webR Node.js session ───────────────────────────────────────────
# Writes a temp package.json linking to the local webR source, runs
# npm install (creates a symlink, fast), then runs the Node.js ESM runner.

phase_webr_session() {
    step "Phase 3: webR Node.js session"

    log "Writing Node.js runner files..."
    write_runner_files

    log "Installing npm deps (local webR link)..."
    docker_run "
        set -euo pipefail
        cd ${SMOKE_TMP}
        npm install --prefer-offline 2>&1
    "

    log "Running Node.js smoke runner..."
    docker_run "
        set -euo pipefail
        cd ${SMOKE_TMP}
        timeout 1800 node smoke-runner.mjs
    "

    ok "webR session complete."

    # Print the tail of the testthat log if it was produced.
    if docker_run "test -f ${SMOKE_TMP}/testthat.log" 2>/dev/null; then
        log "--- testthat log (last 40 lines) ---"
        docker_run "tail -n 40 ${SMOKE_TMP}/testthat.log" || true
        log "--- end of log ---"
    fi
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
    printf "\n${CLR_BOLD}miniextendr webR smoke test${CLR_RESET}\n"
    printf "Image:    %s\n" "${IMAGE}"
    printf "Repo:     %s\n" "${MX_ROOT}"
    printf "R:        %s\n" "${R_VERSION}"
    printf "\n"

    preflight
    phase_native_install
    phase_wasm_build
    phase_webr_session

    printf "\n${CLR_GREEN}${CLR_BOLD}Smoke test PASSED.${CLR_RESET}\n\n"
}

main
