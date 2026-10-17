# Method capture must survive R's dispatch frames and Rust's trait shims.
capture_cases <- list(
  env = list(new = "QuotedEnv", quoted = "qe_quoted", quosure = "qe_quosure",
             optional = "qe_optional"),
  r6 = list(new = "QuotedR6", quoted = "qr_quoted", quosure = "qr_quosure",
            optional = "qr_optional"),
  s3 = list(new = "new_quoteds3", quoted = "qs3_quoted", quosure = "qs3_quosure",
            optional = "qs3_optional"),
  s4 = list(new = "QuotedS4", quoted = "s4_qs4_quoted", quosure = "s4_qs4_quosure",
            optional = "s4_qs4_optional"),
  s7 = list(new = "QuotedS7", quoted = "qs7_quoted", quosure = "qs7_quosure",
            optional = "qs7_optional")
)

# Assemble the actual method call in the caller's frame, without a forwarding
# helper adding another promise/environment to the behavior under test.
capture_call <- function(system, method, args) {
  receiver <- if (system %in% c("env", "r6")) {
    call("$", quote(object), as.name(method))
  } else {
    as.name(method)
  }
  if (!system %in% c("env", "r6")) args <- c(list(quote(object)), args)
  as.call(c(list(receiver), args))
}

for (system in names(capture_cases)) local({
  system <- system
  case <- capture_cases[[system]]
  test_that(paste(system, "methods preserve unevaluated expressions and caller frames"), {
    caller <- new.env(parent = asNamespace("miniextendr"))
    caller$local_value <- 41L
    ctor_head <- if (system %in% c("env", "r6")) {
      call("$", as.name(case$new), quote(new))
    } else as.name(case$new)
    caller$object <- eval(as.call(list(ctor_head, quote(local_value + 1L))), caller)
    expect_identical(eval(capture_call(system, case$quoted, list(quote(ignored), 3L)), caller), 42L)
    for (kind in c("quoted", "quosure")) {
      method <- case[[kind]]
      expr_call <- capture_call(system, method, list(quote(stop("not forced")), 0L))
      expect_identical(eval(expr_call, caller), quote(stop("not forced")))
      env_call <- capture_call(system, method, list(quote(local_value), 1L))
      expect_identical(eval(env_call, caller), caller)
      value_call <- capture_call(system, method, list(quote(local_value + 1L), 2L))
      expect_identical(eval(value_call, caller), 42L)
      missing_call <- capture_call(system, method, list(mode = 0L))
      error <- tryCatch(eval(missing_call, caller), error = identity)
      expect_s3_class(error, "rust_error")
      expect_identical(error$kind, "conversion")
      expect_identical(error$param, "expr")
    }
    omitted <- capture_call(system, case$optional, list())
    expect_identical(eval(omitted, caller), c(TRUE, TRUE))
    supplied <- capture_call(system, case$optional,
                             list(quote(stop("base not forced")), quote(stop("tidy not forced"))))
    expect_identical(eval(supplied, caller), c(FALSE, FALSE))
    trait_name <- switch(system, env = NULL, r6 = NULL, s3 = "capture",
                         s4 = "s4_trait_MethodCapture_capture", s7 = "s7_trait_MethodCapture_capture")
    trait_head <- if (system %in% c("env", "r6")) {
      call("$", call("$", as.name(case$new), quote(MethodCapture)), quote(capture))
    } else as.name(trait_name)
    for (mode in c(0L, 1L, 2L, 3L, 4L, 5L)) {
      result <- eval(as.call(list(trait_head, quote(object), quote(local_value + 1L),
                                  quote(local_value + 1L), mode)), caller)
      expect_identical(result, switch(as.character(mode),
        "0" = quote(local_value + 1L), "1" = caller, "2" = 42L,
        "3" = quote(local_value + 1L), "4" = caller, "5" = 42L))
    }
    static_head <- if (system %in% c("env", "r6")) {
      call("$", as.name(case$new), quote(static_capture))
    } else as.name(paste0(switch(system, s3 = "quoteds3", s4 = "QuotedS4", s7 = "QuotedS7"),
                         "_static_capture"))
    expect_identical(eval(as.call(list(static_head, quote(stop("not forced")))), caller),
                     quote(stop("not forced")))
    if (system == "r6") {
      caller$Child <- R6::R6Class("QuotedChild", inherit = miniextendr:::QuotedR6, cloneable = FALSE)
      caller$child <- eval(quote(Child$new(local_value + 2L)), caller)
      expect_identical(eval(quote(child$qr_quoted(ignored, 3L)), caller), 43L)
      eval(quote(object$initialize(local_value + 2L)), caller)
      expect_identical(eval(capture_call(system, case$quoted, list(quote(ignored), 3L)), caller), 43L)
    }
    # A real forwarding function is intentional here: Quosure can embrace the
    # forwarded argument, whereas Quoted retains the helper's expression and frame.
    forward <- function(expr) {
      owner <- parent.env(environment())
      owner$forward_frame <- environment()
      eval(capture_call(system, case$quoted, list(quote(expr), 1L)), environment())
    }
    environment(forward) <- caller
    # Install the call helpers explicitly, leaving the namespace parent intact.
    caller$capture_call <- capture_call
    caller$system <- system
    caller$case <- case
    expect_identical(forward(local_value), caller$forward_frame)
    tidy_forward <- function(expr) NULL
    body(tidy_forward) <- capture_call(system, case$quosure, list(quote({{ expr }}), 0L))
    environment(tidy_forward) <- caller
    expect_identical(tidy_forward(local_value + 1L), quote(local_value + 1L))
  })
})

