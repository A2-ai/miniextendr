test_that("force_invisible_i32 returns 42 invisibly", {
  res <- withVisible(force_invisible_i32())
  expect_false(res$visible)
  expect_identical(res$value, 42L)
})

test_that("force_visible_unit returns visibly", {
  res <- withVisible(force_visible_unit())
  expect_true(res$visible)
  expect_null(res$value)
})

test_that("with_interrupt_check doubles input", {
  expect_equal(with_interrupt_check(5L), 10L)
})

test_that("greet_hidden uses default and explicit names", {
  expect_equal(miniextendr:::greet_hidden(), "Hello, World!")
  expect_equal(miniextendr:::greet_hidden("Bob"), "Hello, Bob!")
})
