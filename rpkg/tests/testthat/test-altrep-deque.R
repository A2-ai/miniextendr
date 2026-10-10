test_that("native deque ALTREPs preserve ring order, aliases and serialization", {
  cases <- list(
    list(make = altrep_deque_integers, values = c(1L, NA_integer_, 3L, 4L)),
    list(make = altrep_deque_reals, values = c(1.5, NA_real_, NaN, Inf)),
    list(make = altrep_deque_raw, values = as.raw(c(0, 1, 128, 255))),
    list(make = altrep_deque_complex, values = c(1+2i, NA_complex_, 3-4i, 0+0i))
  )
  for (case in cases) {
    expected <- case$values[c(3L, 4L, 1L, 2L)]
    x <- case$make(case$values, 2L)
    expect_identical(x, expected)
    expect_identical(unserialize(serialize(x, NULL)), expected)
    expect_identical(x[c(4L, 1L, 3L, 2L)], expected[c(4L, 1L, 3L, 2L)])
    names(x) <- letters[1:4]
    names(expected) <- letters[1:4]
    expect_identical(unserialize(serialize(x, NULL)), expected)
    alias <- x
    x[1L] <- case$values[[1L]]
    expect_identical(alias, expected)
    expected[1L] <- case$values[[1L]]
    expect_identical(x, expected)
    expect_identical(unserialize(serialize(x, NULL)), expected)
    expect_identical(case$make(case$values[FALSE], 999L), case$values[FALSE])
  }
})

test_that("deque region reads support R summaries before and after mutation", {
  x <- altrep_deque_integers(c(1L, NA_integer_, 3L, 4L), 1L)
  expect_true(anyNA(x))
  expect_identical(sum(x, na.rm = TRUE), 8L)
  expect_identical(min(x, na.rm = TRUE), 1L)
  expect_identical(max(x, na.rm = TRUE), 4L)
  x[1L] <- 2L
  expect_false(anyNA(x))
  expect_identical(sum(x), 10L)
})
