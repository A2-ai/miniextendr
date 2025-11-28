# https://just.systems

default:
    @just --list

# Clean build artifacts
clean *cargo_flags:
    cargo clean -p miniextendr-api {{cargo_flags}}
    cargo clean -p miniextendr-macros {{cargo_flags}}
    cargo clean --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check all crates``
check *cargo_flags:
    cargo check --workspace {{cargo_flags}}
    cargo check --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

alias cargo-check := check

# Build all crates
build *cargo_flags:
    cargo build --workspace {{cargo_flags}}
    cargo build --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Run clippy on all crates
clippy *cargo_flags:
    cargo clippy --workspace {{cargo_flags}}
    cargo clippy --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check documentation builds
doc-check *cargo_flags:
    cargo doc --no-deps --document-private-items --workspace {{cargo_flags}}
    cargo doc --no-deps --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Build and open documentation
doc *cargo_flags:
    cargo doc --document-private-items --workspace {{cargo_flags}}
    cargo doc --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml --open {{cargo_flags}}

alias cargo-doc := doc

# Check formatting
fmt-check *cargo_flags:
    cargo fmt --all {{cargo_flags}} -- --check
    cargo fmt --all --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}} -- --check

# Format all code
fmt *cargo_flags:
    cargo fmt --all {{cargo_flags}}
    cargo fmt --all --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

alias cargo-fmt := fmt

# Run tests
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
tree *cargo_flags:
    cargo tree --workspace {{cargo_flags}}
    cargo tree --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Expand macros for rpkg (requires cargo-expand)
expand *cargo_flags:
    cargo expand --lib -p miniextendr-api {{cargo_flags}}
    cargo expand --lib -p miniextendr-macros {{cargo_flags}}
    cargo expand --lib --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

alias cargo-expand := expand

# Run configure for rpkg
configure:
    cd rpkg && autoconf && ./configure

# Vendor dependencies
vendor:
    cargo vendor \
        --sync=Cargo.toml \
        --sync=miniextendr-api/Cargo.toml \
        --sync=miniextendr-macros/Cargo.toml \
        --sync=rpkg/src/rust/Cargo.toml \
        vendor

# Load and test rpkg with devtools
devtools-test FILTER="":
    @cd rpkg && \
    if [ "{{FILTER}}" = "" ]; then \
      Rscript -e 'devtools::test()'; \
    else \
      Rscript -e 'devtools::test(filter = "{{FILTER}}")'; \
    fi

# Load rpkg with devtools::load_all
load:
    cd rpkg && Rscript -e 'devtools::load_all()'

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
    && cd rpkg \
    && Rscript -e "rcmdcheck::rcmdcheck(args = c('--as-cran','--no-manual'), error_on = '${ERROR_ON}', check_dir = ${CHECK_DIR_ARG})"

# Build R package tarball
test-r-build:
    R CMD build --compression=none rpkg
    tar -xvzf rpkg_0.0.0.9000.tar -C "$(mkdir -p rpkg_build && echo rpkg_build)"

# Quick smoke test of rpkg functions
smoke-test:
    cd rpkg && Rscript -e ' \
      devtools::load_all(".", quiet = TRUE); \
      cat("add(2L, 3L) =", add(2L, 3L), "\n"); \
      cat("drop_message_on_success() =", drop_message_on_success(), "\n"); \
      tryCatch(add_panic(1L, 2L), error = function(e) cat("Caught panic:", conditionMessage(e), "\n")); \
      cat("All smoke tests passed!\n") \
    '
