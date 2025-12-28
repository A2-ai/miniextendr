test_that("has_miniextendr detects scaffold (pending isolated harness)", {
  skip("requires running in a clean project sandbox with scaffolded files")
  expect_true(has_miniextendr())
})

test_that("miniextendr_status runs (pending sandbox)", {
  skip("needs temporary project with generated files; set up harness first")
  expect_silent(miniextendr_status())
})
