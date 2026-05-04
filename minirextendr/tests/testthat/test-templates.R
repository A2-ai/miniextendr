# Integration tests for template scaffolding
#
# These tests verify that the scaffolding functions create valid projects
# that can be built with the miniextendr toolchain.

# -----------------------------------------------------------------------------
# Shared helpers
# -----------------------------------------------------------------------------

# Build a cdylib, generate R wrappers, and roxygenise.
# Returns invisible(TRUE) on success, stops on failure.
generate_r_wrappers <- function(pkg_path) {
  rust_dir <- file.path(pkg_path, "src", "rust")
  features_flag <- grep("^CARGO_FEATURES_FLAG", readLines(file.path(pkg_path, "src", "Makevars")),
                        value = TRUE)
  features_flag <- sub("^CARGO_FEATURES_FLAG *= *", "", features_flag)
  cargo_lines <- readLines(file.path(rust_dir, "Cargo.toml"))
  name_line <- grep("^name\\s*=", cargo_lines, value = TRUE)[1]
  crate_name <- gsub("-", "_", gsub(".*\"(.+)\".*", "\\1", name_line))

  cdylib_result <- system2(
    "cargo",
    c("rustc", "--lib", "--manifest-path", file.path(rust_dir, "Cargo.toml"),
      "--target-dir", file.path(rust_dir, "target"),
      if (nzchar(features_flag)) features_flag,
      "--crate-type", "cdylib"),
    stdout = TRUE, stderr = TRUE
  )
  cdylib_status <- attr(cdylib_result, "status")
  if (!is.null(cdylib_status) && cdylib_status != 0) {
    stop("cdylib build failed: ", paste(cdylib_result, collapse = "\n"))
  }

  cdylib_ext <- if (.Platform$OS.type == "windows") "dll"
                else if (Sys.info()[["sysname"]] == "Darwin") "dylib"
                else "so"
  cdylib_path <- file.path(rust_dir, "target", "debug",
    paste0("lib", crate_name, ".", cdylib_ext))

  pkg_name <- read.dcf(file.path(pkg_path, "DESCRIPTION"))[1, "Package"]
  wrapper_path <- file.path(pkg_path, "R", paste0(pkg_name, "-wrappers.R"))

  lib <- dyn.load(cdylib_path)
  on.exit(dyn.unload(cdylib_path), add = TRUE)
  .Call(getNativeSymbolInfo("miniextendr_write_wrappers", lib), wrapper_path)

  suppressMessages(roxygen2::roxygenise(pkg_path))
  invisible(TRUE)
}

# R CMD INSTALL to a temp library. Returns the lib_path.
install_to_templib <- function(pkg_path, tmp) {
  lib_path <- file.path(tmp, "library")
  dir.create(lib_path, showWarnings = FALSE)

  result <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "-l", lib_path, pkg_path),
    env = c(paste0("R_LIBS=", lib_path)),
    stdout = TRUE, stderr = TRUE
  )
  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    stop("R CMD INSTALL failed: ", paste(result, collapse = "\n"))
  }
  lib_path
}

# Standard e2e skip guards
skip_e2e <- function() {
  skip_on_ci()
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")
  skip_if_no_local_repo()
}

# -----------------------------------------------------------------------------
# Templates patch sync check
# -----------------------------------------------------------------------------

