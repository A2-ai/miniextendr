test_that("factor constructors retain cold and repeated allocations under gctorture", {
  skip_gc_stress_if_disabled()
  # Match the existing subprocess tests: Windows callr can retain pipe handles
  # after R exits (#94). The dynamic fixture sweep still covers Windows.
  skip_on_os("windows")

  results <- run_isolated({
    # Loading miniextendr does not initialize its Rust factor caches. No factor
    # conversion may precede this block: the first call must hit the cold cache.
    gctorture(TRUE)
    on.exit(gctorture(FALSE), add = TRUE)
    results <- vector("list", 3L)
    for (i in seq_along(results)) {
      results[[i]] <- gc_stress_factor_construction()
    }
    gc()
    gctorture(FALSE)
    results
  }, timeout = 120)

  codes <- c(8L, NA_integer_, 1L, 4L, 8L)
  suffixes <- c("alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta")
  for (i in seq_along(results)) {
    expect_length(results[[i]], 3L)
    expected_levels <- list(
      paste0("miniextendr_factor_1408_", i - 1L, "_raw_", suffixes),
      paste0("miniextendr_factor_1408_", i - 1L, "_one_shot_", suffixes),
      paste0("miniextendr_factor_1408_cached_", suffixes)
    )
    for (j in seq_along(expected_levels)) {
      value <- results[[i]][[j]]
      expected_codes <- if (j == 1L) codes[1:2] else codes
      expect_identical(class(value), "factor")
      expect_identical(as.integer(value), expected_codes)
      expect_identical(levels(value), expected_levels[[j]])
      expect_identical(as.character(value), expected_levels[[j]][expected_codes])
    }
  }
})
