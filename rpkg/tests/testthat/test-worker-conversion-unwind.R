# Real R errors must unwind converter locals and completed arguments (#1302).
# Main-thread error cleanup is tracked separately in #1507; only successful
# main-thread calls are used here as reentry controls.

conversion_unwind_condition <- function() {
  errorCondition(
    "R input lookup failed", call = quote(original_input_lookup()),
    class = c("conversion_input_error", "conversion_secondary"),
    values = c(1L, NA_integer_, 3L)
  )
}

conversion_unwind_env <- function(kind, condition, binding = "input") {
  force(condition)
  if (kind == "missing") {
    if (binding == "input") return((function(input) environment())())
    return((function(.ptr) environment())())
  }
  env <- new.env(parent = emptyenv())
  if (kind == "active") {
    makeActiveBinding(binding, function() stop(condition), env)
  } else {
    delayedAssign(binding, stop(condition), eval.env = environment(),
                  assign.env = env)
  }
  env
}

expect_conversion_unwind_condition <- function(actual, expected) {
  expect_identical(class(actual), class(expected))
  expect_identical(conditionMessage(actual), conditionMessage(expected))
  expect_identical(conditionCall(actual), conditionCall(expected))
  expect_identical(actual$values, expected$values)
}

test_that("active bindings and promises drop all worker conversion resources", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  methods <- list(ordinary = worker_conversion_binding,
                  nested = worker_conversion_binding_nested)
  conditions <- list(custom = conversion_unwind_condition(),
                     plain = "plain input lookup failed")
  success <- list2env(list(input = 1L), parent = emptyenv())

  for (method in names(methods)) {
    convert <- methods[[method]]
    for (kind in c("active", "promise")) {
      for (condition_kind in names(conditions)) {
        expected <- conditions[[condition_kind]]
        info <- paste(method, kind, condition_kind)
        env <- conversion_unwind_env(kind, expected)
        worker_conversion_reset()
        err <- tryCatch(convert(NULL, env), error = identity)
        if (condition_kind == "custom") {
          expect_conversion_unwind_condition(err, expected)
        } else {
          expect_identical(class(err), c("simpleError", "error", "condition"),
                           info = info)
          expect_identical(conditionMessage(err), expected, info = info)
        }
        expect_identical(worker_conversion_counts(), c(2L, 2L, 0L), info = info)
        expect_identical(telemetry_get_count(), 0L, info = info)

        expect_identical(convert(NULL, success), 42L, info = info)
        expect_identical(worker_conversion_counts(), c(4L, 4L, 1L), info = info)
        expect_identical(test_worker_input_condition("ok"), 42L, info = info)
      }
    }
  }
})


test_that("cleanup preserves plain error text without signalling another error", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  withr::local_options(list(warning.length = 8170L))
  methods <- list(ordinary = worker_conversion_binding,
                  nested = worker_conversion_binding_nested)
  messages <- c(
    percent = "input is 100% invalid: %s %n %d %%",
    multiline = "first input line\nsecond input line\nthird input line",
    multibyte = "input: caf\u00e9, \u00e6\u00f8\u00e5, \u65e5\u672c\u8a9e, \U0001f9ea",
    long_ascii = paste0(strrep("0123456789", 799L), "-end")
  )

  for (method in names(methods)) {
    convert <- methods[[method]]
    for (kind in c("active", "promise")) {
      for (message_kind in names(messages)) {
        expected <- messages[[message_kind]]
        info <- paste(method, kind, message_kind)
        env <- conversion_unwind_env(kind, expected)
        observed <- list()
        worker_conversion_reset()
        err <- tryCatch(
          withCallingHandlers(convert(NULL, env), error = function(condition) {
            observed[[length(observed) + 1L]] <<- condition
          }),
          error = identity
        )
        expect_identical(class(err), c("simpleError", "error", "condition"),
                         info = info)
        expect_identical(conditionMessage(err), expected, info = info)
        expect_identical(length(observed), 1L, info = info)
        expect_identical(conditionMessage(observed[[1L]]), expected, info = info)
        expect_identical(conditionCall(err), conditionCall(observed[[1L]]),
                         info = info)
        expect_identical(worker_conversion_counts(), c(2L, 2L, 0L), info = info)
        expect_identical(telemetry_get_count(), 0L, info = info)
      }
    }
  }
  expect_identical(test_worker_input_condition("ok"), 42L)
})