test_that("S7 shortcuts and vctrs static methods capture the caller", {
  caller <- new.env(parent = asNamespace("miniextendr"))
  caller$object <- miniextendr:::QuotedS7()
  caller$local_value <- 42L
  for (method in c("QuotedS7_qs7_quoted", "QuotedS7_qs7_quosure")) {
    expect_identical(eval(as.call(list(as.name(method), quote(object), quote(local_value), 1L)),
                          caller), caller)
    expect_identical(eval(as.call(list(as.name(method), quote(object), quote(local_value), 2L)),
                          caller), 42L)
  }
  caller$direct <- S7::method(miniextendr:::qs7_quoted, S7::S7_class(caller$object))
  expect_identical(eval(quote(direct(object, local_value, 1L)), caller), caller)
  for (method in c("quotedvctrs_quoted", "quotedvctrs_quosure")) {
    expect_identical(eval(as.call(list(as.name(method), quote(local_value), 1L)), caller), caller)
    expect_identical(eval(as.call(list(as.name(method), quote(local_value), 2L)), caller), 42L)
  }
})

test_that("trait Views transport captures and missing sentinels", {
  object <- miniextendr:::quoted_method_owner()
  local_value <- 42L
  expect_identical(miniextendr:::quoted_method_view(
    object, stop("not forced"), stop("not forced"), 0L), quote(stop("not forced")))
  expect_identical(miniextendr:::quoted_method_view(
    object, local_value, local_value, 1L), environment())
  expect_identical(miniextendr:::quoted_method_view(
    object, local_value, local_value, 2L), 42L)
  expect_identical(miniextendr:::quoted_method_view(
    object, stop("not forced"), stop("not forced"), 3L), quote(stop("not forced")))
  expect_identical(miniextendr:::quoted_method_view(
    object, local_value, local_value, 4L), environment())
  expect_identical(miniextendr:::quoted_method_view(
    object, local_value, local_value, 5L), 42L)
  expect_identical(miniextendr:::quoted_method_view_absent(object), c(TRUE, TRUE))
  expect_identical(miniextendr:::quoted_method_view_absent(
    object, stop("not forced"), stop("not forced")), c(FALSE, FALSE))
})

test_that("trait capture transport remains rooted under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  expect_identical(miniextendr:::gc_stress_quoted_method_view(), 42L)
})
