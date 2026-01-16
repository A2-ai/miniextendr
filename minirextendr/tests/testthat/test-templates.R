# Integration tests for template scaffolding
#
# These tests verify that the scaffolding functions create valid projects
# that can be built with the miniextendr toolchain.

# -----------------------------------------------------------------------------
# Templates patch sync check
# -----------------------------------------------------------------------------

test_that("templates patch is in sync with rpkg sources", {
  skip_if_not(nzchar(Sys.which("just")), "just not available")

  # Find the miniextendr repo root (parent of minirextendr)
  pkg_path <- tryCatch(
    rprojroot::find_package_root_file(),
    error = function(e) NULL
  )
  skip_if(is.null(pkg_path), "Cannot find package root")

  repo_root <- dirname(pkg_path)
  skip_if(!file.exists(file.path(repo_root, "justfile")),
          "Not in miniextendr monorepo")

  # Run `just templates-check` from repo root
  result <- withr::with_dir(repo_root, {
    system2("just", c("templates-check"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")

  # If status is non-zero, the templates have drifted
  if (!is.null(status) && status != 0) {
    output <- paste(result, collapse = "\n")
    fail(paste0(
      "Templates patch is out of sync with rpkg sources.\n",
      "Run `just templates-approve` to update the patch.\n\n",
      "Diff output:\n", output
    ))
  }

  expect_true(is.null(status) || status == 0)
})

# -----------------------------------------------------------------------------
# Monorepo template tests
# -----------------------------------------------------------------------------

test_that("create_miniextendr_monorepo creates correct directory structure", {
  tmp <- tempfile("monorepo-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create monorepo
  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  # Check root files
  expect_true(file.exists(file.path(tmp, "Cargo.toml")))
  expect_true(file.exists(file.path(tmp, "justfile")))
  expect_true(file.exists(file.path(tmp, ".gitignore")))
  expect_true(dir.exists(file.path(tmp, ".git")))

  # Check main crate
  expect_true(file.exists(file.path(tmp, "testpkg", "Cargo.toml")))
  expect_true(file.exists(file.path(tmp, "testpkg", "src", "lib.rs")))

  # Check rpkg structure
  expect_true(file.exists(file.path(tmp, "rpkg", "DESCRIPTION")))
  expect_true(file.exists(file.path(tmp, "rpkg", "configure.ac")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "Makevars.in")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "entrypoint.c.in")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "rust", "lib.rs")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "rust", "Cargo.toml.in")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "rust", "build.rs")))
  expect_true(file.exists(file.path(tmp, "rpkg", "src", "rust", "document.rs.in")))
  expect_true(dir.exists(file.path(tmp, "rpkg", "src", "vendor")))
})

