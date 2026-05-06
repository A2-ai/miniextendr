# https://just.systems
#
# Quick reference:
#   Setup (Windows-only):
#     just install-deps-windows - Install Windows system tooling via Scoop (rv, etc.)
#
#   Rust:
#     just check              - Run cargo check
#     just check-features     - Check feature combinations compile
#     just test               - Run cargo tests
#     just clippy             - Run lints
#     just fmt                - Format Rust code
#     just lint               - Run miniextendr-lint on rpkg
#
#   rpkg (example R package):
#     just configure          - Configure R package build (dev mode, no vendoring)
#     just vendor             - Vendor deps for CRAN release prep
#     just devtools-test      - Run R package tests
#     just devtools-document  - Run roxygen2 (NAMESPACE + man pages)
#     just rcmdinstall        - Build and install R package
#
#   minirextendr (helper R package):
#     just minirextendr-document  - Generate documentation
#     just minirextendr-test      - Run tests
#     just minirextendr-check     - Run R CMD check
#     just minirextendr-install   - Install package
#
#   Cross-package trait ABI tests:
#     just cross-document     - Regenerate docs for both packages
#     just cross-install      - Build + install both packages
#     just cross-test         - Run tests for both packages
#     just cross-check        - Run checks for both packages
#     just cross-clean        - Clean both packages
#
#   Templates:
#     just templates-check    - Verify templates haven't drifted
#     just templates-approve  - Accept template changes
#
#   Benchmarks:
#     just bench              - Run all Rust benchmarks
#     just bench-core         - Core benchmarks (high-signal, default features)
#     just bench-features     - Feature-gated benchmarks (connections, rayon, etc.)
#     just bench-full         - Full suite (core + feature matrix)
#     just bench-r            - Run R-side benchmarks (requires rpkg installed)
#     just bench-save         - Save structured baseline (text + CSV + metadata)
#     just bench-compare      - Show top benchmarks from a baseline
#     just bench-drift        - Check for regressions between last 2 baselines
#     just bench-info         - List saved baselines with metadata
#     just bench-compile      - Macro compile-time perf (synthetic crates)
#     just bench-lint         - Lint scan performance
#     just bench-check        - Check benchmark crate compiles
#
#   Documentation site:
#     just site-docs           - Regenerate site/content/manual/ + site/data/plans.json
#     just site-build          - Build Zola site (run site-docs first for doc changes)
#     just site-serve          - Local preview server (run site-docs first for doc changes)
#     just bump-version <v>    - Bump version across all Cargo.toml + DESCRIPTION files
#
#   Vendor sync:
#     just vendor-sync-check  - Verify vendored crates match workspace
#     just vendor-sync-diff   - Show diff between workspace and vendor
#     just lock-shape-check   - Verify Cargo.lock is in tarball-shape (git sources, no checksums)
#     just clean-vendor-leak  - Remove a leaked inst/vendor.tar.xz that would flip configure into tarball mode
#
set shell := ["bash", "-euo", "pipefail", "-c"]
set windows-shell := ["bash", "-euo", "pipefail", "-c"]

# On Windows (Git Bash / MSYS2), cargo and R may not be in /bin/bash's PATH.
# Export CARGO_HOME/bin so recipes can find cargo/rustc.
export PATH := if os() == "windows" {
    env("CARGO_HOME", env("USERPROFILE", "") / ".cargo") / "bin" + ":" + env("PATH", "")
} else {
    env("PATH", "")
}

# All optional features for testing (excluding nonapi which causes CRAN warnings).
# This mirrors the default CARGO_FEATURES list in rpkg/configure.ac.
all_features := "worker-thread,rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,tinyvec,raw_conversions,vctrs,borsh,log"

# Directory for devtools::check output (preserved for investigation)
check_output_dir := justfile_directory() / "rpkg-check-output"

[default]
default:
    @just --list

clean:
    just configure
    just cargo-clean
    cd rpkg && ./cleanup
    cd tests/cross-package && just clean

