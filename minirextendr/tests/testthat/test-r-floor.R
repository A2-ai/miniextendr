# R version floor in scaffolded DESCRIPTIONs (#1366)
#
# Any package linking miniextendr-api inherits R >= 4.5 at load time
# (R_getVarEx, see MX_R_FLOOR in R/utils.R), so every scaffold path must
# declare the floor, and the standalone path must merge it into pre-existing
# DESCRIPTIONs without lowering a stricter one. The CLI mirror is guarded
# from the Rust side (miniextendr-cli/src/scaffold.rs,
# r_floor_matches_minirextendr_and_rpkg).

floor_entry <- sprintf("R (>= %s)", minirextendr:::MX_R_FLOOR)

local_desc <- function(lines, .local_envir = parent.frame()) {
  withr::local_tempfile(lines = lines, .local_envir = .local_envir)
}

# mx_desc_get_field() returns a named character (read.dcf column name);
# drop the name so expect_identical() compares values only.
get_depends <- function(path) {
  unname(minirextendr:::mx_desc_get_field("Depends", file = path))
}

# region: mx_desc_ensure_r_floor() merge semantics ---------------------------

test_that("mx_desc_ensure_r_floor() adds Depends when absent", {
  path <- local_desc(c("Package: p", "Version: 1.0.0"))
  expect_true(minirextendr:::mx_desc_ensure_r_floor(path))
  expect_identical(
    get_depends(path),
    floor_entry
  )
})

test_that("mx_desc_ensure_r_floor() prepends to a Depends without an R entry", {
  path <- local_desc(c("Package: p", "Depends: methods, utils"))
  expect_true(minirextendr:::mx_desc_ensure_r_floor(path))
  expect_identical(
    get_depends(path),
    paste0(floor_entry, ", methods, utils")
  )
})

test_that("mx_desc_ensure_r_floor() raises a lower or missing floor", {
  for (lower in c("R (>= 4.4)", "R (>=4.4)", "R (> 4.4)", "R")) {
    path <- local_desc(c("Package: p", paste0("Depends: ", lower, ", methods")))
    expect_true(minirextendr:::mx_desc_ensure_r_floor(path), info = lower)
    expect_identical(
      get_depends(path),
      paste0(floor_entry, ", methods"),
      info = lower
    )
  }
})

test_that("mx_desc_ensure_r_floor() keeps an equal or higher floor untouched", {
  for (kept in c("R (>= 4.5)", "R (>= 4.5.0)", "R (>= 4.6)", "R (== 4.4)")) {
    path <- local_desc(c("Package: p", paste0("Depends: ", kept, ", methods")))
    before <- readLines(path, warn = FALSE)
    expect_false(minirextendr:::mx_desc_ensure_r_floor(path), info = kept)
    expect_identical(readLines(path, warn = FALSE), before, info = kept)
  }
})

# endregion -------------------------------------------------------------------

# region: the three scaffold paths declare the floor --------------------------

test_that("use_miniextendr_description() writes the floor into a Depends-less DESCRIPTION", {
  tmp <- withr::local_tempdir()
  writeLines(c(
    "Package: floorpkg",
    "Title: Floor Test",
    "Version: 1.0.0",
    "License: MIT + file LICENSE",
    "Encoding: UTF-8"
  ), file.path(tmp, "DESCRIPTION"))

  suppressMessages(use_miniextendr_description(path = tmp))

  expect_identical(
    get_depends(file.path(tmp, "DESCRIPTION")),
    floor_entry
  )
})

test_that("use_miniextendr_description() never lowers a stricter existing floor", {
  tmp <- withr::local_tempdir()
  writeLines(c(
    "Package: floorpkg",
    "Title: Floor Test",
    "Version: 1.0.0",
    "Depends: R (>= 4.6), methods",
    "License: MIT + file LICENSE",
    "Encoding: UTF-8"
  ), file.path(tmp, "DESCRIPTION"))

  suppressMessages(use_miniextendr_description(path = tmp))

  expect_identical(
    get_depends(file.path(tmp, "DESCRIPTION")),
    "R (>= 4.6), methods"
  )
})

test_that("create_rpkg_subdirectory() DESCRIPTION declares the floor", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  minirextendr:::set_template_type("monorepo")
  on.exit(minirextendr:::set_template_type("rpkg"), add = TRUE)

  data <- list(
    package = "mypkg",
    package_rs = "mypkg",
    Package = "Mypkg",
    crate_name = "mypkg-rs",
    rpkg_name = "rpkg",
    features_var = "CARGO_FEATURES",
    year = "2026"
  )
  suppressMessages(
    minirextendr:::create_rpkg_subdirectory(data, rpkg_name = "rpkg")
  )

  desc <- readLines(file.path(tmp, "rpkg", "DESCRIPTION"), warn = FALSE)
  expect_true(any(desc == paste0("Depends: ", floor_entry)))
})

test_that("scaffold_inline_package() DESCRIPTION declares the floor", {
  tmp <- withr::local_tempdir()
  hash <- "floorhash1234567890abcdef1234567"

  minirextendr:::scaffold_inline_package(
    "#[miniextendr]\npub fn f() -> i32 { 1 }\n",
    hash, character(), "mxfloor", "mxfloor",
    tmp, quiet = TRUE
  )

  desc <- readLines(fs::path(tmp, hash, "pkg", "DESCRIPTION"), warn = FALSE)
  expect_true(any(desc == paste0("Depends: ", floor_entry)))
})

# endregion -------------------------------------------------------------------

# region: cross-source parity --------------------------------------------------

test_that("MX_R_FLOOR matches rpkg/DESCRIPTION's Depends floor", {
  skip_if_no_local_repo()

  desc <- readLines(
    file.path(find_miniextendr_repo(), "rpkg", "DESCRIPTION"),
    warn = FALSE
  )
  expect_true(any(trimws(desc) == paste0("Depends: ", floor_entry)))
})

# endregion -------------------------------------------------------------------