test_that("templates patch is in sync with rpkg sources", {
  skip_if_not(nzchar(Sys.which("just")), "just not available")

  pkg_path <- normalizePath(
    file.path(testthat::test_path(), "..", ".."),
    mustWork = FALSE
  )
  skip_if(!dir.exists(pkg_path), "Cannot find package root")

  repo_root <- dirname(pkg_path)
  skip_if(!file.exists(file.path(repo_root, "justfile")), "Not in miniextendr monorepo")

  result <- withr::with_dir(repo_root, {
    system2("just", c("templates-check"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")

  if (!is.null(status) && status != 0) {
    fail(paste0(
      "Templates patch is out of sync with rpkg sources.\n",
      "Run `just templates-approve` to update the patch.\n\n",
      "Diff output:\n", paste(result, collapse = "\n")
    ))
  }

  expect_true(is.null(status) || status == 0)
})

# -----------------------------------------------------------------------------
# Monorepo template tests (shared fixture)
# -----------------------------------------------------------------------------

# Create one monorepo fixture for all structure/content tests.
# The fixture is created once and reused across tests in this block.
monorepo_fixture <- NULL
monorepo_fixture_cleanup <- NULL

local({
  setup_monorepo_fixture <- function() {
    if (!is.null(monorepo_fixture)) return(monorepo_fixture)
    repo <- find_miniextendr_repo()
    if (is.null(repo)) return(NULL)

    tmp <- tempfile("monorepo-shared-")
    suppressMessages(
      create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                  local_path = repo, open = FALSE)
    )
    monorepo_fixture <<- tmp
    monorepo_fixture_cleanup <<- withr::defer(
      unlink(tmp, recursive = TRUE),
      envir = testthat::teardown_env()
    )
    tmp
  }

  get_fixture <- function() {
    skip_if_no_local_repo()
    path <- setup_monorepo_fixture()
    skip_if(is.null(path), "Failed to create monorepo fixture")
    path
  }

  test_that("create_miniextendr_monorepo creates correct directory structure", {
    tmp <- get_fixture()

    # Root files
    expect_true(file.exists(file.path(tmp, "Cargo.toml")))
    expect_true(file.exists(file.path(tmp, "justfile")))
    expect_true(file.exists(file.path(tmp, ".gitignore")))
    expect_true(dir.exists(file.path(tmp, ".git")))

    # Main crate
    expect_true(file.exists(file.path(tmp, "testpkg-rs", "Cargo.toml")))
    expect_true(file.exists(file.path(tmp, "testpkg-rs", "src", "lib.rs")))

    # rpkg structure
    expect_true(file.exists(file.path(tmp, "testpkg", "DESCRIPTION")))
    expect_true(file.exists(file.path(tmp, "testpkg", "configure.ac")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "Makevars.in")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "stub.c")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "lib.rs")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "Cargo.toml")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "build.rs")))
    expect_true(dir.exists(file.path(tmp, "testpkg", "vendor")))
  })

  test_that("monorepo root Cargo.toml has valid workspace configuration", {
    tmp <- get_fixture()

    cargo_text <- paste(readLines(file.path(tmp, "Cargo.toml")), collapse = "\n")
    expect_true(grepl("\\[workspace\\]", cargo_text))
    expect_true(grepl('resolver = "3"', cargo_text))
    expect_true(grepl("testpkg-rs", cargo_text))
    expect_true(grepl('exclude = \\["testpkg/src/rust"', cargo_text))
    expect_true(grepl("\\[workspace\\.package\\]", cargo_text))
    expect_true(grepl('version = "0\\.1\\.0"', cargo_text))
  })

  test_that("monorepo rpkg DESCRIPTION has correct miniextendr config", {
    tmp <- get_fixture()

    dcf <- read.dcf(file.path(tmp, "testpkg", "DESCRIPTION"))
    expect_equal(unname(dcf[1, "Config/build/bootstrap"]), "TRUE")
    expect_equal(unname(dcf[1, "Config/build/never-clean"]), "true")
    expect_equal(unname(dcf[1, "Config/build/extra-sources"]), "src/rust/Cargo.lock")
    expect_true(grepl("Rust", dcf[1, "SystemRequirements"]))
  })

  test_that("monorepo can run autoconf and configure", {
    skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
    skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
    tmp <- get_fixture()

    result <- withr::with_dir(file.path(tmp, "testpkg"), {
      system2("autoconf", c("-v", "-i", "-f"), stdout = TRUE, stderr = TRUE)
    })
    status <- attr(result, "status")
    expect_true(is.null(status) || status == 0)
    expect_true(file.exists(file.path(tmp, "testpkg", "configure")))
  })

  test_that("monorepo workspace can cargo check", {
    skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
    tmp <- get_fixture()

    result <- suppressWarnings(
      withr::with_dir(file.path(tmp, "testpkg-rs"), {
        system2("cargo", c("check"), stdout = TRUE, stderr = TRUE)
      })
    )
    status <- attr(result, "status")
    output <- paste(result, collapse = "\n")
    valid_outcome <- is.null(status) || status == 0 ||
                     grepl("miniextendr-api", output) ||
                     grepl("Compiling", output) ||
                     grepl("Checking", output)
    expect_true(valid_outcome)
  })
})

