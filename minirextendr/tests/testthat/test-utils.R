# Tests for utility functions

test_that("to_rust_name converts package names correctly", {
  expect_equal(to_rust_name("my.package"), "my_package")
  expect_equal(to_rust_name("my-package"), "my_package")
  expect_equal(to_rust_name("mypackage"), "mypackage")
  expect_equal(to_rust_name("my.cool-pkg"), "my_cool_pkg")
})

test_that("template_path returns valid paths", {
  skip_if_not_installed("minirextendr")

  # Should not error for known templates
  expect_true(file.exists(template_path("configure.ac")))
  expect_true(file.exists(template_path("lib.rs")))
  expect_true(file.exists(template_path("bootstrap.R")))
})
