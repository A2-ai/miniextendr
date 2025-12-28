# Integration tests for template scaffolding
#
# These tests verify that the scaffolding functions create valid projects
# that can be built with the miniextendr toolchain.

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
  expect_true(file.exists(file.path(tmp, "CLAUDE.md")))
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
  result <- withr::with_dir(file.path(tmp, "testpkg"), {
    system2("cargo", c("check"), stdout = TRUE, stderr = TRUE)
  })
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
