# Test ALTREP DATAPTR materialization
#
# These tests exercise operations that call DATAPTR() on ALTREP vectors,
# forcing materialization. This catches ALTREP types that error on DATAPTR
# ("cannot access data pointer for this ALTVEC object").
#
# Operations that trigger DATAPTR:
#   identical()  — compares via DATAPTR
#   c()          — concatenation materializes
#   sort()       — needs contiguous data
#   rev()        — needs contiguous data
#   x + y        — arithmetic on ALTREP vectors
#   as.numeric() — coercion may trigger DATAPTR
#   data.frame() — column assignment triggers DATAPTR

# Helper: test that DATAPTR-triggering operations work on a vector
expect_dataptr_works <- function(x, label = deparse(substitute(x))) {
  # identical() forces DATAPTR
  expect_true(identical(x, x), info = paste(label, "identical(x, x)"))

  # c() forces DATAPTR
  y <- c(x, x)
  expect_equal(length(y), 2L * length(x), info = paste(label, "c(x, x)"))

  # rev() forces DATAPTR
  r <- rev(x)
  expect_equal(length(r), length(x), info = paste(label, "rev(x)"))
  if (length(x) > 0) {
    expect_equal(r[1], x[length(x)], info = paste(label, "rev first == last"))
  }
}

# region: Builtin Vec<T> types

test_that("Vec<i32> ALTREP supports DATAPTR operations", {
  x <- iter_int_range(1L, 6L)
  expect_dataptr_works(x)
  expect_equal(sort(x), 1:5)
  expect_true(identical(x, 1:5))
})

test_that("Vec<f64> ALTREP supports DATAPTR operations", {
  x <- iter_real_squares(4L)  # [0,1,4,9]
  expect_dataptr_works(x)
  expect_equal(x + 1, c(1, 2, 5, 10))
})

test_that("Vec<bool> ALTREP supports DATAPTR operations", {
  x <- iter_logical_alternating(4L)
  expect_dataptr_works(x)
  expect_true(identical(x, c(TRUE, FALSE, TRUE, FALSE)))
})

test_that("Vec<u8> ALTREP supports DATAPTR operations", {
  x <- iter_raw_bytes(4L)
  expect_dataptr_works(x)
  expect_true(identical(x, as.raw(0:3)))
})

test_that("Vec<String> ALTREP supports DATAPTR operations", {
  x <- iter_string_items(3L)
  expect_dataptr_works(x)
  expect_true(identical(x, c("item_0", "item_1", "item_2")))
  expect_equal(paste(x, collapse = ","), "item_0,item_1,item_2")
})

test_that("Vec<Rcomplex> ALTREP supports DATAPTR operations", {
  x <- vec_complex_altrep(3L)
  expect_dataptr_works(x)
})

# endregion

# region: Box<[T]> types

test_that("Box<[i32]> ALTREP supports DATAPTR operations", {
  x <- boxed_ints(5L)
  expect_dataptr_works(x)
  expect_true(identical(x, 1:5))
})

test_that("Box<[f64]> ALTREP supports DATAPTR operations", {
  x <- boxed_reals(3L)
  expect_dataptr_works(x)
  expect_equal(x + 0.5, c(2.0, 3.5, 5.0))
})

test_that("Box<[bool]> ALTREP supports DATAPTR operations", {
  x <- boxed_logicals(4L)
  expect_dataptr_works(x)
  expect_true(identical(x, c(TRUE, FALSE, TRUE, FALSE)))
})

test_that("Box<[u8]> ALTREP supports DATAPTR operations", {
  x <- boxed_raw(4L)
  expect_dataptr_works(x)
  expect_true(identical(x, as.raw(0:3)))
})

test_that("Box<[String]> ALTREP supports DATAPTR operations", {
  x <- boxed_strings(3L)
  expect_dataptr_works(x)
  expect_true(identical(x, c("boxed_0", "boxed_1", "boxed_2")))
})

test_that("Box<[Rcomplex]> ALTREP supports DATAPTR operations", {
  x <- boxed_complex(3L)
  expect_dataptr_works(x)
})

# endregion

# region: Range<T> types

test_that("Range<i32> ALTREP supports DATAPTR operations", {
  x <- range_int_altrep(1L, 6L)
  expect_dataptr_works(x)
  expect_true(identical(x, 1:5))
  expect_equal(sort(x, decreasing = TRUE), 5:1)
})

test_that("Range<i64> ALTREP supports DATAPTR operations", {
  x <- range_i64_altrep(1, 6)
  expect_dataptr_works(x)
  expect_true(identical(x, 1:5))
})

test_that("Range<f64> ALTREP supports DATAPTR operations", {
  x <- range_real_altrep(0, 5)
  expect_dataptr_works(x)
  expect_equal(x + 0.5, c(0.5, 1.5, 2.5, 3.5, 4.5))
})

# endregion

# region: Proc-macro-derived ALTREP

test_that("ConstantInt proc-macro ALTREP supports DATAPTR operations", {
  x <- constant_int()  # 10 elements, all 42
  expect_dataptr_works(x)
  expect_true(identical(x, rep(42L, 10)))
})

test_that("ConstantReal proc-macro ALTREP supports DATAPTR operations", {
  x <- constant_real()  # 10 elements, all pi
  expect_dataptr_works(x)
  expect_true(identical(x, rep(pi, 10)))
})

# endregion

# region: Arrow ALTREP

test_that("Float64Array ALTREP supports DATAPTR operations", {
  x <- zero_copy_arrow_f64_altrep(c(1.0, 2.0, 3.0))
  expect_dataptr_works(x)
  expect_true(identical(x, c(10.0, 20.0, 30.0)))
})

test_that("Int32Array ALTREP supports DATAPTR operations", {
  x <- zero_copy_arrow_i32_altrep(c(1L, 2L, 3L))
  expect_dataptr_works(x)
  expect_true(identical(x, c(101L, 102L, 103L)))
})

# endregion

# region: ALTREP in data.frame (forces DATAPTR for column assignment)

test_that("ALTREP vectors work as data.frame columns", {
  ints <- iter_int_range(1L, 4L)
  reals <- boxed_reals(3L)
  lgl <- iter_logical_alternating(3L)

  df <- data.frame(
    i = ints,
    r = reals,
    l = lgl
  )

  expect_equal(nrow(df), 3L)
  expect_equal(df$i, 1:3)
  expect_equal(df$r[1], 1.5)
  expect_true(df$l[1])
  expect_false(df$l[2])
})

# endregion
