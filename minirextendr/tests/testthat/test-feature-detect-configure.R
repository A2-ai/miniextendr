# Tests for configure-time feature detection (tools/detect-features.R)

test_that("generate_empty_detect_script produces valid structure", {
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  text <- paste(lines, collapse = "\n")

  expect_true(any(grepl("^## BEGIN RULES", lines)))
  expect_true(any(grepl("^## END RULES", lines)))
  expect_true(any(grepl("CARGO_FEATURES", lines)))
  expect_true(any(grepl("mypkg", lines)))
  expect_true(any(grepl('cat\\(paste\\(features', lines)))
  # toolchain-gate helper available to rules
  expect_true(any(grepl("rustc_at_least <- function", lines)))
})

test_that("use_configure_feature_detection accepts an existing marked script", {
  # Regression: the marker check pasted the whole script into one string and
  # ran grepl("^## BEGIN RULES", text, multiline = TRUE). `multiline` is not a
  # grepl() argument (byte-compile note), and `^` against the pasted text only
  # matched line 1 -- so a valid script (marker ~line 150) was mis-reported as
  # unmarked, firing a spurious "...has no marker, delete and re-run" warning.
  # The word "marker" appears only in that warning, never in the happy path.
  proj <- withr::local_tempdir()
  dir.create(file.path(proj, "tools"), recursive = TRUE)
  writeLines(
    minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES"),
    file.path(proj, "tools", "detect-features.R")
  )
  writeLines(c(
    "AC_INIT([mypkg], [1.0])",
    'if test -z "${CARGO_FEATURES+x}"; then',
    "  dnl CARGO_FEATURES not set - use empty (no extra features)",
    '  CARGO_FEATURES=""',
    "fi"
  ), file.path(proj, "configure.ac"))

  local_mocked_bindings(
    with_project = function(...) invisible(NULL),
    template_data = function(...) list(package = "mypkg", features_var = "CARGO_FEATURES"),
    miniextendr_autoconf = function(...) invisible(TRUE),
    .package = "minirextendr"
  )
  usethis::local_project(proj, force = TRUE, setwd = FALSE)

  msgs <- testthat::capture_messages(use_configure_feature_detection())
  expect_false(any(grepl("marker", msgs)))
  expect_true(any(grepl("already exists", msgs)))
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
  # The robust skeleton requires src/rust/Cargo.toml, so we build a fixture tree.
  tmp <- withr::local_tempdir()
  dir.create(file.path(tmp, "tools"), recursive = TRUE)
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)

  # Cargo.toml with exactly the features under test
  writeLines(c(
    "[package]",
    'name = "mypkg"',
    'version = "0.1.0"',
    "",
    "[features]",
    "rayon = []",
    "vctrs = []"
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  script_path <- file.path(tmp, "tools", "detect-features.R")
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, script_path)
  # rayon: always enable; vctrs: rule returns FALSE -> disabled
  minirextendr:::append_feature_rule(script_path, "rayon", TRUE)
  minirextendr:::append_feature_rule(script_path, "vctrs", "FALSE")

  # Execute from the fixture root so relative Cargo.toml lookup works
  output <- withr::with_dir(tmp, {
    system2(
      file.path(R.home("bin"), "Rscript"), script_path,
      stdout = TRUE, stderr = FALSE
    )
  })
  expect_equal(output, "rayon")
})

test_that("generated detect script outputs multiple features comma-separated", {
  tmp <- withr::local_tempdir()
  dir.create(file.path(tmp, "tools"), recursive = TRUE)
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)

  writeLines(c(
    "[package]",
    'name = "mypkg"',
    'version = "0.1.0"',
    "",
    "[features]",
    "rayon = []",
    "serde = []"
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  script_path <- file.path(tmp, "tools", "detect-features.R")
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, script_path)
  minirextendr:::append_feature_rule(script_path, "rayon", TRUE)
  minirextendr:::append_feature_rule(script_path, "serde", TRUE)

  output <- withr::with_dir(tmp, {
    system2(
      file.path(R.home("bin"), "Rscript"), script_path,
      stdout = TRUE, stderr = FALSE
    )
  })
  expect_equal(output, "rayon,serde")
})