# Clean build artifacts
#
# NOTE: The `tmp="$(mktemp -d)" && (cd "$tmp" && cargo ...)` pattern is used
# throughout this file to run cargo from a neutral directory, preventing it
# from picking up the wrong .cargo/config.toml. These temp dirs are empty
# (just used as cwd) and cleaned by the OS periodically - not a significant leak.
cargo-clean *cargo_flags:
    cargo clean -p miniextendr-api {{cargo_flags}}
    cargo clean -p miniextendr-macros {{cargo_flags}}
    cargo clean -p miniextendr-bench {{cargo_flags}}
    cargo clean -p miniextendr-lint {{cargo_flags}}
    cargo clean -p miniextendr-engine {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo clean --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo clean --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clean --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Check feature combinations compile (F2: feature interaction testing)
# Tests important feature combos that might interact but are only tested independently.
[script("bash")]
check-features:
    set -euo pipefail
    manifest="miniextendr-api/Cargo.toml"
    combos=(
        # serde interacts with many types
        "serde,ndarray"
        "serde,nalgebra"
        "serde,indexmap"
        "serde_json,ndarray"
        # rayon + type features
        "rayon,vctrs"
        "rayon,ndarray"
        "rayon,serde"
        # connections + strict mode
        "connections,default-strict"
        # numeric ecosystem
        "num-bigint,num-complex,num-traits,ordered-float,rust_decimal"
        # string/pattern features together
        "regex,aho-corasick,url,uuid"
        # serialization combos
        "serde,serde_json,borsh,toml"
        # collection combos
        "indexmap,tinyvec,bitvec,bitflags"
        # defaults combos
        "default-strict,default-coerce,default-r6"
        "default-strict,default-s7,default-worker"
        # diagnostic features
        "growth-debug,materialization-tracking"
        # all features together
        "rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs,tinyvec,borsh,connections,nonapi"
    )
    total=${#combos[@]}
    passed=0
    for combo in "${combos[@]}"; do
        echo "--- Checking: $combo ---"
        if cargo check --manifest-path="$manifest" --features "$combo" 2>&1; then
            ((passed++))
        else
            echo "FAILED: $combo"
            exit 1
        fi
    done
    echo ""
    echo "=== All $passed/$total feature combinations passed ==="

# Update Cargo.lock files across every tracked manifest.
# rpkg's lock must stay in tarball-shape (no `path+...` sources for
# miniextendr-{api,lint,macros}). We move .cargo/config.toml aside so the
# [patch."git+url"] override doesn't bleed into the lock.
# Checksums are NO LONGER stripped: cargo-revendor now writes valid
# `.cargo-checksum.json` files that match the registry checksums, so the
# committed lock can retain `checksum = "..."` lines.
alias cargo-update := update
[script("bash")]
update *cargo_flags:
    set -euo pipefail
    cargo update {{cargo_flags}}
    cargo update --manifest-path=cargo-revendor/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/shared-traits/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/consumer.pkg/src/rust/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/producer.pkg/src/rust/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/model_project/src/rust/Cargo.toml {{cargo_flags}}
    rust_dir="{{justfile_directory()}}/rpkg/src/rust"
    cargo_cfg="$rust_dir/.cargo/config.toml"
    if [[ -f "$cargo_cfg" ]]; then mv "$cargo_cfg" "$cargo_cfg.tmp_just_update"; fi
    trap "[[ -f '$cargo_cfg.tmp_just_update' ]] && mv '$cargo_cfg.tmp_just_update' '$cargo_cfg'" EXIT
    cargo update --manifest-path "$rust_dir/Cargo.toml" {{cargo_flags}}
    just lock-shape-check

# Check all crates
alias cargo-check := check
check *cargo_flags:
    cargo check --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo check --benches --tests --examples --workspace --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Build all crates
alias cargo-build := build
build *cargo_flags:
    cargo build --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo build --benches --tests --examples --workspace --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run clippy on all crates
alias cargo-clippy := clippy
clippy *cargo_flags:
    cargo clippy --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clippy --benches --tests --examples --workspace --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run miniextendr-lint on rpkg (checks #[miniextendr] consistency)
# The lint runs as a build script; this command triggers it via cargo check.
# Lint output appears as cargo warnings. Errors indicate:
# - Multiple unlabeled impl blocks for the same type
# - Class system incompatibilities between inherent and trait impls
[script("bash")]
lint:
    set -euo pipefail
    cd rpkg
    output=$(cargo check --manifest-path=src/rust/Cargo.toml 2>&1) || {
        echo "$output"
        echo ""
        echo "::error::cargo check failed (see output above)"
        exit 1
    }
    lint_issues=$(echo "$output" | grep -E "warning.*miniextendr.*\[MXL|warning.*miniextendr-lint" || true)
    if [[ -n "$lint_issues" ]]; then
        echo "$lint_issues"
        echo ""
        echo "miniextendr-lint found issues (see above)"
    else
        echo "miniextendr-lint: no issues found"
    fi

# Check documentation builds
alias cargo-doc-check := doc-check
doc-check *cargo_flags: configure-all
    cargo doc --no-deps --document-private-items --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo doc --no-deps --document-private-items --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Build and open documentation
alias cargo-doc := doc
doc *cargo_flags: configure-all
    cargo doc --document-private-items --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo doc --document-private-items --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})
    if command -v open >/dev/null 2>&1; then \
      open rpkg/src/rust/target/doc/rpkg/index.html >/dev/null 2>&1 || \
        echo "doc: unable to open generated docs (skipping)"; \
    else \
      echo "doc: open not found; docs at rpkg/src/rust/target/doc/rpkg/index.html"; \
    fi

# Check formatting
alias cargo-fmt-check := fmt-check
fmt-check *cargo_flags:
    cargo fmt --all {{cargo_flags}} -- --check
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/rpkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)

# Format all code
alias cargo-fmt := fmt
fmt *cargo_flags:
    cargo fmt --all {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/rpkg/src/rust/Cargo.toml" {{cargo_flags}})

# Run tests
alias cargo-test := test
test *args:
    cargo_flags="" \
    && test_args="" \
    && sep=0 \
    && for arg in {{args}}; do \
      if [ "$arg" = "--" ]; then sep=1; continue; fi; \
      if [ "$sep" = "0" ]; then cargo_flags="$cargo_flags $arg"; else test_args="$test_args $arg"; fi; \
    done \
    && cargo test --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args \
    && root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo test --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args) \
    && root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo test --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args) \
    && root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo test --workspace --no-fail-fast $cargo_flags --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" -- --no-capture $test_args)

# Run benchmarks (miniextendr-bench)
alias cargo-bench := bench
bench *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}

