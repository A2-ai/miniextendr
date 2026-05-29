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

    log "Running cleanup: scrubbing build objects + restoring native Makevars..."
    docker_run "
        set -euo pipefail
        rm -f /work/rpkg/src/*.o /work/rpkg/src/*.so
        ( cd /work/rpkg && bash ./configure ) 2>/dev/null || true
    " || warn "Cleanup configure failed — rpkg/src/Makevars may still be in wasm-mode. Run: cd rpkg && bash ./configure"

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
        # Scrub stale build objects: the bind-mounted /work is shared with the
        # host, so host-arch (e.g. macOS arm64) *.o/*.so from a prior
        # 'just rcmdinstall' have newer mtimes than their .c sources. R then
        # skips recompiling and the native build links the wrong object format
        # ('unknown file type'). cargo's target dir is triple-namespaced, so it
        # is unaffected.
        rm -f /work/rpkg/src/*.o /work/rpkg/src/*.so
        ( cd /work/rpkg && bash ./configure )
        ${R_NATIVE_EXE} CMD INSTALL --no-test-load \
            --library=${SMOKE_TMP}/native-lib \
            /work/rpkg
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
        # Scrub Phase 1's native objects so emcc recompiles .c -> wasm .o
        # (same stale-mtime trap as Phase 1, but native-poisoning-wasm here).
        rm -f /work/rpkg/src/*.o /work/rpkg/src/*.so
        ( cd /work/rpkg && CC=emcc bash ./configure )
        # Install to an empty host-side temp library, NOT directly into the
        # wasm tree. INSTALL's tail-end lazy-load spawns a sub-R that adds
        # --library to .libPaths(); if that path contains wasm grDevices.so
        # the sub-R dies with 'invalid ELF header' on startup. rwasm uses
        # tempfile() lib_dir for the same reason (see
        # r-wasm/rwasm:R/build.R, 'empty library otherwise R might try to
        # load wasm packages' comment), then copies into the final tree.
        # --no-byte-compile mirrors rwasm's flag set.
        rm -rf /tmp/wasm-lib && mkdir -p /tmp/wasm-lib
        WASM_TOOLS=${WASM_TOOLS} \
        R_SOURCE=${R_SOURCE} \
        R_MAKEVARS_USER=${WEBR_VARS_MK} \
        ${R_HOST_EXE} CMD INSTALL \
            --library=/tmp/wasm-lib \
            --no-docs --no-test-load --no-staged-install --no-byte-compile \
            /work/rpkg
    "

    # Verify the wasm library landed. Phase 3 NODEFS-mounts /tmp/wasm-lib
    # directly (matching CI tier-3 / smoke.mjs's HOST_WASM_LIB), so this is
    # the canonical location — no copy into the wasm site-lib needed.
    log "Verifying wasm library installation..."
    if ! docker_run "test -d '/tmp/wasm-lib/miniextendr'"; then
        fail "wasm library not found at /tmp/wasm-lib/miniextendr"
        exit 1
    fi

    ok "wasm32 side-module installed to /tmp/wasm-lib/miniextendr"
}

# ── Phase 3: webR Node.js session ───────────────────────────────────────────
# Mirrors the green webr.yml tier-3 job step-for-step: rebuild webR's Node
# bundle (the published image strips /opt/webr/src/dist via `make clean`), then
# run the CANONICAL runner tests/webr-node-smoke/smoke.mjs — the same script CI
# uses, so there is no second, drifting copy of the runner to maintain. smoke.mjs
# NODEFS-mounts /tmp/wasm-lib (Phase 2's output) and drives library(miniextendr).
# See docs/WEBR.md "Running a webR session in Node (the two-bundle gotcha)".

phase_webr_session() {
    step "Phase 3: webR Node.js session"

    # Only the /opt/webr/src/dist bundle runs in Node (the /opt/webr/dist one is
    # the browser build); the image deletes src/dist + src/node_modules to stay
    # small, so rebuild before importing. ~20s on a warm image.
    log "Rebuilding webR's Node bundle (cd /opt/webr/src && make ...)..."
    docker_run "
        set -euo pipefail
        cd /opt/webr/src
        if [ ! -f package-lock.json ]; then
            echo 'ERROR: /opt/webr/src/package-lock.json absent — base image layout changed.' >&2
            ls -la /opt/webr/src/ >&2 || true
            exit 1
        fi
        make /opt/webr/src/dist/webr.mjs
        test -f /opt/webr/src/dist/webr.mjs
        test -f /opt/webr/src/dist/R.wasm   # confirms the asset-copy step ran
    "

    log "Running canonical Node smoke runner (tests/webr-node-smoke/smoke.mjs)..."
    docker_run "
        set -euo pipefail
        cd /work/tests/webr-node-smoke
        timeout 900 node smoke.mjs
    "

    ok "webR session complete (library(miniextendr) loaded)."
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
