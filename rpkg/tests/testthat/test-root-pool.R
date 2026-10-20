# ProtectPool as a per-call root set (#1909): a struct owns a pool and roots
# each column it builds. See rpkg/src/rust/root_pool_tests.rs.

root_pool_columns_expected <- list(
  id = 1:3,
  value = c(0.5, 1.5, 2.5),
  label = c("a", "b", "c"),
  flag = c(TRUE, FALSE, NA)
)

root_pool_out_of_order_expected <- list(
  reals = c(0.5, 1.5),
  ints = 1:3,
  strings = c("x", "y", "z")
)

check_root_pool_stress <- function(result) {
  expect_named(result, c("columns", "out_of_order", "many"))
  expect_identical(result$columns, root_pool_columns_expected)
  expect_identical(result$out_of_order, root_pool_out_of_order_expected)
  expect_identical(names(result$many), paste0("c", 1:20))
  for (i in 1:20) {
    expect_identical(result$many[[i]], rep(i, 3L))
  }
}

test_that("a struct owning a ProtectPool roots each column it builds", {
  expect_identical(miniextendr:::root_pool_columns(), root_pool_columns_expected)
})

test_that("pools owned by two structs drop out of creation order", {
  expect_identical(
    miniextendr:::root_pool_drop_out_of_order(),
    root_pool_out_of_order_expected
  )
})

test_that("gc_stress_root_pool returns every column", {
  check_root_pool_stress(miniextendr:::gc_stress_root_pool())
})

test_that("root pools hold their columns across allocations under gctorture", {
  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture: see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  columns <- miniextendr:::root_pool_columns()
  out_of_order <- miniextendr:::root_pool_drop_out_of_order()
  result <- miniextendr:::gc_stress_root_pool()
  gctorture(FALSE)
  expect_identical(columns, root_pool_columns_expected)
  expect_identical(out_of_order, root_pool_out_of_order_expected)
  check_root_pool_stress(result)
})