test_that("generated detect script auto-enables features with no rule", {
  # A feature present in Cargo.toml but with NO rule is enabled by default
  # (auto-discovery). This is the new default-enable paradigm.
  tmp <- withr::local_tempdir()
  dir.create(file.path(tmp, "tools"), recursive = TRUE)
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)

  writeLines(c(
    "[package]",
    'name = "mypkg"',
    'version = "0.1.0"',
    "",
    "[features]",
    "rayon = []",
    "auto_enabled = []"
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  script_path <- file.path(tmp, "tools", "detect-features.R")
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES")
  writeLines(lines, script_path)
  # Only add a rule for rayon; auto_enabled has no rule -> enabled by default
  minirextendr:::append_feature_rule(script_path, "rayon", TRUE)

  output <- withr::with_dir(tmp, {
    system2(
      file.path(R.home("bin"), "Rscript"), script_path,
      stdout = TRUE, stderr = FALSE
    )
  })
  # Both features should be present (sorted)
  expect_equal(output, "auto_enabled,rayon")
})

test_that("rustc_at_least gate enables/disables a feature by toolchain version", {
  skip_on_os("windows") # fake rustc shim is a POSIX shell script

  tmp <- withr::local_tempdir()
  dir.create(file.path(tmp, "tools"), recursive = TRUE)
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    "[package]", 'name = "mypkg"', 'version = "0.1.0"', "",
    "[features]", "datafusion = []", "serde = []"
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  script_path <- file.path(tmp, "tools", "detect-features.R")
  writeLines(
    minirextendr:::generate_empty_detect_script("mypkg", "CARGO_FEATURES"),
    script_path
  )
  minirextendr:::append_feature_rule(script_path, "datafusion", 'rustc_at_least("1.82.0")')

  fake_bin <- file.path(tmp, "fakebin")
  dir.create(fake_bin)
  run_with_rustc <- function(version) {
    writeLines(
      c("#!/bin/sh", sprintf('echo "rustc %s (abc123 2026-01-01)"', version)),
      file.path(fake_bin, "rustc")
    )
    Sys.chmod(file.path(fake_bin, "rustc"), "0755")
    withr::with_path(fake_bin, action = "prefix", {
      withr::with_dir(tmp, {
        system2(
          file.path(R.home("bin"), "Rscript"), script_path,
          stdout = TRUE, stderr = FALSE
        )
      })
    })
  }

  # datafusion MSRV is 1.82: old toolchain drops it, new toolchain keeps it.
  expect_equal(run_with_rustc("1.70.0"), "serde")
  expect_equal(run_with_rustc("1.85.0"), "datafusion,serde")
})

test_that("rpkg detect-features.R matches generator output (regenerable invariant)", {
  # The shipped rpkg/tools/detect-features.R must equal what the generator
  # produces for ("miniextendr", "CARGO_FEATURES") plus the two canonical rules.
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("miniextendr", "CARGO_FEATURES")
  writeLines(lines, tmp)
  minirextendr:::append_feature_rule(tmp, "vctrs", 'requireNamespace("vctrs", quietly = TRUE)')
  minirextendr:::append_feature_rule(tmp, "connections", 'getRversion() >= "4.3.0"')
  minirextendr:::append_feature_rule(tmp, "arrow", 'rustc_at_least("1.81.0")')
  minirextendr:::append_feature_rule(tmp, "datafusion", 'rustc_at_least("1.82.0")')

  generated <- readLines(tmp, warn = FALSE)
  # Path to the shipped rpkg file (relative to the test file's package root)
  rpkg_path <- system.file("../../rpkg/tools/detect-features.R",
    package = "minirextendr", mustWork = FALSE
  )
  if (!nzchar(rpkg_path) || !file.exists(rpkg_path)) {
    # Fallback: find it relative to this test file
    rpkg_path <- file.path(
      dirname(dirname(dirname(testthat::test_path()))),
      "rpkg", "tools", "detect-features.R"
    )
  }
  skip_if(!file.exists(rpkg_path), "rpkg/tools/detect-features.R not found (running outside repo)")

  shipped <- readLines(rpkg_path, warn = FALSE)
  expect_equal(generated, shipped)
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
