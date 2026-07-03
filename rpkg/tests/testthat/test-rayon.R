# Tests for rayon parallel computation feature

# Skip all tests if rayon feature is not enabled
skip_if_missing_feature("rayon")

test_that("rayon_parallel_sum computes correct sum", {
  x <- as.numeric(1:1000)
  result <- rayon_parallel_sum(x)
  expect_equal(result, sum(x))
})
test_that("rayon_parallel_sum handles empty vector", {
  result <- rayon_parallel_sum(numeric(0))
  expect_equal(result, 0)
})

test_that("rayon_parallel_sqrt computes element-wise sqrt", {
  x <- c(1, 4, 9, 16, 25)
  result <- rayon_parallel_sqrt(x)
  expect_equal(result, sqrt(x))
})

test_that("rayon_parallel_sqrt handles large vector", {
  x <- as.numeric(1:10000)
  result <- rayon_parallel_sqrt(x)
  expect_equal(result, sqrt(x))
})

test_that("rayon_parallel_filter_positive filters correctly", {
  x <- c(-2, -1, 0, 1, 2, 3)
  result <- rayon_parallel_filter_positive(x)
  expect_equal(sort(result), c(1, 2, 3))
})

test_that("rayon_parallel_filter_positive returns empty for all negative", {
  x <- c(-3, -2, -1)
  result <- rayon_parallel_filter_positive(x)
  expect_length(result, 0)
})

test_that("rayon_vec_collect returns correct length and values", {
  result <- rayon_vec_collect(100L)
  expect_length(result, 100)
  expect_equal(result[1], 0)
  expect_equal(result[2], 1)
  expect_equal(result[5], 2) # sqrt(4) = 2
})

test_that("rayon_with_r_vec returns correct R vector", {
  result <- rayon_with_r_vec(100L)
  expect_type(result, "double")
  expect_length(result, 100)
  expect_equal(result[1], 0)
  expect_equal(result[2], 1)
  expect_equal(result[5], 2) # sqrt(4) = 2
})

test_that("rayon_with_r_vec_map returns correct R vector", {
  result <- rayon_with_r_vec_map(100L)
  expect_type(result, "double")
  expect_length(result, 100)
  expect_equal(result, sqrt(as.numeric(0:99)))
})

test_that("rayon_par_map computes element-wise sqrt", {
  x <- as.numeric(1:1000)
  result <- rayon_par_map(x)
  expect_type(result, "double")
  expect_length(result, 1000)
  expect_equal(result, sqrt(x))
})

test_that("rayon_par_map handles empty vector", {
  result <- rayon_par_map(numeric(0))
  expect_length(result, 0)
})

test_that("rayon_par_map2 computes element-wise addition", {
  a <- as.numeric(1:500)
  b <- as.numeric(501:1000)
  result <- rayon_par_map2(a, b)
  expect_type(result, "double")
  expect_length(result, 500)
  expect_equal(result, a + b)
})

test_that("rayon_par_map3 computes fused multiply-add", {
  a <- c(1, 2, 3, 4, 5)
  b <- c(10, 20, 30, 40, 50)
  c_vec <- c(100, 200, 300, 400, 500)
  result <- rayon_par_map3(a, b, c_vec)
  expect_type(result, "double")
  expect_length(result, 5)
  expect_equal(result, a * b + c_vec)
})

test_that("rayon_with_r_matrix returns correct matrix", {
  result <- rayon_with_r_matrix(3L, 4L)
  expect_true(is.matrix(result))
  expect_equal(dim(result), c(3, 4))
  # Value at [row, col] should be (row-1) * (col-1) (0-indexed in Rust)
  expect_equal(result[1, 1], 0) # 0 * 0 = 0
  expect_equal(result[2, 2], 1) # 1 * 1 = 1
  expect_equal(result[3, 3], 4) # 2 * 2 = 4
  expect_equal(result[3, 4], 6) # 2 * 3 = 6
})

test_that("rayon_parallel_stats returns correct statistics", {
  x <- c(1, 2, 3, 4, 5)
  result <- rayon_parallel_stats(x)
  expect_length(result, 4)
  expect_equal(result[1], sum(x))    # sum = 15

  expect_equal(result[2], min(x))    # min = 1
  expect_equal(result[3], max(x))    # max = 5
  expect_equal(result[4], mean(x))   # mean = 3
})

test_that("rayon_parallel_stats handles single element", {
  x <- c(42)
  result <- rayon_parallel_stats(x)
  expect_equal(result[1], 42)  # sum
  expect_equal(result[2], 42)  # min
  expect_equal(result[3], 42)  # max
  expect_equal(result[4], 42)  # mean
})

