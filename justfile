# https://just.systems

default:
    @just --list

# Clean build artifacts
alias cargo-clean := clean
clean *cargo_flags:
    cargo clean -p miniextendr-api {{cargo_flags}}
    cargo clean -p miniextendr-macros {{cargo_flags}}
    cargo clean --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check all crates
alias cargo-check := check
check *cargo_flags:
    cargo check --workspace {{cargo_flags}}
    cargo check --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}


# Build all crates
alias cargo-build := build
build *cargo_flags:
    cargo build --workspace {{cargo_flags}}
    cargo build --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Run clippy on all crates
alias cargo-clippy := clippy
clippy *cargo_flags:
    cargo clippy --workspace {{cargo_flags}}
    cargo clippy --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check documentation builds
alias cargo-doc-check := doc-check
doc-check *cargo_flags: configure
    cargo doc --no-deps --document-private-items --workspace {{cargo_flags}}
    cargo doc --no-deps --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Build and open documentation
alias cargo-doc := doc
doc *cargo_flags: configure
    cargo doc --document-private-items --workspace {{cargo_flags}}
    cargo doc --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml --open {{cargo_flags}}

# Check formatting
alias cargo-fmt-check := fmt-check
fmt-check *cargo_flags:
    cargo fmt --all {{cargo_flags}} -- --check
    cargo fmt --all --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}} -- --check

# Format all code
alias cargo-fmt := fmt
fmt *cargo_flags:
    cargo fmt --all {{cargo_flags}}
    cargo fmt --all --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

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
    && cargo test --manifest-path=rpkg/src/rust/Cargo.toml --no-fail-fast $cargo_flags -- --no-capture $test_args

# Show dependency tree
alias cargo-tree := tree
tree *cargo_flags:
    cargo tree --workspace {{cargo_flags}}
    cargo tree --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Expand macros for rpkg (requires cargo-expand)
alias cargo-expand := expand
expand *cargo_flags:
    cargo expand --lib -p miniextendr-api {{cargo_flags}}
    cargo expand --lib -p miniextendr-macros {{cargo_flags}}
    cargo expand --lib --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Run ./configure and vendor rpkg deps
configure:
    cd rpkg && autoconf && ./configure
    cargo vendor --manifest-path rpkg/src/rust/Cargo.toml rpkg/src/vendor

# Vendor dependencies (workspace-level)
alias cargo-vendor := vendor
vendor:
    cargo vendor \
        --sync=Cargo.toml \
        --sync=miniextendr-api/Cargo.toml \
        --sync=miniextendr-bench/Cargo.toml \
        --sync=miniextendr-macros/Cargo.toml \
        --sync=rpkg/src/rust/Cargo.toml \
        vendor

# Vendor crates.io dependencies for rpkg (into inst/vendor)
# Local crates (miniextendr-api, miniextendr-macros) are handled by:
# - Makevars.in copy_crates (dev builds)
# - bootstrap.R (CRAN tarball builds)
vendor-rpkg:
    cargo vendor --manifest-path rpkg/src/rust/Cargo.toml rpkg/inst/vendor

# Load and test rpkg with devtools
devtools-test FILTER="":
    if [ -z "{{FILTER}}" ]; then \
      Rscript -e 'devtools::test("rpkg")'; \
    else \
      Rscript -e 'devtools::test("rpkg", filter = "{{FILTER}}")'; \
    fi

# Load rpkg with devtools::load_all
alias devtools-load_all := devtools-load
devtools-load:
    Rscript -e 'devtools::load_all("rpkg")'

# Install rpkg with devtools::install
devtools-install:
    Rscript -e 'devtools::install("rpkg")'

# Build rpkg with devtools::build
devtools-build:
    Rscript -e 'devtools::build("rpkg")'

# Check rpkg with devtools::check
devtools-check:
    Rscript -e 'devtools::check("rpkg")'

# Document rpkg with devtools::document
devtools-document:
    Rscript -e 'devtools::document("rpkg")'

alias rcmdinstall := r-cmd-install
r-cmd-install *args:
    R CMD INSTALL {{args}} rpkg 

# Build R package tarball
alias rcmdbuild := r-cmd-build
r-cmd-build *args:
    R CMD build {{args}} --no-manual --log --debug rpkg

# Run R CMD check on rpkg
alias rcmdcheck := r-cmd-check
r-cmd-check *args:
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

# REVIEW THIS SLOP:
# Build R package tarball
test-r-build:
    pkg="$$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Package"]])')" \
    && ver="$$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Version"]])')" \
    && R CMD build --compression=none rpkg \
    && tarball="$${pkg}_$${ver}.tar" \
    && [ -f "$$tarball" ] || tarball="$${tarball}.gz" \
    && out_dir="rpkg_build/$${pkg}_$${ver}" \
    && mkdir -p "$$out_dir" \
    && Rscript -e 'args <- commandArgs(trailingOnly = TRUE); utils::untar(args[[1]], exdir = args[[2]])' "$$tarball" "$$out_dir"
