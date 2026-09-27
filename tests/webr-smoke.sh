#!/usr/bin/env bash
# tests/webr-smoke.sh
#
# Local smoke test: builds rpkg as a wasm32 side-module inside the
# miniextendr-webr-dev container and loads it in a webR Node.js session
# via the canonical runner tests/webr-node-smoke/smoke.mjs. By default the
# runner then finishes with an informational testthat pass (#1255): suite
# counts are reported but test failures never gate — many tests legitimately
# fail or skip under wasm (worker-thread / fork / subprocess assumptions).
# Disable with SMOKE_TESTTHAT=0; only a harness error before the counts
# line turns the gate red.
#
# Every phase runs in ONE container (`docker run -d ... sleep infinity`, then
# `docker exec` per step): each phase reads what the previous one left under
# /tmp (the wasm library, the scaffolds) and /opt/webr/src/dist (the Node
# bundle), which a fresh `docker run --rm` per step would lose (#1290). CI
# gets the same effect from its single job container.
#
# On Apple Silicon the amd64 image runs under Rosetta, which can kill node
# with SIGTRAP after printing "BasicBlock requested for unrecognized address"
# (#1254). A phase that fails with that assertion in its output is rerun once
# (tests/webr-smoke-retry.sh); a second trap fails the run. The arm64-native
# image (WEBR_ARM64=1) does not run under Rosetta.
#
# Exit codes:
#   0     miniextendr loaded in the webR session
#   else  the failing step's exit status: 1 for an infrastructure failure
#         (docker, build error, library() crash); 133 is SIGTRAP
#
# Usage:
#   bash tests/webr-smoke.sh [--rebuild-image] [--no-cache] [--keep] [--scaffold] [-h|--help]
#
# Options:
#   --rebuild-image   Force re-build of the docker image before running.
#   --no-cache        Pass --no-cache to docker build (implies --rebuild-image).
#   --keep            Leave the container running on exit, with every phase's
#                     output in place; the script prints how to enter and
#                     remove it.
#   --scaffold        Add the scaffold-leg phase (#1270), reproducing the CI
#                     scaffold legs (#1259 standalone, #1271 monorepo) locally:
#                     installs minirextendr from this checkout, scaffolds a
#                     fresh end-user package (mxsmoke) via
#                     create_miniextendr_package() AND a fresh monorepo
#                     (mxmono) via create_miniextendr_monorepo(), points their
#                     framework git deps at this checkout with
#                     use_local_miniextendr(), and repeats the native -> wasm
#                     two-step install on each into the same /tmp/wasm-lib
#                     Phase 2 uses. Phase 3 then also loads mxsmoke + mxmono.
#                     Off by default — without this flag, behavior is
#                     byte-identical to today.
#   -h, --help        Show this help text and exit.
#
# Environment:
#   WEBR_ARM64=1      arm64-native dev path (#788, ⚠️ DRAFT: Phase 1 needs
#                     miniextendr-api to compile for aarch64 Linux, #1254).
#                     Selects Dockerfile.webr-arm64 + the
#                     arm64 image, and orchestrates both R passes through the
#                     native arm64 R on PATH (the donor's amd64 host R trees
#                     are not copied into the arm64 image, only the portable
#                     wasm sysroot emcc links against). Unset/default = the
#                     amd64 path (Rosetta under Docker Desktop).
#   WEBR_IMAGE=<tag>  Image to run, and to build from the Dockerfile when the
#                     tag is missing locally or --rebuild-image is given.
#                     Default: miniextendr-webr-dev:latest, or
#                     miniextendr-webr-dev-arm64:latest with WEBR_ARM64=1.
#   WEBR_SCAFFOLD=1   Same as --scaffold.
#   SMOKE_TESTTHAT=0  Skip the informational testthat pass in Phase 3
#                     (default: enabled, #1255). Test failures never gate
#                     either way — the pass only reports counts.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MX_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── arm64 vs amd64 selection (#788) ──────────────────────────────────────────
# Guarded so the existing amd64 CI / Rosetta path is byte-for-byte unchanged
# when WEBR_ARM64 is unset.
WEBR_ARM64="${WEBR_ARM64:-0}"

# ── Constants ───────────────────────────────────────────────────────────────

