test_that("standalone None errors use r_name and postfix", {
  expect_error(miniextendr:::none_named_main(), "`none_named_main()` returned no value", fixed = TRUE)
  expect_error(miniextendr:::none_postfix_main_entry(), "`none_postfix_main_entry()` returned no value", fixed = TRUE)
  # Renaming changes diagnostics only; scalar Option returns still use NA.
  expect_identical(miniextendr:::none_named_scalar(), NA_integer_)
})

test_that("inherent None errors use the R member name", {
  lookup <- miniextendr:::NoneNameLookup$new()
  expect_error(lookup$find_value(), "`find_value()` returned no value", fixed = TRUE)
  expect_error(lookup$find_entry(), "`find_entry()` returned no value", fixed = TRUE)
})
