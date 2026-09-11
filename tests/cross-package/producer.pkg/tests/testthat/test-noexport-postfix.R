# Crate-level `noexport_postfix` default (#1454)
#
# producer.pkg's Cargo.toml sets `[package.metadata.miniextendr]
# noexport_postfix = "_impl"`. Every `noexport` free function without its own
# `r_name` / `postfix` gets that suffix; explicit names and exported functions
# are untouched.

test_that("noexport free functions take the crate-level postfix", {
  ns <- asNamespace("producer.pkg")
  expect_true(exists("producer_internal_probe_impl", envir = ns, inherits = FALSE))
  expect_false(exists("producer_internal_probe", envir = ns, inherits = FALSE))
  expect_identical(producer.pkg:::producer_internal_probe_impl(), 1454L)
  expect_false("producer_internal_probe_impl" %in% getNamespaceExports(ns))
})

test_that("per-item postfix and r_name win over the crate default", {
  ns <- asNamespace("producer.pkg")
  expect_identical(producer.pkg:::producer_internal_probe_explicit_own(), 1L)
  expect_false(exists("producer_internal_probe_explicit_impl", envir = ns, inherits = FALSE))
  expect_identical(producer.pkg:::producer_internal_probe_renamed(), 2L)
  expect_false(exists("producer_internal_probe_r_name_impl", envir = ns, inherits = FALSE))
})

test_that("exported functions keep their Rust name", {
  ns <- asNamespace("producer.pkg")
  expect_identical(producer_exported_probe(), 3L)
  expect_false(exists("producer_exported_probe_impl", envir = ns, inherits = FALSE))
})
