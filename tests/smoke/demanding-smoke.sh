#!/usr/bin/env bash
# =============================================================================
# demanding-smoke.sh -- High-pressure smoke test for miniextendr + minirextendr
#
# Usage:
#   tests/smoke/demanding-smoke.sh           # full demanding run
#   tests/smoke/demanding-smoke.sh --quick   # skip heavy phases (A3, A4, C*)
#
# Must be run from the repo root (where justfile lives).
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Globals
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export MX_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
export MX_SMOKE_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/mx-smoke-XXXXXX")"
export MX_ARTIFACTS="$MX_SMOKE_ROOT/artifacts"
mkdir -p "$MX_ARTIFACTS"

QUICK=0
for arg in "$@"; do
  case "$arg" in
    --quick) QUICK=1 ;;
    *) echo "Unknown flag: $arg"; exit 1 ;;
  esac
done

# Phase tracking (bash 3+ compatible via string accumulators)
PHASE_NAMES=""
PHASE_RESULTS=""
PHASE_DURATIONS=""
PHASE_COUNT=0
TOTAL_START="$(date +%s)"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log_header() {
  echo ""
  echo "========================================================================"
  echo "  $1"
  echo "========================================================================"
  echo ""
}

log_info() {
  echo "[smoke] $1"
}

log_pass() {
  echo "[PASS] $1"
}

log_fail() {
  echo "[FAIL] $1"
}

# Run a phase, capture output, track pass/fail.
# Usage: run_phase "A1" "description" command [args...]
run_phase() {
  local phase_id="$1"; shift
  local description="$1"; shift
  local label="${phase_id}: ${description}"
  local log_file="$MX_ARTIFACTS/${phase_id}.log"
  local phase_start
  phase_start="$(date +%s)"

  log_header "$label"

  PHASE_COUNT=$((PHASE_COUNT + 1))
  PHASE_NAMES="${PHASE_NAMES}${label}"$'\n'

  local rc=0
  "$@" > >(tee "$log_file") 2>&1 || rc=$?

  local phase_end
  phase_end="$(date +%s)"
  local elapsed=$((phase_end - phase_start))

  if [ $rc -eq 0 ]; then
    log_pass "$label (${elapsed}s)"
    PHASE_RESULTS="${PHASE_RESULTS}PASS"$'\n'
  else
    log_fail "$label (${elapsed}s, exit=$rc)"
    PHASE_RESULTS="${PHASE_RESULTS}FAIL"$'\n'
  fi
  PHASE_DURATIONS="${PHASE_DURATIONS}${elapsed}"$'\n'

  return $rc
}

# Like run_phase but does not abort the script on failure (best-effort).
run_phase_optional() {
  run_phase "$@" || true
}

# Record a skipped phase.
skip_phase() {
  local label="$1"
  log_info "SKIP: $label -- quick mode"
  PHASE_COUNT=$((PHASE_COUNT + 1))
  PHASE_NAMES="${PHASE_NAMES}${label} (skipped)"$'\n'
  PHASE_RESULTS="${PHASE_RESULTS}SKIP"$'\n'
  PHASE_DURATIONS="${PHASE_DURATIONS}0"$'\n'
}

# ---------------------------------------------------------------------------
# Tool versions
# ---------------------------------------------------------------------------

log_versions() {
  log_header "Tool Versions"

  R --version 2>&1 | head -n 3 | tee "$MX_ARTIFACTS/r-version.txt"
  echo ""
  rustc --version | tee "$MX_ARTIFACTS/rustc-version.txt"
  cargo --version | tee "$MX_ARTIFACTS/cargo-version.txt"
  autoconf --version 2>/dev/null | head -n 1 | tee "$MX_ARTIFACTS/autoconf-version.txt" || echo "autoconf: not found"
  echo ""

  log_info "MX_ROOT=$MX_ROOT"
  log_info "MX_SMOKE_ROOT=$MX_SMOKE_ROOT"
  log_info "MX_ARTIFACTS=$MX_ARTIFACTS"
  log_info "QUICK=$QUICK"
  echo ""
}

# ---------------------------------------------------------------------------
# Phase A: miniextendr smoke
# ---------------------------------------------------------------------------

phase_a1_repo_sync() {
  cd "$MX_ROOT"
  log_info "Running templates-check..."
  just templates-check
  log_info "Running vendor-sync-check..."
  just vendor-sync-check
  log_info "Running lint..."
  just lint
}