# Save structured benchmark baseline (raw text + CSV + metadata)
bench-save *cargo_flags:
    bash tests/perf/bench_baseline.sh save -- {{cargo_flags}}

# Show top benchmarks from most recent baseline
bench-compare *csv_file:
    bash tests/perf/bench_baseline.sh compare {{csv_file}}

# Check for performance regressions between last 2 baselines
bench-drift *flags:
    bash tests/perf/bench_baseline.sh drift {{flags}}

# List saved benchmark baselines with metadata
bench-info:
    bash tests/perf/bench_baseline.sh info

# Check benchmark crate
bench-check *cargo_flags:
    cargo check --manifest-path=miniextendr-bench/Cargo.toml --benches --tests --examples {{cargo_flags}}

# Run macro compile-time benchmark (synthetic crates, measures cold/warm/incremental)
bench-compile *flags:
    bash tests/perf/macro_compile_bench.sh {{flags}}

# Run lint scan benchmark
bench-lint *cargo_flags:
    cargo bench --manifest-path=miniextendr-lint/Cargo.toml --bench lint_scan {{cargo_flags}}

# Run core benchmarks (default features, high-signal targets)
bench-core *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --bench ffi_calls --bench into_r --bench from_r --bench translate --bench strings --bench externalptr --bench worker --bench unwind_protect {{cargo_flags}}

# Run feature-gated benchmarks (connections, rayon, refcount-fast-hash)
bench-features *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections,rayon,refcount-fast-hash {{cargo_flags}}

# Run full benchmark suite (core + feature matrix)
bench-full *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections,rayon,refcount-fast-hash --bench connections --bench rayon --bench refcount_protect {{cargo_flags}}

# Run R-side benchmarks (requires rpkg installed)
[script("bash")]
bench-r:
    set -euo pipefail
    for f in rpkg/tests/testthat/bench-*.R; do
      echo "=== Running $f ==="
      Rscript "$f"
      echo ""
    done

# Show dependency tree
alias cargo-tree := tree
tree *cargo_flags:
    cargo tree --workspace {{cargo_flags}}
    cargo tree --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Expand macros for rpkg (requires cargo-expand)
alias cargo-expand := expand
expand *cargo_flags:
    cargo expand --lib -p miniextendr-api {{cargo_flags}}
    cargo expand --lib -p miniextendr-macros {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run ./configure
#
# Generates build configuration files (Makevars, cargo config, etc.) using
# the unified install-mode detection in configure.ac:
#   - source mode (default): cargo resolves miniextendr deps from monorepo
#     siblings via [patch] in .cargo/config.toml. No vendor/.
#   - tarball mode: kicks in automatically when inst/vendor.tar.xz exists
#     (i.e. inside R CMD INSTALL <built-tarball>). Configure unpacks the
#     tarball and writes a vendored source-replacement config.
#
# See docs/CRAN_COMPATIBILITY.md for the full table.
configure:
    cd rpkg && \
    if command -v autoconf >/dev/null 2>&1; then autoconf; else echo "autoconf not found; using existing configure"; fi && \
    bash ./configure

# Vendor dependencies into inst/vendor.tar.xz for CRAN release preparation.
# Requires cargo-revendor: `just revendor-install`.
#
# Only needed when producing a build-tarball intended for offline install
# (i.e. before `R CMD build .` for a CRAN submission). Day-to-day dev
# (R CMD INSTALL ., devtools::install/test/load) never calls this recipe.
#
# Steps:
#   1. Regenerate Cargo.lock against the bare git URL (no [patch] override),
#      so the lockfile entries for miniextendr-{api,lint,macros} carry the
#      `git+https://...#<commit>` source — required by cargo's source
#      replacement mechanism when the tarball is later installed offline.
#   2. Run cargo-revendor against the freshly-regenerated lockfile.
#      cargo-revendor recomputes `.cargo-checksum.json` with real SHA-256s
#      after CRAN-trim, so the committed Cargo.lock can retain its registry
#      `checksum = "..."` lines (no post-vendor sed stripping needed).
#   3. Compress vendor/ to inst/vendor.tar.xz.
#
# --force re-runs the full vendor even when cargo-revendor's cache thinks the
# output is current. Cheap insurance against a stale committed tarball when
# the workspace crates have edits that don't bump Cargo.lock.
[script("bash")]
vendor:
    set -euo pipefail
    rust_dir="{{justfile_directory()}}/rpkg/src/rust"
    cargo_cfg="$rust_dir/.cargo/config.toml"
    # Regenerate lockfile in tarball-shape: temporarily move .cargo aside so
    # cargo doesn't see the [patch] override and resolves git deps as-is.
    # Entries for miniextendr-{api,lint,macros} get source = "git+url#<commit>",
    # which is what cargo's source-replacement mechanism needs at offline
    # install time.
    if [[ -f "$cargo_cfg" ]]; then
      mv "$cargo_cfg" "$cargo_cfg.tmp_just_vendor"
    fi
    rm -f "$rust_dir/Cargo.lock"
    cargo generate-lockfile --manifest-path "$rust_dir/Cargo.toml"
    if [[ -f "$cargo_cfg.tmp_just_vendor" ]]; then
      mv "$cargo_cfg.tmp_just_vendor" "$cargo_cfg"
    fi
    # cargo-revendor auto-reads [patch."git+url"] from .cargo/config.toml
    # (written by `configure` in dev-monorepo mode) to copy miniextendr-{api,
    # lint,macros} from this workspace checkout instead of fetching the git URL
    # pinned in Cargo.lock. PRs that edit a workspace crate alongside rpkg get
    # their edits reflected in vendor/ and inst/vendor.tar.xz, so
    # `vendor-sync-check` passes and the offline tarball ships the PR's code.
    # `--source-root` is no longer needed here (kept as CLI flag for back-compat).
    cargo revendor \
      --manifest-path rpkg/src/rust/Cargo.toml \
      --output rpkg/vendor \
      --compress rpkg/inst/vendor.tar.xz \
      --blank-md \
      --source-marker \
      --force \
      -v
    echo ""
    echo "Created rpkg/inst/vendor.tar.xz — DELETE THIS BEFORE RESUMING DEV ITERATION"
    echo "(run 'just clean-vendor-leak' or 'unlink(\"rpkg/inst/vendor.tar.xz\")' in R)"

# Remove a leaked rpkg/inst/vendor.tar.xz.
# inst/vendor.tar.xz is the single signal that flips configure into tarball mode.
# A leftover tarball makes subsequent dev installs silently use stale vendored
# sources instead of workspace edits. Run this if you're seeing
# "tarball install — vendor/ already populated" during dev iteration and you
# didn't intend to stay in tarball mode.
[script("bash")]
clean-vendor-leak:
    set -euo pipefail
    if [ -f rpkg/inst/vendor.tar.xz ]; then
        rm -f rpkg/inst/vendor.tar.xz
        echo "Removed rpkg/inst/vendor.tar.xz (tarball-mode leak)."
    else
        echo "No tarball leak to clean."
    fi

# Verify committed Cargo.lock, vendor/, and vendor.tar.xz agree (#157).
# Runs in CI/pre-release to guarantee the offline build artifact is fresh.
vendor-verify:
    cargo run --manifest-path cargo-revendor/Cargo.toml --quiet -- \
      --verify \
      --manifest-path rpkg/src/rust/Cargo.toml \
      --output rpkg/vendor \
      --compress rpkg/inst/vendor.tar.xz \
      -v

# Load and test rpkg with devtools
devtools-test FILTER="": devtools-document
    if [ -z "{{FILTER}}" ]; then \
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg")'; \
    else \
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg", filter = "{{FILTER}}")'; \
    fi