R_VERSION="4.6.0"
WEBR_ROOT="/opt/webr"
WASM_TOOLS="${WEBR_ROOT}/tools"
R_SOURCE="${WEBR_ROOT}/R/build/R-${R_VERSION}"
WEBR_VARS_MK="${WEBR_ROOT}/packages/webr-vars.mk"
SMOKE_TMP="/tmp/webr-smoke"
# Library both R binaries read (R_LIBS_USER) for rpkg's Imports in Phase 2 and
# the scaffold tooling, as in CI's job-level R_LIBS_USER (webr.yml).
SHARED_R_LIBS="/tmp/r-shared-lib"

if [[ "$WEBR_ARM64" == "1" ]]; then
    IMAGE="${WEBR_IMAGE:-miniextendr-webr-dev-arm64:latest}"
    DOCKERFILE="${MX_ROOT}/Dockerfile.webr-arm64"
    # The arm64 image carries no amd64 host R (/opt/webr/host/R-4.6.0 and the
    # donor's /opt/R are not copied) — use the native rig-installed arm64 R on
    # PATH for BOTH passes. webr-vars.mk still drives CC=emcc for the wasm pass,
    # so which R orchestrates the install doesn't change the compiler.
    R_HOST_EXE="R"
    R_NATIVE_EXE="R"
else
    IMAGE="${WEBR_IMAGE:-miniextendr-webr-dev:latest}"
    DOCKERFILE="${MX_ROOT}/Dockerfile.webr"
    R_HOST_EXE="${WEBR_ROOT}/host/R-${R_VERSION}/bin/R"
    R_NATIVE_EXE="/opt/R/current/bin/R"
fi
# Rscript siblings of the two R binaries above (#1270 scaffold leg: roxygen2
# and minirextendr calls run via -e, not CMD INSTALL).
R_HOST_RSCRIPT="${R_HOST_EXE%R}Rscript"
R_NATIVE_RSCRIPT="${R_NATIVE_EXE%R}Rscript"

# ── Scaffold leg (#1270) constants ──────────────────────────────────────────
# Local parity with the CI scaffold leg (#1259, .github/workflows/webr.yml).
# Paths match the CI step names 1:1 so container-side debug output lines up
# with the workflow logs.
SCAFFOLD_PKG_NAME="mxsmoke"
SCAFFOLD_DIR="/tmp/scaffold"
SCAFFOLD_PKG_DIR="${SCAFFOLD_DIR}/${SCAFFOLD_PKG_NAME}"
SCAFFOLD_NATIVE_LIB="/tmp/scaffold-native-lib"
SCAFFOLD_R_LIBS="${SHARED_R_LIBS}"

# ── Monorepo scaffold leg (#1271) constants ─────────────────────────────────
# Local parity with the CI "Monorepo scaffold leg" steps (main-push/dispatch
# only in CI; locally it rides along whenever --scaffold is given — a local
# run wants the coverage). Paths match the CI steps 1:1.
MONO_PKG_NAME="mxmono"
MONO_DIR="/tmp/scaffold-mono"
MONO_ROOT_DIR="${MONO_DIR}/monorepo"
MONO_PKG_DIR="${MONO_ROOT_DIR}/${MONO_PKG_NAME}"
MONO_NATIVE_LIB="/tmp/scaffold-mono-native-lib"

# ── Argument parsing ─────────────────────────────────────────────────────────

REBUILD_IMAGE=0
NO_CACHE=""
KEEP=0
SCAFFOLD="${WEBR_SCAFFOLD:-0}"

usage() {
    grep '^#' "$0" | grep -v '^#!/' | sed 's/^# \{0,1\}//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rebuild-image) REBUILD_IMAGE=1 ;;
        --no-cache)      NO_CACHE="--no-cache"; REBUILD_IMAGE=1 ;;
        --keep)          KEEP=1 ;;
        --scaffold)      SCAFFOLD=1 ;;
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

# run_with_rosetta_retry (uses warn / fail above).
# shellcheck source=tests/webr-smoke-retry.sh
source "${SCRIPT_DIR}/webr-smoke-retry.sh"

# ── docker helpers ────────────────────────────────────────────────────────────

# The one container every phase runs in (#1290). Set by start_container only
# once `docker run` succeeded, so cleanup knows whether there is one.
CONTAINER=""

