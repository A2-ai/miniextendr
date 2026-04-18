#!/usr/bin/env bash
# Macro Compile-Time Performance Benchmark (B3)
#
# Creates synthetic crates with varying macro density and measures
# cargo check timing for cold builds, warm builds, and incremental
# rebuilds after a single-file edit.
#
# Usage:
#   bash tests/perf/macro_compile_bench.sh
#   bash tests/perf/macro_compile_bench.sh --quick   # fewer iterations
#
# Requires: cargo, hyperfine (optional, for statistical rigor)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK_DIR="$(mktemp -d)"
QUICK=false

if [[ "${1:-}" == "--quick" ]]; then
  QUICK=true
fi

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

echo "=== Macro Compile-Time Performance Benchmark ==="
echo "Repo:     $REPO_ROOT"
echo "Work dir: $WORK_DIR"
echo "Quick:    $QUICK"
echo ""

# ---------------------------------------------------------------------------
# Helper: generate a synthetic crate
# ---------------------------------------------------------------------------

generate_crate() {
  local name="$1"
  local n_files="$2"
  local fns_per_file="$3"
  local impls_per_file="$4"
  local traits_per_file="$5"

  local crate_dir="$WORK_DIR/$name"
  mkdir -p "$crate_dir/src"

  # Cargo.toml with workspace path deps
  cat > "$crate_dir/Cargo.toml" <<TOML
[package]
name = "$name"
version = "0.1.0"
edition = "2021"

[dependencies]
miniextendr-api = { path = "$REPO_ROOT/miniextendr-api" }
miniextendr-macros = { path = "$REPO_ROOT/miniextendr-macros" }

[patch.crates-io]
miniextendr-api = { path = "$REPO_ROOT/miniextendr-api" }
miniextendr-macros = { path = "$REPO_ROOT/miniextendr-macros" }
miniextendr-lint = { path = "$REPO_ROOT/miniextendr-lint" }
TOML

  # Generate sub-module files
  for i in $(seq 0 $((n_files - 1))); do
    local mod_file="$crate_dir/src/mod_${i}.rs"
    {
      echo "use miniextendr_api::{miniextendr, miniextendr_module};"
      echo ""

      # Functions
      for j in $(seq 0 $((fns_per_file - 1))); do
        echo "#[miniextendr]"
        echo "pub fn mod_${i}_fn_${j}(x: i32) -> i32 { x + $j }"
        echo ""
      done

      # Impl blocks
      for j in $(seq 0 $((impls_per_file - 1))); do
        echo "#[derive(miniextendr_api::ExternalPtr)]"
        echo "pub struct Type${i}_${j} { value: i32 }"
        echo ""
        echo "#[miniextendr]"
        echo "impl Type${i}_${j} {"
        echo "    pub fn new(v: i32) -> Self { Type${i}_${j} { value: v } }"
        echo "    pub fn get(&self) -> i32 { self.value }"
        echo "    pub fn set(&mut self, v: i32) { self.value = v; }"
        echo "}"
        echo ""
      done

      # Trait impls
      for j in $(seq 0 $((traits_per_file - 1))); do
        echo "#[miniextendr]"
        echo "pub trait Trait${i}_${j} {"
        echo "    fn trait_get(&self) -> i32;"
        echo "    fn trait_set(&mut self, v: i32);"
        echo "    fn trait_static() -> i32;"
        echo "}"
        echo ""
      done

      # Module declaration
      echo "miniextendr_module! {"
      echo "    mod mod_${i};"
      for j in $(seq 0 $((fns_per_file - 1))); do
        echo "    fn mod_${i}_fn_${j};"
      done
      for j in $(seq 0 $((impls_per_file - 1))); do
        echo "    impl Type${i}_${j};"
      done
      echo "}"
    } > "$mod_file"
  done

  # lib.rs
  {
    echo "use miniextendr_api::miniextendr_module;"
    echo ""
    for i in $(seq 0 $((n_files - 1))); do
      echo "mod mod_${i};"
    done
    echo ""
    echo "miniextendr_module! {"
    echo "    mod ${name};"
    for i in $(seq 0 $((n_files - 1))); do
      echo "    use mod_${i};"
    done
    echo "}"
  } > "$crate_dir/src/lib.rs"

  echo "$crate_dir"
}

# ---------------------------------------------------------------------------
# Helper: time a cargo check
# ---------------------------------------------------------------------------

