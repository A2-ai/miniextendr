# A `&Dots` parameter is R's `...` wherever it sits in the Rust signature.
# The formals follow the Rust order, and a formal after `...` is matched by
# its full name only, as R does for any formal after the dots.

# region: standalone functions

test_that("a formal after `...` keeps its default and is matched by full name only", {
  expect_equal(names(formals(dots_mid)), c("x", "...", "overwrite"))
  expect_false(formals(dots_mid)$overwrite)
  expect_equal(dots_mid(1L, 2, 3, overwrite = TRUE), "x=1 dots=2 overwrite=true")
  expect_equal(dots_mid(1L), "x=1 dots=0 overwrite=false")
  # A partial name does not match a formal after `...`: `over` is a dot.
  expect_equal(dots_mid(1L, over = TRUE), "x=1 dots=1 overwrite=false")
  # A positional extra is a dot too.
  expect_equal(dots_mid(1L, TRUE), "x=1 dots=1 overwrite=false")
})

test_that("dots first: every formal after them is matched by name", {
  f <- miniextendr:::dots_first
  expect_equal(names(formals(f)), c("...", "x"))
  expect_equal(f(1, 2, x = 3L), "dots=2 x=3")
})

test_that("an explicit trailing `&Dots` is `...`", {
  f <- miniextendr:::dots_trailing_explicit
  expect_equal(names(formals(f)), c("x", "..."))
  expect_equal(f(1L, 2, 3), 3L)
})

test_that("typed_list! sugar reads a middle `&Dots`", {
  f <- miniextendr:::dots_mid_typed
  expect_equal(names(formals(f)), c("x", "...", "flag"))
  expect_equal(f(1L, a = 2), 3)
  expect_equal(f(1L, a = 2, flag = TRUE), 4)
  expect_error(f(1L, a = "x"), "dots validation failed")
})

test_that("on a `caller` wrapper, `.call` goes after the formal that follows the dots", {
  f <- miniextendr:::dots_mid_caller_impl
  expect_equal(names(formals(f)), c("x", "...", "flag", ".call"))
  expect_false(formals(f)$flag)
  expect_null(formals(f)$.call)
  expect_equal(f(1L, 2, 3, flag = TRUE), "x=1 dots=2 flag=true")
  via <- function(v) miniextendr:::dots_mid_caller_impl(v, 2, .call = environment())
  e <- tryCatch(via(-1L), error = identity)
  expect_s3_class(e, "error")
  expect_identical(conditionCall(e)[[1]], quote(via))
})

test_that("`worker` on a function with dots runs on the main thread", {
  skip_if_not(
    exists("dots_mid_worker", envir = asNamespace("miniextendr"), inherits = FALSE),
    "built without the worker-thread feature"
  )
  f <- miniextendr:::dots_mid_worker
  expect_equal(names(formals(f)), c("x", "...", "flag"))
  expect_equal(f(1L, 2, flag = TRUE), "x=1 dots=1 flag=true")
})

test_that("the page of an explicit `&Dots` documents `...`, not the Rust name", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_text <- vapply(
    rd_db,
    function(rd) paste(capture.output(print(rd)), collapse = "\n"),
    character(1)
  )
  page <- rd_text[grepl("dots_trailing_explicit(x, ...)", rd_text, fixed = TRUE)]
  skip_if(length(page) == 0L, "dots_trailing_explicit page not found — package not documented")
  expect_match(page, "\\item{...}{", fixed = TRUE)
  expect_no_match(page, "\\item{rest}", fixed = TRUE)
})

# endregion

# region: methods, one class per class system

test_that("env: a method's formal after its dots", {
  obj <- miniextendr:::DotsPosEnv$new(1L)
  expect_equal(names(formals(obj$collect)), c("n", "...", "flag"))
  expect_equal(obj$collect(2L, 3, 4, flag = TRUE), "base=1 n=2 dots=2 flag=true")
  expect_equal(obj$collect(2L, TRUE), "base=1 n=2 dots=1 flag=false")
  expect_equal(obj$collect(2L, fl = TRUE), "base=1 n=2 dots=1 flag=false")
})

test_that("R6: a method's formal after its dots", {
  obj <- miniextendr:::DotsPosR6$new(1L)
  expect_equal(names(formals(obj$collect)), c("n", "...", "flag"))
  expect_equal(obj$collect(2L, 3, 4, flag = TRUE), "base=1 n=2 dots=2 flag=true")
  expect_equal(obj$collect(2L, TRUE), "base=1 n=2 dots=1 flag=false")
  expect_equal(obj$collect(2L, fl = TRUE), "base=1 n=2 dots=1 flag=false")
})

test_that("S3: a method's formal after its dots, and one `...`", {
  obj <- miniextendr:::new_dotsposs3(1L)
  method <- miniextendr:::dots_pos_s3_collect.DotsPosS3
  expect_equal(names(formals(method)), c("x", "n", "...", "flag"))
  collect <- miniextendr:::dots_pos_s3_collect
  expect_equal(collect(obj, 2L, 3, 4, flag = TRUE), "base=1 n=2 dots=2 flag=true")
  expect_equal(collect(obj, 2L, TRUE), "base=1 n=2 dots=1 flag=false")
  expect_equal(collect(obj, 2L, fl = TRUE), "base=1 n=2 dots=1 flag=false")
})

test_that("S4: a method's formal after its dots", {
  obj <- miniextendr:::DotsPosS4(1L)
  collect <- miniextendr:::s4_dots_pos_collect
  expect_equal(collect(obj, 2L, 3, 4, flag = TRUE), "base=1 n=2 dots=2 flag=true")
  expect_equal(collect(obj, 2L, TRUE), "base=1 n=2 dots=1 flag=false")
  expect_equal(collect(obj, 2L, fl = TRUE), "base=1 n=2 dots=1 flag=false")
})

test_that("S7: a method's formal after its dots", {
  obj <- miniextendr:::DotsPosS7(1L)
  collect <- miniextendr:::dots_pos_s7_collect
  expect_equal(collect(obj, 2L, 3, 4, flag = TRUE), "base=1 n=2 dots=2 flag=true")
  expect_equal(collect(obj, 2L, TRUE), "base=1 n=2 dots=1 flag=false")
  expect_equal(collect(obj, 2L, fl = TRUE), "base=1 n=2 dots=1 flag=false")
})

test_that("vctrs: a static helper's formal after its dots", {
  f <- miniextendr:::dotsposvctrs_collect
  expect_equal(names(formals(f)), c("n", "...", "flag"))
  expect_equal(f(2L, 3, 4, flag = TRUE), "n=2 dots=2 flag=true")
  expect_equal(f(2L, TRUE), "n=2 dots=1 flag=false")
  expect_equal(f(2L, fl = TRUE), "n=2 dots=1 flag=false")
})

# endregion