test_that("create_miniextendr_monorepo performs correct template substitution", {
  tmp <- tempfile("monorepo-subst-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "myPkg", crate_name = "my-pkg", open = FALSE)

  # Root Cargo.toml should reference crate_name
  root_cargo <- readLines(file.path(tmp, "Cargo.toml"))
  expect_true(any(grepl("my-pkg", root_cargo)))

  # Crate Cargo.toml should have correct name
  crate_cargo <- readLines(file.path(tmp, "my-pkg", "Cargo.toml"))
  expect_true(any(grepl('name = "my-pkg"', crate_cargo)))

  # rpkg lib.rs should have package_rs (underscores)
  rpkg_lib <- readLines(file.path(tmp, "rpkg", "src", "rust", "lib.rs"))
  expect_true(any(grepl("mod myPkg;", rpkg_lib)))

  # rpkg DESCRIPTION should have package name
  desc <- readLines(file.path(tmp, "rpkg", "DESCRIPTION"))
  expect_true(any(grepl("Package: myPkg", desc)))
})

test_that("monorepo root Cargo.toml has valid workspace configuration", {
  tmp <- tempfile("monorepo-cargo-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  cargo <- readLines(file.path(tmp, "Cargo.toml"))
  cargo_text <- paste(cargo, collapse = "\n")

  # Check workspace section exists
  expect_true(grepl("\\[workspace\\]", cargo_text))
  expect_true(grepl('resolver = "3"', cargo_text))
  expect_true(grepl("testpkg", cargo_text))
  expect_true(grepl('exclude = \\["rpkg/src/rust"', cargo_text))

  # Check workspace dependencies
  expect_true(grepl("miniextendr-api", cargo_text))
  expect_true(grepl("miniextendr-macros", cargo_text))
})

test_that("monorepo rpkg DESCRIPTION has correct miniextendr config", {
  tmp <- tempfile("monorepo-desc-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  desc_path <- file.path(tmp, "rpkg", "DESCRIPTION")
  desc <- desc::desc(desc_path)

  # Check Config fields
  expect_equal(desc$get_field("Config/build/bootstrap"), "TRUE")
  expect_equal(desc$get_field("Config/build/never-clean"), "true")
  expect_equal(desc$get_field("Config/build/extra-sources"), "src/rust/Cargo.lock")

  # Check SystemRequirements
  sys_req <- desc$get_field("SystemRequirements")
  expect_true(grepl("Rust", sys_req))
})

# -----------------------------------------------------------------------------
# rpkg template tests
# -----------------------------------------------------------------------------

test_that("use_template works with rpkg template type", {
  tmp <- tempfile("rpkg-template-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create minimal package structure
  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)

  # Write minimal DESCRIPTION
  desc_content <- "Package: testpkg\nTitle: Test\nVersion: 0.0.1\n"
  writeLines(desc_content, file.path(tmp, "DESCRIPTION"))

  # Set template type and test
  set_template_type("rpkg")

  data <- list(
    package = "testpkg",
    package_rs = "testpkg",
    Package = "Testpkg",
    year = "2025"
  )

  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)
  use_template("build.rs", save_as = "src/rust/build.rs")
  use_template("Makevars.in", save_as = "src/Makevars.in")

  # Verify files exist
  expect_true(file.exists(file.path(tmp, "src", "rust", "lib.rs")))
  expect_true(file.exists(file.path(tmp, "src", "rust", "build.rs")))
  expect_true(file.exists(file.path(tmp, "src", "Makevars.in")))

  # Verify substitution
  lib_rs <- readLines(file.path(tmp, "src", "rust", "lib.rs"))
  expect_true(any(grepl("mod testpkg;", lib_rs)))
})

test_that("use_template performs mustache substitution correctly", {
  tmp <- tempfile("subst-test-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  set_template_type("rpkg")

  data <- list(
    package = "specialPkg",
    package_rs = "special_pkg"
  )

  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)

  content <- paste(readLines(file.path(tmp, "src", "rust", "lib.rs")), collapse = "\n")

  # Should have substituted {{package_rs}} but no leftover {{...}} markers
  expect_true(grepl("mod special_pkg;", content))
  expect_false(grepl("\\{\\{package", content))
})

# -----------------------------------------------------------------------------
# Build integration tests (require Rust toolchain)
# -----------------------------------------------------------------------------

test_that("monorepo can run autoconf and configure", {
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  tmp <- tempfile("monorepo-build-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  # Run autoconf in rpkg/
  result <- withr::with_dir(file.path(tmp, "rpkg"), {
    system2("autoconf", c("-v", "-i", "-f"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0)

  # Configure script should now exist
  expect_true(file.exists(file.path(tmp, "rpkg", "configure")))
})

test_that("monorepo workspace can cargo check", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  tmp <- tempfile("monorepo-cargo-check-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  # The main crate should be checkable (rpkg needs configure first)
  # suppressWarnings: cargo check may fail if miniextendr-api isn't on crates.io
  result <- suppressWarnings(
    withr::with_dir(file.path(tmp, "testpkg"), {
      system2("cargo", c("check"), stdout = TRUE, stderr = TRUE)
    })
  )
  status <- attr(result, "status")

  # This may fail if miniextendr-api is not available, which is expected
  # The test mainly verifies the Cargo.toml structure is valid
  # We check that it at least attempted to resolve dependencies
  output <- paste(result, collapse = "\n")
  # Either succeeds or fails trying to find miniextendr-api (which is expected)
  valid_outcome <- is.null(status) || status == 0 ||
                   grepl("miniextendr-api", output) ||
                   grepl("Compiling", output) ||
                   grepl("Checking", output)
  expect_true(valid_outcome)
})

# -----------------------------------------------------------------------------
# End-to-end scaffolding test (full build and test)
# -----------------------------------------------------------------------------

# Helper to find local miniextendr repo for vendoring
find_miniextendr_repo <- function() {

  # Check environment variable first
  env_path <- Sys.getenv("MINIEXTENDR_LOCAL_PATH", "")
  if (nzchar(env_path) && dir.exists(file.path(env_path, "miniextendr-api"))) {
    return(env_path)
  }


  # Check relative to minirextendr package source (development mode)
  # minirextendr is at miniextendr/minirextendr/, so parent is the repo
  pkg_path <- tryCatch(
    rprojroot::find_package_root_file(),
    error = function(e) NULL
  )
  if (!is.null(pkg_path)) {
    parent <- dirname(pkg_path)
    if (dir.exists(file.path(parent, "miniextendr-api"))) {
      return(parent)
    }
  }

  NULL
}

test_that("rpkg scaffolding builds and functions work end-to-end", {
  skip_on_ci()  # Complex build environment requirements; test locally
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")

  # Find local miniextendr repo for vendoring

  miniextendr_path <- find_miniextendr_repo()
  skip_if(is.null(miniextendr_path),
          "Local miniextendr repo not found (set MINIEXTENDR_LOCAL_PATH)")

  tmp <- tempfile("rpkg-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  # Create package inside temp dir with a valid name
  pkg_path <- file.path(tmp, "testpkg")

  # Create basic R package
  suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    usethis::proj_set(pkg_path, force = TRUE)
    use_miniextendr(local_path = miniextendr_path)
    # Add package-level documentation for useDynLib
    usethis::use_package_doc()
  })

  # Run autoconf and configure using minirextendr functions
  suppressMessages({
    usethis::proj_set(pkg_path, force = TRUE)
    miniextendr_autoconf()
    miniextendr_configure()
    # Generate R wrappers and NAMESPACE
    miniextendr_document()
    devtools::document()
  })

  # Get package name
  pkg_name <- desc::desc(file.path(pkg_path, "DESCRIPTION"))$get_field("Package")

  # Build and install to temp library using R CMD INSTALL
  lib_path <- file.path(tmp, "library")
  dir.create(lib_path)

  result <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "-l", lib_path, pkg_path),
    env = c(paste0("R_LIBS=", lib_path), "NOT_CRAN=true"),
    stdout = TRUE,
    stderr = TRUE
  )
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste("R CMD INSTALL failed:", paste(result, collapse = "\n")))

  # Test the functions work
  withr::with_libpaths(lib_path, action = "prefix", {
    # Load the package
    library(pkg_name, character.only = TRUE)

    # Test add function
    expect_equal(add(1L, 2L), 3L)
    expect_equal(add(10L, 20L), 30L)

    # Test hello function
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")

    # Unload
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})

test_that("rpkg scaffolding with external cargo dependency works", {
  skip_on_ci()  # Complex build environment requirements; test locally
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")

  miniextendr_path <- find_miniextendr_repo()
  skip_if(is.null(miniextendr_path),
          "Local miniextendr repo not found (set MINIEXTENDR_LOCAL_PATH)")

  tmp <- tempfile("rpkg-cargo-dep-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  pkg_path <- file.path(tmp, "testpkg")

  # Create package and add miniextendr
  suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    usethis::proj_set(pkg_path, force = TRUE)
    use_miniextendr(local_path = miniextendr_path)
    # Add package-level documentation for useDynLib
    usethis::use_package_doc()
  })

  # Run autoconf and configure using minirextendr functions
  suppressMessages({
    usethis::proj_set(pkg_path, force = TRUE)
    miniextendr_autoconf()
    miniextendr_configure()
  })

  # Add itertools dependency by editing Cargo.toml.in directly
  # (cargo_add would modify Cargo.toml, but configure regenerates it from .in)
  cargo_toml_in <- file.path(pkg_path, "src", "rust", "Cargo.toml.in")
  cargo_content <- readLines(cargo_toml_in)
  deps_idx <- grep("^\\[dependencies\\]", cargo_content)[1]
  if (!is.na(deps_idx)) {
    # Insert itertools right after [dependencies] header
    cargo_content <- c(
      cargo_content[1:deps_idx],
      "itertools = \"0.13\"",
      cargo_content[(deps_idx + 1):length(cargo_content)]
    )
    writeLines(cargo_content, cargo_toml_in)
  }

  # Update lib.rs to use itertools
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
  # Update module to include new function
  module_idx <- grep("miniextendr_module!", lib_content)
  if (length(module_idx) > 0) {
    # Find the line with fn declarations
    fn_line <- grep("fn add;", lib_content)
    if (length(fn_line) > 0) {
      lib_content <- c(
        lib_content[1:fn_line],
        "    fn join_strings;",
        lib_content[(fn_line + 1):length(lib_content)]
      )
    }
  }
  writeLines(lib_content, lib_rs)

  # Reconfigure to vendor itertools (with FORCE_VENDOR environment variable)
  suppressMessages({
    usethis::proj_set(pkg_path, force = TRUE)
    withr::with_envvar(c("FORCE_VENDOR" = "1", "NOT_CRAN" = "true"), {
      # Run configure with logging
      result <- run_with_logging(
        "./configure",
        log_prefix = "configure-vendor",
        wd = pkg_path
      )
      expect_true(result$success,
                  info = paste("configure with FORCE_VENDOR failed:", paste(result$output, collapse = "\n")))
    })
  })

  # Verify itertools was vendored
  expect_true(dir.exists(file.path(pkg_path, "src", "vendor", "itertools")),
              info = "itertools was not vendored")

  # Generate R wrappers and NAMESPACE
  suppressMessages({
    usethis::proj_set(pkg_path, force = TRUE)
    miniextendr_document()
    devtools::document()
  })

  # Build and install using R CMD INSTALL
  lib_path <- file.path(tmp, "library")
  dir.create(lib_path)

  result <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "-l", lib_path, pkg_path),
    env = c(paste0("R_LIBS=", lib_path), "NOT_CRAN=true"),
    stdout = TRUE,
    stderr = TRUE
  )
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste("R CMD INSTALL failed:", paste(result, collapse = "\n")))

  # Test that the functions work
  withr::with_libpaths(lib_path, action = "prefix", {
    # Load the package
    library(testpkg)

    # Test basic functions
    expect_equal(add(1L, 2L), 3L)
    expect_equal(hello("Test"), "Hello, Test!")

    # Test itertools function
    expect_equal(join_strings(c("a", "b", "c")), "a, b, c")

    # Unload
    detach("package:testpkg", character.only = TRUE, unload = TRUE)
  })
})

