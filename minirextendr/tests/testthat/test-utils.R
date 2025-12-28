# Tests for utility functions

test_that("to_rust_name converts package names correctly", {
  expect_equal(to_rust_name("my.package"), "my_package")
  expect_equal(to_rust_name("my-package"), "my_package")
  expect_equal(to_rust_name("mypackage"), "mypackage")
  expect_equal(to_rust_name("my.cool-pkg"), "my_cool_pkg")
})

test_that("template_path returns valid paths for rpkg template type", {
  skip_if_not_installed("minirextendr")

  set_template_type("rpkg")

  # Should not error for known templates
  expect_true(file.exists(template_path("configure.ac")))
  expect_true(file.exists(template_path("lib.rs")))
  expect_true(file.exists(template_path("bootstrap.R")))
  expect_true(file.exists(template_path("Makevars.in")))
})

test_that("template_path returns valid paths for monorepo template type", {
  skip_if_not_installed("minirextendr")

  set_template_type("monorepo")
  on.exit(set_template_type("rpkg"))

  # Root templates
  expect_true(file.exists(template_path("Cargo.toml")))
  expect_true(file.exists(template_path("justfile")))
  expect_true(file.exists(template_path("CLAUDE.md")))

  # Nested rpkg templates
  expect_true(file.exists(template_path("lib.rs", subdir = "rpkg")))
  expect_true(file.exists(template_path("configure.ac", subdir = "rpkg")))
  expect_true(file.exists(template_path("Makevars.in", subdir = "rpkg")))

  # my-crate templates
  expect_true(file.exists(template_path("Cargo.toml", subdir = "my-crate")))
  expect_true(file.exists(template_path("lib.rs", subdir = "my-crate/src")))
})

test_that("set_template_type and get_template_type work", {
  old_type <- get_template_type()
  on.exit(set_template_type(old_type))

  set_template_type("rpkg")
  expect_equal(get_template_type(), "rpkg")

  set_template_type("monorepo")
  expect_equal(get_template_type(), "monorepo")

  expect_error(set_template_type("invalid"), "should be one of")
})

test_that("detect_project_type identifies monorepo from workspace Cargo.toml", {
  tmp <- tempfile("monorepo-detect-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)

  # Create workspace Cargo.toml
  cargo_content <- "[workspace]\nmembers = [\"crate1\"]\n"
  writeLines(cargo_content, file.path(tmp, "Cargo.toml"))

  usethis::proj_set(tmp, force = TRUE)
  expect_equal(detect_project_type(tmp), "monorepo")
})

test_that("detect_project_type identifies standalone rpkg", {
  tmp <- tempfile("rpkg-detect-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)

  # Create DESCRIPTION (R package)
  desc_content <- "Package: testpkg\nTitle: Test\nVersion: 0.1.0\n"
  writeLines(desc_content, file.path(tmp, "DESCRIPTION"))

  usethis::proj_set(tmp, force = TRUE)
  expect_equal(detect_project_type(tmp), "rpkg")
})

test_that("detect_project_type identifies rpkg inside monorepo", {
  tmp <- tempfile("monorepo-rpkg-detect-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create monorepo structure
  dir.create(file.path(tmp, "rpkg"), recursive = TRUE)

  # Create workspace Cargo.toml at root
  cargo_content <- "[workspace]\nmembers = [\"my-crate\"]\nexclude = [\"rpkg\"]\n"
  writeLines(cargo_content, file.path(tmp, "Cargo.toml"))

  # Create DESCRIPTION in rpkg/
  desc_content <- "Package: testpkg\nTitle: Test\nVersion: 0.1.0\n"
  writeLines(desc_content, file.path(tmp, "rpkg", "DESCRIPTION"))

  # When in rpkg/ subdirectory, should still detect as monorepo
  usethis::proj_set(file.path(tmp, "rpkg"), force = TRUE)
  expect_equal(detect_project_type(file.path(tmp, "rpkg")), "monorepo")
})

test_that("is_in_rust_workspace returns TRUE for monorepo", {
  tmp <- tempfile("workspace-check-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)

  # Create workspace Cargo.toml
  cargo_content <- "[workspace]\nmembers = []\n"
  writeLines(cargo_content, file.path(tmp, "Cargo.toml"))

  expect_true(is_in_rust_workspace(tmp))
})

test_that("is_in_rust_workspace returns FALSE for standalone rpkg", {
  tmp <- tempfile("no-workspace-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)

  # Just a DESCRIPTION, no Cargo.toml
  desc_content <- "Package: testpkg\nTitle: Test\nVersion: 0.1.0\n"
  writeLines(desc_content, file.path(tmp, "DESCRIPTION"))

  expect_false(is_in_rust_workspace(tmp))
})
