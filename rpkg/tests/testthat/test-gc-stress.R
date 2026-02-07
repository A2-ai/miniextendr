# GC protection stress tests
#
# These tests verify that ExternalPtr and ALTREP objects survive garbage
# collection under various stress conditions.

# =============================================================================
# ExternalPtr GC stress
# =============================================================================

test_that("many ExternalPtr objects survive GC", {
  # Create many objects in a loop with GC between batches
  ptrs <- vector("list", 100)

  for (batch in 1:10) {
    for (i in ((batch - 1) * 10 + 1):(batch * 10)) {
      ptrs[[i]] <- SharedData$create(as.double(i), as.double(i * 2), paste("item", i))
    }
    gc()
  }

  # All objects should still be accessible
  for (i in 1:100) {
    expect_equal(ptrs[[i]]$get_x(), as.double(i))
    expect_equal(ptrs[[i]]$get_label(), paste("item", i))
  }
})

test_that("ExternalPtr survives repeated GC cycles", {
  data <- SharedData$create(1.0, 2.0, "gc-test")

  # Hammer GC multiple times
  for (i in 1:20) {
    gc()
    gc()
  }

  # Object should still work
  expect_equal(data$get_x(), 1.0)
  expect_equal(data$get_y(), 2.0)
  expect_equal(data$get_label(), "gc-test")
})

test_that("temporary ExternalPtr objects are collected without crashing", {
  # Create and immediately drop many ExternalPtr objects
  for (i in 1:200) {
    SharedData$create(as.double(i), as.double(i), "temp")
  }

  gc()

  # Should not crash - if we get here, finalizers ran successfully
  succeed()
})

# =============================================================================
# ALTREP GC stress
# =============================================================================

test_that("ALTREP vector elements accessible after GC", {
  v <- into_sexp_altrep(1:1000)

  for (i in 1:10) {
    gc()
  }

  # Elements should still be accessible
  expect_equal(v[1], 1L)
  expect_equal(v[500], 500L)
  expect_equal(v[1000], 1000L)
  expect_equal(length(v), 1000L)
})

test_that("many ALTREP vectors survive concurrent GC", {
  vectors <- vector("list", 50)

  for (i in 1:50) {
    vectors[[i]] <- into_sexp_altrep(seq_len(i * 10))
    if (i %% 10 == 0) gc()
  }

  # All vectors should be accessible
  for (i in 1:50) {
    expect_equal(length(vectors[[i]]), i * 10)
    expect_equal(vectors[[i]][1], 1L)
  }
})

# =============================================================================
# Mixed GC stress (ExternalPtr + ALTREP interleaved)
# =============================================================================

test_that("ExternalPtr and ALTREP interleaved with GC", {
  ptrs <- vector("list", 20)
  vecs <- vector("list", 20)

  for (i in 1:20) {
    ptrs[[i]] <- SharedData$create(as.double(i), 0.0, paste0("p", i))
    vecs[[i]] <- into_sexp_altrep(seq_len(i))
    gc()
  }

  # All should be accessible
  for (i in 1:20) {
    expect_equal(ptrs[[i]]$get_x(), as.double(i))
    expect_equal(length(vecs[[i]]), i)
  }
})