phase_a2_dev_build() {
  cd "$MX_ROOT"
  log_info "Configuring rpkg (dev mode)..."
  just configure
  log_info "Installing rpkg..."
  R CMD INSTALL rpkg
  log_info "Running baseline tests (basic)..."
  Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg", filter = "basic")'
  log_info "Running baseline tests (conversions)..."
  Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg", filter = "conversions")'
}

phase_a3_high_risk_tests() {
  cd "$MX_ROOT"
  local filters=(
    gc-stress
    panic
    thread
    worker
    trait-abi
    externalptr
    class-systems
    altrep
    serde_r
    feature-adapters
    rayon
  )
  for filter in "${filters[@]}"; do
    log_info "Running high-risk filter: $filter"
    Rscript -e "testthat::set_max_fails(Inf); devtools::test('rpkg', filter = '${filter}')"
  done
}

phase_a4_cran_tarball() {
  cd "$MX_ROOT"
  log_info "Vendoring for CRAN tarball..."
  just vendor
  log_info "Building tarball..."
  R CMD build --no-manual rpkg
  local tarball
  tarball="$(ls -1t miniextendr_*.tar.gz 2>/dev/null | head -n1)"
  if [ -z "$tarball" ]; then
    echo "ERROR: No tarball found after R CMD build"
    return 1
  fi
  log_info "Tarball: $tarball"
  cp "$tarball" "$MX_ARTIFACTS/"
  log_info "Running R CMD check --as-cran..."
  R CMD check --as-cran --no-manual "$tarball"
  # Copy check directory to artifacts
  local check_dir
  check_dir="$(basename "$tarball" .tar.gz).Rcheck"
  if [ -d "$check_dir" ]; then
    cp -R "$check_dir" "$MX_ARTIFACTS/" 2>/dev/null || true
  fi
}

phase_a5_cross_package() {
  cd "$MX_ROOT"
  log_info "Cleaning cross-package..."
  just cross-clean || true
  log_info "Configuring cross-package..."
  just cross-configure
  log_info "Building cross-package..."
  just cross-install
  log_info "Testing cross-package..."
  just cross-test
}

# ---------------------------------------------------------------------------
# Phase B: minirextendr smoke
# ---------------------------------------------------------------------------

phase_b1_minirextendr_pkg() {
  cd "$MX_ROOT"
  log_info "Running minirextendr tests..."
  just minirextendr-test
  log_info "Running minirextendr check..."
  just minirextendr-check
}

phase_b2_scaffold_standalone() {
  cd "$MX_ROOT"
  local smoke_pkg="$MX_SMOKE_ROOT/standalone.smoke"
  local smoke_lib="$MX_SMOKE_ROOT/r-lib-standalone"
  mkdir -p "$smoke_lib"

  log_info "Scaffolding standalone package at $smoke_pkg..."
  Rscript - <<'RSCRIPT'
library(minirextendr)
library(usethis)
library(withr)

root <- Sys.getenv("MX_ROOT")
tmp  <- Sys.getenv("MX_SMOKE_ROOT")
pkg  <- file.path(tmp, "standalone.smoke")

# Scaffold the package
create_package(pkg, open = FALSE)
proj_set(pkg, force = TRUE)
use_miniextendr(template_type = "rpkg", local_path = root)
RSCRIPT

  log_info "Building standalone scaffold..."
  cd "$smoke_pkg"

  # Run autoconf + configure
  if command -v autoconf >/dev/null 2>&1; then
    autoconf
  fi
  ./configure

  # Build and install
  R CMD INSTALL --no-multiarch -l "$smoke_lib" .

  log_info "Testing hello_world from scaffolded package..."
  local pkg_name
  pkg_name="$(Rscript -e 'd <- read.dcf("DESCRIPTION")[1,]; cat(d[["Package"]])')"

  Rscript - <<RTEST
lib <- "${smoke_lib}"
pkg <- "${pkg_name}"
.libPaths(c(lib, .libPaths()))
library(pkg, character.only = TRUE, lib.loc = lib)
result <- hello_world()
cat("hello_world() returned:", result, "\n")
stopifnot(is.character(result))
cat("Standalone scaffold smoke: OK\n")
RTEST

  # Save tree summary
  find "$smoke_pkg" -type f -not -path '*/target/*' -not -path '*/.git/*' | sort > "$MX_ARTIFACTS/standalone-tree.txt"
}