test_that("on.exit can reenter the worker and collect during an R error", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  methods <- list(ordinary = worker_conversion_binding,
                  nested = worker_conversion_binding_nested)
  expected <- conversion_unwind_condition()
  success <- list2env(list(input = 1L), parent = emptyenv())

  for (method in names(methods)) {
    convert <- methods[[method]]
    for (torture in c(FALSE, TRUE)) {
      info <- paste(method, "gctorture", torture)
      env <- new.env(parent = emptyenv())
      # Make a fresh condition here so the expected value does not root it.
      makeActiveBinding("input", function() {
        stop(conversion_unwind_condition())
      }, env)
      nested_result <- counts_before_reentry <- counts_after_reentry <- NULL
      fail_with_exit <- function() {
        on.exit({
          counts_before_reentry <<- worker_conversion_counts()
          nested_result <<- worker_conversion_binding(NULL, success)
          gc()
          counts_after_reentry <<- worker_conversion_counts()
        })
        convert(NULL, env)
      }
      worker_conversion_reset()
      err <- tryCatch({
        if (torture) gctorture(TRUE)
        tryCatch(fail_with_exit(), error = identity)
      }, finally = gctorture(FALSE))

      expect_conversion_unwind_condition(err, expected)
      expect_identical(counts_before_reentry, c(2L, 2L, 0L), info = info)
      expect_identical(nested_result, 42L, info = info)
      expect_identical(counts_after_reentry, c(4L, 4L, 1L), info = info)
      expect_identical(worker_conversion_counts(), c(4L, 4L, 1L), info = info)
      expect_identical(telemetry_get_count(), 0L, info = info)
      expect_identical(test_worker_input_condition("ok"), 42L, info = info)
    }
  }
})


test_that("warnings converted to errors unwind worker conversion resources", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  withr::local_options(warn = 2)
  worker_conversion_reset()

  err <- tryCatch(worker_conversion_warning(NULL, NULL), error = identity)
  expect_s3_class(err, "error")
  expect_match(conditionMessage(err), "conversion warning", fixed = TRUE)
  expect_false(inherits(err, "rust_error"))
  expect_identical(worker_conversion_counts(), c(2L, 2L, 0L))
  expect_identical(telemetry_get_count(), 0L)
  expect_identical(test_worker_input_condition("ok"), 42L)
})

