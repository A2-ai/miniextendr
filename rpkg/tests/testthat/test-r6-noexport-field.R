# Tests for `@field name NULL` opt-out for noexported R6 active bindings.
#
# roxygen2 8.0.0: `@field name NULL` in an R6 active-binding block tells roxygen2
# to exclude that binding from the generated `.Rd`.  miniextendr emits this form
# automatically when the Rust method is tagged `#[miniextendr(noexport)]` or
# `#[miniextendr(internal)]`.

# region: runtime behaviour

test_that("R6SensorReading exported active binding is accessible", {
  s <- R6SensorReading$new(3.14, 1023L)
  expect_equal(s$value, 3.14)
})

test_that("R6SensorReading noexported active binding is still accessible at runtime", {
  # The binding is excluded from docs but NOT from the R6 object.  Users can
  # still reach it — suppressing docs does not suppress the binding.
  s <- R6SensorReading$new(2.71, 512L)
  expect_equal(s$raw_bytes, 512L)
})

# endregion

# region: documentation opt-out

test_that("noexported active binding is absent from the rendered Rd", {
  # Read the installed .Rd for R6SensorReading and check that `raw_bytes`
  # does not appear in the Active bindings section.
  rd_db <- tools::Rd_db("miniextendr")
  sensor_rd_name <- grep("R6SensorReading", names(rd_db), value = TRUE)[1]
  skip_if(is.na(sensor_rd_name), "R6SensorReading.Rd not found — package not documented")

  rd <- rd_db[[sensor_rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_false(
    grepl("raw_bytes", rd_text),
    info = paste(
      "`raw_bytes` (noexported active binding) must not appear in R6SensorReading.Rd.",
      "Found in:\n", rd_text
    )
  )
})

test_that("exported active binding is present in the rendered Rd", {
  rd_db <- tools::Rd_db("miniextendr")
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