# Load rpkg with devtools::load_all
alias devtools-load_all := devtools-load
devtools-load: devtools-document
    Rscript -e 'devtools::load_all("rpkg")'

# Install rpkg with devtools::install
devtools-install: devtools-document
    Rscript -e 'devtools::install("rpkg")'

# Install R dependencies used by the repo (devtools, roxygen2, testthat, R6, S7, vctrs, etc.)
install_deps:
    Rscript -e 'install.packages(c("devtools","roxygen2","rcmdcheck","pkgbuild","processx","testthat","R6","S7","vctrs"), repos = "https://cloud.r-project.org")'

# Install Rust dev tools used for day-to-day iteration.
# cargo-limit: provides `cargo lcheck`/`lclippy`/`ltest`/`lbuild` that surface the first
# few errors/warnings without dumping thousands of lines of "noise" into the terminal.
# Prefer these aliases in CLI one-offs; CI still runs full `cargo clippy ... -D warnings`.
# cargo-revendor: required by `just configure` in dev-monorepo mode to sync
# workspace crates into rpkg/vendor/ (otherwise cargo metadata fails on the
# frozen path = "../../vendor/..." deps in rpkg/src/rust/Cargo.toml).
dev-tools-install: revendor-install
    cargo install cargo-limit

# Install Windows system tooling via Scoop (https://scoop.sh). Currently installs
# `rv` (https://github.com/a2-ai/rv), the R version manager used on this repo.
[windows]
install-deps-windows:
    powershell.exe -NoProfile -Command "scoop install rv"

# Install minirextendr dependencies (for scaffolding helper package)
minirextendr-install-deps:
    Rscript -e 'install.packages(c("cli","curl","desc","fs","gh","glue","rappdirs","rlang","rprojroot","usethis","withr","devtools","roxygen2","testthat"), repos = "https://cloud.r-project.org")'

# Build rpkg with devtools::build
# Depends on `vendor` for the same reason as r-cmd-build — devtools::build
# wraps R CMD build, and the resulting tarball is meaningful only with
# inst/vendor.tar.xz inside.
# Cleanup: inst/vendor.tar.xz is removed on exit so the source tree is never
# left in tarball mode after this recipe finishes.
[script("bash")]
devtools-build: configure vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    Rscript -e 'devtools::build("rpkg")'

# Check rpkg with devtools::check
# error_on = "error" matches CI behavior (ignore warnings/notes)
# check_dir preserves output for investigation (not auto-cleaned)
devtools-check: devtools-document
    Rscript -e 'devtools::check("rpkg", error_on = "error", check_dir = "{{check_output_dir}}")'

# Document rpkg with devtools::document (roxygen2 → NAMESPACE + man pages)
# R wrappers are generated automatically by Makevars during R CMD INSTALL.
devtools-document: configure
    Rscript -e 'devtools::document("rpkg")'

# Document ALL R packages in the workspace
# This includes: rpkg, minirextendr, and cross-package test packages
document-all: devtools-document minirextendr-document
    cd tests/cross-package && just document-all

