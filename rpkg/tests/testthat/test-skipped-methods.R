test_that("Rust-only methods stay callable without R conversion contracts", {
  expect_identical(
    miniextendr:::skipped_methods_rust_only(),
    c(1L, 2L, 3L, 4L, 11L, 12L, 13L, 14L)
  )
})

test_that("skipped signatures add no R methods or native registrations", {
  cls <- miniextendr:::SkippedMethodHost
  host <- cls$new(7L)
  expect_identical(host$exposed(), 7L)
  expect_identical(host$SkippedPlain$visible(), 7L)
  expect_identical(host$SkippedGeneric$generic_visible(), 8L)
  expect_identical(host$SkippedDefaults$default_visible(), 40L)
  expect_identical(ls(cls$SkippedPlain), "visible")
  expect_identical(ls(cls$SkippedGeneric), "generic_visible")
  expect_identical(ls(cls$SkippedDefaults), "default_visible")
  expect_length(ls(cls$SkippedOnly), 0L)

  calls <- names(getDLLRegisteredRoutines(getLoadedDLLs()[["miniextendr"]])$.Call)
  calls <- calls[grepl("SkippedMethodHost", calls, fixed = TRUE)]
  expect_false(any(grepl(
    "unchecked|boxed|missing|two_dots|visibility|consume|map|array|fetch|lazy|__only$",
    calls
  )))
})
