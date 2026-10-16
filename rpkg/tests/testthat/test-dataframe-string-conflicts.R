test_that("expanded and map string conflicts preserve cells", {
  expect_no_error(gc_stress_dataframe_string_conflicts())
})
