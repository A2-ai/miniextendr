# Tests for the webR namespace-import lint (#925).
#
# All fixtures are hermetic: fake "installed" dependencies live in a temp
# library passed via lib_paths, so the tests never depend on what happens to
# be installed in the running session's .libPaths().

# Creates a fake installed package in `lib`: a directory with a DESCRIPTION
# (and optionally a libs/ dir and Depends/Imports fields), which is all the
# static probe ever reads.
make_fake_installed_pkg <- function(lib, name,
                                    needs_compilation = NULL,
                                    with_libs = FALSE,
                                    imports = NULL,
                                    depends = NULL) {
  pkg_dir <- file.path(lib, name)
  dir.create(pkg_dir, recursive = TRUE)
  desc <- c(
    paste0("Package: ", name),
    "Version: 0.1.0",
    "Title: Fake Package",
    "Description: Fixture for webr-lint tests.",
    "License: MIT + file LICENSE"
  )
  if (!is.null(needs_compilation)) {
    desc <- c(desc, paste0("NeedsCompilation: ", needs_compilation))
  }
  if (!is.null(imports)) {
    desc <- c(desc, paste0("Imports: ", paste(imports, collapse = ", ")))
  }
  if (!is.null(depends)) {
    desc <- c(desc, paste0("Depends: ", paste(depends, collapse = ", ")))
  }
  writeLines(desc, file.path(pkg_dir, "DESCRIPTION"))
  if (with_libs) {
    dir.create(file.path(pkg_dir, "libs"))
  }
  invisible(pkg_dir)
}

# Creates the package under lint: DESCRIPTION + a NAMESPACE built from the
# given directive lines.
make_target_pkg <- function(tmp, namespace_lines, pkg_name = "webrlinttarget") {
  pkg_dir <- file.path(tmp, pkg_name)
  dir.create(pkg_dir, recursive = TRUE)
  writeLines(
    c(
      paste0("Package: ", pkg_name),
      "Version: 0.1.0",
      "Title: Webr Lint Target",
      "Description: Fixture package for webr-lint tests.",
      "License: MIT + file LICENSE",
      "Encoding: UTF-8"
    ),
    file.path(pkg_dir, "DESCRIPTION")
  )
  writeLines(namespace_lines, file.path(pkg_dir, "NAMESPACE"))
  invisible(pkg_dir)
}

test_that("webr_pkg_compiled_status probes libs/, NeedsCompilation, and absence", {
  lib <- withr::local_tempdir("webr-lint-lib-")
  make_fake_installed_pkg(lib, "haslibs", with_libs = TRUE)
  make_fake_installed_pkg(lib, "needsyes", needs_compilation = "yes")
  make_fake_installed_pkg(lib, "needsno", needs_compilation = "no")
  make_fake_installed_pkg(lib, "nofield")

  expect_true(webr_pkg_compiled_status("haslibs", lib))
  expect_true(webr_pkg_compiled_status("needsyes", lib))
  expect_false(webr_pkg_compiled_status("needsno", lib))
  # NeedsCompilation absent -> cannot tell
  expect_identical(webr_pkg_compiled_status("nofield", lib), NA)
  # not installed at all -> cannot tell
  expect_identical(webr_pkg_compiled_status("absentpkg", lib), NA)
})

test_that("webr_hard_deps strips version constraints and the R pseudo-dep", {
  lib <- withr::local_tempdir("webr-lint-lib-")
  make_fake_installed_pkg(
    lib, "umbrellaish",
    needs_compilation = "no",
    imports = c("depone (>= 1.0)", "deptwo"),
    depends = c("R (>= 4.1)", "depthree")
  )

  deps <- webr_hard_deps("umbrellaish", lib)
  expect_setequal(deps, c("depone", "deptwo", "depthree"))
})

test_that("webr_load_graph_status walks pure-R umbrellas down to compiled deps", {
  lib <- withr::local_tempdir("webr-lint-lib-")
  make_fake_installed_pkg(lib, "compiledleaf", with_libs = TRUE)
  make_fake_installed_pkg(
    lib, "pureumbrella",
    needs_compilation = "no", imports = "compiledleaf"
  )
  make_fake_installed_pkg(lib, "pureleaf", needs_compilation = "no")
  make_fake_installed_pkg(
    lib, "allpure",
    needs_compilation = "no", imports = "pureleaf"
  )

  umbrella <- webr_load_graph_status("pureumbrella", lib)
  expect_identical(umbrella$status, "compiled")
  expect_identical(umbrella$via, "compiledleaf")

  clean <- webr_load_graph_status("allpure", lib)
  expect_identical(clean$status, "pure-r")
  expect_length(clean$via, 0L)
})