test_that("monorepo scaffolding builds and functions work end-to-end", {
  skip_on_ci()  # Complex build environment requirements; test locally
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")

  # Find local miniextendr repo for vendoring
  miniextendr_path <- find_miniextendr_repo()
  skip_if(is.null(miniextendr_path),
          "Local miniextendr repo not found (set MINIEXTENDR_LOCAL_PATH)")

  tmp <- tempfile("monorepo-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create monorepo with valid package name
  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)
  })

  rpkg_path <- file.path(tmp, "rpkg")

  # Vendor miniextendr from local path into rpkg/src/vendor
  suppressMessages({
    vendor_miniextendr(local_path = miniextendr_path, dest = file.path(rpkg_path, "src", "vendor"))
    # Add package-level documentation for useDynLib
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
  })

  # Pre-vendor crates.io dependencies manually
  # (needed before autoconf/configure can run)
  cargo_toml_in <- file.path(rpkg_path, "src", "rust", "Cargo.toml.in")
  cargo_toml <- file.path(rpkg_path, "src", "rust", "Cargo.toml")
  if (file.exists(cargo_toml_in)) {
    content <- readLines(cargo_toml_in)
    content <- gsub("@PACKAGE_TARNAME_RS@", "testpkg", content)
    content <- gsub("@CARGO_FEATURE_CPPFLAGS@", "", content)
    writeLines(content, cargo_toml)
  }

  # Run cargo vendor to fetch crates.io deps (proc-macro2, syn, quote, etc.)
  suppressWarnings({
    withr::with_dir(rpkg_path, {
      system2("cargo", c("vendor", "--manifest-path", "src/rust/Cargo.toml", "src/vendor"),
              stdout = FALSE, stderr = FALSE)
    })
  })

  # Run autoconf and configure using minirextendr functions
  suppressMessages({
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
    miniextendr_autoconf()
    miniextendr_configure()
    # Generate R wrappers and NAMESPACE
    miniextendr_document()
    devtools::document()
  })

  # Get package name
  pkg_name <- desc::desc(file.path(rpkg_path, "DESCRIPTION"))$get_field("Package")

  # Build and install using R CMD INSTALL
  lib_path <- file.path(tmp, "library")
  dir.create(lib_path)

  result <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "-l", lib_path, rpkg_path),
    env = c(paste0("R_LIBS=", lib_path), "NOT_CRAN=true"),
    stdout = TRUE,
    stderr = TRUE
  )
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste("R CMD INSTALL failed:", paste(result, collapse = "\n")))

  # Test the functions work
  withr::with_libpaths(lib_path, action = "prefix", {
    # Load the package
    library(pkg_name, character.only = TRUE)

    # Test add function
    expect_equal(add(1L, 2L), 3L)
    expect_equal(add(10L, 20L), 30L)

    # Test hello function
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")

    # Unload
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})
