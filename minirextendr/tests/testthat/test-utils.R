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