test_that("webr_load_graph_status falls back to the curated list for uninstalled deps", {
  lib <- withr::local_tempdir("webr-lint-lib-")

  # data.table is on the curated list and not installed in the temp lib.
  denied <- webr_load_graph_status("data.table", lib)
  expect_identical(denied$status, "known-compiled")
  expect_identical(denied$via, "data.table")

  # An unheard-of package is unverifiable, not flagged.
  unknown <- webr_load_graph_status("definitelynotapkg", lib)
  expect_identical(unknown$status, "unknown")
  expect_identical(unknown$via, "definitelynotapkg")
})

test_that("webr_import_findings classifies NAMESPACE directives and skips base", {
  lib <- withr::local_tempdir("webr-lint-lib-")
  make_fake_installed_pkg(lib, "fakecompiled", with_libs = TRUE)
  make_fake_installed_pkg(lib, "fakepure", needs_compilation = "no")

  tmp <- withr::local_tempdir("webr-lint-target-")
  pkg_dir <- make_target_pkg(tmp, c(
    "importFrom(fakecompiled, something)",
    "import(fakepure)",
    "importFrom(data.table, fread)",
    "importFrom(definitelynotapkg, mystery)",
    "importFrom(stats, median)",
    "importFrom(utils, head)"
  ))

  findings <- webr_import_findings(pkg_dir, lib)

  # Base-priority imports never appear.
  expect_false(any(findings$package %in% c("stats", "utils")))
  expect_setequal(
    findings$package,
    c("fakecompiled", "fakepure", "data.table", "definitelynotapkg")
  )

  status_of <- function(pkg) findings$status[findings$package == pkg]
  expect_identical(status_of("fakecompiled"), "compiled")
  expect_identical(status_of("fakepure"), "pure-r")
  expect_identical(status_of("data.table"), "known-compiled")
  expect_identical(status_of("definitelynotapkg"), "unknown")

  directive_of <- function(pkg) findings$directive[findings$package == pkg]
  expect_identical(directive_of("fakecompiled"), "importFrom")
  expect_identical(directive_of("fakepure"), "import")
})

test_that("webr_import_findings returns zero rows for base-only or empty NAMESPACE", {
  tmp <- withr::local_tempdir("webr-lint-target-")

  base_only <- make_target_pkg(
    tmp, "importFrom(stats, median)",
    pkg_name = "baseonlypkg"
  )
  expect_identical(nrow(webr_import_findings(base_only, character())), 0L)

  no_imports <- make_target_pkg(
    tmp, "export(placeholder)",
    pkg_name = "noimportspkg"
  )
  expect_identical(nrow(webr_import_findings(no_imports, character())), 0L)
})

test_that("webr_report_findings maps statuses onto pass/warn/fail", {
  findings <- data.frame(
    package = c("a", "b", "c", "d"),
    directive = c("importFrom", "importFrom", "import", "importFrom"),
    status = c("compiled", "known-compiled", "pure-r", "unknown"),
    via = c("a", "b", "", "d"),
    stringsAsFactors = FALSE
  )

  results <- suppressMessages(webr_report_findings(findings))
  expect_length(results$fail, 1L)
  expect_match(results$fail, "importFrom\\(a\\)")
  expect_length(results$warn, 1L)
  expect_match(results$warn, "importFrom\\(b\\)")
  expect_length(results$pass, 1L)
  expect_match(results$pass, "import\\(c\\)")
})

test_that("miniextendr_webr_import_lint runs end-to-end on a fixture package", {
  lib <- withr::local_tempdir("webr-lint-lib-")
  make_fake_installed_pkg(lib, "fakecompiled", with_libs = TRUE)

  tmp <- withr::local_tempdir("webr-lint-target-")
  pkg_dir <- make_target_pkg(tmp, c(
    "importFrom(fakecompiled, something)",
    "importFrom(stats, median)"
  ))

  findings <- suppressMessages(
    miniextendr_webr_import_lint(pkg_dir, lib_paths = lib)
  )
  expect_s3_class(findings, "data.frame")
  expect_identical(findings$package, "fakecompiled")
  expect_identical(findings$status, "compiled")
})