test_that("ordinary conversion warnings permit worker dispatch", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  withr::local_options(warn = 0)
  worker_conversion_reset()
  observed <- list()

  result <- withCallingHandlers(worker_conversion_warning(NULL, NULL),
    warning = function(condition) {
      observed[[length(observed) + 1L]] <<- condition
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(result, 42L)
  expect_identical(length(observed), 1L)
  expect_s3_class(observed[[1L]], "warning")
  expect_identical(conditionMessage(observed[[1L]]), "conversion warning")
  expect_identical(worker_conversion_counts(), c(2L, 2L, 1L))
  expect_identical(telemetry_get_count(), 0L)
})

test_that("missing R arguments unwind completed and local conversion resources", {
  env <- conversion_unwind_env("missing", conversion_unwind_condition())
  worker_conversion_reset()
  err <- tryCatch(worker_conversion_binding(NULL, env), error = identity)
  expect_s3_class(err, "error")
  expect_match(conditionMessage(err), "input", fixed = TRUE)
  expect_match(conditionMessage(err), "missing", fixed = TRUE)
  expect_false(inherits(err, "rust_error"))
  expect_identical(worker_conversion_counts(), c(2L, 2L, 0L))
  expect_identical(test_worker_input_condition("ok"), 42L)
})

test_that("class handle lookup participates in worker conversion cleanup", {
  expected <- conversion_unwind_condition()
  for (kind in c("active", "promise", "missing")) {
    # ExternalPtr conversion first reads a direct .ptr environment binding.
    env <- conversion_unwind_env(kind, expected, binding = ".ptr")
    worker_conversion_reset()
    err <- tryCatch(worker_conversion_class(NULL, env), error = identity)
    if (kind == "missing") {
      expect_s3_class(err, "error")
      expect_match(conditionMessage(err), ".ptr", fixed = TRUE)
      expect_match(conditionMessage(err), "missing", fixed = TRUE)
    } else {
      expect_conversion_unwind_condition(err, expected)
    }
    expect_identical(worker_conversion_counts(), c(1L, 1L, 0L), info = kind)
  }

  success <- list2env(list(.ptr = extptr_counter_new(19L)), parent = emptyenv())
  worker_conversion_reset()
  expect_identical(worker_conversion_class(NULL, success), 42L)
  expect_identical(worker_conversion_counts(), c(1L, 1L, 1L))
})

test_that("repeated conversion errors leave the scope and worker reusable", {
  telemetry_install_counter()
  withr::defer(telemetry_clear_hook())
  expected <- conversion_unwind_condition()
  failing <- conversion_unwind_env("active", expected)
  success <- list2env(list(input = 1L), parent = emptyenv())
  worker_conversion_reset()

  for (i in seq_len(5L)) {
    err <- tryCatch(worker_conversion_binding(NULL, failing), error = identity)
    expect_conversion_unwind_condition(err, expected)
    expect_identical(worker_conversion_counts(), c(2L * i, 2L * i, 0L))
  }
  expect_identical(telemetry_get_count(), 0L)
  expect_identical(worker_conversion_binding(NULL, success), 42L)
  expect_identical(worker_conversion_counts(), c(12L, 12L, 1L))
  expect_identical(main_conversion_binding(NULL, success), 42L)
  expect_identical(worker_conversion_counts(), c(14L, 14L, 2L))
  expect_identical(miniextendr:::gc_stress_worker_roundtrip(), 2234L)

  # A subsequent real Rust panic still has its ordinary transport and telemetry.
  err <- tryCatch(test_worker_input_condition("panic"), error = identity)
  expect_identical(err$kind, "panic")
  expect_match(conditionMessage(err), "panic before worker dispatch", fixed = TRUE)
  expect_identical(telemetry_get_count(), 1L)
  expect_identical(test_worker_input_condition("ok"), 42L)
})

test_that("conversion fences survive reentry through main and worker wrappers", {
  expected <- conversion_unwind_condition()
  success <- list2env(list(input = 1L), parent = emptyenv())
  for (nested in c("main", "worker")) {
    for (fail_after in c(FALSE, TRUE)) {
      nested_result <- NULL
      env <- new.env(parent = emptyenv())
      makeActiveBinding("input", function() {
        nested_result <<- if (nested == "main") {
          main_conversion_binding(NULL, success)
        } else {
          test_worker_input_condition("ok")
        }
        if (fail_after) stop(expected)
        1L
      }, env)
      worker_conversion_reset()
      result <- tryCatch(worker_conversion_binding(NULL, env), error = identity)
      expect_identical(nested_result, 42L)
      if (fail_after) {
        expect_conversion_unwind_condition(result, expected)
      } else {
        expect_identical(result, 42L)
      }
      resources <- if (nested == "main") 4L else 2L
      dispatched <- as.integer(nested == "main") + as.integer(!fail_after)
      expect_identical(worker_conversion_counts(),
                       c(resources, resources, dispatched))
      expect_identical(test_worker_input_condition("ok"), 42L)
    }
  }
})

test_that("RNG cleanup commits the converter draw after an R input error", {
  withr::local_seed(1302L)
  expected <- conversion_unwind_condition()
  failing <- conversion_unwind_env("active", expected)
  invisible(runif(1L))
  expected_seed <- .Random.seed
  expected_next <- runif(1L)
  set.seed(1302L)
  saved_seed <- .Random.seed
  saved_bytes <- serialize(saved_seed, NULL)
  worker_conversion_reset()

  err <- tryCatch(worker_conversion_binding_rng(NULL, failing), error = identity)
  expect_conversion_unwind_condition(err, expected)
  expect_identical(worker_conversion_counts(), c(2L, 2L, 0L))
  expect_identical(.Random.seed, expected_seed)
  expect_identical(serialize(saved_seed, NULL), saved_bytes)
  expect_identical(runif(1L), expected_next)

  success <- list2env(list(input = 1L), parent = emptyenv())
  set.seed(1302L)
  expect_identical(worker_conversion_binding_rng(NULL, success), 42L)
  expect_identical(.Random.seed, expected_seed)
  expect_identical(worker_conversion_counts(), c(4L, 4L, 1L))
})

test_that("pending R continuations survive allocating drops and shared-seed GC", {
  withr::local_seed(1507L)
  expected <- conversion_unwind_condition()
  failing <- conversion_unwind_env("active", expected)
  invisible(runif(3L))
  expected_seed <- .Random.seed
  expected_next <- runif(1L)
  set.seed(1507L)
  saved_seed <- .Random.seed
  saved_bytes <- serialize(saved_seed, NULL)
  errors <- vector("list", 3L)
  worker_conversion_reset()

  tryCatch({
    gctorture(TRUE)
    for (i in seq_along(errors)) {
      errors[[i]] <- tryCatch(worker_conversion_binding_rng(NULL, failing),
                              error = identity)
    }
  }, finally = gctorture(FALSE))

  for (err in errors) expect_conversion_unwind_condition(err, expected)
  expect_identical(worker_conversion_counts(), c(6L, 6L, 0L))
  expect_identical(.Random.seed, expected_seed)
  expect_identical(serialize(saved_seed, NULL), saved_bytes)
  expect_identical(runif(1L), expected_next)
  expect_identical(test_worker_input_condition("ok"), 42L)
})
