# Tests for configure-time feature detection (tools/detect-features.R)

test_that("generate_empty_detect_script produces valid structure", {
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  text <- paste(lines, collapse = "\n")

  expect_true(any(grepl("^## BEGIN RULES", lines)))
  expect_true(any(grepl("^## END RULES", lines)))
  expect_true(any(grepl("CARGO_FEATURES", lines)))
  expect_true(any(grepl("mypkg", lines)))
  expect_true(any(grepl('cat\\(paste\\(features', lines)))
})

test_that("append and parse feature rules round-trip", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)

  # Add a TRUE rule

  minirextendr:::append_feature_rule(tmp, "rayon", TRUE)
  rules <- minirextendr:::parse_detect_features_script(tmp)
  expect_equal(rules$rayon, "TRUE")

  # Add a string rule
  minirextendr:::append_feature_rule(tmp, "vctrs", 'requireNamespace("vctrs", quietly = TRUE)')
  rules <- minirextendr:::parse_detect_features_script(tmp)
  expect_equal(length(rules), 2)
  expect_equal(rules$rayon, "TRUE")
  expect_equal(rules$vctrs, 'requireNamespace("vctrs", quietly = TRUE)')
})

test_that("remove_feature_rule_from_script removes correct rule", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)
  minirextendr:::append_feature_rule(tmp, "rayon", TRUE)
  minirextendr:::append_feature_rule(tmp, "vctrs", 'requireNamespace("vctrs", quietly = TRUE)')

  # Remove rayon
  result <- minirextendr:::remove_feature_rule_from_script(tmp, "rayon")
  expect_true(result)

  rules <- minirextendr:::parse_detect_features_script(tmp)
  expect_equal(length(rules), 1)
  expect_null(rules$rayon)
  expect_equal(rules$vctrs, 'requireNamespace("vctrs", quietly = TRUE)')
})

test_that("remove_feature_rule_from_script returns FALSE for missing rule", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)

  result <- minirextendr:::remove_feature_rule_from_script(tmp, "nonexistent")
  expect_false(result)
})

test_that("parse_detect_features_script returns empty list for no rules", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)

  rules <- minirextendr:::parse_detect_features_script(tmp)
  expect_equal(length(rules), 0)
  expect_type(rules, "list")
})

test_that("generated detect script is valid R and outputs features", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)
  minirextendr:::append_feature_rule(tmp, "rayon", TRUE)
  minirextendr:::append_feature_rule(tmp, "vctrs", "FALSE")

  # Execute the script
  output <- system2(
    file.path(R.home("bin"), "Rscript"), tmp,
    stdout = TRUE, stderr = FALSE
  )
  expect_equal(output, "rayon")
})

test_that("generated detect script outputs multiple features comma-separated", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, tmp)
  minirextendr:::append_feature_rule(tmp, "rayon", TRUE)
  minirextendr:::append_feature_rule(tmp, "serde", TRUE)

  output <- system2(
    file.path(R.home("bin"), "Rscript"), tmp,
    stdout = TRUE, stderr = FALSE
  )
  expect_equal(output, "rayon,serde")
})

test_that("patch_configure_ac_for_detection patches old-style block", {
  tmp <- tempfile(fileext = ".ac")
  on.exit(unlink(tmp), add = TRUE)

  # Write a configure.ac with the old-style block
  writeLines(c(
    'AC_INIT([mypkg], [1.0])',
    'if test -z "${CARGO_FEATURES+x}"; then',
    '  dnl CARGO_FEATURES not set - use empty (no extra features)',
    '  CARGO_FEATURES=""',
    'fi',
    'AC_OUTPUT'
  ), tmp)

  result <- minirextendr:::patch_configure_ac_for_detection(tmp, "CARGO_FEATURES")
  expect_true(result)

  text <- paste(readLines(tmp, warn = FALSE), collapse = "\n")
  expect_true(grepl("detect-features\\.R", text))
  expect_true(grepl("auto-detect", text))
  expect_true(grepl("Rscript", text))
  # Old comment should be gone
  expect_false(grepl("use empty \\(no extra features\\)", text))
})

test_that("patch_configure_ac_for_detection is idempotent", {
  tmp <- tempfile(fileext = ".ac")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(c(
    'AC_INIT([mypkg], [1.0])',
    'if test -z "${CARGO_FEATURES+x}"; then',
    '  dnl CARGO_FEATURES not set - use empty (no extra features)',
    '  CARGO_FEATURES=""',
    'fi',
    'AC_OUTPUT'
  ), tmp)

  minirextendr:::patch_configure_ac_for_detection(tmp, "CARGO_FEATURES")
  text_after_first <- paste(readLines(tmp, warn = FALSE), collapse = "\n")

  # Second call should detect it's already patched
  result <- minirextendr:::patch_configure_ac_for_detection(tmp, "CARGO_FEATURES")
  expect_false(result)

  text_after_second <- paste(readLines(tmp, warn = FALSE), collapse = "\n")
  expect_equal(text_after_first, text_after_second)
})

test_that("add_feature_rule validates optional_dep parameter", {
  proj <- withr::local_tempdir()
  dir.create(file.path(proj, "tools"), recursive = TRUE)
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, file.path(proj, "tools", "detect-features.R"))

  local_mocked_bindings(
    with_project = function(...) invisible(NULL),
    .package = "minirextendr"
  )
  withr::local_options(usethis.quiet = TRUE)
  usethis::local_project(proj, force = TRUE, setwd = FALSE)

  # Invalid optional_dep values should error
  expect_error(
    add_feature_rule("foo", detect = TRUE, optional_dep = 42),
    "optional_dep"
  )
  expect_error(
    add_feature_rule("foo", detect = TRUE, optional_dep = ""),
    "optional_dep"
  )
  expect_error(
    add_feature_rule("foo", detect = TRUE, optional_dep = c("a", "b")),
    "optional_dep"
  )
})

