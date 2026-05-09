# Tests for use_release_workflow()

test_that("use_release_workflow() creates .github/workflows/r-release.yml", {
  tmp <- tempfile("use-release-wf-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  result <- use_release_workflow(path = tmp)

  expected <- file.path(tmp, ".github", "workflows", "r-release.yml")
  expect_true(fs::file_exists(expected))
  expect_equal(result, expected)
})

test_that("use_release_workflow() content matches the bundled template", {
  tmp <- tempfile("use-release-wf-content-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  use_release_workflow(path = tmp)

  written <- readLines(file.path(tmp, ".github", "workflows", "r-release.yml"),
                       warn = FALSE)
  template <- readLines(
    system.file("templates", "r-release.yml", package = "minirextendr"),
    warn = FALSE
  )
  expect_identical(written, template)
})

test_that("use_release_workflow() errors when file exists and overwrite = FALSE", {
  tmp <- tempfile("use-release-wf-no-overwrite-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  use_release_workflow(path = tmp)

  expect_error(
    use_release_workflow(path = tmp, overwrite = FALSE),
    "already exists"
  )
})

test_that("use_release_workflow() succeeds when overwrite = TRUE", {
  tmp <- tempfile("use-release-wf-overwrite-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  use_release_workflow(path = tmp)

  expect_no_error(use_release_workflow(path = tmp, overwrite = TRUE))

  expected <- file.path(tmp, ".github", "workflows", "r-release.yml")
  expect_true(fs::file_exists(expected))
})
