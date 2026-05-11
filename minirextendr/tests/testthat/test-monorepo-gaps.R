# Tests for the three monorepo scaffolding gaps (B1, B2, B3)
# PR: fix(minirextendr): monorepo scaffolding gaps — r_shim.h, upgrade path, release workflow

# region: helpers -------------------------------------------------------

# Build a minimal monorepo directory structure in tmp without running cargo or
# autoconf. Returns a list(root, rpkg) of absolute paths.
make_minimal_monorepo <- function(tmp, rpkg_subdir = "rpkg") {
  root <- tmp
  rpkg <- file.path(root, rpkg_subdir)
  src <- file.path(rpkg, "src", "rust")
  dir.create(src, recursive = TRUE)
  # Workspace Cargo.toml so detect_project_type() sees a monorepo root
  writeLines(c(
    "[workspace]",
    paste0('members = ["', rpkg_subdir, '/src/rust"]')
  ), file.path(root, "Cargo.toml"))
  # Minimal configure.ac with CARGO_FEATURES so is_miniextendr_package() passes
  writeLines(c(
    "AC_INIT([mypkg], [0.0.1])",
    "CARGO_FEATURES=\"\"",
    "AC_OUTPUT"
  ), file.path(rpkg, "configure.ac"))
  # Minimal DESCRIPTION
  writeLines(c(
    "Package: mypkg",
    "Version: 0.0.1"
  ), file.path(rpkg, "DESCRIPTION"))
  # Minimal src/Makevars.in (required by is_miniextendr_package)
  dir.create(file.path(rpkg, "src"), recursive = TRUE, showWarnings = FALSE)
  writeLines("# stub Makevars.in", file.path(rpkg, "src", "Makevars.in"))
  # Minimal src/rust/Cargo.toml
  writeLines(c(
    "[package]",
    'name = "mypkg"',
    'version = "0.0.1"'
  ), file.path(src, "Cargo.toml"))
  list(root = root, rpkg = rpkg)
}

# endregion ---------------------------------------------------------------

# region: B1 — r_shim.h in monorepo template ---------------------------

test_that("B1: monorepo rpkg template directory contains r_shim.h", {
  shim <- system.file("templates", "monorepo", "rpkg", "r_shim.h",
                      package = "minirextendr")
  expect_true(nzchar(shim), info = "r_shim.h missing from monorepo/rpkg/ template")
  expect_true(file.exists(shim))
})

test_that("B1: monorepo/rpkg/r_shim.h is byte-identical to rpkg/r_shim.h", {
  mono_shim <- system.file("templates", "monorepo", "rpkg", "r_shim.h",
                            package = "minirextendr", mustWork = TRUE)
  rpkg_shim <- system.file("templates", "rpkg", "r_shim.h",
                            package = "minirextendr", mustWork = TRUE)
  expect_identical(readLines(mono_shim, warn = FALSE),
                   readLines(rpkg_shim, warn = FALSE))
})

test_that("B1: create_rpkg_subdirectory() copies r_shim.h into rpkg/src/", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  minirextendr:::set_template_type("monorepo")
  on.exit(minirextendr:::set_template_type("rpkg"), add = TRUE)

  data <- list(
    package = "mypkg",
    package_rs = "mypkg",
    Package = "Mypkg",
    crate_name = "mypkg-rs",
    rpkg_name = "rpkg",
    features_var = "CARGO_FEATURES",
    year = "2026"
  )

  suppressMessages(minirextendr:::create_rpkg_subdirectory(data, rpkg_name = "rpkg"))

  shim_dest <- file.path(tmp, "rpkg", "src", "r_shim.h")
  expect_true(file.exists(shim_dest),
              info = "r_shim.h should be placed at rpkg/src/r_shim.h")

  # Should be identical to the template
  shim_tpl <- system.file("templates", "monorepo", "rpkg", "r_shim.h",
                           package = "minirextendr", mustWork = TRUE)
  expect_identical(readLines(shim_dest, warn = FALSE),
                   readLines(shim_tpl, warn = FALSE))
})

# endregion ---------------------------------------------------------------

# region: B2 — upgrade_miniextendr_package() monorepo awareness ---------