# start_container: start the smoke container with the repo bind-mounted at
# /work. `sleep infinity` keeps it alive between `docker exec` calls; --init
# reaps the exec'd processes; --rm removes it once it is stopped.
start_container() {
    local name="miniextendr-webr-smoke-$$"
    log "Starting container ${name} from ${IMAGE}..."
    docker run -d --rm --init --name "$name" \
        -v "${MX_ROOT}:/work" \
        -w /work \
        "${IMAGE}" \
        sleep infinity >/dev/null
    CONTAINER="$name"
    ok "Container ${CONTAINER} running."
}

# docker_run: run a bash -c script inside the smoke container.
# Extra positional args (after $script) are passed through to `docker exec` as
# additional flags (before the container name) — used by phase_scaffold to
# inject `-e VAR=value` env vars so its (single-quoted, verbatim) R heredocs
# never need to fight bash's own double-quote/backslash escaping rules.
docker_run() {
    local script="$1"
    shift || true
    docker exec -w /work "$@" "$CONTAINER" bash -c "$script"
}

# docker_step <label> <script> [docker exec flags...]: run a phase body. On a
# Rosetta trap the step is rerun once (tests/webr-smoke-retry.sh); every
# phase body is safe to rerun in the same container. Verification queries
# whose stdout is captured stay plain docker_run.
docker_step() {
    local label="$1"
    shift
    run_with_rosetta_retry "$label" docker_run "$@"
}

# ── Cleanup trap ─────────────────────────────────────────────────────────────
# Runs native configure inside the container to restore rpkg/src/Makevars
# to host (non-wasm) state. Phase 2 leaves Makevars in wasm-mode; this
# ensures the user's checkout isn't left in a broken state. Then removes the
# container, unless --keep.

_cleanup_done=0
cleanup() {
    if [[ $_cleanup_done -eq 1 ]]; then return; fi
    _cleanup_done=1
    # No container yet: preflight (or the image build) failed.
    if [[ -z "$CONTAINER" ]]; then return; fi

    log "Running cleanup: scrubbing build objects + restoring native Makevars..."
    docker_run "
        set -euo pipefail
        rm -f /work/rpkg/src/*.o /work/rpkg/src/*.so
        ( cd /work/rpkg && bash ./configure ) 2>/dev/null || true
    " || warn "Cleanup configure failed — rpkg/src/Makevars may still be in wasm-mode. Run: cd rpkg && bash ./configure"

    if [[ $KEEP -eq 0 ]]; then
        log "Removing container ${CONTAINER}..."
        docker rm -f "$CONTAINER" >/dev/null 2>&1 \
            || warn "Could not remove container ${CONTAINER}. Run: docker rm -f ${CONTAINER}"
    else
        log "--keep set: container ${CONTAINER} is still running, every phase's output in place."
        log "  Enter it:  docker exec -it ${CONTAINER} bash"
        log "  Remove it: docker rm -f ${CONTAINER}"
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
        log "Building docker image ${IMAGE} from ${DOCKERFILE##*/} (--rebuild-image)..."
        # shellcheck disable=SC2086
        docker build $NO_CACHE -f "${DOCKERFILE}" -t "${IMAGE}" "${MX_ROOT}"
        ok "Image built."
    else
        if ! docker image inspect "${IMAGE}" &>/dev/null; then
            log "Image ${IMAGE} not found locally — building from ${DOCKERFILE##*/}..."
            docker build -f "${DOCKERFILE}" -t "${IMAGE}" "${MX_ROOT}"
            ok "Image built."
        else
            ok "Image ${IMAGE} present."
        fi
    fi

    start_container
}

# ── Phase 1: Native install (regenerates wasm_registry.rs) ──────────────────
# R CMD INSTALL with the host R triggers the cdylib build path which writes
# wasm_registry.rs with the live slice contents. Without this, the wasm build
# compiles but registers zero R routines.

