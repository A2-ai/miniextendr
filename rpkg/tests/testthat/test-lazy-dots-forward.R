# Captured promises forwarded through RCall (#2158).
forward_drops <- function() miniextendr:::lazy_dots_sentinel_drops()

test_that("forwarding preserves empty, unnamed and repeated arguments without forcing", {
  inspect <- function(...) as.list(substitute(list(...)))[-1L]
  expected <- alist(a = not_bound, , a = 2L, not_bound + 1L)
  expect_identical(miniextendr:::lazy_dots_forward(inspect, a = not_bound, , a = 2L,
                                                 not_bound + 1L), expected)
  expect_identical(miniextendr:::lazy_dots_try_forward(inspect, a = not_bound, , a = 2L,
                                                     not_bound + 1L), expected)
  expect_identical(miniextendr:::lazy_dots_forward(inspect), list())
  expect_identical(miniextendr:::lazy_dots_forward(function(...) 42L, stop("never")), 42L)
  relay <- function(arg) miniextendr:::lazy_dots_forward(function(x) missing(x), arg)
  expect_true(relay())
})

test_that("forwarded promises retain their environments and cached values", {
  x <- 1L
  relay <- function(...) {
    x <- 100L
    miniextendr:::lazy_dots_forward(function(...) list(...), local = x, ...)
  }
  expect_identical(relay(original = x), list(local = 100L, original = 1L))
  state <- new.env(parent = emptyenv())
  state$count <- 0L
  out <- miniextendr:::lazy_dots_forward_twice(function(...) list(...), {
    state$count <- state$count + 1L
    state$count
  })
  expect_identical(out, list(list(1L), list(1L)))
  expect_identical(state$count, 1L)
  relay <- compiler::cmpfun(relay)
  expect_identical(relay(original = x), list(local = 100L, original = 1L))
})

test_that("leading Rust values remain rooted and do not shadow promise environments", {
  label <- "from caller"
  target <- function(values, label, ...) list(values, label, list(...))
  expect_identical(miniextendr:::lazy_dots_forward_values(target, seen = label),
                   list(1:3, "from Rust", list(seen = "from caller")))
  caller <- environment()
  captures <- miniextendr:::lazy_dots_forward(function(...) rlang::enquos(...), x + 1L)
  expect_identical(rlang::quo_get_expr(captures[[1L]]), quote(x + 1L))
  expect_identical(rlang::quo_get_env(captures[[1L]]), caller)
})

test_that("namespace forwarding supports a data-masking callee", {
  data <- data.frame(ID = c(2L, 1L, 2L), value = 1:3)
  bump <- 10L
  out <- miniextendr:::lazy_dots_group_by(data, key = ID + bump)
  expect_identical(dplyr::group_vars(out), "key")
  expect_identical(out$key, c(12L, 11L, 12L))
  relay <- function(...) miniextendr:::lazy_dots_group_by(data, ...)
  expect_identical(dplyr::group_vars(relay(ID)), "ID")
})

test_that("forwarding retains calling handlers and original error conditions", {
  for (forward in list(miniextendr:::lazy_dots_forward, miniextendr:::lazy_dots_try_forward)) {
    seen <- character()
    target <- function(...) {
      warning("callee warning")
      message("callee message")
      list(...)
    }
    before <- forward_drops()
    value <- withCallingHandlers(
      forward(target, x = 3L),
      warning = function(w) {
        seen <<- c(seen, conditionMessage(w))
        invokeRestart("muffleWarning")
      },
      message = function(m) {
        seen <<- c(seen, conditionMessage(m))
        invokeRestart("muffleMessage")
      }
    )
    expect_identical(value, list(x = 3L))
    expect_identical(seen, c("callee warning", "callee message\n"))
    expect_identical(forward_drops(), before + 1L)
    before <- forward_drops()
    out <- tryCatch(forward(function(...) warning("exit")), warning = identity)
    expect_s3_class(out, "warning")
    expect_identical(forward_drops(), before + 1L)
    before <- forward_drops()
    expect_identical(withRestarts(forward(function(...) invokeRestart("escaped", 7L)),
                                 escaped = identity), 7L)
    expect_identical(forward_drops(), before + 1L)
  }
  condition <- errorCondition("forwarded error", code = 11L, class = "mx_forwarded_error")
  target <- function(...) stop(condition)
  before <- forward_drops()
  expect_identical(tryCatch(miniextendr:::lazy_dots_forward(target), error = identity), condition)
  expect_identical(forward_drops(), before + 1L)
  before <- forward_drops()
  expect_identical(miniextendr:::lazy_dots_try_forward(target), condition)
  expect_identical(forward_drops(), before + 1L)
})

test_that("every class dispatch path forwards the original expressions", {
  inspect <- function(...) as.list(substitute(list(...)))[-1L]
  expected <- alist(a = not_bound, )
  env <- miniextendr:::LazyDotsEnv$new(0)
  r6 <- miniextendr:::LazyDotsR6$new(0)
  s3 <- miniextendr::new_lazydotss3(0)
  s4 <- miniextendr:::LazyDotsS4(0)
  s7 <- miniextendr:::LazyDotsS7(0)
  expect_identical(env$sum_forward(inspect, a = not_bound, ), expected)
  expect_identical(r6$sum_forward(inspect, a = not_bound, ), expected)
  expect_identical(miniextendr::lazy_s3_sum_forward(s3, inspect, a = not_bound, ), expected)
  expect_identical(miniextendr:::s4_lazy_sum_forward(s4, inspect, a = not_bound, ), expected)
  expect_identical(miniextendr:::lazy_s7_sum_forward(s7, inspect, a = not_bound, ), expected)
  expect_identical(miniextendr:::LazyDotsS7_lazy_s7_sum_forward(s7, inspect, a = not_bound, ),
                   expected)
  expect_identical(miniextendr:::lazydotsvctrs_forward(inspect, a = not_bound, ), expected)
})

test_that("forwarding NextMethod keeps the active S3 dispatch frame", {
  mx_forward_next <- function(x, ...) UseMethod("mx_forward_next")
  mx_forward_next.LazyDotsS3 <- utils::getS3method(
    "lazy_s3_sum_forward", "LazyDotsS3", envir = asNamespace("miniextendr")
  )
  mx_forward_next.mx_next <- function(x, fun, ...) {
    as.list(substitute(list(...)))[-1L]
  }
  x <- miniextendr::new_lazydotss3(0)
  class(x) <- c("LazyDotsS3", "mx_next")
  expect_identical(mx_forward_next(x, base::NextMethod, a = not_bound, , 3L),
                   alist(a = not_bound, , 3L))
  expect_identical(mx_forward_next(x, base::NextMethod), list())
})

test_that("forwarded calls and their roots survive gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in 1:3) {
    miniextendr:::gc_stress_lazy_dots_forward()
  }
  expect_true(TRUE)
})