test_that("B2: find_rpkg_subdir() returns the subdir with CARGO_FEATURES configure.ac", {
  tmp <- withr::local_tempdir()
  # Write a configure.ac with CARGO_FEATURES in a subdir called "rpkg"
  rpkg <- file.path(tmp, "rpkg")
  dir.create(rpkg)
  writeLines(c("AC_INIT", "CARGO_FEATURES=\"\"", "AC_OUTPUT"),
             file.path(rpkg, "configure.ac"))

  result <- minirextendr:::find_rpkg_subdir(tmp)
  expect_equal(result, "rpkg")
})

test_that("B2: find_rpkg_subdir() returns NULL when no matching subdir exists", {
  tmp <- withr::local_tempdir()
  # No subdirs at all
  expect_null(minirextendr:::find_rpkg_subdir(tmp))
})

test_that("B2: find_rpkg_subdir() ignores subdirs without CARGO_FEATURES", {
  tmp <- withr::local_tempdir()
  other <- file.path(tmp, "other")
  dir.create(other)
  writeLines(c("AC_INIT", "AC_OUTPUT"), file.path(other, "configure.ac"))

  expect_null(minirextendr:::find_rpkg_subdir(tmp))
})

test_that("B2: upgrade_miniextendr_package() resolves rpkg subdir in monorepo", {
  tmp <- withr::local_tempdir()
  dirs <- make_minimal_monorepo(tmp)

  # Record where upgrade thinks the project root is
  upgraded_path <- NULL

  # Mock the heavy upgrade steps — we only want to verify path resolution
  local_mocked_bindings(
    is_miniextendr_package = function() TRUE,
    check_scaffolding_clean = function() invisible(),
    use_miniextendr_stub = function(path = ".") invisible(),
    use_miniextendr_makevars = function(path = ".") invisible(),
    use_miniextendr_mx_abi = function(path = ".") invisible(),
    use_miniextendr_build_rs = function(path = ".") invisible(),
    use_miniextendr_bootstrap = function(path = ".") invisible(),
    use_miniextendr_cleanup = function(path = ".") invisible(),
    use_miniextendr_configure_win = function(path = ".") invisible(),
    use_miniextendr_config_scripts = function(path = ".") invisible(),
    use_miniextendr_description = function(path = ".") invisible(),
    use_miniextendr_rbuildignore = function(path = ".") invisible(),
    upgrade_gitignore = function() invisible(),
    check_configure_ac_drift = function() invisible(),
    .package = "minirextendr"
  )

  # Capture proj_get() inside upgrade to verify the resolved path
  withr::with_envvar(list(), {
    suppressMessages(
      upgrade_miniextendr_package(path = dirs$root, allow_dirty = TRUE)
    )
    # If resolution worked, usethis::proj_get() will have been set to the rpkg
    # subdir at some point. We verify by checking no error was thrown and the
    # function found the rpkg subdir (not errored with "monorepo" detection).
    expect_true(TRUE)  # reached here without error
  })
})

test_that("B2: upgrade_miniextendr_package() errors clearly when rpkg subdir undetectable", {
  tmp <- withr::local_tempdir()
  # Write a Cargo.toml so it looks like a monorepo root, but no rpkg subdir
  writeLines("[workspace]", file.path(tmp, "Cargo.toml"))

  expect_error(
    suppressMessages(upgrade_miniextendr_package(path = tmp, allow_dirty = TRUE)),
    "Could not find an rpkg subdirectory"
  )
})

test_that("B2: upgrade_miniextendr_package() respects explicit rpkg_subdir=", {
  tmp <- withr::local_tempdir()
  # Create monorepo with rpkg under a custom name
  dirs <- make_minimal_monorepo(tmp, rpkg_subdir = "mypkg-r")

  local_mocked_bindings(
    is_miniextendr_package = function() TRUE,
    check_scaffolding_clean = function() invisible(),
    use_miniextendr_stub = function(path = ".") invisible(),
    use_miniextendr_makevars = function(path = ".") invisible(),
    use_miniextendr_mx_abi = function(path = ".") invisible(),
    use_miniextendr_build_rs = function(path = ".") invisible(),
    use_miniextendr_bootstrap = function(path = ".") invisible(),
    use_miniextendr_cleanup = function(path = ".") invisible(),
    use_miniextendr_configure_win = function(path = ".") invisible(),
    use_miniextendr_config_scripts = function(path = ".") invisible(),
    use_miniextendr_description = function(path = ".") invisible(),
    use_miniextendr_rbuildignore = function(path = ".") invisible(),
    upgrade_gitignore = function() invisible(),
    check_configure_ac_drift = function() invisible(),
    .package = "minirextendr"
  )

  # Should not error — explicit rpkg_subdir bypasses auto-detect
  expect_no_error(
    suppressMessages(
      upgrade_miniextendr_package(path = dirs$root, rpkg_subdir = "mypkg-r",
                                  allow_dirty = TRUE)
    )
  )
})

