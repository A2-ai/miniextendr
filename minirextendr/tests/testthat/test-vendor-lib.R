# Tests for vendor-lib functions

# =============================================================================
# Helper: create minimal project structure for testing
# =============================================================================

make_test_project <- function() {
  tmp <- tempfile("vendor-lib-")
  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)

  # Minimal DESCRIPTION
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.1.0\n", file.path(tmp, "DESCRIPTION"))
  writeLines("", file.path(tmp, "NAMESPACE"))

  # Minimal Cargo.toml
  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    '[package]',
    'name = "testpkg"',
    'version = "0.1.0"',
    'edition = "2024"',
    '',
    '[lib]',
    'crate-type = ["staticlib"]',
    '',
    '[dependencies]',
    'miniextendr-api = { git = "https://github.com/CGMossa/miniextendr" }',
    '',
    '[features]',
    'default = []'
  ), file.path(cargo_dir, "Cargo.toml"))

  # Minimal configure.ac (matching template structure)
  writeLines(c(
    'AC_INIT([testpkg], [1.0])',
    'AC_CONFIG_AUX_DIR([tools])',
    ': ${NOT_CRAN:=false}',
    'AC_SUBST([NOT_CRAN])',
    'CARGO_CMD="$CARGO"',
    'AC_SUBST([CARGO_CMD])',
    'abs_top_srcdir="$(cd "$srcdir" && pwd)"',
    'VENDOR_OUT="$abs_top_srcdir/vendor"',
    'AC_SUBST([VENDOR_OUT])',
    'VENDOR_OUT_CARGO="$abs_top_srcdir/vendor"',
    'AC_SUBST([VENDOR_OUT_CARGO])',
    'CARGO_FEATURES=""',
    '',
    'AC_CONFIG_FILES([src/Makevars:src/Makevars.in])',
    '',
    'AC_CONFIG_COMMANDS([dev-cargo-config],',
    '[',
    '  RPKG_CFG="src/rust/.cargo/config.toml"',
    '',
    '  if test "$NOT_CRAN" = "true"; then',
    '    if test -f "$RPKG_CFG"; then',
    '      rm "$RPKG_CFG"',
    '      echo "configure: removed cargo config (dev mode - using git deps)"',
    '    fi',
    '  fi',
    '],',
    '[NOT_CRAN="$NOT_CRAN"])',
    '',
    'AC_CONFIG_COMMANDS([cargo-vendor],',
    '[',
    '  echo "configure: vendor step"',
    '],',
    '[NOT_CRAN="$NOT_CRAN"])',
    '',
    'AC_CONFIG_COMMANDS([post-vendor],',
    '[',
    '  touch src/rust/Cargo.toml',
    '],',
    '[ABS_RPKG_SRC="$abs_rpkg_src"])',
    '',
    'AC_OUTPUT'
  ), file.path(tmp, "configure.ac"))

  # stub.c + Makevars.in (required by is_miniextendr_package)
  src_dir <- file.path(tmp, "src")
  writeLines("// placeholder", file.path(src_dir, "stub.c"))
  writeLines("# placeholder", file.path(src_dir, "Makevars.in"))

  tmp
}

# =============================================================================
# add_cargo_dependency tests
# =============================================================================

test_that("add_cargo_dependency adds to [dependencies]", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- add_cargo_dependency("dvs", "*")

  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  expect_true(any(grepl('^dvs = "\\*"', cargo)))
  expect_true(result)
})

test_that("add_cargo_dependency is idempotent", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_cargo_dependency("dvs", "*")
  result <- add_cargo_dependency("dvs", "*")

  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  matches <- grep('^dvs = "\\*"', cargo)
  expect_length(matches, 1)
  expect_false(result)
})

test_that("add_cargo_dependency inserts before next section", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_cargo_dependency("dvs", "0.1.0")

  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  dep_line <- grep('^dvs = "0.1.0"', cargo)
  features_line <- grep("^\\[features\\]", cargo)

  # dvs dependency should appear before [features] section

  expect_true(dep_line < features_line)
})

# =============================================================================
# add_cargo_patch tests
# =============================================================================

test_that("add_cargo_patch creates [patch.crates-io] if missing", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- add_cargo_patch("dvs", "../../../dvs")

  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  expect_true(any(grepl("^\\[patch\\.crates-io\\]", cargo)))
  expect_true(any(grepl('^dvs = \\{ path = "\\.\\./\\.\\./\\.\\./dvs" \\}', cargo)))
  expect_true(result)
})

