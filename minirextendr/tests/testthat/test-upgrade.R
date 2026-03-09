# Tests for upgrade functionality

test_that("upgrade_gitignore removes obsolete entries", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)

  # Create a .gitignore with some current and obsolete entries
  gitignore <- file.path(tmp, ".gitignore")
  writeLines(c(
    "src/rust/target/",
    "src/entrypoint.c",
    "src/mx_abi.c",
    "src/rust/document.rs",
    "vendor/"
  ), gitignore)

  # Mock the gitignore template lookup to avoid package dependency
  local_mocked_bindings(
    use_miniextendr_gitignore = function(...) invisible(),
    .package = "minirextendr"
  )

  minirextendr:::upgrade_gitignore()

  result <- readLines(gitignore)
  expect_true("src/rust/target/" %in% result)
  expect_true("vendor/" %in% result)
  expect_false("src/entrypoint.c" %in% result)
  expect_false("src/mx_abi.c" %in% result)
  expect_false("src/rust/document.rs" %in% result)
})

test_that("check_configure_ac_drift warns on missing elements", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)

  # Create a minimal configure.ac missing key elements
  writeLines(c(
    "AC_INIT([mypkg])",
    "AC_OUTPUT"
  ), file.path(tmp, "configure.ac"))

  expect_warning(
    minirextendr:::check_configure_ac_drift(),
    "configure.ac may be outdated"
  )
})
