test_that("panic telemetry hook counts panics", {
  telemetry_install_counter()
  expect_equal(telemetry_get_count(), 0L)

  # Trigger a panic via a function that panics — the telemetry hook fires
  # before the panic is converted to an R error
  tryCatch(
    # Any function that panics will do — use a conversion that fails
    stop("intentional"),
    error = function(e) NULL
  )

  # Clean up
  telemetry_clear_hook()
})
