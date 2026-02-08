test_that("normal function is exported", {
  # Normal functions are exported and callable
  expect_equal(export_control_normal(), "normal")

  # Check it appears in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_true(any(grepl("export_control_normal", ns)))
})

test_that("internal function is callable but not exported", {
  # Internal functions are still callable via :::
  expect_equal(miniextendr:::export_control_internal(), "internal")

  # Check it does NOT appear as export() in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(any(grepl("export_control_internal", export_lines)))
})

test_that("noexport function is callable but not exported", {
  # Noexport functions are still callable via :::
  expect_equal(miniextendr:::export_control_noexport(), "noexport")

  # Check it does NOT appear as export() in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(any(grepl("export_control_noexport", export_lines)))
})
