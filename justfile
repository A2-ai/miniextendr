# https://just.systems

default:
    @just --list

clean:
    -just configure
    -just cargo-clean
    -cd rpkg && NOT_CRAN=false ./cleanup

# Clean build artifacts
cargo-clean *cargo_flags:
    cargo clean -p miniextendr-api {{cargo_flags}}
    cargo clean -p miniextendr-macros {{cargo_flags}}
    cargo clean -p miniextendr-bench {{cargo_flags}}
    cargo clean --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check all crates
alias cargo-check := check
check *cargo_flags:
    cargo check --benches --tests --examples --workspace {{cargo_flags}}
    cargo check --benches --tests --examples -p miniextendr-bench {{cargo_flags}}
    cargo check --benches --tests --examples --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Build all crates
alias cargo-build := build
build *cargo_flags:
    cargo build --benches --tests --examples --workspace {{cargo_flags}}
    cargo build --benches --tests --examples -p miniextendr-bench {{cargo_flags}}
    cargo build --benches --tests --examples --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Run clippy on all crates
alias cargo-clippy := clippy
clippy *cargo_flags:
    cargo clippy --benches --tests --examples --workspace {{cargo_flags}}
    cargo clippy --benches --tests --examples -p miniextendr-bench {{cargo_flags}}
    cargo clippy --benches --tests --examples --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Check documentation builds
alias cargo-doc-check := doc-check
doc-check *cargo_flags: configure
    cargo doc --no-deps --document-private-items -p miniextendr-bench {{cargo_flags}}
    cargo doc --no-deps --document-private-items --workspace {{cargo_flags}}
    cargo doc --no-deps --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Build and open documentation
alias cargo-doc := doc
doc *cargo_flags: configure
    cargo doc --document-private-items -p miniextendr-bench {{cargo_flags}}
    cargo doc --document-private-items --workspace {{cargo_flags}}
    cargo doc --document-private-items --manifest-path=rpkg/src/rust/Cargo.toml --open {{cargo_flags}}

# Check formatting
alias cargo-fmt-check := fmt-check
fmt-check *cargo_flags:
    cargo fmt --all {{cargo_flags}} -- --check
    cargo fmt -p miniextendr-bench {{cargo_flags}} -- --check
    cargo fmt --all --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}} -- --check

# Format all code
alias cargo-fmt := fmt
fmt *cargo_flags:
    cargo fmt --all {{cargo_flags}}
    cargo fmt -p miniextendr-bench {{cargo_flags}}
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
    cargo tree -p miniextendr-bench {{cargo_flags}}
    cargo tree --workspace {{cargo_flags}}
    cargo tree --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

# Expand macros for rpkg (requires cargo-expand)
alias cargo-expand := expand
expand *cargo_flags:
    cargo expand --lib -p miniextendr-api {{cargo_flags}}
    cargo expand --lib -p miniextendr-macros {{cargo_flags}}
    cargo expand --lib --manifest-path=rpkg/src/rust/Cargo.toml {{cargo_flags}}

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

# Templates / drift check
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

    # Only include files where rpkg is the source of truth.
    # Templates with @PLACEHOLDER@ markers (document.rs.in, entrypoint.c.in)
    # are NOT compared - they are the source, rpkg has expanded versions.
    cat <<'EOF'
    # rel	src
    # Build system files (rpkg is source of truth)
    Makevars.in	rpkg/src/Makevars.in
    configure.ac	rpkg/configure.ac
    build.rs	rpkg/src/rust/build.rs
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
    (cd "$tmp" && diff -ruN a b) > "{{patch_file}}" || true
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