test_that("add_cargo_patch adds to existing [patch.crates-io]", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Add initial patch section
  cargo_path <- file.path(tmp, "src", "rust", "Cargo.toml")
  lines <- readLines(cargo_path, warn = FALSE)
  lines <- c(lines, "", "[patch.crates-io]", 'other-crate = { path = "../other" }')
  writeLines(lines, cargo_path)

  add_cargo_patch("dvs", "../../../dvs")

  cargo <- readLines(cargo_path, warn = FALSE)
  expect_true(any(grepl('^dvs = \\{ path = "\\.\\./\\.\\./\\.\\./dvs" \\}', cargo)))
  expect_true(any(grepl('^other-crate = ', cargo)))
})

test_that("add_cargo_patch is idempotent", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_cargo_patch("dvs", "../../../dvs")
  result <- add_cargo_patch("dvs", "../../../dvs")

  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  matches <- grep('^dvs = \\{ path = ', cargo)
  expect_length(matches, 1)
  expect_false(result)
})

# =============================================================================
# add_vendor_lib_to_configure_ac tests
# =============================================================================

test_that("add_vendor_lib_to_configure_ac inserts VENDOR_LIB variable", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_vendor_lib_to_configure_ac("dvs", "../../../dvs")

  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  expect_true(any(grepl('VENDOR_LIB=.*dvs-lib\\.tar\\.gz', conf)))
  expect_true(any(grepl("AC_SUBST\\(\\[VENDOR_LIB\\]\\)", conf)))
})

test_that("add_vendor_lib_to_configure_ac inserts vendor-lib block", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_vendor_lib_to_configure_ac("dvs", "../../../dvs")

  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  expect_true(any(grepl("AC_CONFIG_COMMANDS\\(\\[vendor-lib-dvs\\]", conf)))
  expect_true(any(grepl('_lib_crate="dvs"', conf)))
  expect_true(any(grepl('_lib_dev_path="\\.\\./\\.\\./\\.\\./dvs"', conf)))
})

test_that("add_vendor_lib_to_configure_ac vendor-lib block is before post-vendor", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_vendor_lib_to_configure_ac("dvs", "../../../dvs")

  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  vendor_lib_line <- grep("AC_CONFIG_COMMANDS\\(\\[vendor-lib-dvs\\]", conf)
  post_vendor_line <- grep("AC_CONFIG_COMMANDS\\(\\[post-vendor\\]", conf)

  expect_true(length(vendor_lib_line) > 0)
  expect_true(length(post_vendor_line) > 0)
  expect_true(vendor_lib_line[1] < post_vendor_line[1])
})

test_that("add_vendor_lib_to_configure_ac updates dev-cargo-config", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_vendor_lib_to_configure_ac("dvs", "../../../dvs")

  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  # Should have the vendor-lib-only check
  expect_true(any(grepl("vendor-lib-only", conf)))
  # Should pass abs_rpkg_dir to dev-cargo-config
  expect_true(any(grepl('abs_rpkg_dir="\\$abs_top_srcdir"', conf)))
})

test_that("add_vendor_lib_to_configure_ac is idempotent", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  add_vendor_lib_to_configure_ac("dvs", "../../../dvs")
  result <- add_vendor_lib_to_configure_ac("dvs", "../../../dvs")

  expect_false(result)

  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  vendor_lib_blocks <- grep("AC_CONFIG_COMMANDS\\(\\[vendor-lib-dvs\\]", conf)
  expect_length(vendor_lib_blocks, 1)
})

# =============================================================================
# use_vendor_lib end-to-end test
# =============================================================================

test_that("use_vendor_lib end-to-end", {
  tmp <- make_test_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Initialize git so usethis::use_git_ignore works
  withr::local_options(usethis.quiet = TRUE)
  system2("git", c("init", tmp), stdout = FALSE, stderr = FALSE)

  use_vendor_lib("dvs", "*", "../../../dvs")

  # Check Cargo.toml has dependency
  cargo <- readLines(file.path(tmp, "src", "rust", "Cargo.toml"), warn = FALSE)
  expect_true(any(grepl('^dvs = "\\*"', cargo)))
  expect_true(any(grepl("^\\[patch\\.crates-io\\]", cargo)))
  expect_true(any(grepl('^dvs = \\{ path = "\\.\\./\\.\\./\\.\\./dvs" \\}', cargo)))

  # Check configure.ac has VENDOR_LIB and vendor-lib block
  conf <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  expect_true(any(grepl("VENDOR_LIB", conf)))
  expect_true(any(grepl("vendor-lib-dvs", conf)))

  # Check .gitignore has tarball pattern
  gitignore <- readLines(file.path(tmp, ".gitignore"), warn = FALSE)
  expect_true(any(grepl("inst/dvs-lib\\.tar\\.gz", gitignore)))
})

test_that("use_vendor_lib rejects non-miniextendr packages", {
  tmp <- tempfile("not-mx-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))
  writeLines("", file.path(tmp, "NAMESPACE"))

  expect_error(use_vendor_lib("dvs", "*", "../dvs"), "Not a miniextendr package")
})