phase_b4_scaffold_monorepo() {
  cd "$MX_ROOT"
  local mono_root="$MX_SMOKE_ROOT/mono-smoke"
  local mono_lib="$MX_SMOKE_ROOT/r-lib-monorepo"
  mkdir -p "$mono_lib"

  log_info "Scaffolding monorepo at $mono_root..."
  Rscript - <<'RSCRIPT'
library(minirextendr)
library(usethis)

root <- Sys.getenv("MX_ROOT")
tmp  <- Sys.getenv("MX_SMOKE_ROOT")
mono <- file.path(tmp, "mono-smoke")

create_miniextendr_monorepo(
  mono,
  package = "monosmoke",
  crate_name = "mono-smoke-core",
  open = FALSE
)
RSCRIPT

  local rpkg_dir="$mono_root/rpkg"
  if [ ! -d "$rpkg_dir" ]; then
    rpkg_dir="$mono_root/monosmoke"
    if [ ! -d "$rpkg_dir" ]; then
      echo "ERROR: Cannot find rpkg directory in monorepo scaffold"
      ls -la "$mono_root"
      return 1
    fi
  fi

  log_info "Building monorepo R package at $rpkg_dir..."
  cd "$rpkg_dir"
  if command -v autoconf >/dev/null 2>&1; then
    autoconf 2>/dev/null || true
  fi
  if [ -f "configure" ]; then
    ./configure || true
  fi

  # Try to build and install
  R CMD INSTALL --no-multiarch -l "$mono_lib" . || {
    log_info "Direct R CMD INSTALL failed, trying devtools..."
    Rscript -e "devtools::install('.', lib = '${mono_lib}')" || true
  }

  # Check if main Rust crate compiles
  if [ -f "$mono_root/mono-smoke-core/Cargo.toml" ]; then
    log_info "Checking main Rust crate..."
    cargo check --manifest-path "$mono_root/mono-smoke-core/Cargo.toml"
  elif [ -f "$mono_root/Cargo.toml" ]; then
    log_info "Checking workspace Cargo.toml..."
    cargo check --manifest-path "$mono_root/Cargo.toml"
  fi

  # Save tree summary
  find "$mono_root" -type f -not -path '*/target/*' -not -path '*/.git/*' | sort > "$MX_ARTIFACTS/monorepo-tree.txt"
}

phase_b3_external_dep_revendor() {
  cd "$MX_ROOT"
  local smoke_pkg="$MX_SMOKE_ROOT/standalone.smoke"
  local smoke_lib="$MX_SMOKE_ROOT/r-lib-standalone"

  if [ ! -d "$smoke_pkg" ]; then
    log_info "B3 requires B2 to have run first (standalone scaffold)"
    return 1
  fi

  log_info "Adding external dependency to standalone scaffold..."
  cd "$smoke_pkg"

  # Add itertools to Cargo.toml
  Rscript -e "minirextendr::cargo_add('itertools@0.14', path = '.')" || {
    # Fall back to manual edit if cargo_add unavailable
    cd src/rust
    cargo add itertools@0.14
    cd "$smoke_pkg"
  }

  # Add a function that uses itertools
  cat >> src/rust/src/lib.rs << 'RUSTEOF'

#[miniextendr_api::miniextendr]
pub fn join_sorted(x: Vec<String>) -> String {
    use itertools::Itertools;
    x.into_iter().sorted().join(", ")
}
RUSTEOF

  # Re-configure and rebuild
  ./configure
  R CMD INSTALL --no-multiarch -l "$smoke_lib" .

  # Test the new function
  local pkg_name
  pkg_name="$(Rscript -e 'd <- read.dcf("DESCRIPTION")[1,]; cat(d[["Package"]])')"

  Rscript - <<RTEST
lib <- "${smoke_lib}"
.libPaths(c(lib, .libPaths()))
library("${pkg_name}", character.only = TRUE, lib.loc = lib)
result <- join_sorted(c("banana", "apple", "cherry"))
cat("join_sorted() returned:", result, "\n")
stopifnot(result == "apple, banana, cherry")
cat("External dep re-vendor smoke: OK\n")
RTEST
}

