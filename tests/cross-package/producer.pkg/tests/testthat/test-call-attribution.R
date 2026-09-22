# Crate-level `call_attribution` default (#1566)
#
# producer.pkg's Cargo.toml sets `[package.metadata.miniextendr]
# call_attribution = "caller"`. Every `noexport` / `internal` free function
# without its own `call = ...` (or `Call` / `CallerCall` marker) attributes
# conditions to its caller; a per-item attribute wins, and exported functions
# keep the wrapper's own call.

test_that("noexport free functions take the crate-level caller attribution", {
  e <- tryCatch(producer.pkg:::producer_attributed(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be positive, got -1", fixed = TRUE)
  expect_equal(conditionCall(e), quote(producer.pkg:::producer_attributed(value = -1L)))
  expect_identical(producer.pkg:::producer_attributed(3L), 3L)
})

test_that("a per-item `call = wrapper` wins over the crate default", {
  e <- tryCatch(producer.pkg:::producer_self_attributed(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(producer_self_attributed_probe_impl(x = value)))
})

test_that("exported functions keep the wrapper's own call under a caller default", {
  via <- function(value) producer_exported_attributed_probe(value)
  e <- tryCatch(via(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(producer_exported_attributed_probe(x = value)))
  expect_identical(producer_exported_attributed_probe(2L), 2L)
})
