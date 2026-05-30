# Tests for system command execution helpers (R/system.R)

# Resolve the Rscript that ships with the running R, or "" if absent.
# Use R.home("bin") rather than Sys.which("Rscript"): under `R CMD check` the
# bare name resolves to a wrapper on PATH that exits non-zero with "'Rscript'
# should not be used without a path -- see par. 1.6 of the manual". Addressing
# it by its home path (the method Writing R Extensions sec. 1.6 prescribes)
# invokes the real interpreter and sidesteps the wrapper.
find_rscript <- function() {
  exe <- if (.Platform$OS.type == "windows") "Rscript.exe" else "Rscript"
  cand <- file.path(R.home("bin"), exe)
  if (file.exists(cand)) cand else ""
}

test_that("run_with_logging captures a non-zero exit without leaking a warning", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  # Point Rscript at a file that does not exist: deterministic non-zero exit,
  # no shell metacharacters in the argument. base R raises a
  # "running command ... had status N" warning on non-zero exit when output is
  # captured; run_with_logging() suppresses it because the status is reported
  # through $status/$success instead. Regression guard for #798.
  missing <- file.path(tempdir(), "minirextendr-no-such-script-9f3c.R")

  result <- expect_no_warning(
    minirextendr:::run_with_logging(rscript, missing, log_prefix = "test-nonzero")
  )

  # Status is still reported correctly despite the silenced warning.
  expect_false(result$success)
  expect_false(is.null(result$status))
  expect_gt(result$status, 0L)
})

test_that("run_with_logging reports success for a zero-exit command", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  # Run an *empty* R script: a no-op program that exits 0. The path is
  # metacharacter-free, mirroring the non-zero sibling above, since system2()
  # with captured output goes through `sh -c` and does not shell-quote its
  # arguments.
  empty <- file.path(tempdir(), "minirextendr-empty-9f3c.R")
  file.create(empty)
  on.exit(unlink(empty), add = TRUE)

  result <- minirextendr:::run_with_logging(rscript, empty, log_prefix = "test-zero")

  expect_true(result$success)
  expect_null(result$status)
})