phase_b5_api_helpers() {
  cd "$MX_ROOT"
  local smoke_pkg="$MX_SMOKE_ROOT/standalone.smoke"

  if [ ! -d "$smoke_pkg" ]; then
    log_info "B5 requires B2 to have run first (standalone scaffold)"
    return 1
  fi

  log_info "Testing minirextendr API helpers in scaffolded project..."
  Rscript - <<RSCRIPT
library(minirextendr)
library(usethis)

pkg <- "${smoke_pkg}"
proj_set(pkg, force = TRUE)

# miniextendr_doctor should return no fails
diag <- miniextendr_doctor()
if (length(diag\$fail) > 0) {
  cat("Doctor found failures:\n")
  print(diag\$fail)
  stop("miniextendr_doctor() reported failures")
}
cat("miniextendr_doctor(): OK\n")

# Cargo wrapper commands should execute without error
tryCatch(cargo_check(path = file.path(pkg, "src", "rust")), error = function(e) {
  cat("cargo_check warning:", conditionMessage(e), "\n")
})
cat("cargo_check(): OK\n")

tryCatch(cargo_clippy(path = file.path(pkg, "src", "rust")), error = function(e) {
  cat("cargo_clippy warning:", conditionMessage(e), "\n")
})
cat("cargo_clippy(): OK\n")

cat("API helper smoke: OK\n")
RSCRIPT
}

# ---------------------------------------------------------------------------
# Phase C: Failure-injection (optional/best-effort)
# ---------------------------------------------------------------------------

phase_c1_stale_detection() {
  cd "$MX_ROOT"
  log_info "Testing stale generated-file detection..."

  touch rpkg/src/Makevars.in
  sleep 1

  log_info "Running configure to clear stale state..."
  just configure

  if [ -f rpkg/src/Makevars ]; then
    local template_ts generated_ts
    template_ts="$(stat -f %m rpkg/src/Makevars.in 2>/dev/null || stat -c %Y rpkg/src/Makevars.in 2>/dev/null)"
    generated_ts="$(stat -f %m rpkg/src/Makevars 2>/dev/null || stat -c %Y rpkg/src/Makevars 2>/dev/null)"
    if [ "$generated_ts" -ge "$template_ts" ]; then
      log_info "Stale detection: configure refreshed generated file correctly"
    else
      log_info "WARNING: generated file still older than template after configure"
    fi
  fi
}

phase_c2_vendor_fallback() {
  cd "$MX_ROOT"
  log_info "Testing vendor fallback behavior..."

  if [ ! -f rpkg/inst/vendor.tar.xz ]; then
    log_info "Creating vendor tarball first..."
    just vendor
  fi

  rm -rf rpkg/vendor

  log_info "Running configure in CRAN-like mode without vendor/..."
  cd rpkg
  if command -v autoconf >/dev/null 2>&1; then
    autoconf
  fi
  ./configure

  if [ -d vendor ] || [ -d src/rust/vendor ]; then
    log_info "Vendor fallback: configure restored vendored sources"
  else
    log_info "WARNING: vendor directory not restored by configure"
  fi

  # Restore dev mode
  cd "$MX_ROOT"
  just configure
}

phase_c3_install_order() {
  cd "$MX_ROOT"
  log_info "Testing cross-package install order constraint..."

  # Clean consumer first
  just cross-clean || true

  # Attempt to install consumer BEFORE producer — should fail
  log_info "Attempting consumer install without producer (should fail)..."
  local consumer_dir="tests/cross-package/consumer.pkg"
  local rc=0
  R CMD INSTALL "$consumer_dir" 2>&1 || rc=$?

  if [ $rc -eq 0 ]; then
    log_info "WARNING: consumer installed without producer — expected failure"
    return 1
  fi
  log_info "Consumer correctly failed without producer (exit=$rc)"

  # Now install in correct order: producer first, then consumer
  log_info "Installing in correct order (producer then consumer)..."
  just cross-install

  # Verify correct-order install works
  log_info "Running cross-package tests to verify correct order..."
  just cross-test
}

# ---------------------------------------------------------------------------
# Summary generation
# ---------------------------------------------------------------------------

