test_that("explicit conversions accept custom views and aliases", {
  describe <- miniextendr:::converted_refs_describe
  expect_identical(describe(c(2L, 3L), "sum"), "sum:5")
  expect_identical(describe(integer(), NULL), "missing:0")
  expect_identical(describe(4L, NA_character_), "missing:4")
  e <- tryCatch(describe(1.5, "sum"), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "values")
  expect_error(describe(1L, 1L), class = "rust_error")
  expect_identical(miniextendr:::converted_refs_optional(NULL), -1L)
  expect_identical(miniextendr:::converted_refs_optional(c(1L, 2L)), 5L)
  expect_identical(miniextendr:::converted_refs_strings("a", "b", "c"), "a:b:c")
  expect_null(miniextendr:::gc_stress_converted_references())
})

test_that("alias metadata is checked before explicit converters create references", {
  for (pair in list(miniextendr:::converted_refs_alias_pair,
                    miniextendr:::converted_refs_custom_pair)) {
    x <- c(2L, 3L)
    expect_error(pair(x, x), "aliasing")
    expect_identical(x, c(2L, 3L))
    expect_identical(pair(c(2L, 3L), c(4L, 5L)), 16L)
    expect_identical(pair(integer(), integer()), 0L)
  }
})

test_that("inherent and trait method wrappers honor convert", {
  host <- miniextendr:::ConvertedRefHost$new()
  expect_identical(host$describe(c(3L, 4L), "method"), "method:7")
  expect_identical(host$describe(integer(), NULL), "missing:0")
  expect_error(host$describe(1.5, "bad"), class = "rust_error")
  x <- c(2L, 3L)
  expect_error(host$pair(x, x), "aliasing")
  expect_identical(x, c(2L, 3L))
  expect_identical(host$ConvertedReads$total(c(3L, 4L)), 7L)
  expect_identical(host$GenericConvertedReads$generic_total(c(5L, 6L)), 11L)
  expect_error(host$ConvertedReads$total("bad"), class = "rust_error")
  expect_identical(miniextendr:::converted_refs_through_view(), 10L)
})

test_that("explicit borrowed conversions survive GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  expect_null(miniextendr:::gc_stress_converted_references())
})
