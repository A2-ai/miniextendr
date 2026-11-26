# Same-named enum fields spelled otherwise share one column (#2141): a type
# alias, a renamed struct and a module-relative path join the first variant's
# column, which rustc checks through `SameColumn`.

test_that("fields spelled otherwise fill one column per name", {
  df <- miniextendr:::df_shared_columns()
  # A nested struct's columns come after the enum's other columns.
  expect_identical(
    names(df),
    c("kind", "id", "pair_1", "pair_2", "at_x", "at_y")
  )
  expect_identical(df$kind, c("Here", "There"))
  expect_identical(df$id, c(1L, 2L))
  expect_identical(df$at_x, c(1, 3))
  expect_identical(df$at_y, c(2, 4))
  expect_identical(df$pair_1, c(10L, 30L))
  expect_identical(df$pair_2, c(20L, 40L))
})

test_that("the reader reads fields spelled otherwise back from their columns", {
  df <- miniextendr:::df_shared_columns_read()
  expect_identical(df$id, c(1L, 2L))
  expect_identical(miniextendr:::df_shared_columns_roundtrip(df), df)
})