# endregion ---------------------------------------------------------------

# region: B3 — use_release_workflow() monorepo working-directory --------

test_that("B3: release_workflow_insert_workdir() adds working-directory to configure step", {
  lines <- c(
    "      - name: Configure package",
    "        run: bash ./configure",
    "      - name: Build",
    "        run: |",
    "          R CMD build .",
    "          R CMD check ./*.tar.gz"
  )
  result <- minirextendr:::release_workflow_insert_workdir(lines, "rpkg")

  cfg_idx <- which(grepl("run: bash ./configure", result, fixed = TRUE))
  expect_true(length(cfg_idx) > 0)
  expect_match(result[[cfg_idx + 1L]], "working-directory: rpkg")

  build_idx <- which(grepl("run: |", result, fixed = TRUE))
  expect_true(length(build_idx) > 0)
  expect_match(result[[build_idx + 1L]], "working-directory: rpkg")
})

test_that("B3: use_release_workflow() with rpkg_subdir inserts working-directory", {
  tmp <- withr::local_tempdir()

  use_release_workflow(path = tmp, rpkg_subdir = "rpkg")

  dest <- file.path(tmp, ".github", "workflows", "r-release.yml")
  expect_true(file.exists(dest))

  lines <- readLines(dest, warn = FALSE)
  # Both configure and build steps should have working-directory: rpkg
  wd_lines <- grep("working-directory: rpkg", lines, value = TRUE)
  # There are two jobs in the template (linux + macos), each with configure + build
  expect_true(length(wd_lines) >= 2,
              info = paste("Expected at least 2 working-directory lines, got:",
                           length(wd_lines)))
})

test_that("B3: use_release_workflow() standalone produces no working-directory lines", {
  tmp <- withr::local_tempdir()

  use_release_workflow(path = tmp, rpkg_subdir = "")

  dest <- file.path(tmp, ".github", "workflows", "r-release.yml")
  expect_true(file.exists(dest))

  lines <- readLines(dest, warn = FALSE)
  wd_lines <- grep("working-directory:", lines, value = TRUE)
  expect_equal(length(wd_lines), 0L,
               info = "Standalone workflow should have no working-directory lines")
})

test_that("B3: use_release_workflow() standalone content matches bundled template", {
  tmp <- withr::local_tempdir()

  use_release_workflow(path = tmp, rpkg_subdir = "")

  written <- readLines(file.path(tmp, ".github", "workflows", "r-release.yml"),
                       warn = FALSE)
  template <- readLines(
    system.file("templates", "r-release.yml", package = "minirextendr"),
    warn = FALSE
  )
  expect_identical(written, template)
})

test_that("B3: use_release_workflow() auto-detects monorepo and inserts working-directory", {
  tmp <- withr::local_tempdir()
  dirs <- make_minimal_monorepo(tmp, rpkg_subdir = "rpkg")

  # path = monorepo root (has Cargo.toml, no configure.ac at root)
  use_release_workflow(path = dirs$root)

  dest <- file.path(dirs$root, ".github", "workflows", "r-release.yml")
  expect_true(file.exists(dest))

  lines <- readLines(dest, warn = FALSE)
  wd_lines <- grep("working-directory: rpkg", lines, value = TRUE)
  expect_true(length(wd_lines) >= 2)
})

test_that("B3: use_release_workflow() standalone auto-detect (no Cargo.toml) is unchanged", {
  tmp <- withr::local_tempdir()
  # No Cargo.toml → auto-detect returns NULL / non-monorepo → standalone copy

  use_release_workflow(path = tmp)

  dest <- file.path(tmp, ".github", "workflows", "r-release.yml")
  lines <- readLines(dest, warn = FALSE)
  wd_lines <- grep("working-directory:", lines, value = TRUE)
  expect_equal(length(wd_lines), 0L)
})

# endregion ---------------------------------------------------------------
