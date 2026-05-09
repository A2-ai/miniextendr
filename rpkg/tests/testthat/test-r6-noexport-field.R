# Tests for the `noexport` / `internal` opt-out on R6 active bindings.
#
# roxygen2 8.0.0's NEWS documents `@field name NULL` as the opt-out for an
# active binding, but in practice `r6_resolve_fields` still warns "Undocumented
# R6 active binding" for that form because `expected` is introspected from the
# class and isn't pruned in sync with the NULL-description discard. miniextendr
# therefore emits a minimal `#' @field <name> (internal)` description for
# `#[miniextendr(noexport)]` / `#[miniextendr(internal)]` active bindings —
# satisfies roxygen2 (no warning), and clearly tags the binding as internal in
# the rendered Rd.

# region: runtime behaviour

test_that("R6SensorReading exported active binding is accessible", {
  s <- R6SensorReading$new(3.14, 1023L)
  expect_equal(s$value, 3.14)
})

test_that("R6SensorReading noexported active binding is still accessible at runtime", {
  # The binding is marked internal in the docs but is still a real public
  # active binding on the R6 object — users can reach it.
  s <- R6SensorReading$new(2.71, 512L)
  expect_equal(s$raw_bytes, 512L)
})

# endregion

# region: documentation opt-out

test_that("noexported active binding renders as (internal) in the rendered Rd", {
  # The binding appears in `\section{Active bindings}` with the description
  # text `(internal)` — the minimal-real-description form that satisfies
  # roxygen2 8.0.0 without claiming to be user-facing documentation.
  rd_db <- tryCatch(
    tools::Rd_db("miniextendr"),
    error = function(e) NULL
  )
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  sensor_rd_name <- grep("R6SensorReading", names(rd_db), value = TRUE)[1]
  skip_if(is.na(sensor_rd_name), "R6SensorReading.Rd not found — package not documented")

  rd <- rd_db[[sensor_rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_true(
    grepl("raw_bytes", rd_text),
    info = paste(
      "`raw_bytes` (noexported active binding) must appear in R6SensorReading.Rd",
      "as a marked-internal entry. Got:\n", rd_text
    )
  )
  expect_match(
    rd_text,
    "\\\\item\\{\\\\code\\{raw_bytes\\}\\}\\{\\(internal\\)\\}",
    info = paste(
      "`raw_bytes` must render as `\\item{\\code{raw_bytes}}{(internal)}` in the",
      "Active bindings section. Got:\n", rd_text
    )
  )
})

test_that("exported active binding is present in the rendered Rd", {
  rd_db <- tryCatch(
    tools::Rd_db("miniextendr"),
    error = function(e) NULL
  )
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  sensor_rd_name <- grep("R6SensorReading", names(rd_db), value = TRUE)[1]
  skip_if(is.na(sensor_rd_name), "R6SensorReading.Rd not found — package not documented")

  rd <- rd_db[[sensor_rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_true(
    grepl("value", rd_text),
    info = paste(
      "`value` (exported active binding) must appear in R6SensorReading.Rd.",
      "Got:\n", rd_text
    )
  )
})

# endregion
