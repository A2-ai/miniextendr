# Every impl of a trait takes the declaration's parameter names: consumer.pkg's
# `impl Counter for DoubleCounter` calls `add`'s parameter `amount`, the
# declaration in shared-traits calls it `n`.

test_that("an impl's R wrapper takes the trait's parameter names", {
  expect_identical(names(formals(DoubleCounter$Counter$add)), c(".mx_self", "n"))
  dc <- DoubleCounter$create(1L)
  dc$Counter$add(n = 2L)
  expect_identical(dc$Counter$value(), 3L)
  expect_error(dc$Counter$add(amount = 2L), "unused argument")
  expect_identical(dc$Counter$value(), 3L)
})

test_that("an impl's argument errors name the trait's parameter on every path", {
  direct <- tryCatch(DoubleCounter$create(1L)$Counter$add("x"), error = identity)
  through_vtable <- tryCatch(counter_add_raw(new_double_counter(1L), "x"), error = identity)
  for (e in list(direct, through_vtable)) {
    expect_identical(e$kind, "conversion")
    expect_identical(e$param, "n")
    expect_true(startsWith(conditionMessage(e), "'n' must be"), info = conditionMessage(e))
  }
})