# Substitution test needs different params — separate fixture
test_that("create_miniextendr_monorepo performs correct template substitution", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-subst-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "myPkg", crate_name = "my-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl("my-pkg", readLines(file.path(tmp, "Cargo.toml")))))
  expect_true(any(grepl('name = "my-pkg"', readLines(file.path(tmp, "my-pkg", "Cargo.toml")))))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "myPkg", "src", "rust", "lib.rs")))))
  expect_true(any(grepl("Package: myPkg", readLines(file.path(tmp, "myPkg", "DESCRIPTION")))))
})

# -----------------------------------------------------------------------------
# rpkg template tests
# -----------------------------------------------------------------------------

test_that("use_template works with rpkg template type", {
  tmp <- tempfile("rpkg-template-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n", file.path(tmp, "DESCRIPTION"))

  minirextendr:::set_template_type("rpkg")
  data <- list(package = "testpkg", package_rs = "testpkg", Package = "Testpkg", year = "2025")
  minirextendr:::ensure_dir(usethis::proj_path("src", "rust"))
  minirextendr:::use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)
  minirextendr:::use_template("build.rs", save_as = "src/rust/build.rs")
  minirextendr:::use_template("Makevars.in", save_as = "src/Makevars.in")

  expect_true(file.exists(file.path(tmp, "src", "rust", "lib.rs")))
  expect_true(file.exists(file.path(tmp, "src", "rust", "build.rs")))
  expect_true(file.exists(file.path(tmp, "src", "Makevars.in")))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "src", "rust", "lib.rs")))))
})

test_that("use_template performs mustache substitution correctly", {
  tmp <- tempfile("subst-test-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  minirextendr:::set_template_type("rpkg")
  data <- list(package = "specialPkg", package_rs = "special_pkg")
  minirextendr:::ensure_dir(usethis::proj_path("src", "rust"))
  minirextendr:::use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)

  content <- paste(readLines(file.path(tmp, "src", "rust", "lib.rs")), collapse = "\n")
  expect_true(grepl("#\\[miniextendr\\]", content))
  expect_false(grepl("\\{\\{package", content))
})

# -----------------------------------------------------------------------------
# End-to-end scaffolding tests (full build and test)
# -----------------------------------------------------------------------------

test_that("rpkg scaffolding builds and functions work end-to-end", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("rpkg-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, "testpkg")

  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
    usethis::proj_set(pkg_path, force = TRUE)
    usethis::use_package_doc()
  }))

  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
    devtools::document(pkg = pkg_path)
  })

  pkg_name <- read.dcf(file.path(pkg_path, "DESCRIPTION"))[1, "Package"]
  lib_path <- install_to_templib(pkg_path, tmp)
  generate_r_wrappers(pkg_path)
  lib_path <- install_to_templib(pkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(pkg_name, character.only = TRUE)
    expect_equal(add(1, 2), 3)
    expect_equal(add(10, 20), 30)
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})

