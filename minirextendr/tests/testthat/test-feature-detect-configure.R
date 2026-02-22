# Tests for configure-time feature detection (tools/detect-features.R)

test_that("generate_empty_detect_script produces valid structure", {
  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
  text <- paste(lines, collapse = "\n")

  expect_true(any(grepl("^## BEGIN RULES", lines)))
  expect_true(any(grepl("^## END RULES", lines)))
  expect_true(any(grepl("MYPKG_FEATURES", lines)))
  expect_true(any(grepl("mypkg", lines)))
  expect_true(any(grepl('cat\\(paste\\(features', lines)))
})

test_that("append and parse feature rules round-trip", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
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

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
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

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
  writeLines(lines, tmp)

  result <- minirextendr:::remove_feature_rule_from_script(tmp, "nonexistent")
  expect_false(result)
})

test_that("parse_detect_features_script returns empty list for no rules", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
  writeLines(lines, tmp)

  rules <- minirextendr:::parse_detect_features_script(tmp)
  expect_equal(length(rules), 0)
  expect_type(rules, "list")
})

test_that("generated detect script is valid R and outputs features", {
  tmp <- tempfile(fileext = ".R")
  on.exit(unlink(tmp), add = TRUE)

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
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

  lines <- minirextendr:::generate_empty_detect_script("mypkg", "MYPKG_FEATURES")
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
    'if test -z "${MYPKG_FEATURES+x}"; then',
    '  dnl MYPKG_FEATURES not set - use empty (no extra features)',
    '  MYPKG_FEATURES=""',
    'fi',
    'AC_OUTPUT'
  ), tmp)

  result <- minirextendr:::patch_configure_ac_for_detection(tmp, "MYPKG_FEATURES")
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
    'if test -z "${MYPKG_FEATURES+x}"; then',
    '  dnl MYPKG_FEATURES not set - use empty (no extra features)',
    '  MYPKG_FEATURES=""',
    'fi',
    'AC_OUTPUT'
  ), tmp)

  minirextendr:::patch_configure_ac_for_detection(tmp, "MYPKG_FEATURES")
  text_after_first <- paste(readLines(tmp, warn = FALSE), collapse = "\n")

  # Second call should detect it's already patched
  result <- minirextendr:::patch_configure_ac_for_detection(tmp, "MYPKG_FEATURES")
  expect_false(result)

  text_after_second <- paste(readLines(tmp, warn = FALSE), collapse = "\n")
  expect_equal(text_after_first, text_after_second)
})
