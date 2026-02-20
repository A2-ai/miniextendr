test_that("mx_config_defaults() returns expected structure", {
  defaults <- mx_config_defaults()

  expect_type(defaults, "list")
  expect_named(defaults, c("class_system", "strict", "coerce", "features",
                            "rust_version", "vendor"))
  expect_equal(defaults$class_system, "env")
  expect_false(defaults$strict)
  expect_false(defaults$coerce)
  expect_equal(defaults$features, character(0))
  expect_equal(defaults$rust_version, "stable")
  expect_true(defaults$vendor)
})

test_that("mx_config() returns defaults when no file exists", {
  tmp <- withr::local_tempdir()
  config <- mx_config(path = tmp)
  expect_equal(config, mx_config_defaults())
})

test_that("mx_config() merges user overrides over defaults", {
  skip_if_not_installed("yaml")

  tmp <- withr::local_tempdir()
  writeLines(
    c("class_system: r6", "strict: true"),
    file.path(tmp, "miniextendr.yml")
  )

  config <- mx_config(path = tmp)
  expect_equal(config$class_system, "r6")
  expect_true(config$strict)
  # Unspecified keys keep defaults

  expect_false(config$coerce)
  expect_equal(config$rust_version, "stable")
})

test_that("mx_config() warns on unknown keys", {
  skip_if_not_installed("yaml")

  tmp <- withr::local_tempdir()
  writeLines(
    c("class_system: s3", "bogus_key: 42"),
    file.path(tmp, "miniextendr.yml")
  )

  expect_warning(
    config <- mx_config(path = tmp),
    class = "mx_config_unknown_keys"
  )
  # Known keys still applied
  expect_equal(config$class_system, "s3")
})

test_that("mx_config() handles features as character vector", {
  skip_if_not_installed("yaml")

  tmp <- withr::local_tempdir()
  writeLines(
    c("features:", "  - rayon", "  - serde"),
    file.path(tmp, "miniextendr.yml")
  )

  config <- mx_config(path = tmp)
  expect_equal(config$features, c("rayon", "serde"))
})

test_that("mx_config() handles malformed yaml gracefully", {
  skip_if_not_installed("yaml")

  tmp <- withr::local_tempdir()
  writeLines("{{{{invalid yaml", file.path(tmp, "miniextendr.yml"))

  expect_warning(
    config <- mx_config(path = tmp),
    class = "mx_config_parse_error"
  )
  expect_equal(config, mx_config_defaults())
})
