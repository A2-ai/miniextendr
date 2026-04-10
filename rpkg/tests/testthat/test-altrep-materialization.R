# Materialization stress tests for ALTREP dataptr path.
#
# These tests exercise the altrep_data1_mut -> AltrepDataptr::dataptr()
# materialization path under various conditions to catch heap corruption
# from pointer provenance or aliasing issues.

# =============================================================================
# Repeated materialization access
# =============================================================================

test_that("materialized ALTREP survives repeated dataptr access", {
  lazy <- lazy_int_seq(1L, 100L, 1L)
  # Force materialization via dataptr (arithmetic)
  y <- lazy + 0L
  expect_equal(y, 1:100)
  # Access dataptr again — should return cached materialized data

  y2 <- lazy + 0L
  expect_equal(y2, 1:100)
  # Third access
  y3 <- lazy + 0L
  expect_equal(y3, 1:100)
})

test_that("materialized ALTREP values correct after GC", {
  lazy <- lazy_int_seq(1L, 50L, 1L)
  y <- lazy + 0L
  expect_equal(y, 1:50)
  gc()
  # After GC, the materialized data should still be valid
  y2 <- lazy + 0L
  expect_equal(y2, 1:50)
  expect_equal(lazy[25], 25L)
})

# =============================================================================
# GC pressure during materialization
# =============================================================================

test_that("materialization under GC pressure (integers)", {
  # Create many ALTREP objects then materialize them all
  seqs <- lapply(1:20, function(i) lazy_int_seq(1L, as.integer(i * 10), 1L))
  gc()
  # Materialize each one via dataptr
  results <- lapply(seqs, function(s) s + 0L)
  gc()
  # Verify correctness
  for (i in seq_along(results)) {
    expect_equal(results[[i]], seq_len(i * 10L))
  }
})

test_that("materialization under GC pressure (reals)", {
  vecs <- lapply(1:20, function(i) altrep_from_doubles(as.double(seq_len(i * 10))))
  gc()
  results <- lapply(vecs, function(v) v + 0)
  gc()
  for (i in seq_along(results)) {
    expect_equal(results[[i]], as.double(seq_len(i * 10)))
  }
})

# =============================================================================
# Interleaved access patterns
# =============================================================================

test_that("interleaved element and dataptr access", {
  lazy <- lazy_int_seq(1L, 20L, 1L)
  # Element access (via Elt, no materialization)
  expect_equal(lazy[1], 1L)
  expect_equal(lazy[10], 10L)
  # Dataptr access (triggers materialization)
  y <- lazy + 0L
  expect_equal(y, 1:20)
  # Element access after materialization
  expect_equal(lazy[1], 1L)
  expect_equal(lazy[20], 20L)
  # Another dataptr access
  y2 <- lazy * 2L
  expect_equal(y2, (1:20) * 2L)
})

test_that("sum then dataptr on same object", {
  lazy <- lazy_int_seq(1L, 100L, 1L)
  # sum uses O(1) formula, no materialization
  expect_equal(sum(lazy), 5050L)
  expect_false(unsafe_C_lazy_int_seq_is_materialized(lazy))
  # Now force materialization
  y <- lazy + 0L
  expect_true(unsafe_C_lazy_int_seq_is_materialized(lazy))
  expect_equal(y, 1:100)
  # sum again after materialization
  expect_equal(sum(lazy), 5050L)
})

# =============================================================================
# Multiple ALTREP types materializing together
# =============================================================================

test_that("mixed ALTREP types materialize without corruption", {
  ints <- lazy_int_seq(1L, 50L, 1L)
  reals <- altrep_from_doubles(as.double(1:50))
  strings <- altrep_from_strings(as.character(1:50))
  gc()

  # Materialize integers
  int_result <- ints + 0L
  gc()
  # Materialize reals
  real_result <- reals + 0
  gc()
  # Access strings
  str_result <- strings[1:5]

  expect_equal(int_result, 1:50)
  expect_equal(real_result, as.double(1:50))
  expect_equal(str_result, as.character(1:5))
})

# =============================================================================
# Serialization round-trip then materialization
# =============================================================================

test_that("serialized ALTREP materializes correctly after round-trip", {
  lazy <- lazy_int_seq(1L, 30L, 1L)
  # Serialize + unserialize
  lazy2 <- unserialize(serialize(lazy, NULL))
  # The unserialized version should not be materialized
  expect_false(unsafe_C_lazy_int_seq_is_materialized(lazy2))
  # Materialize the round-tripped version
  y <- lazy2 + 0L
  expect_equal(y, 1:30)
  expect_true(unsafe_C_lazy_int_seq_is_materialized(lazy2))
  # Original should still work
  expect_equal(lazy[15], 15L)
})

# =============================================================================
# Large vector materialization
# =============================================================================

test_that("large vector materialization stress", {
  n <- 100000L
  lazy <- lazy_int_seq(1L, n, 1L)
  y <- lazy + 0L
  expect_equal(length(y), n)
  expect_equal(y[1], 1L)
  expect_equal(y[n], n)
  expect_equal(sum(y), sum(1:n))
})

# =============================================================================
# Rapid create-materialize cycles
# =============================================================================

test_that("rapid create-materialize-discard cycles", {
  for (i in 1:50) {
    lazy <- lazy_int_seq(1L, as.integer(i), 1L)
    y <- lazy + 0L
    expect_equal(length(y), i)
    expect_equal(y[1], 1L)
  }
  gc()
  # Final check — heap should be intact
  final <- lazy_int_seq(1L, 100L, 1L)
  expect_equal(sum(final + 0L), 5050L)
})

# =============================================================================
# Subprocess isolation for crash detection
# =============================================================================

test_that("materialization does not corrupt heap (subprocess)", {
  skip_on_os("windows")
  skip_if_not_installed("callr")
  lib_paths <- .libPaths()
  result <- callr::r(function(lp) {
    .libPaths(lp)
    library(miniextendr)
    tryCatch({
      # Create and materialize several ALTREP objects
      for (i in 1:10) {
        lazy <- lazy_int_seq(1L, as.integer(i * 100), 1L)
        y <- lazy + 0L
        stopifnot(length(y) == i * 100L)
        stopifnot(y[1] == 1L)
      }
      gc()
      # Mixed types
      reals <- altrep_from_doubles(as.double(1:1000))
      r <- reals + 0
      stopifnot(length(r) == 1000L)
      gc()
      TRUE
    }, error = function(e) {
      paste0("ERROR: ", conditionMessage(e))
    })
  }, args = list(lp = lib_paths), timeout = 60)
  expect_true(result)
})
