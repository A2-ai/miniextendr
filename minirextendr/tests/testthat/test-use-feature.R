# Tests for feature detection and code generation

# =============================================================================
# detect_cargo_features tests
# =============================================================================

test_that("detect_cargo_features extracts features from Cargo.toml", {
  tmp <- tempfile("feature-detect-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  # Create Cargo.toml with features
  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    '[package]',
    'name = "testpkg"',
    '',
    '[features]',
    'default = []',
    'vctrs = ["miniextendr-api/vctrs"]',
    'rayon = ["miniextendr-api/rayon"]',
    'serde = ["miniextendr-api/serde"]',
    '',
    '[dependencies]',
    'miniextendr-api = "0.1"'
  ), file.path(cargo_dir, "Cargo.toml"))

  features <- detect_cargo_features()
  expect_type(features, "character")
  expect_true("vctrs" %in% features)
  expect_true("rayon" %in% features)
  expect_true("serde" %in% features)
  # "default" should be filtered out
  expect_false("default" %in% features)
})

test_that("detect_cargo_features returns empty for no features section", {
  tmp <- tempfile("feature-none-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    '[package]',
    'name = "testpkg"',
    '',
    '[dependencies]'
  ), file.path(cargo_dir, "Cargo.toml"))

  features <- detect_cargo_features()
  expect_equal(features, character())
})

test_that("detect_cargo_features warns on missing Cargo.toml", {
  tmp <- tempfile("feature-missing-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  expect_warning(detect_cargo_features(), "not found")
})

# =============================================================================
# Generated code structure tests
# =============================================================================

test_that("generate_feature_detection_rust produces valid structure", {
  code <- generate_feature_detection_rust("testpkg", c("rayon", "serde"))

  # Should contain the function name
  expect_true(grepl("testpkg_enabled_features", code))
  # Should contain cfg! checks for each feature
  expect_true(grepl('cfg!\\(feature = "rayon"\\)', code))
  expect_true(grepl('cfg!\\(feature = "serde"\\)', code))
  # Should be parseable Rust (at least no obvious syntax errors)
  expect_true(grepl("#\\[miniextendr\\]", code))
})

test_that("generate_feature_detection_r produces valid R code", {
  code <- generate_feature_detection_r("testpkg")

  # Should be parseable R
  expect_silent(parse(text = code))
  # Should contain the helper functions
  expect_true(grepl("testpkg_has_feature", code))
  expect_true(grepl("skip_if_missing_feature", code))
})

test_that("generate_feature_detection_rust handles package name with dots", {
  code <- generate_feature_detection_rust("my.pkg", c("vctrs"))

  # Dots should be converted to underscores in Rust identifiers
  expect_true(grepl("my_pkg_enabled_features", code))
})

test_that("generate_feature_detection_rust handles empty features", {
  code <- generate_feature_detection_rust("testpkg", character())

  # Should still produce valid code
  expect_true(grepl("testpkg_enabled_features", code))
})