test_that("rayon_parallel_sum_int computes integer sum", {
  x <- 1:100L
  result <- rayon_parallel_sum_int(x)
  expect_equal(result, sum(x))
})

test_that("rayon_num_threads returns positive number", {
  result <- rayon_num_threads()
  expect_true(result >= 1)
})

test_that("rayon_in_thread returns FALSE when called from R", {
  # When called from R main thread, we should NOT be in a rayon thread

  result <- rayon_in_thread()
  expect_false(result)
})

# ---------------------------------------------------------------------------
# Thread pool control (docs/RAYON.md "Controlling Parallelism from R"):
# MINIEXTENDR_NUM_THREADS > RAYON_NUM_THREADS > _R_CHECK_LIMIT_CORES_ cap >
# available_parallelism(). The global rayon pool builds once per process, so
# each scenario needs its own fresh subprocess. callr merges `env` over the
# inherited environment, so every scenario pins all three resolver vars
# (blank = unset to the resolver) — otherwise values inherited from the
# calling process skew baselines: R CMD check --as-cran exports
# _R_CHECK_LIMIT_CORES_=TRUE into this very test run.
# ---------------------------------------------------------------------------

run_with_env <- function(expr, env_vars = character()) {
  skip_if_not_installed("callr")
  vars <- c(
    MINIEXTENDR_NUM_THREADS = "",
    RAYON_NUM_THREADS = "",
    `_R_CHECK_LIMIT_CORES_` = ""
  )
  vars[names(env_vars)] <- env_vars
  callr::r(
    function(expr_to_eval) {
      library(miniextendr)
      eval(expr_to_eval)
    },
    args = list(expr_to_eval = substitute(expr)),
    env = c(callr::rcmd_safe_env(), vars),
    timeout = 30
  )
}

test_that("miniextendr_num_threads honors MINIEXTENDR_NUM_THREADS", {
  result <- run_with_env(
    miniextendr_num_threads(),
    c(MINIEXTENDR_NUM_THREADS = "3")
  )
  expect_equal(result, 3L)
})

test_that("miniextendr_num_threads caps at 2 under _R_CHECK_LIMIT_CORES_", {
  result <- run_with_env(
    miniextendr_num_threads(),
    c(`_R_CHECK_LIMIT_CORES_` = "TRUE")
  )
  expect_true(result <= 2L)
})

test_that("miniextendr_num_threads ignores _R_CHECK_LIMIT_CORES_ = FALSE/empty", {
  uncapped <- run_with_env(miniextendr_num_threads())
  for (v in c("FALSE", "false", "")) {
    result <- run_with_env(
      miniextendr_num_threads(),
      c(`_R_CHECK_LIMIT_CORES_` = v)
    )
    expect_equal(result, uncapped)
  }
})

test_that("miniextendr_num_threads defaults to available parallelism", {
  result <- run_with_env(miniextendr_num_threads())
  expect_true(result >= 1L)
})

test_that("miniextendr_set_threads changes the reported count before first use", {
  result <- run_with_env({
    miniextendr_set_threads(2L)
    miniextendr_num_threads()
  })
  expect_equal(result, 2L)
})

test_that("miniextendr_set_threads errors once the pool is already built", {
  msg <- run_with_env({
    miniextendr_num_threads() # builds the pool
    tryCatch(
      miniextendr_set_threads(4L),
      error = function(e) conditionMessage(e)
    )
  })
  expect_match(msg, "already built", fixed = TRUE)
})

test_that("miniextendr_set_threads rejects non-positive input", {
  # Validation panics in the wrapper before any pool interaction, so this is
  # safe to run in-process — no subprocess needed.
  expect_error(miniextendr_set_threads(0L), "positive integer")
})

test_that("rayon_with_r_dataframe builds a correct heterogeneous data.frame", {
  n <- 50L
  df <- rayon_with_r_dataframe(n)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), n)
  expect_equal(ncol(df), 3)
  expect_identical(names(df), c("x", "y", "label"))

  # Column types.
  expect_type(df$x, "double")
  expect_type(df$y, "integer")
  expect_type(df$label, "character")

  # Column values (filled in parallel).
  expect_equal(df$x, sqrt(as.numeric(0:(n - 1))))
  expect_equal(df$y, (0:(n - 1)) * 2L)

  # Character column: every fifth row (i %% 5 == 4) is NA, else "row_<i>".
  expected_label <- ifelse((0:(n - 1)) %% 5 == 4, NA_character_,
                           paste0("row_", 0:(n - 1)))
  expect_identical(df$label, expected_label)

  # Compact row.names form (1:n).
  expect_identical(rownames(df), as.character(seq_len(n)))
})