# Configure ALL R packages that need vendoring/configure
# This includes: rpkg and cross-package test packages (minirextendr has no configure step)
configure-all: configure cross-configure

alias rcmdinstall := r-cmd-install
r-cmd-install *args: configure
    R CMD INSTALL {{args}} rpkg 

# Build R package tarball
# Depends on `vendor` so the tarball ships inst/vendor.tar.xz, which is what
# triggers tarball-mode install (offline, vendored sources). Without this dep
# a maintainer can silently produce a tarball that source-mode-installs over
# the network — defeating the point of `R CMD build` for CRAN submission.
#
# Cleanup: rpkg/inst/vendor.tar.xz must be removed after R CMD build copies
# it into the built tarball. Otherwise the leftover in the source tree makes
# the next `just rcmdinstall` / `devtools::install("rpkg")` silently switch
# to tarball mode (configure's only signal is `[ -f inst/vendor.tar.xz ]`),
# which freezes out monorepo workspace-crate edits via `[patch."git+url"]`.
alias rcmdbuild := r-cmd-build
[script("bash")]
r-cmd-build *args: configure vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    R CMD build {{args}} --no-manual --log --debug rpkg

# Run R CMD check on rpkg
# Depends on vendor to ensure inst/vendor.tar.xz exists in the tarball.
# R CMD check copies the tarball to a temp dir where monorepo [patch] paths
# are unavailable — configure detects this and uses vendored sources instead.
#
# Cleanup: same reason as r-cmd-build — see comment there.
alias rcmdcheck := r-cmd-check
[script("bash")]
r-cmd-check *args: vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    ERROR_ON="warning"
    CHECK_DIR=""
    for arg in {{args}}; do
      case "$arg" in
        ERROR_ON=*) ERROR_ON="${arg#ERROR_ON=}" ;;
        CHECK_DIR=*) CHECK_DIR="${arg#CHECK_DIR=}" ;;
        *) echo "Ignoring unknown arg '$arg'" ;;
      esac
    done
    CHECK_DIR_ARG="NULL"
    if [ -n "$CHECK_DIR" ]; then
      case "$CHECK_DIR" in
        /*) CHECK_DIR_ARG="'$CHECK_DIR'" ;;
        *)  CHECK_DIR_ARG="'$(pwd)/$CHECK_DIR'" ;;
      esac
    fi
    Rscript -e "rcmdcheck::rcmdcheck('rpkg', args = c('--as-cran','--no-manual'), error_on = '${ERROR_ON}', check_dir = ${CHECK_DIR_ARG})"

# Extract and inspect R package tarball contents (for debugging build artifacts)
#
# Builds tarball with --compression=none and extracts to rpkg_build/ for inspection.
# Useful for verifying what gets included in CRAN submissions.
[script("bash")]
test-r-build: configure
    set -euo pipefail
    # MSYS2 tar interprets D: as remote host; --force-local treats all as local
    TAR_FORCE_LOCAL=""
    case "$(uname -s)" in MSYS*|MINGW*|CYGWIN*) TAR_FORCE_LOCAL="--force-local";; esac
    # Extract package info from DESCRIPTION
    pkg=$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Package"]])')
    ver=$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Version"]])')
    # Build tarball
    R CMD build --compression=none rpkg
    # Determine tarball name (.tar or .tar.gz)
    tarball="${pkg}_${ver}.tar"
    [[ -f "$tarball" ]] || tarball="${tarball}.gz"
    # Extract for inspection
    out_dir="rpkg_build/${pkg}_${ver}"
    mkdir -p "$out_dir"
    (cd "$out_dir" && tar $TAR_FORCE_LOCAL -xf "$tarball" --strip-components=1)
    echo "Extracted to: $out_dir"

# ============================================================================
# minirextendr R package development
# ============================================================================

# Generate documentation for minirextendr R package
minirextendr-document:
    Rscript -e 'devtools::document("minirextendr")'

# Run tests for minirextendr R package
[script("bash")]
minirextendr-test FILTER="":
    export MINIEXTENDR_LOCAL_PATH="$(pwd)"
    if [ -z "{{FILTER}}" ]; then
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("minirextendr")'
    else
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("minirextendr", filter = "{{FILTER}}")'
    fi

# Check minirextendr R package with devtools::check
minirextendr-check:
    MINIEXTENDR_LOCAL_PATH="$(pwd)" Rscript -e 'devtools::check("minirextendr", error_on = "error")'

# Install minirextendr R package with devtools::install
minirextendr-install:
    Rscript -e 'devtools::install("minirextendr")'

# Load minirextendr with devtools::load_all (for interactive development)
minirextendr-load:
    Rscript -e 'devtools::load_all("minirextendr")'

# Build minirextendr R package tarball
minirextendr-build:
    R CMD build --no-manual minirextendr

# Run R CMD check on minirextendr package
[script("bash")]
minirextendr-rcmdcheck:
    export MINIEXTENDR_LOCAL_PATH="$(pwd)"
    Rscript -e "rcmdcheck::rcmdcheck('minirextendr', args = c('--no-manual'), error_on = 'warning')"

# Full development cycle for minirextendr: document, test, check
minirextendr-dev: minirextendr-document minirextendr-test minirextendr-check

# ============================================================================
# Cross-package trait dispatch testing (tests/cross-package)
# ============================================================================

cross-document:
    cd tests/cross-package && just document-all

cross-configure:
    cd tests/cross-package && just configure-all

alias cross-build := cross-install
cross-install:
    cd tests/cross-package && just install-all

cross-test:
    cd tests/cross-package && just test-all

cross-check:
    cd tests/cross-package && just check-all

cross-clean:
    cd tests/cross-package && just clean

cross-dev:
    cd tests/cross-package && just dev

# ============================================================================
# Templates / drift check
# ============================================================================
#
# Pattern:
# - upstream snapshot   : built from sources within this repo (see templates-sources)
# - inst/templates/**   : your edited copies
# - patches/templates.patch : the *approved* delta
#
# Workflow:
#   just templates-check         # fails if inst/templates drift beyond approved patch
#   just templates-approve       # accept current delta as approved (regen patch)

local_root  := "minirextendr/inst/templates"
patch_file  := "patches/templates.patch"

# Configure your upstream locations here.
#
# Use TAB-separated pairs: <relative/path/in/templates>\t<source/path>
# - For a directory source, end BOTH sides with a trailing slash.
# - Paths with spaces are OK (TAB is the separator).
#
# The templates are scaffolding for new packages. The rpkg files are the
# "upstream" source, and templates may have intentional differences like
# {{package_rs}} placeholders. The patch file captures approved differences.

[script("bash")]
templates-sources:
    set -euo pipefail

    # Two template types exist:
    #   - rpkg/          : Standalone R package template
    #   - monorepo/      : Rust workspace with embedded R package
    #
    # Only include files where rpkg is the source of truth.
    cat <<'EOF'
    # rel	src
    # === R Package Template (rpkg/) ===
    rpkg/bootstrap.R	rpkg/bootstrap.R
    rpkg/build.rs	rpkg/src/rust/build.rs
    rpkg/cdylib-exports.def	rpkg/src/cdylib-exports.def
    rpkg/cleanup	rpkg/cleanup
    rpkg/cleanup.ucrt	rpkg/cleanup.ucrt
    rpkg/cleanup.win	rpkg/cleanup.win
    rpkg/configure.ac	rpkg/configure.ac
    rpkg/configure.ucrt	rpkg/configure.ucrt
    rpkg/configure.win	rpkg/configure.win
    rpkg/Makevars.in	rpkg/src/Makevars.in
    rpkg/Makevars.win	rpkg/src/Makevars.win
    rpkg/stub.c	rpkg/src/stub.c
    rpkg/tools/lock-shape-check.R	rpkg/tools/lock-shape-check.R
    rpkg/win.def.in	rpkg/src/win.def.in
    # === Monorepo Template (monorepo/) ===
    # The embedded R package uses same sources as rpkg/ template
    monorepo/rpkg/bootstrap.R	rpkg/bootstrap.R
    monorepo/rpkg/build.rs	rpkg/src/rust/build.rs
    monorepo/rpkg/cdylib-exports.def	rpkg/src/cdylib-exports.def
    monorepo/rpkg/cleanup	rpkg/cleanup
    monorepo/rpkg/cleanup.ucrt	rpkg/cleanup.ucrt
    monorepo/rpkg/cleanup.win	rpkg/cleanup.win
    monorepo/rpkg/configure.ac	rpkg/configure.ac
    monorepo/rpkg/configure.ucrt	rpkg/configure.ucrt
    monorepo/rpkg/configure.win	rpkg/configure.win
    monorepo/rpkg/Makevars.in	rpkg/src/Makevars.in
    monorepo/rpkg/Makevars.win	rpkg/src/Makevars.win
    monorepo/rpkg/stub.c	rpkg/src/stub.c
    monorepo/rpkg/tools/lock-shape-check.R	rpkg/tools/lock-shape-check.R
    monorepo/rpkg/win.def.in	rpkg/src/win.def.in
    EOF

# Internal helper: populate an upstream snapshot into DEST.
# The snapshot is a tree laid out to match inst/templates.
[script("bash")]
_templates-upstream-populate dest:
    set -euo pipefail

    dest="{{dest}}"
    mkdir -p "$dest"

    manifest="$(just --quiet templates-sources)"

    add() {
      local rel="$1" src="$2" dst="$dest/$rel"
      if [[ "$rel" == */ ]]; then
        mkdir -p "$dst"
        rsync -a "$src" "$dst"
      else
        mkdir -p "$(dirname "$dst")"
        cp -a "$src" "$dst"
      fi
    }

    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue

      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      src="$(printf '%s' "$src" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"

      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      if [[ -z "$src" ]]; then
        echo "_templates-upstream-populate: missing source path for rel='$rel'" >&2
        exit 2
      fi
      if [[ ! -e "$src" ]]; then
        echo "_templates-upstream-populate: source not found: $src (for rel='$rel')" >&2
        exit 2
      fi

      # Disallow absolute paths to keep this repo-portable
      if [[ "$src" = /* ]]; then
        echo "_templates-upstream-populate: absolute paths are not allowed (got: $src)" >&2
        exit 2
      fi

      add "$rel" "$src"
    done <<<"$manifest"

# Accept the current delta as approved by regenerating patches/templates.patch
# (Builds an upstream snapshot from templates-sources before diffing.)
[script("bash")]
templates-approve:
    set -euo pipefail

    mkdir -p "$(dirname "{{patch_file}}")"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/a" "$tmp/b"

    just _templates-upstream-populate "$tmp/a"

    # Populate b/ with template versions of same files (not entire template dir)
    manifest="$(just --quiet templates-sources)"
    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue
      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      template_file="{{local_root}}/$rel"
      if [[ -e "$template_file" ]]; then
        mkdir -p "$(dirname "$tmp/b/$rel")"
        cp -a "$template_file" "$tmp/b/$rel"
      fi
    done <<<"$manifest"

    # diff exits 1 when differences exist; that's expected here.
    # -U2 = 2 context lines (default is 3)
    (cd "$tmp" && diff -ruN -U2 a b) > "{{patch_file}}" || true
    echo "Wrote {{patch_file}}"

# Verify: upstream snapshot + approved patch == inst/templates
# - exits nonzero on drift
# - exits nonzero if the patch no longer applies cleanly
[script("bash")]
templates-check:
    set -euo pipefail

    test -f "{{patch_file}}"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    just _templates-upstream-populate "$tmp"

    # Apply approved delta (no-op if patch is empty)
    if [[ -s "{{patch_file}}" ]]; then
      patch -d "$tmp" -p1 --forward --batch < "{{patch_file}}" >/dev/null
    fi

    # Compare only files defined in templates-sources (not entire templates dir)
    tmp2="$(mktemp -d)"
    trap 'rm -rf "$tmp" "$tmp2"' EXIT

    manifest="$(just --quiet templates-sources)"
    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue
      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      template_file="{{local_root}}/$rel"
      if [[ -e "$template_file" ]]; then
        mkdir -p "$(dirname "$tmp2/$rel")"
        cp -a "$template_file" "$tmp2/$rel"
      fi
    done <<<"$manifest"

    diff -ruN "$tmp" "$tmp2"

# CI-friendly: only prints diff when failing
[script("bash")]
templates-check-ci:
    set -euo pipefail

    test -f "{{patch_file}}"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    just _templates-upstream-populate "$tmp"

    if [[ -s "{{patch_file}}" ]]; then
      patch -d "$tmp" -p1 --forward --batch < "{{patch_file}}" >/dev/null
    fi

    if ! diff -ruN "$tmp" "{{local_root}}" >/dev/null; then
      diff -ruN "$tmp" "{{local_root}}"
      exit 1
    fi

# ==============================================================================
# Vendor sync check (ensure vendored crates match workspace)
# ==============================================================================
# After `just vendor`, rpkg/vendor/ should contain synced copies of workspace
# crates (miniextendr-api, miniextendr-macros, etc.) alongside external deps.
#
# This check verifies the vendored copies haven't drifted from the workspace.
# Run `just vendor` to refresh if this check fails.

# Check that vendored miniextendr crates match workspace sources
[script("bash")]
vendor-sync-check:
    set -euo pipefail

    vendor_dir="rpkg/vendor"
    drift_found=0

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      # Accept both flat (vendor/<name>/) and versioned (vendor/<name>-<version>/) layouts
      crate_dir=""
      if [[ -d "$vendor_dir/$crate" ]]; then
        crate_dir="$vendor_dir/$crate"
      else
        versioned=$(find "$vendor_dir" -maxdepth 1 -type d -name "${crate}-[0-9]*" | head -1)
        if [[ -n "$versioned" ]]; then
          crate_dir="$versioned"
        fi
      fi

      if [[ -z "$crate_dir" ]]; then
        echo "WARNING: $vendor_dir/$crate (or versioned) not found (run 'just configure' first)"
        continue
      fi

      # Compare src directories (the actual code)
      if ! diff -rq "$crate/src" "$crate_dir/src" >/dev/null 2>&1; then
        echo "DRIFT: $crate/src differs from $crate_dir/src"
        drift_found=1
      fi
    done

    if [[ $drift_found -eq 1 ]]; then
      echo ""
      echo "Vendored crates have drifted from workspace sources."
      echo "Run 'just vendor' to refresh vendored copies."
      exit 1
    else
      echo "Vendor sync check passed: all miniextendr crates match."
    fi

# Verify lint crate compiles against the current miniextendr-macros parser
lint-sync-check:
    cargo check --manifest-path=miniextendr-lint/Cargo.toml

# Show diff between workspace and vendored crates
[script("bash")]
vendor-sync-diff:
    set -euo pipefail

    vendor_dir="rpkg/vendor"

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      # Accept both flat (vendor/<name>/) and versioned (vendor/<name>-<version>/) layouts
      crate_dir=""
      if [[ -d "$vendor_dir/$crate" ]]; then
        crate_dir="$vendor_dir/$crate"
      else
        versioned=$(find "$vendor_dir" -maxdepth 1 -type d -name "${crate}-[0-9]*" | head -1)
        if [[ -n "$versioned" ]]; then
          crate_dir="$versioned"
        fi
      fi

      if [[ -n "$crate_dir" ]]; then
        echo "=== $crate ==="
        diff -ruN "$crate/src" "$crate_dir/src" || true
        echo ""
      fi
    done

# Check that rpkg/src/rust/Cargo.lock is in tarball-shape.
#
# One invariant remains after cargo-revendor item 2:
#   - miniextendr-{api,lint,macros} must use git+url#<sha> sources, not path+.
#
# checksum = "..." lines are now ALLOWED (cargo-revendor writes valid
# .cargo-checksum.json files whose `package` field matches them).
[script("bash")]
lock-shape-check:
    set -euo pipefail
    lock=rpkg/src/rust/Cargo.lock
    if [ ! -f "$lock" ]; then
        echo "lock-shape-check: $lock not found — skipping"
        exit 0
    fi
    bad=0
    if grep -q 'source = "path+' "$lock"; then
        echo "lock-shape-check: $lock has path+... sources (tarball-shape violation)" >&2
        bad=1
    fi
    if [ $bad -eq 1 ]; then
        echo "  Run: just vendor    # regenerates canonical shape" >&2
        exit 1
    fi
    echo "lock-shape-check: OK"

# ============================================================================
# mx CLI (miniextendr-cli)
# ============================================================================

# Build the miniextendr CLI binary (with dev commands)
cli-build *cargo_flags:
    cargo build -p miniextendr-cli --features dev {{cargo_flags}}

# Install the miniextendr CLI binary (with dev commands)
cli-install *cargo_flags:
    cargo install --path miniextendr-cli --features dev {{cargo_flags}}

# ============================================================================
# cargo-revendor (standalone workspace, not part of miniextendr workspace)
# ============================================================================

# Install cargo-revendor
revendor-install:
    cargo install --path cargo-revendor

# Build cargo-revendor (without installing)
revendor-build *cargo_flags:
    cargo build --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}

# Clean build cargo-revendor with timing analysis
revendor-timings:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Cleaning..."
    cd cargo-revendor && cargo clean 2>/dev/null || true
    echo "Building with --timings..."
    cd cargo-revendor && cargo build --timings 2>&1 | tail -3
    echo ""
    python3 -c "
    import re, json
    html = open('cargo-revendor/target/cargo-timings/cargo-timing.html').read()
    m = re.search(r'UNIT_DATA\s*=\s*(\[.+?\]);\s', html, re.DOTALL)
    if not m:
        print('Could not parse timing data')
        exit()
    data = json.loads(m.group(1))
    data.sort(key=lambda x: x.get('duration', 0), reverse=True)
    total = sum(d.get('duration', 0) for d in data)
    print(f'Compile units: {len(data)}')
    print(f'Total CPU time: {total:.1f}s')
    print()
    print(f'{\"TIME\":>6}  CRATE')
    print('-' * 50)
    for d in data[:15]:
        print(f'  {d.get(\"duration\", 0):5.1f}s  {d.get(\"name\", \"?\")} {d.get(\"version\", \"\")}')
    "

# Run cargo-revendor tests (offline only)
revendor-test:
    cargo test --manifest-path cargo-revendor/Cargo.toml

# Run cargo-revendor tests (including network tests)
revendor-test-all:
    cargo test --manifest-path cargo-revendor/Cargo.toml -- --include-ignored --test-threads=1

# ── Native R package bindgen corpus ─────────────────────────────────────────

# Run bindgen corpus test (C-only mode, ~69 packages)
bindgen-corpus:
    bash dev/run_bindgen_corpus.sh

# Run bindgen corpus test (C + C++ mode, ~308 packages)
bindgen-corpus-v3:
    bash dev/run_bindgen_corpus_v3.sh

# Add all CRAN corpus packages (with inst/include) to rv, excluding Rcpp/cpp11 ecosystem
bindgen-corpus-add-packages:
    #!/usr/bin/env bash
    set -euo pipefail
    PKGS=$(awk -F',' 'NR==1{for(i=1;i<=NF;i++){gsub(/"/,"",$i);if($i=="excluded_rcpp_ecosystem")col=i}} NR>1{gsub(/"/,"",$col);gsub(/"/,"",$1);if($col=="FALSE")print $1}' dev/cran_native_packages_full.csv | tr '\n' ' ')
    echo "Adding $(echo $PKGS | wc -w | tr -d ' ') packages to rv..."
    rv add --no-sync $PKGS
    echo "Run 'rv sync' to install them."

# Remove all CRAN corpus packages from rv (keeps core dev deps)
bindgen-corpus-remove-packages:
    #!/usr/bin/env bash
    set -euo pipefail
    PKGS=$(awk -F',' 'NR==1{for(i=1;i<=NF;i++){gsub(/"/,"",$i);if($i=="excluded_rcpp_ecosystem")col=i}} NR>1{gsub(/"/,"",$col);gsub(/"/,"",$1);if($col=="FALSE")print $1}' dev/cran_native_packages_full.csv | tr '\n' ' ')
    echo "Removing $(echo $PKGS | wc -w | tr -d ' ') corpus packages from rv..."
    rv remove --no-sync $PKGS
    echo "Run 'rv sync' to clean the library."

# ── Documentation site ──────────────────────────────────────────────────────

# Regenerate site/content/manual/ from docs/ and site/data/plans.json from plans/.
# Run before site-build or site-serve when previewing doc or plan changes locally.
# CI runs docs-to-site.sh automatically before each deploy.
site-docs:
    bash scripts/docs-to-site.sh
    bash scripts/plans-to-json.sh > site/data/plans.json

# Build Zola site (output in site/public/). Run `just site-docs` first if docs or plans changed.
site-build:
    cd site && zola build

# Local preview server (http://127.0.0.1:1111). Run `just site-docs` first if docs or plans changed.
site-serve:
    cd site && zola serve

# Bump version across all Cargo.toml and DESCRIPTION files.
bump-version version:
    bash scripts/bump-version.sh {{version}}
