# https://just.systems
#
# Quick reference:
#   Rust:
#     just check              - Run cargo check
#     just test               - Run cargo tests
#     just clippy             - Run lints
#     just fmt                - Format Rust code
#     just lint               - Run miniextendr-lint on rpkg
#
#   rpkg (example R package):
#     just configure          - Configure R package build
#     just devtools-test      - Run R package tests
#     just devtools-document  - Generate R documentation
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
#   Vendor sync:
#     just vendor-sync-check  - Verify vendored crates match workspace
#     just vendor-sync-diff   - Show diff between workspace and vendor
#
#   Lint sync:
#     just lint-sync-check    - Check lint parser vs macros parser
#     just lint-sync-diff     - Show diff between parsers

default:
    @just --list
# TODO: add the vendor checksum file to this recipe!
clean:
    -just configure
    -just cargo-clean
    -cd rpkg && NOT_CRAN=false ./cleanup
    -cd tests/cross-package && just clean

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
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo clean --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo clean --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clean --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Check all crates
alias cargo-check := check
check *cargo_flags:
    cargo check --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo check --benches --tests --examples --workspace --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Build all crates
alias cargo-build := build
build *cargo_flags:
    cargo build --benches --tests --examples --workspace {{cargo_flags}}
    cargo build --manifest-path=miniextendr-bench/Cargo.toml --benches --tests --examples {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo build --benches --tests --examples --workspace --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run clippy on all crates
alias cargo-clippy := clippy
clippy *cargo_flags:
    cargo clippy --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run miniextendr-lint on rpkg (checks #[miniextendr] ↔ miniextendr_module! consistency)
# The lint runs as a build script; this command triggers it via cargo check.
# Lint output appears as cargo warnings. Errors indicate:
# - #[miniextendr] items missing from miniextendr_module!
# - miniextendr_module! items without #[miniextendr] attribute
# - Multiple unlabeled impl blocks for the same type
# - Class system incompatibilities between inherent and trait impls
lint:
    #!/usr/bin/env bash
    set -euo pipefail
    cd rpkg
    output=$(NOT_CRAN=true cargo check --manifest-path=src/rust/Cargo.toml 2>&1) || {
        echo "$output"
        echo ""
        echo "::error::cargo check failed (see output above)"
        exit 1
    }
    lint_issues=$(echo "$output" | grep -E "warning.*miniextendr-lint" || true)
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
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo doc --no-deps --document-private-items --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Build and open documentation
alias cargo-doc := doc
doc *cargo_flags: configure-all
    cargo doc --document-private-items --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
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
    && root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/src/rust/target" cargo test --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args) \
    && root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/src/rust/target" cargo test --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args) \
    && root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo test --manifest-path="$root/rpkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" -- --no-capture $test_args)

# Run benchmarks (miniextendr-bench)
alias cargo-bench := bench
bench *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}

# Check benchmark crate
bench-check *cargo_flags:
    cargo check --manifest-path=miniextendr-bench/Cargo.toml --benches --tests --examples {{cargo_flags}}

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

# Run ./configure and vendor rpkg dependencies
#
# This prepares the R package for building by:
# 1. Running autoconf + configure script
#    - Syncs miniextendr-api and miniextendr-macros to rpkg/src/vendor/ (via rsync)
#    - Generates build configuration files
# 2. Vendoring crates.io dependencies to rpkg/src/vendor/
#    - proc-macro2, quote, syn, unicode-ident
#
# This is the only vendoring needed - R packages must be self-contained for CRAN.
# Workspace crates use normal cargo dependency resolution (no vendoring needed).
configure:
    cd rpkg && \
    if command -v autoconf >/dev/null 2>&1; then autoconf; else echo "autoconf not found; using existing configure"; fi && \
    NOT_CRAN=true ./configure

# Configure in CRAN/offline mode (do NOT force NOT_CRAN=true)
configure-cran:
    cd rpkg && \
    if command -v autoconf >/dev/null 2>&1; then autoconf; else echo "autoconf not found; using existing configure"; fi && \
    ./configure

# Load and test rpkg with devtools
devtools-test FILTER="": configure
    if [ -z "{{FILTER}}" ]; then \
      Rscript -e 'devtools::test("rpkg")'; \
    else \
      Rscript -e 'devtools::test("rpkg", filter = "{{FILTER}}")'; \
    fi

# Load rpkg with devtools::load_all
alias devtools-load_all := devtools-load
devtools-load: configure
    Rscript -e 'devtools::load_all("rpkg")'