test_that("rayon_with_r_dataframe handles zero rows", {
  df <- rayon_with_r_dataframe(0L)
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 0)
  expect_equal(ncol(df), 3)
  expect_identical(names(df), c("x", "y", "label"))
})

test_that("rayon_dataframe_wide fills a wide (many-column) frame correctly", {
  # Wide shape: more columns than threads; column-dominated work-list.
  nrow <- 20L
  ncol <- 32L
  df <- rayon_dataframe_wide(nrow, ncol)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), nrow)
  expect_equal(ncol(df), ncol)
  expect_identical(names(df), paste0("c", 0:(ncol - 1)))

  # Every cell c<j>[i] == i * 1000 + j — verifies no cross-column scatter.
  for (j in seq_len(ncol)) {
    expected <- (0:(nrow - 1)) * 1000 + (j - 1)
    expect_equal(df[[j]], as.numeric(expected),
                 info = paste("column", j - 1))
  }
})

test_that("rayon_dataframe_wide handles a tall single chunk per column", {
  # Tall shape: few columns x many rows -> each column shatters into chunks.
  nrow <- 200000L
  ncol <- 3L
  skip_on_cran()
  df <- rayon_dataframe_wide(nrow, ncol)

  expect_equal(nrow(df), nrow)
  expect_equal(ncol(df), ncol)
  # Spot-check first/last rows of each column.
  for (j in seq_len(ncol)) {
    expect_equal(df[[j]][1], as.numeric(j - 1))
    expect_equal(df[[j]][nrow], as.numeric((nrow - 1) * 1000 + (j - 1)))
  }
})

test_that("rayon_dataframe_skewed fills one long + several tiny columns", {
  # Skewed shape: one long numeric column + two short character columns.
  nrow <- 5000L
  df <- rayon_dataframe_skewed(nrow)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), nrow)
  expect_equal(ncol(df), 3)
  expect_identical(names(df), c("big", "t0", "t1"))

  expect_type(df$big, "double")
  expect_equal(df$big, as.numeric(0:(nrow - 1)))

  # Character columns: NA on every third row (i %% 3 == 2).
  is_na <- (0:(nrow - 1)) %% 3 == 2
  expected_t0 <- ifelse(is_na, NA_character_, paste0("a", 0:(nrow - 1)))
  expected_t1 <- ifelse(is_na, NA_character_, paste0("b", 0:(nrow - 1)))
  expect_identical(df$t0, expected_t0)
  expect_identical(df$t1, expected_t1)
})

test_that("rayon handles large parallel workload", {
  skip_on_cran()

  # Stress test with moderately large data (reduced for CI speed)
  n <- 100000L
  set.seed(42)
  x <- runif(n)

  # Parallel reduction order can introduce floating point differences
  # Use relative tolerance based on magnitude of result
  sum_result <- rayon_parallel_sum(x)
  expected_sum <- sum(x)

  expect_equal(sum_result, expected_sum, tolerance = abs(expected_sum) * 1e-10)

  # Parallel sqrt - element-wise, so order doesn't matter
  sqrt_result <- rayon_parallel_sqrt(x)
  expect_equal(sqrt_result, sqrt(x), tolerance = 1e-14)
})

# region: RParallelIterator / RParallelExtend adapter traits (audit A7)

test_that("RParallelIterator aggregations work on a materialized source", {
  res <- rayon_trait_par_stats(c(1, 2, 3, 4))
  expect_equal(res, c(4, 10, 2.5, 1, 4))
})

test_that("RParallelIterator handles the empty source", {
  res <- rayon_trait_par_stats(numeric(0))
  expect_equal(res[1], 0) # par_len
  expect_equal(res[2], 0) # par_sum
  expect_true(is.nan(res[3])) # par_mean of nothing
  expect_equal(res[4], Inf) # par_min_f64 identity
  expect_equal(res[5], -Inf) # par_max_f64 identity
})

test_that("RParallelExtend combines par_extend and par_extend_from_slice", {
  expect_equal(rayon_trait_par_extend(c(3, 1), c(2)), c(1, 2, 3))
  expect_equal(rayon_trait_par_extend(numeric(0), numeric(0)), numeric(0))
})

test_that("RParallelExtend par_clear/par_is_empty reset the collection", {
  expect_identical(rayon_trait_par_clear(c(1, 2, 3)), c(3L, 0L, 1L))
})

# endregion
