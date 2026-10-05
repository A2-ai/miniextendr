# The monorepo scaffold's tools/bump-version.R keeps DESCRIPTION and the
# Cargo.toml versions in lockstep. It leaves configure.ac alone: nothing reads
# the AC_INIT version, and rewriting it made the upgrade's configure.ac drift
# check warn on every run (#1789).

test_that("bump-version.R updates DESCRIPTION and Cargo.toml, not configure.ac", {
  ws <- withr::local_tempdir("bump-version-")
  dir.create(file.path(ws, "tools"))
  dir.create(file.path(ws, "rpkg", "src", "rust"), recursive = TRUE)
  file.copy(
    system.file("templates", "monorepo", "tools", "bump-version.R",
                package = "minirextendr", mustWork = TRUE),
    file.path(ws, "tools", "bump-version.R")
  )
  writeLines(c("Package: demo", "Version: 0.1.0"), file.path(ws, "rpkg", "DESCRIPTION"))
  writeLines(c("[package]", 'name = "demo"', 'version = "0.1.0"'),
             file.path(ws, "rpkg", "src", "rust", "Cargo.toml"))
  configure_ac <- c("AC_INIT([demo], [0.1.0])", "AC_OUTPUT")
  writeLines(configure_ac, file.path(ws, "rpkg", "configure.ac"))

  out <- withr::with_dir(ws, system2(
    file.path(R.home("bin"), "Rscript"),
    c("tools/bump-version.R", "rpkg", "--set=0.2.0.9000"),
    stdout = TRUE, stderr = TRUE
  ))
  expect_null(attr(out, "status"))

  expect_identical(readLines(file.path(ws, "rpkg", "DESCRIPTION"))[[2]], "Version: 0.2.0.9000")
  expect_true('version = "0.2.0-9000"' %in%
                readLines(file.path(ws, "rpkg", "src", "rust", "Cargo.toml")))
  expect_identical(readLines(file.path(ws, "rpkg", "configure.ac")), configure_ac)
})
