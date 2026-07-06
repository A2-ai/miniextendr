# Tests for the unified .Rbuildignore / .gitignore scaffolding (#1151)
#
# The standalone path (use_miniextendr_rbuildignore() /
# use_miniextendr_gitignore()) and the monorepo path
# (create_rpkg_subdirectory()) share the "read template, filter comments"
# prep step — mx_ignore_patterns() in R/utils.R. The standalone path keeps
# usethis's append + dedupe write into a possibly pre-existing file; the
# monorepo path keeps its fresh-file overwrite. For a brand-new file the two
# writes are byte-identical, so fresh scaffolds produce the same content on
# both paths.

test_that("mx_ignore_patterns() filters blank and comment lines, preserving order", {
  minirextendr:::set_template_type("rpkg")
  withr::defer(minirextendr:::set_template_type("rpkg"))

  for (tpl in c("Rbuildignore", "gitignore")) {
    raw <- readLines(
      system.file("templates", "rpkg", tpl, package = "minirextendr", mustWork = TRUE)
    )
    patterns <- minirextendr:::mx_ignore_patterns(tpl)

    expect_true(length(patterns) > 0)
    expect_true(all(nzchar(patterns)))
    expect_false(any(grepl("^#", patterns)))
    # Every pattern comes from the template, in template order
    expect_identical(patterns, raw[raw %in% patterns])
  }

  # Spot-check load-bearing patterns survive the filter
  expect_true("^vendor$" %in% minirextendr:::mx_ignore_patterns("Rbuildignore"))
  expect_true("vendor/" %in% minirextendr:::mx_ignore_patterns("gitignore"))
})

test_that("standalone fresh scaffold writes exactly the filtered template patterns", {
  tmp <- withr::local_tempdir()
  writeLines(c("Package: ignpkg", "Version: 0.0.1"), file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  minirextendr:::set_template_type("rpkg")
  withr::defer(minirextendr:::set_template_type("rpkg"))

  suppressMessages({
    minirextendr:::use_miniextendr_rbuildignore(path = tmp)
    minirextendr:::use_miniextendr_gitignore(path = tmp)
  })

  # Brand-new files: usethis writes exactly the filtered patterns, in order.
  # This is the byte-identity that lets the monorepo path's fresh
  # writeLines() produce the same content (#1151).
  expect_identical(readLines(file.path(tmp, ".Rbuildignore")),
                   minirextendr:::mx_ignore_patterns("Rbuildignore"))
  expect_identical(readLines(file.path(tmp, ".gitignore")),
                   minirextendr:::mx_ignore_patterns("gitignore"))
})

test_that("monorepo scaffold ignore files match the standalone fresh-scaffold content", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")

  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  minirextendr:::set_template_type("monorepo")
  withr::defer(minirextendr:::set_template_type("rpkg"))

  data <- list(
    package = "mypkg",
    package_rs = "mypkg",
    Package = "Mypkg",
    crate_name = "mypkg-rs",
    rpkg_name = "rpkg",
    features_var = "CARGO_FEATURES",
    year = "2026"
  )
  suppressMessages(minirextendr:::create_rpkg_subdirectory(data, rpkg_name = "rpkg"))

  rbuild <- readLines(file.path(tmp, "rpkg", ".Rbuildignore"))
  gitign <- readLines(file.path(tmp, "rpkg", ".gitignore"))

  # Written content is the filtered monorepo template ...
  expect_identical(rbuild,
                   minirextendr:::mx_ignore_patterns("Rbuildignore", subdir = "rpkg"))
  expect_identical(gitign,
                   minirextendr:::mx_ignore_patterns("gitignore", subdir = "rpkg"))

  # ... and identical to what the standalone path writes on a fresh scaffold
  # (the rpkg and monorepo/rpkg templates are kept in sync).
  minirextendr:::set_template_type("rpkg")
  expect_identical(rbuild, minirextendr:::mx_ignore_patterns("Rbuildignore"))
  expect_identical(gitign, minirextendr:::mx_ignore_patterns("gitignore"))
})

test_that("standalone path appends and dedupes into a pre-existing ignore file", {
  tmp <- withr::local_tempdir()
  writeLines(c("Package: ignpkg", "Version: 0.0.1"), file.path(tmp, "DESCRIPTION"))
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  minirextendr:::set_template_type("rpkg")
  withr::defer(minirextendr:::set_template_type("rpkg"))

  rb_patterns <- minirextendr:::mx_ignore_patterns("Rbuildignore")
  gi_patterns <- minirextendr:::mx_ignore_patterns("gitignore")

  # Pre-existing files with user content plus one already-present pattern
  rb_pre <- c("# user comment", "^my_own_file$", rb_patterns[1])
  gi_pre <- c("# user comment", "my_own_file", gi_patterns[1])
  writeLines(rb_pre, file.path(tmp, ".Rbuildignore"))
  writeLines(gi_pre, file.path(tmp, ".gitignore"))

  suppressMessages({
    minirextendr:::use_miniextendr_rbuildignore(path = tmp)
    minirextendr:::use_miniextendr_gitignore(path = tmp)
  })

  rb_after <- readLines(file.path(tmp, ".Rbuildignore"))
  gi_after <- readLines(file.path(tmp, ".gitignore"))

  # User content preserved verbatim at the top
  expect_identical(rb_after[seq_along(rb_pre)], rb_pre)
  expect_identical(gi_after[seq_along(gi_pre)], gi_pre)

  # Already-present pattern was not re-appended
  expect_identical(sum(rb_after == rb_patterns[1]), 1L)
  expect_identical(sum(gi_after == gi_patterns[1]), 1L)

  # All template patterns present, nothing duplicated
  expect_true(all(rb_patterns %in% rb_after))
  expect_true(all(gi_patterns %in% gi_after))
  expect_false(any(duplicated(rb_after)))
  expect_false(any(duplicated(gi_after)))
})
