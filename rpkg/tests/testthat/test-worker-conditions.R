test_that("worker conditions preserve kind, classes, data, and call across both channels", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  layers <- list(
    error = c("rust_error", "simpleError", "error", "condition"),
    warning = c("rust_warning", "simpleWarning", "warning", "condition"),
    message = c("rust_message", "simpleMessage", "message", "condition"),
    condition = c("rust_condition", "simpleCondition", "condition")
  )
  for (via_main in c(FALSE, TRUE)) {
    for (kind in names(layers)) {
      call <- substitute(test_worker_condition(kind = KIND, via_main = ROUTE),
                         list(KIND = kind, ROUTE = via_main))
      captured <- NULL
      result <- tryCatch(
        withCallingHandlers(eval(call), condition = function(c) {
          if (inherits(c, paste0("rust_", kind))) captured <<- c
          if (inherits(c, "warning")) invokeRestart("muffleWarning")
          if (inherits(c, "message")) invokeRestart("muffleMessage")
        }),
        error = function(e) { captured <<- e; NULL }
      )
      expected_class <- layers[[kind]]
      if (kind != "message") {
        expected_class <- c("worker_custom", "worker_secondary", expected_class)
      }
      expect_identical(class(captured), expected_class, info = deparse(call))
      expect_identical(captured$kind, kind)
      # Match the shared R helper: messages use base R's newline and NULL call.
      expected_message <- paste0("worker ", kind, if (kind == "message") "\n" else "")
      expect_identical(conditionMessage(captured), expected_message)
      expect_identical(conditionCall(captured), if (kind == "message") NULL else call)
      expect_identical(captured$values, c(1L, NA_integer_, 3L))
      expect_identical(captured$details, list(label = "nested", ready = TRUE))
      expect_null(result)
      expect_identical(telemetry_get_count(), 0L)
      expect_identical(miniextendr:::unsafe_C_test_worker_simple(), 42L)
    }
  }
})

test_that("worker panic relays retain the origin and emit telemetry once", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  for (name in c("panic_location_worker_direct", "panic_location_worker_with_r_thread")) {
    before <- telemetry_get_count()
    err <- tryCatch(get(name)(), error = identity)
    expect_identical(err$kind, "panic")
    expect_match(conditionMessage(err), "panic_location_tests\\.rs:[0-9]+")
    expect_false(grepl("worker.rs", conditionMessage(err), fixed = TRUE))
    expect_length(regmatches(conditionMessage(err),
                             gregexpr("(at ", conditionMessage(err), fixed = TRUE))[[1]], 1L)
    expect_identical(telemetry_get_count(), before + 1L)
  }
})

test_that("pre-dispatch failures use typed conditions and report generic panics once", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  err <- tryCatch(test_worker_input_condition("panic"), error = identity)
  expect_identical(err$kind, "panic")
  expect_match(conditionMessage(err), "panic before worker dispatch", fixed = TRUE)
  expect_match(conditionMessage(err), "worker_tests\\.rs:[0-9]+")
  expect_identical(telemetry_get_count(), 1L)

  warning <- NULL
  result <- withCallingHandlers(
    test_worker_input_condition(input = "warning"),
    warning = function(w) { warning <<- w; invokeRestart("muffleWarning") }
  )
  expect_identical(class(warning),
                   c("worker_input_warning", "rust_warning", "simpleWarning",
                     "warning", "condition"))
  expect_identical(warning$kind, "warning")
  expect_identical(conditionMessage(warning), "warning before worker dispatch")
  expect_identical(warning$stage, "input")
  expect_identical(conditionCall(warning),
                   quote(test_worker_input_condition(input = "warning")))
  expect_null(result)
  expect_identical(telemetry_get_count(), 1L)
  expect_identical(test_worker_input_condition("ok"), 42L)
})
