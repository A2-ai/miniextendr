# Tests for system command execution helpers (R/system.R)

# Resolve a usable Rscript binary, or skip. Returns "" when unavailable.
find_rscript <- function() {
  rscript <- Sys.which("Rscript")
  if (!nzchar(rscript)) {
    cand <- file.path(
      R.home("bin"),
      if (.Platform$OS.type == "windows") "Rscript.exe" else "Rscript"
    )
    if (file.exists(cand)) rscript <- cand
  }
  rscript
}

test_that("run_with_logging captures a non-zero exit without leaking a warning", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found on PATH")

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
  skip_if_not(nzchar(rscript), "Rscript not found on PATH")

  result <- minirextendr:::run_with_logging(rscript, "--version", log_prefix = "test-zero")

  expect_true(result$success)
  expect_null(result$status)
})