test_that("add_feature_rule with optional_dep = FALSE skips cargo_add", {
  proj <- withr::local_tempdir()
  dir.create(file.path(proj, "tools"), recursive = TRUE)
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, file.path(proj, "tools", "detect-features.R"))

  cargo_add_called <- FALSE

  local_mocked_bindings(
    with_project = function(...) invisible(NULL),
    cargo_add = function(...) { cargo_add_called <<- TRUE; invisible(TRUE) },
    .package = "minirextendr"
  )
  withr::local_options(usethis.quiet = TRUE)
  usethis::local_project(proj, force = TRUE, setwd = FALSE)

  add_feature_rule("rayon", detect = TRUE, optional_dep = FALSE)

  expect_false(cargo_add_called)
  rules <- minirextendr:::parse_detect_features_script(
    file.path(proj, "tools", "detect-features.R")
  )
  expect_equal(rules$rayon, "TRUE")
})

test_that("add_feature_rule with optional_dep = TRUE calls cargo_add", {
  proj <- withr::local_tempdir()
  dir.create(file.path(proj, "tools"), recursive = TRUE)
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, file.path(proj, "tools", "detect-features.R"))

  captured_args <- NULL

  local_mocked_bindings(
    with_project = function(...) invisible(NULL),
    cargo_add = function(...) { captured_args <<- list(...); invisible(TRUE) },
    .package = "minirextendr"
  )
  withr::local_options(usethis.quiet = TRUE)
  usethis::local_project(proj, force = TRUE, setwd = FALSE)

  add_feature_rule("rayon", detect = TRUE, optional_dep = TRUE)

  expect_false(is.null(captured_args))
  expect_equal(captured_args$dep, "rayon")
  expect_true(captured_args$optional)

  rules <- minirextendr:::parse_detect_features_script(
    file.path(proj, "tools", "detect-features.R")
  )
  expect_equal(rules$rayon, "TRUE")
})

test_that("add_feature_rule with optional_dep string uses it as dep spec", {
  proj <- withr::local_tempdir()
  dir.create(file.path(proj, "tools"), recursive = TRUE)
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, file.path(proj, "tools", "detect-features.R"))

  captured_args <- NULL

  local_mocked_bindings(
    with_project = function(...) invisible(NULL),
    cargo_add = function(...) { captured_args <<- list(...); invisible(TRUE) },
    .package = "minirextendr"
  )
  withr::local_options(usethis.quiet = TRUE)
  usethis::local_project(proj, force = TRUE, setwd = FALSE)

  add_feature_rule("rayon", detect = TRUE, optional_dep = "rayon@1.10")

  expect_false(is.null(captured_args))
  expect_equal(captured_args$dep, "rayon@1.10")
  expect_true(captured_args$optional)
})

# =============================================================================
# parse_cargo_metadata_json tests
# =============================================================================

test_that("parse_cargo_metadata_json extracts features", {
  json <- '{"packages":[{"features":{"default":[],"rayon":["miniextendr-api/rayon"],"serde":["miniextendr-api/serde","dep:serde"]},"dependencies":[]}]}'
  result <- minirextendr:::parse_cargo_metadata_json(json)

  expect_equal(result$features$default, character())
  expect_equal(result$features$rayon, "miniextendr-api/rayon")
  expect_equal(result$features$serde, c("miniextendr-api/serde", "dep:serde"))
  expect_equal(length(result$optional_deps), 0)
})

test_that("parse_cargo_metadata_json extracts optional deps", {
  json <- '{"packages":[{"features":{},"dependencies":[{"name":"serde","req":"^1","optional":true,"features":["derive"]},{"name":"miniextendr-api","req":"*","optional":false,"features":[]}]}]}'
  result <- minirextendr:::parse_cargo_metadata_json(json)

  expect_equal(length(result$optional_deps), 1)
  expect_equal(result$optional_deps$serde$version, "^1")
  expect_equal(result$optional_deps$serde$features, "derive")
})

test_that("parse_cargo_metadata_json handles empty features and deps", {
  json <- '{"packages":[{"features":{},"dependencies":[]}]}'
  result <- minirextendr:::parse_cargo_metadata_json(json)

  expect_equal(length(result$features), 0)
  expect_equal(length(result$optional_deps), 0)
})

test_that("parse_cargo_metadata_json handles multiple optional deps", {
  json <- '{"packages":[{"features":{"bitflags":["dep:bitflags"],"time":["dep:time"]},"dependencies":[{"name":"bitflags","req":"^2","optional":true,"features":[]},{"name":"time","req":"^0.3","optional":true,"features":["macros","formatting"]},{"name":"core-dep","req":"*","optional":false,"features":[]}]}]}'
  result <- minirextendr:::parse_cargo_metadata_json(json)

  expect_equal(length(result$optional_deps), 2)
  expect_equal(result$optional_deps$bitflags$version, "^2")
  expect_equal(result$optional_deps$bitflags$features, character())
  expect_equal(result$optional_deps$time$version, "^0.3")
  expect_equal(result$optional_deps$time$features, c("macros", "formatting"))
})
