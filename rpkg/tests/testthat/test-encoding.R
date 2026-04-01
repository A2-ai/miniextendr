test_that("encoding_info_available returns logical", {
  result <- encoding_info_available()
  expect_type(result, "logical")
  # On most R package builds, encoding_init is disabled so this is FALSE.
  # On embedded R or nonapi builds, it may be TRUE.
})
