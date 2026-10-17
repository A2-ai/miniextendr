# Call suppression belongs to each impl block or individual method (#1868).
method_call_cases <- function() {
  list(
    env = list(
      new = NoCallEnv$new,
      read = function(x, value, mode = "fast") x$nc_env_read(value, mode),
      plain = function(x, value) x$nc_env_plain(value),
      quiet = function(x, value) x$nc_env_quiet(value),
      static = NoCallEnv$static_probe
    ),
    r6 = list(
      new = NoCallR6$new,
      read = function(x, value, mode = "fast") x$nc_r6_read(value, mode),
      plain = function(x, value) x$nc_r6_plain(value),
      quiet = function(x, value) x$nc_r6_quiet(value),
      static = NoCallR6$static_probe
    ),
    s3 = list(new = new_nocalls3, read = nc_s3_read, plain = nc_s3_plain,
              quiet = nc_s3_quiet, static = nocalls3_static_probe),
    s4 = list(new = NoCallS4, read = s4_nc_s4_read, plain = s4_nc_s4_plain,
              quiet = s4_nc_s4_quiet, static = NoCallS4_static_probe),
    s7 = list(new = NoCallS7, read = nc_s7_read, plain = nc_s7_plain,
              quiet = nc_s7_quiet, static = NoCallS7_static_probe)
  )
}

method_error <- function(expr) {
  e <- tryCatch(expr, error = identity)
  expect_s3_class(e, "error")
  e
}

method_warning <- function(expr) {
  warnings <- list()
  value <- withCallingHandlers(expr, warning = function(w) {
    warnings[[length(warnings) + 1L]] <<- w
    invokeRestart("muffleWarning")
  })
  expect_length(warnings, 1L)
  list(condition = warnings[[1L]], value = value)
}

test_that("impl-level call suppression covers constructors, statics and methods", {
  for (case in method_call_cases()) {
    x <- case$new(10L)
    for (value in list(-1L, 1L, 2L, NA_integer_, 1.5)) {
      expect_null(conditionCall(method_error(case$new(value))))
      expect_null(conditionCall(method_error(case$read(x, value, "fast"))))
      expect_null(conditionCall(method_error(case$static(value))))
    }
    e <- method_error(case$read(x, 10L, "unknown"))
    expect_identical(e$param, "mode")
    expect_null(conditionCall(e))
    expect_identical(case$read(x, 10L, "fast"), 10L)
    expect_identical(case$static(10L), 10L)
    for (value in c(0L, 3L)) {
      w <- method_warning(case$read(x, value, "fast"))
      expect_null(conditionCall(w$condition))
      expect_s3_class(w$condition, "rust_warning")
    }
    m <- NULL
    result <- withCallingHandlers(case$read(x, 4L, "fast"), message = function(c) {
      m <<- c
      invokeRestart("muffleMessage")
    })
    expect_identical(result, 4L)
    expect_s3_class(m, "message")
    expect_null(conditionCall(m))
  }
})

test_that("method options are local and another impl block keeps default attribution", {
  for (case in method_call_cases()) {
    x <- case$new(10L)
    for (value in list(-1L, 1L, NA_integer_, 1.5)) {
      expect_null(conditionCall(method_error(case$quiet(x, value))))
      expect_type(conditionCall(method_error(case$plain(x, value))), "language")
    }
  }
  x <- new_nocalls3(10L)
  e <- method_error(nc_s3_plain(x, -1L))
  expect_identical(conditionCall(e), quote(nc_s3_plain(x, -1L)))
  expect_null(conditionCall(method_error(nc_s3_read.NoCallS3(x, -1L, "fast"))))
  x <- NoCallS7(10L)
  expect_null(conditionCall(method_error(NoCallS7_nc_s7_read(x, -1L, "fast"))))
})

test_that("populated and empty trait impls keep the call policy", {
  x <- NoCallEnv$new(10L)
  methods <- list(
    function(value) x$NoCallTrait$trait_probe(value),
    NoCallEnv$NoCallTrait$trait_static,
    function(value) x$NoCallInherited$inherited_probe(value)
  )
  for (method in methods) {
    for (value in list(-1L, 1L, 2L, NA_integer_, 1.5)) {
      expect_null(conditionCall(method_error(method(value))))
    }
    expect_null(conditionCall(method_warning(method(0L))$condition))
    expect_identical(method(10L), 10L)
  }
  e <- method_error(NoCallEnv$NoCallTrait$FAILURE())
  expect_match(conditionMessage(e), "constant conversion")
  expect_null(conditionCall(e))
})

test_that("R6 setters and private callbacks re-raise conditions without calls", {
  x <- NoCallR6$new(10L)
  for (value in list(-1L, 1L, NA_integer_, 1.5)) {
    expect_null(conditionCall(method_error(x$reading <- value)))
    expect_identical(x$reading, 10L)
  }
  x$reading <- 12L
  expect_identical(x$reading, 12L)
  callback <- x$.__enclos_env__$private$deep_clone
  expect_null(conditionCall(method_error(callback("x", NULL))))
  x$reading <- 13L
  expect_null(conditionCall(method_error(x$.__enclos_env__$private$finalize())))
  x$reading <- 10L # the automatic GC finalizer must succeed
})

test_that("S7 property getters, setters and validators transport call-less errors", {
  x <- NoCallS7(10L)
  for (value in list(-1L, 1L, NA_integer_)) {
    expect_null(conditionCall(method_error(S7::prop(x, "reading") <- value)))
    expect_identical(S7::prop(x, "reading"), 10L)
  }
  NoCallS7_store_raw(x, -1L)
  expect_null(conditionCall(method_error(S7::prop(x, "reading"))))
  NoCallS7_store_raw(x, 10L)
  # S7 does not automatically validate computed properties. Invoke the actual
  # generated validator retained on its property definition.
  validator <- attr(NoCallS7, "properties")$reading$validator
  expect_null(validator(10L))
  e <- method_error(validator(9L))
  expect_match(conditionMessage(e), "validator error")
  expect_null(conditionCall(e))
})

test_that("vctrs constructor and static method suppress condition calls", {
  skip_if_not(exists("new_nocallvctrs", envir = asNamespace("miniextendr")))
  for (value in list(-1L, 1L, 2L, NA_integer_, 1.5)) {
    expect_null(conditionCall(method_error(new_nocallvctrs(value))))
    expect_null(conditionCall(method_error(nocallvctrs_static_probe(value))))
  }
  expect_s3_class(new_nocallvctrs(10L), "NoCallVctrs")
  expect_identical(nocallvctrs_static_probe(10L), 10L)
})