# Install rpkg with devtools::install
devtools-install: configure
    Rscript -e 'devtools::install("rpkg")'

# Install R dependencies used by the repo (devtools, roxygen2, testthat, R6, S7, etc.)
install_deps:
    Rscript -e 'install.packages(c("devtools","roxygen2","rcmdcheck","pkgbuild","processx","testthat","R6","S7"), repos = "https://cloud.r-project.org")'

# Build rpkg with devtools::build
devtools-build: configure
    Rscript -e 'devtools::build("rpkg")'

# Check rpkg with devtools::check
# NOT_CRAN=true ensures vendor directory is preserved during R CMD build
# error_on = "error" matches CI behavior (ignore warnings/notes)
devtools-check: configure
    NOT_CRAN=true Rscript -e 'devtools::check("rpkg", error_on = "error")'

# Document rpkg with devtools::document
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
alias rcmdbuild := r-cmd-build
r-cmd-build *args: configure
    R CMD build {{args}} --no-manual --log --debug rpkg

# Run R CMD check on rpkg
alias rcmdcheck := r-cmd-check
r-cmd-check *args: configure
    @ERROR_ON="warning" \
    CHECK_DIR="" \
    && for arg in {{args}}; do \
      case "$arg" in \
        ERROR_ON=*) ERROR_ON="${arg#ERROR_ON=}" ;; \
        CHECK_DIR=*) CHECK_DIR="${arg#CHECK_DIR=}" ;; \
        *) echo "Ignoring unknown arg '$arg'" ;; \
      esac; \
    done \
    && CHECK_DIR_ARG="NULL" \
    && if [ -n "$CHECK_DIR" ]; then \
      case "$CHECK_DIR" in \
        /*) CHECK_DIR_ARG="'$CHECK_DIR'" ;; \
        *)  CHECK_DIR_ARG="'$(pwd)/$CHECK_DIR'" ;; \
      esac; \
    fi \
    && Rscript -e "rcmdcheck::rcmdcheck('rpkg', args = c('--as-cran','--no-manual'), error_on = '${ERROR_ON}', check_dir = ${CHECK_DIR_ARG})"

# Extract and inspect R package tarball contents (for debugging build artifacts)
#
# Builds tarball with --compression=none and extracts to rpkg_build/ for inspection.
# Useful for verifying what gets included in CRAN submissions.
test-r-build: configure
    #!/usr/bin/env bash
    set -euo pipefail
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
    tar -xf "$tarball" -C "$out_dir" --strip-components=1
    echo "Extracted to: $out_dir"

# ============================================================================
# minirextendr R package development
# ============================================================================

# Generate documentation for minirextendr R package
minirextendr-document:
    Rscript -e 'devtools::document("minirextendr")'

# Run tests for minirextendr R package
minirextendr-test FILTER="":
    #!/usr/bin/env bash
    if [ -z "{{FILTER}}" ]; then
      Rscript -e 'devtools::test("minirextendr")'
    else
      Rscript -e 'devtools::test("minirextendr", filter = "{{FILTER}}")'
    fi

# Check minirextendr R package with devtools::check
minirextendr-check:
    Rscript -e 'devtools::check("minirextendr", error_on = "error")'

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
minirextendr-rcmdcheck:
    #!/usr/bin/env bash
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

templates-sources:
    #!/usr/bin/env bash
    set -euo pipefail

    # Two template types exist:
    #   - rpkg/          : Standalone R package template
    #   - monorepo/      : Rust workspace with embedded R package
    #
    # Only include files where rpkg is the source of truth.
    # Templates with @PLACEHOLDER@ markers (document.rs.in, entrypoint.c.in)
    # are NOT compared - they are the source, rpkg has expanded versions.
    cat <<'EOF'
    # rel	src
    # === R Package Template (rpkg/) ===
    rpkg/Makevars.in	rpkg/src/Makevars.in
    rpkg/configure.ac	rpkg/configure.ac
    rpkg/build.rs	rpkg/src/rust/build.rs
    # === Monorepo Template (monorepo/) ===
    # Monorepo root files are template-only (no rpkg source)
    # The embedded R package uses same sources as rpkg/ template
    monorepo/rpkg/Makevars.in	rpkg/src/Makevars.in
    monorepo/rpkg/configure.ac	rpkg/configure.ac
    monorepo/rpkg/build.rs	rpkg/src/rust/build.rs
    EOF

# Internal helper: populate an upstream snapshot into DEST.
# The snapshot is a tree laid out to match inst/templates.
_templates-upstream-populate dest:
    #!/usr/bin/env bash
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
templates-approve:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p "$(dirname "{{patch_file}}")"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/a" "$tmp/b"

    just _templates-upstream-populate "$tmp/a"
    rsync -a "{{local_root}}/" "$tmp/b/"

    # diff exits 1 when differences exist; that's expected here.
    # -U2 = 2 context lines (default is 3)
    (cd "$tmp" && diff -ruN -U2 a b) > "{{patch_file}}" || true
    echo "Wrote {{patch_file}}"

# Verify: upstream snapshot + approved patch == inst/templates
# - exits nonzero on drift
# - exits nonzero if the patch no longer applies cleanly
templates-check:
    #!/usr/bin/env bash
    set -euo pipefail

    test -f "{{patch_file}}"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    just _templates-upstream-populate "$tmp"

    # Apply approved delta (no-op if patch is empty)
    if [[ -s "{{patch_file}}" ]]; then
      patch -d "$tmp" -p1 --forward --batch < "{{patch_file}}" >/dev/null
    fi

    diff -ruN "$tmp" "{{local_root}}"

# CI-friendly: only prints diff when failing
templates-check-ci:
    #!/usr/bin/env bash
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
# After `just configure`, rpkg/src/vendor/ should contain synced copies of:
#   - miniextendr-api
#   - miniextendr-macros
#   - miniextendr-lint
#   - miniextendr-engine
#
# This check verifies the vendored copies haven't drifted from the workspace.
# Run `just configure` to refresh vendored copies if this check fails.

# Check that vendored miniextendr crates match workspace sources
vendor-sync-check:
    #!/usr/bin/env bash
    set -euo pipefail

    vendor_dir="rpkg/src/vendor"
    drift_found=0

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      if [[ ! -d "$vendor_dir/$crate" ]]; then
        echo "WARNING: $vendor_dir/$crate not found (run 'just configure' first)"
        continue
      fi

      # Compare src directories (the actual code)
      if ! diff -rq "$crate/src" "$vendor_dir/$crate/src" >/dev/null 2>&1; then
        echo "DRIFT: $crate/src differs from $vendor_dir/$crate/src"
        drift_found=1
      fi
    done

    if [[ $drift_found -eq 1 ]]; then
      echo ""
      echo "Vendored crates have drifted from workspace sources."
      echo "Run 'just configure' to refresh vendored copies."
      exit 1
    else
      echo "Vendor sync check passed: all miniextendr crates match."
    fi

# Show diff between workspace and vendored crates
vendor-sync-diff:
    #!/usr/bin/env bash
    set -euo pipefail

    vendor_dir="rpkg/src/vendor"

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      if [[ -d "$vendor_dir/$crate" ]]; then
        echo "=== $crate ==="
        diff -ruN "$crate/src" "$vendor_dir/$crate/src" || true
        echo ""
      fi
    done

# ==============================================================================
# Lint file sync check (ensure lint crate parser matches macros)
# ==============================================================================
# The miniextendr-lint crate has a similar miniextendr_module.rs from miniextendr-macros.
# The lint version omits codegen helpers (call_method_def_ident, r_wrapper_const_ident)
# that depend on macros-only functions. This check helps identify parser changes that
# need manual porting.

# Check for significant drift between macros and lint parsers (informational)
lint-sync-check:
    #!/usr/bin/env bash
    set -euo pipefail

    macros_file="miniextendr-macros/src/miniextendr_module.rs"
    lint_file="miniextendr-lint/src/miniextendr_module.rs"

    # Count differing lines (excluding known differences)
    diff_lines=$(diff "$macros_file" "$lint_file" 2>/dev/null | grep -c "^[<>]" || echo "0")

    if [[ "$diff_lines" -gt 30 ]]; then
      echo "WARNING: $lint_file differs significantly from $macros_file ($diff_lines lines)"
      echo ""
      echo "The lint crate parser may have drifted from the macros parser."
      echo "Review 'just lint-sync-diff' and manually port any parsing changes."
      exit 1
    else
      echo "Lint sync check passed: parsers are in sync (minor expected differences)."
    fi

# Show diff between macros and lint parsers
lint-sync-diff:
    diff -u miniextendr-macros/src/miniextendr_module.rs miniextendr-lint/src/miniextendr_module.rs || true