phase_native_install() {
    step "Phase 1: Native install (regenerating wasm_registry.rs)"

    log "Running native R CMD INSTALL inside container..."
    docker_step "Phase 1 (native install)" "
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
    docker_step "Phase 2 (wasm32 install)" "
        set -euo pipefail
        # Scrub Phase 1's native objects so emcc recompiles .c -> wasm .o
        # (same stale-mtime trap as Phase 1, but native-poisoning-wasm here).
        rm -f /work/rpkg/src/*.o /work/rpkg/src/*.so
        ( cd /work/rpkg && CC=emcc bash ./configure )
        # INSTALL's lazy-load loads rpkg's Imports in the R that runs it. In
        # the amd64 image that is webR's host R, whose library tree is not
        # the native R's: it cannot see transitive Imports such as rlang
        # (lifecycle -> rlang), which the native R keeps in its own system
        # library. Like CI tier 2 (webr.yml), install the Imports, derived
        # from rpkg/DESCRIPTION, through that R into a shared R_LIBS_USER.
        export R_LIBS_USER=${SHARED_R_LIBS}
        mkdir -p ${SHARED_R_LIBS}
        cat > /tmp/smoke-imports.R <<'RSCRIPT'
imports <- trimws(strsplit(read.dcf('/work/rpkg/DESCRIPTION')[1, 'Imports'], ',')[[1]])
imports <- sub('[ (].*', '', imports)
imports <- setdiff(imports, rownames(installed.packages(priority = 'base')))
install.packages(imports, dependencies = c('Depends', 'Imports', 'LinkingTo'),
                 repos = 'https://packagemanager.posit.co/cran/__linux__/noble/latest',
                 lib = Sys.getenv('R_LIBS_USER'))
miss <- imports[!vapply(imports, requireNamespace, logical(1), quietly = TRUE)]
if (length(miss)) stop('Unloadable after install: ', paste(miss, collapse = ', '))
RSCRIPT
        ${R_HOST_RSCRIPT} /tmp/smoke-imports.R
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
    # the canonical location — no copy into the wasm site-lib needed. The
    # side-module must start with the wasm magic bytes (00 61 73 6d): an ELF
    # file there means the install silently fell back to the native compiler.
    log "Verifying wasm library installation..."
    local so="/tmp/wasm-lib/miniextendr/libs/miniextendr.so"
    if ! docker_run "test -f ${so} && [ \"\$(od -An -tx1 -N4 ${so} | tr -d ' \n')\" = 0061736d ]"; then
        fail "${so} is missing or is not a wasm module."
        docker_run "ls -la /tmp/wasm-lib/miniextendr/libs/; od -An -tx1 -N16 ${so}" >&2 || true
        exit 1
    fi

    ok "wasm32 side-module installed to /tmp/wasm-lib/miniextendr"
}

# ── Phase Scaffold (#1270): end-user scaffolded-package wasm path ───────────
# Local parity with the CI scaffold legs (#1259 standalone + #1271 monorepo,
# .github/workflows/webr.yml): installs minirextendr from this checkout,
# scaffolds a fresh end-user package (mxsmoke) via
# create_miniextendr_package() AND a fresh monorepo (mxmono) via
# create_miniextendr_monorepo(), points their framework git deps at this
# checkout with use_local_miniextendr() (the same
# [patch."https://github.com/A2-ai/miniextendr"] mechanism rpkg's monorepo
# mode uses, so this validates the checkout's code, not published git
# sources), then repeats the native -> wasm two-step install on each into the
# SAME /tmp/wasm-lib Phase 2 uses (so Phase 3's NODEFS mount exposes all
# packages). In CI the monorepo leg is gated to main-push/dispatch (PR wall
# time); locally it always rides along with --scaffold — a local run wants
# the coverage. Gated behind --scaffold / WEBR_SCAFFOLD=1 — main() only calls
# this when $SCAFFOLD == 1; without the flag this phase never runs and
# behavior is byte-identical to today.
#
# The whole leg runs as a single docker_step() invocation, not one call per
# CI step, so a Rosetta rerun repeats the leg from the top (every step is
# idempotent: apt and install.packages, rm -rf before each create, git init
# reinitialises, installs overwrite). /tmp/scaffold, /tmp/scaffold-native-lib,
# /tmp/r-shared-lib and /tmp/wasm-lib are container-local paths (not
# bind-mounted like /work); the shared smoke container keeps them for
# Phase 3. The R snippets are written to temp .R files via
# backslash-escaped (verbatim, no-expansion) heredocs and the few
# container-side values that vary by WEBR_ARM64 are passed in as `docker exec
# -e` env vars — both choices sidestep nesting this script's own R string
# literals (which contain unescaped double quotes and regex `$`/`\` chars)
# inside this file's bash double-quoting.
phase_scaffold() {
    step "Phase Scaffold: end-user scaffolded-package wasm path (#1270 standalone, #1271 monorepo)"

    log "Installing minirextendr + roxygen tooling, scaffolding mxsmoke + mxmono, and building native + wasm..."
    # shellcheck disable=SC2016  # expanded inside the container
    docker_step "Phase Scaffold" '
        set -euo pipefail

        # Scaffold leg — install minirextendr + roxygen tooling into a
        # shared lib both R binaries below can see (mirrors CI job-level
        # R_LIBS_USER). git/libgit2-dev: git-init guard below + usethis
        # git probing / gert (libgit2 runtime dep). file: the wasm-vs-ELF
        # checks below (the webR base does not ship it; CI apt-installs it).
        apt-get update -qq
        apt-get install -y --no-install-recommends git libgit2-dev file
        mkdir -p "$SCAFFOLD_R_LIBS"
        export R_LIBS_USER="$SCAFFOLD_R_LIBS"
        cat > /tmp/scaffold-step1.R <<\RSCRIPT
options(HTTPUserAgent = sprintf("R/%s R (%s)", getRversion(),
  paste(getRversion(), R.version[["platform"]], R.version[["arch"]], R.version[["os"]])))
imports <- trimws(strsplit(read.dcf("minirextendr/DESCRIPTION")[1, "Imports"], ",")[[1]])
imports <- sub("\\s*\\(.*\\)$", "", imports)
imports <- setdiff(imports, rownames(installed.packages(priority = "base")))
pkgs <- union(imports, c("roxygen2", "pkgload", "pkgbuild"))
cat("Installing:", paste(pkgs, collapse = ", "), "\n")
install.packages(pkgs, dependencies = c("Depends", "Imports", "LinkingTo"),
                 repos = "https://packagemanager.posit.co/cran/__linux__/noble/latest",
                 lib = Sys.getenv("R_LIBS_USER"))
miss <- pkgs[!vapply(pkgs, requireNamespace, logical(1), quietly = TRUE)]
if (length(miss)) stop("Unloadable after install: ", paste(miss, collapse = ", "))
RSCRIPT
        "$R_HOST_RSCRIPT" /tmp/scaffold-step1.R
        "$R_HOST_EXE" CMD INSTALL --no-test-load --library="$SCAFFOLD_R_LIBS" /work/minirextendr

        # Scaffold leg — create a fresh end-user package.
        rm -rf "$SCAFFOLD_DIR" && mkdir -p "$SCAFFOLD_DIR"
        cat > /tmp/scaffold-step2.R <<\RSCRIPT
minirextendr::create_miniextendr_package(Sys.getenv("SCAFFOLD_PKG_DIR"), open = FALSE)
minirextendr::use_local_miniextendr("/work", path = Sys.getenv("SCAFFOLD_PKG_DIR"))
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step2.R
        # The template stock fns are kept as-is (NB: this whole phase body
        # is a single-quoted docker_run argument — no apostrophes in
        # comments here). rpkg (loaded first in the tier-3 webR session)
        # also exports an add — an *intentional* name collision: since
        # #1273 the generated C wrapper symbols are crate-prefixed
        # (C_mxsmoke_add vs C_miniextendr_add), so both packages sharing
        # one Emscripten side-module GOT must dispatch to their own
        # implementations. Phase 3 asserts exactly that.
        # configure never vendors, so a missing .git ancestor no longer flips
        # the mode; but the leaked-tarball guard (#1029) only arms in
        # git-tracked trees. git-init the scaffold so a leaked
        # inst/vendor.tar.xz fails loudly instead of freezing published git
        # sources in place of this checkout.
        git init -q "$SCAFFOLD_PKG_DIR"

        # Scaffold leg — native install + roxygen pass (wrappers,
        # wasm_registry.rs, NAMESPACE). C.UTF-8: the miniextendr package
        # init asserts the UTF-8 locale bit and refuses to dyn.load
        # otherwise; the container default locale is C.
        export LANG=C.UTF-8 LC_ALL=C.UTF-8
        ( cd "$SCAFFOLD_PKG_DIR" && bash ./configure )
        # use_local_miniextendr guard (#1309, parity with the CI legs):
        # configure must have emitted the local-checkout [patch] block into
        # the generated cargo config. If a marker regression made
        # use_local_miniextendr silently no-op, this leg would build the
        # *published* git sources instead of /work — and stay green while
        # validating the wrong code.
        cfg="$SCAFFOLD_PKG_DIR/src/rust/.cargo/config.toml"
        if ! grep -qF "[patch.\"https://github.com/A2-ai/miniextendr\"]" "$cfg"; then
            echo "FAIL: use_local_miniextendr [patch] block missing from $cfg — the scaffold would build the published miniextendr git sources, not /work." >&2
            cat "$cfg" >&2 || true
            exit 1
        fi
        mkdir -p "$SCAFFOLD_NATIVE_LIB"
        "$R_NATIVE_EXE" CMD INSTALL --no-test-load --library="$SCAFFOLD_NATIVE_LIB" "$SCAFFOLD_PKG_DIR"
        ch="$(grep "content-hash:" "$SCAFFOLD_PKG_DIR/src/rust/wasm_registry.rs")"
        ch="${ch##* }"
        echo "scaffold wasm_registry content-hash=$ch"
        test "$ch" != "0000000000000000"
        test -s "$SCAFFOLD_PKG_DIR/R/mxsmoke-wrappers.R"
        cat > /tmp/scaffold-step3.R <<\RSCRIPT
roxygen2::roxygenise(Sys.getenv("SCAFFOLD_PKG_DIR"))
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step3.R
        grep -q "useDynLib(mxsmoke" "$SCAFFOLD_PKG_DIR/NAMESPACE"
        "$R_NATIVE_EXE" CMD INSTALL --no-test-load --library="$SCAFFOLD_NATIVE_LIB" "$SCAFFOLD_PKG_DIR"
        cat > /tmp/scaffold-step4.R <<\RSCRIPT
library(mxsmoke, lib.loc = Sys.getenv("SCAFFOLD_NATIVE_LIB"))
stopifnot(identical(mxsmoke::add(2, 3), 5))
stopifnot(identical(mxsmoke::hello("webR"), "Hello, webR!"))
cat("scaffold native runtime sanity OK\n")
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step4.R

        # Scaffold leg — wasm32 R CMD INSTALL (template wasm branches),
        # into the SAME /tmp/wasm-lib Phase 2 uses so Phase 3s existing
        # NODEFS mount exposes both packages.
        rm -f "$SCAFFOLD_PKG_DIR"/src/*.o "$SCAFFOLD_PKG_DIR"/src/*.so
        ( cd "$SCAFFOLD_PKG_DIR" && CC=emcc bash ./configure )
        WASM_TOOLS="$WASM_TOOLS" \
        R_SOURCE="$R_SOURCE" \
        R_MAKEVARS_USER="$WEBR_VARS_MK" \
        "$R_HOST_EXE" CMD INSTALL \
            --library=/tmp/wasm-lib \
            --no-docs --no-test-load --no-staged-install --no-byte-compile \
            "$SCAFFOLD_PKG_DIR"

        # Scaffold leg — verify wasm side-module landed (sanity: must be
        # wasm, not ELF — would mean the wasm pass silently fell back to
        # native compilation).
        libdir="/tmp/wasm-lib/mxsmoke"
        test -d "$libdir"
        ls -la "$libdir/libs/"
        file "$libdir/libs/mxsmoke.so"
        file "$libdir/libs/mxsmoke.so" | grep -qi "WebAssembly\|wasm"

        # ── Monorepo scaffold leg (#1271) ───────────────────────────────
        # Same native -> roxygenise -> native -> CC=emcc sequence on a
        # package scaffolded from the MONOREPO template
        # (create_miniextendr_monorepo()), mirroring the CI "Monorepo
        # scaffold leg" steps. Runs in this same container so it reuses
        # the tooling installed above (same R_LIBS_USER) and lands in the
        # SAME /tmp/wasm-lib.

        # Monorepo scaffold leg — create a fresh end-user monorepo.
        rm -rf "$MONO_DIR" && mkdir -p "$MONO_DIR"
        cat > /tmp/scaffold-step5.R <<\RSCRIPT
minirextendr::create_miniextendr_monorepo(Sys.getenv("MONO_ROOT_DIR"),
  package = "mxmono", crate_name = "mxmono-core", open = FALSE)
minirextendr::use_local_miniextendr("/work", path = Sys.getenv("MONO_PKG_DIR"))
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step5.R
        # The template stock fns are kept as-is, same as mxsmoke above:
        # the monorepo rpkg template ships the same stock add/hello, and
        # rpkg + mxsmoke are loaded first in the tier-3 webR session — an
        # *intentional* name collision: since #1273 the generated C wrapper
        # symbols are crate-prefixed (C_mxmono_core_add here — the prefix
        # is the CRATE name, mxmono-core per the create step above — vs
        # C_mxsmoke_add / C_miniextendr_add), so all three packages sharing
        # one Emscripten side-module GOT must dispatch to their own
        # implementations. Phase 3 asserts exactly that.
        # Source-mode guard: unlike create_miniextendr_package() (which
        # needs the explicit git init above for mxsmoke), the monorepo
        # scaffolder git-inits the workspace root itself
        # (usethis::use_git()), so the rpkg subdir already has a .git
        # ancestor — assert it so a scaffolder regression cannot silently
        # disarm the leaked-tarball guard (#1029).
        test -d "$MONO_ROOT_DIR/.git"

        # Monorepo scaffold leg — native install + roxygen pass (wrappers,
        # wasm_registry.rs, NAMESPACE). LANG/LC_ALL=C.UTF-8 are still
        # exported from the mxsmoke leg above.
        ( cd "$MONO_PKG_DIR" && bash ./configure )
        # use_local_miniextendr guard (#1309): same assert as the mxsmoke
        # leg above, for the monorepo template configure.
        cfg="$MONO_PKG_DIR/src/rust/.cargo/config.toml"
        if ! grep -qF "[patch.\"https://github.com/A2-ai/miniextendr\"]" "$cfg"; then
            echo "FAIL: use_local_miniextendr [patch] block missing from $cfg — the scaffold would build the published miniextendr git sources, not /work." >&2
            cat "$cfg" >&2 || true
            exit 1
        fi
        mkdir -p "$MONO_NATIVE_LIB"
        "$R_NATIVE_EXE" CMD INSTALL --no-test-load --library="$MONO_NATIVE_LIB" "$MONO_PKG_DIR"
        ch="$(grep "content-hash:" "$MONO_PKG_DIR/src/rust/wasm_registry.rs")"
        ch="${ch##* }"
        echo "monorepo scaffold wasm_registry content-hash=$ch"
        test "$ch" != "0000000000000000"
        test -s "$MONO_PKG_DIR/R/mxmono-wrappers.R"
        cat > /tmp/scaffold-step6.R <<\RSCRIPT
roxygen2::roxygenise(Sys.getenv("MONO_PKG_DIR"))
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step6.R
        grep -q "useDynLib(mxmono" "$MONO_PKG_DIR/NAMESPACE"
        "$R_NATIVE_EXE" CMD INSTALL --no-test-load --library="$MONO_NATIVE_LIB" "$MONO_PKG_DIR"
        cat > /tmp/scaffold-step7.R <<\RSCRIPT
library(mxmono, lib.loc = Sys.getenv("MONO_NATIVE_LIB"))
stopifnot(identical(mxmono::add(2, 3), 5))
stopifnot(identical(mxmono::hello("webR"), "Hello, webR!"))
cat("monorepo scaffold native runtime sanity OK\n")
RSCRIPT
        "$R_NATIVE_RSCRIPT" /tmp/scaffold-step7.R

        # Monorepo scaffold leg — wasm32 R CMD INSTALL (monorepo template
        # wasm branches), into the SAME /tmp/wasm-lib.
        rm -f "$MONO_PKG_DIR"/src/*.o "$MONO_PKG_DIR"/src/*.so
        ( cd "$MONO_PKG_DIR" && CC=emcc bash ./configure )
        WASM_TOOLS="$WASM_TOOLS" \
        R_SOURCE="$R_SOURCE" \
        R_MAKEVARS_USER="$WEBR_VARS_MK" \
        "$R_HOST_EXE" CMD INSTALL \
            --library=/tmp/wasm-lib \
            --no-docs --no-test-load --no-staged-install --no-byte-compile \
            "$MONO_PKG_DIR"

        # Monorepo scaffold leg — verify wasm side-module landed.
        libdir="/tmp/wasm-lib/mxmono"
        test -d "$libdir"
        ls -la "$libdir/libs/"
        file "$libdir/libs/mxmono.so"
        file "$libdir/libs/mxmono.so" | grep -qi "WebAssembly\|wasm"
    ' \
        -e "R_HOST_EXE=${R_HOST_EXE}" \
        -e "R_NATIVE_EXE=${R_NATIVE_EXE}" \
        -e "R_HOST_RSCRIPT=${R_HOST_RSCRIPT}" \
        -e "R_NATIVE_RSCRIPT=${R_NATIVE_RSCRIPT}" \
        -e "SCAFFOLD_DIR=${SCAFFOLD_DIR}" \
        -e "SCAFFOLD_PKG_DIR=${SCAFFOLD_PKG_DIR}" \
        -e "SCAFFOLD_NATIVE_LIB=${SCAFFOLD_NATIVE_LIB}" \
        -e "SCAFFOLD_R_LIBS=${SCAFFOLD_R_LIBS}" \
        -e "MONO_DIR=${MONO_DIR}" \
        -e "MONO_ROOT_DIR=${MONO_ROOT_DIR}" \
        -e "MONO_PKG_DIR=${MONO_PKG_DIR}" \
        -e "MONO_NATIVE_LIB=${MONO_NATIVE_LIB}" \
        -e "WASM_TOOLS=${WASM_TOOLS}" \
        -e "R_SOURCE=${R_SOURCE}" \
        -e "WEBR_VARS_MK=${WEBR_VARS_MK}"

    ok "mxsmoke + mxmono scaffolds installed + wasm32 side-modules built at /tmp/wasm-lib/{mxsmoke,mxmono}"
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
    docker_step "Phase 3 (Node bundle rebuild)" "
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
    # Scaffold leg (#1270, #1271): when phase_scaffold ran, also load mxsmoke
    # (standalone template) and mxmono (monorepo template) from the same
    # /tmp/wasm-lib — comma-separated list, split by smoke.mjs.
    # SMOKE_SCAFFOLD_PKG is always passed (empty string when --scaffold
    # wasn't given) — smoke.mjs filters out empty entries, so this is
    # byte-identical to today without the flag.
    local scaffold_pkg_env=""
    if [[ "$SCAFFOLD" == "1" ]]; then
        scaffold_pkg_env="${SCAFFOLD_PKG_NAME},${MONO_PKG_NAME}"
    fi
    # Informational testthat pass (#1255): defaults ON locally — a local run
    # wants the information (SMOKE_TESTTHAT=0 disables; CI keeps per-PR
    # tier-3 load-only and enables it on main-push/dispatch). The wall-clock
    # cap grows 900 -> 2400 when enabled: ~10 min for the load smoke plus the
    # runner's own 20-min in-session testthat budget. Test failures never
    # gate; a cap hit or a harness error before the counts line does.
    local smoke_testthat="${SMOKE_TESTTHAT:-1}"
    local smoke_timeout=900
    if [[ "$smoke_testthat" == "1" ]]; then
        smoke_timeout=2400
    fi
    docker_step "Phase 3 (webR Node session)" "
        set -euo pipefail
        cd /work/tests/webr-node-smoke
        timeout ${smoke_timeout} node smoke.mjs
    " -e "SMOKE_SCAFFOLD_PKG=${scaffold_pkg_env}" -e "SMOKE_TESTTHAT=${smoke_testthat}"

    ok "webR session complete (library(miniextendr) loaded)."
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
    printf '\n%bminiextendr webR smoke test%b\n' "${CLR_BOLD}" "${CLR_RESET}"
    printf "Image:    %s\n" "${IMAGE}"
    printf "Repo:     %s\n" "${MX_ROOT}"
    printf "R:        %s\n" "${R_VERSION}"
    if [[ "$WEBR_ARM64" == "1" ]]; then
        printf 'Arch:     %barm64-native (DRAFT, #1254)%b\n' "${CLR_YELLOW}" "${CLR_RESET}"
    else
        printf "Arch:     amd64 (Rosetta on Apple Silicon)\n"
    fi
    if [[ "$SCAFFOLD" == "1" ]]; then
        printf "Scaffold: yes (%s #1270 + %s monorepo #1271)\n" "${SCAFFOLD_PKG_NAME}" "${MONO_PKG_NAME}"
    else
        printf "Scaffold: no\n"
    fi
    printf "\n"

    preflight
    phase_native_install
    phase_wasm_build
    if [[ "$SCAFFOLD" == "1" ]]; then
        phase_scaffold
    fi
    phase_webr_session

    if [[ $ROSETTA_RERUNS -gt 0 ]]; then
        printf '\n%b%bSmoke test PASSED (%s step(s) rerun after a Rosetta trap, see docs/WEBR.md).%b\n\n' \
            "${CLR_GREEN}" "${CLR_BOLD}" "${ROSETTA_RERUNS}" "${CLR_RESET}"
    else
        printf '\n%b%bSmoke test PASSED.%b\n\n' "${CLR_GREEN}" "${CLR_BOLD}" "${CLR_RESET}"
    fi
}

main