time_cargo_check() {
  local crate_dir="$1"
  local label="$2"
  local clean_first="${3:-false}"

  if [[ "$clean_first" == "true" ]]; then
    cargo clean --manifest-path="$crate_dir/Cargo.toml" 2>/dev/null || true
  fi

  local start end elapsed
  start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  MINIEXTENDR_LINT=0 cargo check --manifest-path="$crate_dir/Cargo.toml" 2>/dev/null
  end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')

  elapsed=$(( (end - start) / 1000000 ))
  printf "  %-50s %6d ms\n" "$label" "$elapsed"
}

# ---------------------------------------------------------------------------
# Helper: time incremental rebuild (touch one file, re-check)
# ---------------------------------------------------------------------------

time_incremental() {
  local crate_dir="$1"
  local label="$2"

  # Ensure warm cache first
  MINIEXTENDR_LINT=0 cargo check --manifest-path="$crate_dir/Cargo.toml" 2>/dev/null

  # Touch a single file to trigger incremental rebuild
  touch "$crate_dir/src/mod_0.rs"

  local start end elapsed
  start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  MINIEXTENDR_LINT=0 cargo check --manifest-path="$crate_dir/Cargo.toml" 2>/dev/null
  end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')

  elapsed=$(( (end - start) / 1000000 ))
  printf "  %-50s %6d ms\n" "$label" "$elapsed"
}

# ---------------------------------------------------------------------------
# Scenario 1: Function-heavy crate
# ---------------------------------------------------------------------------

echo "--- Scenario 1: Function-heavy (20 files × 50 fns = 1000 fns) ---"
FN_CRATE=$(generate_crate "bench_fn_heavy" 20 50 0 0)

time_cargo_check "$FN_CRATE" "cold build (fn-heavy)" true
time_cargo_check "$FN_CRATE" "warm build (fn-heavy, no changes)"
time_incremental "$FN_CRATE" "incremental (fn-heavy, touch mod_0.rs)"
echo ""

# ---------------------------------------------------------------------------
# Scenario 2: Impl-heavy crate
# ---------------------------------------------------------------------------

echo "--- Scenario 2: Impl-heavy (20 files × 10 impls = 200 types) ---"
IMPL_CRATE=$(generate_crate "bench_impl_heavy" 20 5 10 0)

time_cargo_check "$IMPL_CRATE" "cold build (impl-heavy)" true
time_cargo_check "$IMPL_CRATE" "warm build (impl-heavy, no changes)"
time_incremental "$IMPL_CRATE" "incremental (impl-heavy, touch mod_0.rs)"
echo ""

# ---------------------------------------------------------------------------
# Scenario 3: Trait-impl-heavy crate
# ---------------------------------------------------------------------------

echo "--- Scenario 3: Trait-heavy (20 files × 5 traits = 100 traits) ---"
TRAIT_CRATE=$(generate_crate "bench_trait_heavy" 20 5 0 5)

time_cargo_check "$TRAIT_CRATE" "cold build (trait-heavy)" true
time_cargo_check "$TRAIT_CRATE" "warm build (trait-heavy, no changes)"
time_incremental "$TRAIT_CRATE" "incremental (trait-heavy, touch mod_0.rs)"
echo ""

# ---------------------------------------------------------------------------
# Scenario 4: Mixed crate (everything)
# ---------------------------------------------------------------------------

echo "--- Scenario 4: Mixed (20 files × 20 fns + 5 impls + 2 traits) ---"
MIXED_CRATE=$(generate_crate "bench_mixed" 20 20 5 2)

time_cargo_check "$MIXED_CRATE" "cold build (mixed)" true
time_cargo_check "$MIXED_CRATE" "warm build (mixed, no changes)"
time_incremental "$MIXED_CRATE" "incremental (mixed, touch mod_0.rs)"
echo ""

# ---------------------------------------------------------------------------
# Scenario 5: Scaling test (same density, more files)
# ---------------------------------------------------------------------------

if [[ "$QUICK" != "true" ]]; then
  echo "--- Scenario 5: Scaling (5 fns/file, increasing file count) ---"

  for n_files in 5 20 50 100; do
    SCALE_CRATE=$(generate_crate "bench_scale_${n_files}" "$n_files" 5 1 0)
    time_cargo_check "$SCALE_CRATE" "cold build (${n_files} files × 5 fns)" true
  done
  echo ""

  echo "--- Scenario 6: Density test (20 files, increasing fns/file) ---"

  for fns in 5 20 50 100; do
    DENSE_CRATE=$(generate_crate "bench_dense_${fns}" 20 "$fns" 0 0)
    time_cargo_check "$DENSE_CRATE" "cold build (20 files × ${fns} fns)" true
  done
  echo ""
fi

echo "=== Done ==="
echo "Work dir cleaned up: $WORK_DIR"