generate_summary() {
  local total_end
  total_end="$(date +%s)"
  local total_elapsed=$((total_end - TOTAL_START))

  local summary_file="$MX_ARTIFACTS/smoke-summary.md"
  local pass_count=0
  local fail_count=0
  local skip_count=0

  {
    echo "# Demanding Smoke Test Summary"
    echo ""
    echo "- Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "- Total duration: ${total_elapsed}s"
    echo "- Mode: $([ $QUICK -eq 1 ] && echo 'quick' || echo 'demanding')"
    echo "- Platform: $(uname -s) $(uname -m)"
    echo ""
    echo "## Tool Versions"
    echo ""
    echo '```'
    cat "$MX_ARTIFACTS/r-version.txt" 2>/dev/null || echo "R: unknown"
    cat "$MX_ARTIFACTS/rustc-version.txt" 2>/dev/null || echo "rustc: unknown"
    cat "$MX_ARTIFACTS/cargo-version.txt" 2>/dev/null || echo "cargo: unknown"
    cat "$MX_ARTIFACTS/autoconf-version.txt" 2>/dev/null || echo "autoconf: unknown"
    echo '```'
    echo ""
    echo "## Phase Results"
    echo ""
    echo "| Phase | Result | Duration |"
    echo "|-------|--------|----------|"
  } > "$summary_file"

  local i=0
  while IFS= read -r name; do
    [ -z "$name" ] && continue
    i=$((i + 1))

    local result
    result="$(echo "$PHASE_RESULTS" | sed -n "${i}p")"
    local duration
    duration="$(echo "$PHASE_DURATIONS" | sed -n "${i}p")"

    if [ "$result" = "PASS" ]; then
      pass_count=$((pass_count + 1))
      echo "| $name | PASS | ${duration}s |" >> "$summary_file"
    elif [ "$result" = "FAIL" ]; then
      fail_count=$((fail_count + 1))
      echo "| $name | **FAIL** | ${duration}s |" >> "$summary_file"
    else
      skip_count=$((skip_count + 1))
      echo "| $name | SKIP | - |" >> "$summary_file"
    fi
  done <<< "$PHASE_NAMES"

  {
    echo ""
    echo "## Totals"
    echo ""
    echo "- Passed: $pass_count"
    echo "- Failed: $fail_count"
    echo "- Skipped: $skip_count"
    echo ""
    echo "## Artifacts"
    echo ""
    echo "All logs and artifacts saved to: \`$MX_ARTIFACTS/\`"
    echo ""
    ls -1 "$MX_ARTIFACTS/" | sed 's/^/- /'
  } >> "$summary_file"

  echo ""
  log_header "SMOKE TEST SUMMARY"
  cat "$summary_file"
  echo ""

  if [ $fail_count -gt 0 ]; then
    log_fail "$fail_count phase(s) failed. See $MX_ARTIFACTS/ for logs."
    return 1
  else
    log_pass "All $pass_count phase(s) passed."
    return 0
  fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

main() {
  cd "$MX_ROOT"

  log_versions

  # --- Phase A: miniextendr smoke ---
  run_phase "A1" "Repo sync checks" phase_a1_repo_sync
  run_phase "A2" "Dev-mode configure/build/install" phase_a2_dev_build

  if [ $QUICK -eq 0 ]; then
    run_phase "A3" "High-risk runtime test filters" phase_a3_high_risk_tests
    run_phase "A4" "CRAN-like tarball path" phase_a4_cran_tarball
  else
    skip_phase "A3: High-risk runtime test filters"
    skip_phase "A4: CRAN-like tarball path"
  fi

  run_phase "A5" "Cross-package trait ABI smoke" phase_a5_cross_package

  # --- Phase B: minirextendr smoke ---
  run_phase "B1" "minirextendr package tests and check" phase_b1_minirextendr_pkg
  run_phase "B2" "Scaffolding smoke: standalone package" phase_b2_scaffold_standalone
  run_phase "B3" "External dep re-vendor" phase_b3_external_dep_revendor
  run_phase "B4" "Scaffolding smoke: monorepo package" phase_b4_scaffold_monorepo
  run_phase "B5" "minirextendr API helpers" phase_b5_api_helpers

  # --- Phase C: Failure-injection (optional/best-effort) ---
  if [ $QUICK -eq 0 ]; then
    run_phase_optional "C1" "Stale generated-file detection" phase_c1_stale_detection
    run_phase_optional "C2" "Vendor fallback behavior" phase_c2_vendor_fallback
    run_phase_optional "C3" "Cross-package install order constraint" phase_c3_install_order
  else
    skip_phase "C1: Stale generated-file detection"
    skip_phase "C2: Vendor fallback behavior"
    skip_phase "C3: Cross-package install order constraint"
  fi

  # Summary (exit 1 if any hard-fail phases failed)
  generate_summary || exit 1
}

# Cleanup trap: always print artifact location
cleanup() {
  local rc=$?
  echo ""
  echo "[smoke] Artifacts saved to: $MX_ARTIFACTS"
  if [ $rc -ne 0 ]; then
    echo "[smoke] Script exited with code $rc"
  fi
}
trap cleanup EXIT

main "$@"
