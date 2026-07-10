# Debug-only guard against aliasing `&mut [T]` slices from one SEXP (#1104).
#
# `impl TryFromSexp for &mut [T]` hands out a mutable slice over R's data pointer
# without copying. If R binds the same vector to two `&mut [T]` parameters
# (`alias_probe(x, x)`), the macro-generated wrapper would produce two aliasing
# `&mut` slices over one buffer -- undefined behavior. The wrapper emits a
# `debug_assert!` comparing the raw SEXP identities before conversion and panics
# (converted to an R error) when two such parameters share a SEXP.
#
# `debug_assert!` compiles out in release builds, so the error only surfaces when
# the package's Rust was built with `debug_assertions` on. rpkg's default and CI
# build is `release` (debug_assertions off), so the error assertion is skipped
# there via `debug_assertions_enabled()`.

test_that("distinct vectors pass alias_probe (no false positive)", {
  a <- c(1L, 2L, 3L)
  b <- c(10L, 20L)
  # Distinct SEXPs -> guard does not fire; both slices increment in place.
  expect_equal(alias_probe(a, b), 5L)
  expect_equal(a, c(2L, 3L, 4L))
  expect_equal(b, c(11L, 21L))
})

test_that("alias_probe(x, x) errors under a debug build (#1104)", {
  skip_if_not(
    debug_assertions_enabled(),
    "package compiled without debug_assertions; the &mut [T] alias guard is compiled out"
  )
  x <- c(1L, 2L, 3L)
  expect_error(alias_probe(x, x), "aliasing")
})
