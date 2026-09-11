test_that("installation reporting identifies the active copy rather than the first library", {
  root <- withr::local_tempdir()
  active <- file.path(root, "rv", "minirextendr")
  user <- file.path(root, "user", "minirextendr")
  dir.create(active, recursive = TRUE)
  dir.create(user, recursive = TRUE)
  writeLines(c("Package: minirextendr", "Version: 0.2.0", "RemoteSha: abc123"),
             file.path(active, "DESCRIPTION"))
  writeLines(c("Package: minirextendr", "Version: 0.1.0"),
             file.path(user, "DESCRIPTION"))
  copies <- minirextendr:::minirextendr_installations(active, c(dirname(user), dirname(active)))
  expect_length(copies, 2L)
  expect_identical(copies[[1L]]$path, normalizePath(active))
  expect_identical(copies[[1L]]$sha, "abc123")
  expect_identical(copies[[2L]]$version, "0.1.0")
  expect_true(is.na(copies[[2L]]$sha))
})