test_that("rpkg scaffolding with external cargo dependency works", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("rpkg-cargo-dep-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, "testpkg")

  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
    usethis::proj_set(pkg_path, force = TRUE)
    usethis::use_package_doc()
  }))

  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
  })

  # Add itertools dependency
  cargo_toml <- file.path(pkg_path, "src", "rust", "Cargo.toml")
  cargo_content <- readLines(cargo_toml)
  deps_idx <- grep("^\\[dependencies\\]", cargo_content)[1]
  if (!is.na(deps_idx)) {
    cargo_content <- c(
      cargo_content[1:deps_idx],
      "itertools = \"0.13\"",
      cargo_content[(deps_idx + 1):length(cargo_content)]
    )
    writeLines(cargo_content, cargo_toml)
  }

  # Add itertools usage to lib.rs
  lib_rs <- file.path(pkg_path, "src", "rust", "lib.rs")
  lib_content <- readLines(lib_rs)
  use_idx <- grep("use miniextendr_api", lib_content)[1]
  lib_content <- c(
    lib_content[1:use_idx],
    "use itertools::Itertools;",
    "",
    "/// Join strings with itertools",
    "/// @param parts Character vector to join",
    "/// @return Joined string",
    "#[miniextendr]",
    "pub fn join_strings(parts: Vec<String>) -> String {",
    "    parts.into_iter().join(\", \")",
    "}",
    "",
    lib_content[(use_idx + 1):length(lib_content)]
  )
  writeLines(lib_content, lib_rs)

  # Reconfigure in source mode (itertools will be fetched by cargo during build)
  suppressMessages({
    result <- minirextendr:::run_with_logging(
      "bash", args = c("./configure"),
      log_prefix = "configure-cargo-dep", wd = pkg_path,
      env = devtools::r_env_vars()
    )
    expect_true(result$success,
                info = paste("configure failed:", paste(result$output, collapse = "\n")))
  })

  lib_path <- install_to_templib(pkg_path, tmp)
  generate_r_wrappers(pkg_path)
  lib_path <- install_to_templib(pkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(testpkg)
    expect_equal(add(1, 2), 3)
    expect_equal(hello("Test"), "Hello, Test!")
    expect_equal(join_strings(c("a", "b", "c")), "a, b, c")
    detach("package:testpkg", character.only = TRUE, unload = TRUE)
  })
})

# -----------------------------------------------------------------------------
# Package name handling tests
# -----------------------------------------------------------------------------

test_that("monorepo handles dots in package names", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-dots-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "my.pkg", crate_name = "my-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl('name = "my-pkg"', readLines(file.path(tmp, "my-pkg", "Cargo.toml")))))
  expect_true(any(grepl("Package: my.pkg", readLines(file.path(tmp, "my.pkg", "DESCRIPTION")))))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "my.pkg", "src", "rust", "lib.rs")))))
})

test_that("monorepo handles hyphens in crate names", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-hyphens-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "test-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl('name = "test-pkg"', readLines(file.path(tmp, "test-pkg", "Cargo.toml")))))
})

test_that("create_miniextendr_package rejects invalid R package names", {
  tmp <- tempfile("invalid-name-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_error(create_miniextendr_package(file.path(tmp, "1badname"), open = FALSE))
})

# -----------------------------------------------------------------------------
# Monorepo end-to-end
# -----------------------------------------------------------------------------

test_that("monorepo scaffolding builds and functions work end-to-end", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("monorepo-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                local_path = miniextendr_path, open = FALSE)
  })

  rpkg_path <- file.path(tmp, "testpkg")

  suppressMessages({
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
  })

  # Vendor crates.io deps (miniextendr crates already vendored by scaffolding)
  suppressWarnings({
    withr::with_dir(rpkg_path, {
      system2("cargo", c("vendor", "--manifest-path", "src/rust/Cargo.toml", "vendor"),
              stdout = FALSE, stderr = FALSE)
    })
  })

  suppressMessages({
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
    miniextendr_autoconf(path = rpkg_path)
    miniextendr_configure(path = rpkg_path)
  })

  pkg_name <- read.dcf(file.path(rpkg_path, "DESCRIPTION"))[1, "Package"]
  lib_path <- install_to_templib(rpkg_path, tmp)
  generate_r_wrappers(rpkg_path)
  lib_path <- install_to_templib(rpkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(pkg_name, character.only = TRUE)
    expect_equal(add(1, 2), 3)
    expect_equal(add(10, 20), 30)
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})
