# Ingest of a dplyr `grouped_df`'s grouping metadata (`attr(df, "groups")`) via
# DataFrame::group_by_metadata (#1126). The reader honors the caller's grouping
# verbatim — no recomputation — reading key columns + the `.rows` list-column
# (1-based indices) from the `groups` attribute.
#
# Coverage runs two ways so it never depends on dplyr being installed:
#   1. hand-constructed `groups` attribute (exactly as dplyr documents it), and
#   2. a real `dplyr::group_by()` frame (skipped if dplyr is unavailable).

# Build a frame carrying a dplyr-style `groups` attribute by hand. `keys_df` is
# a data.frame of key columns (one row per group, in the desired group order);
# `rows` is a list of 1-based integer index vectors, one per group. dplyr stores
# the `groups` frame as a tbl_df, but the reader accepts any data.frame — a
# plain one is the minimal faithful construction.
make_grouped <- function(df, keys_df, rows) {
  groups <- keys_df
  groups[[".rows"]] <- rows # list-column of integer index vectors
  attr(df, "groups") <- groups
  class(df) <- c("grouped_df", "tbl_df", "tbl", "data.frame")
  df
}

test_that("group_by_metadata reads a single character key in groups-frame order", {
  df <- data.frame(g = c("a", "a", "b"), v = c(10, 20, 30))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "b"), stringsAsFactors = FALSE),
    rows = list(c(1L, 2L), 3L)
  )

  expect_identical(group_metadata_keys(gdf), c("a", "b"))
  expect_identical(group_metadata_sizes(gdf), c(2L, 1L))

  # 1-based -> 0-based conversion is correct by row *content*, not just size.
  frames <- group_metadata_frames(gdf)
  expect_identical(names(frames), c("a", "b"))
  expect_identical(frames[["a"]]$v, c(10, 20))
  expect_identical(frames[["b"]]$v, 30)
})

test_that("group_by_metadata reads a two-column key as `.`-joined tuple labels", {
  df <- data.frame(g = c("a", "a", "b"), k = c(1L, 2L, 1L), v = c(10, 20, 30))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "a", "b"), k = c(1L, 2L, 1L)),
    rows = list(1L, 2L, 3L)
  )

  expect_identical(group_metadata_keys(gdf), c("a.1", "a.2", "b.1"))
  expect_identical(group_metadata_sizes(gdf), c(1L, 1L, 1L))
  frames <- group_metadata_frames(gdf)
  expect_identical(frames[["a.2"]]$v, 20)
})

test_that("group_by_metadata keeps `.drop = FALSE` empty groups", {
  df <- data.frame(g = c("a", "a", "b"), v = c(10, 20, 30))
  # Third group ("z") has zero rows — the empty-factor-level / .drop=FALSE case.
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "b", "z"), stringsAsFactors = FALSE),
    rows = list(c(1L, 2L), 3L, integer(0))
  )

  expect_identical(group_metadata_keys(gdf), c("a", "b", "z"))
  expect_identical(group_metadata_sizes(gdf), c(2L, 1L, 0L))
  frames <- group_metadata_frames(gdf)
  expect_identical(nrow(frames[["z"]]), 0L)
  expect_identical(names(frames[["z"]]), c("g", "v"))
})

test_that("group_by_metadata accepts a double (integerish) `.rows` column", {
  # dplyr uses integer .rows, but a REALSXP of whole numbers is valid too.
  df <- data.frame(g = c("a", "a", "b"), v = c(10, 20, 30))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "b"), stringsAsFactors = FALSE),
    rows = list(c(1, 2), 3) # doubles
  )

  expect_identical(group_metadata_sizes(gdf), c(2L, 1L))
  expect_identical(group_metadata_frames(gdf)[["a"]]$v, c(10, 20))
})

test_that("group_by_metadata errors on an out-of-range `.rows` index", {
  df <- data.frame(g = c("a", "a", "b"), v = c(10, 20, 30))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "b"), stringsAsFactors = FALSE),
    rows = list(c(1L, 99L), 3L) # 99 > nrow(df) == 3
  )
  expect_error(group_metadata_keys(gdf), "out of range")

  gdf0 <- make_grouped(
    df,
    keys_df = data.frame(g = "a", stringsAsFactors = FALSE),
    rows = list(0L) # < 1
  )
  expect_error(group_metadata_keys(gdf0), "out of range")
})

test_that("group_by_metadata errors on a non-integer `.rows` element", {
  df <- data.frame(g = c("a", "a"), v = c(10, 20))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = "a", stringsAsFactors = FALSE),
    rows = list(c(1.5, 2)) # non-integer double
  )
  expect_error(group_metadata_keys(gdf), "not an integer index vector")
})

test_that("group_by_metadata errors on a plain (non-grouped) frame", {
  plain <- data.frame(g = c("a", "b"), v = c(1, 2))
  expect_error(group_metadata_keys(plain), "not a grouped_df")
})

# --- Parity against real dplyr grouping ---------------------------------------

test_that("group_by_metadata matches dplyr::group_by on a single key", {
  skip_if_not_installed("dplyr")
  df <- data.frame(g = c("b", "a", "b", "a"), v = c(1, 2, 3, 4))
  gdf <- dplyr::group_by(df, g)

  expect_identical(
    group_metadata_keys(gdf),
    as.character(dplyr::group_keys(gdf)$g)
  )
  expect_identical(group_metadata_sizes(gdf), dplyr::group_size(gdf))

  frames <- group_metadata_frames(gdf)
  # dplyr sorts keys ascending -> group order a, b.
  expect_identical(sort(frames[["a"]]$v), c(2, 4))
  expect_identical(sort(frames[["b"]]$v), c(1, 3))
})

test_that("group_by_metadata matches dplyr::group_by on two keys", {
  skip_if_not_installed("dplyr")
  df <- data.frame(g = c("a", "a", "b", "b"), k = c(1L, 2L, 1L, 2L), v = 1:4)
  gdf <- dplyr::group_by(df, g, k)

  gk <- dplyr::group_keys(gdf)
  expect_identical(
    group_metadata_keys(gdf),
    paste(gk$g, gk$k, sep = ".")
  )
  expect_identical(group_metadata_sizes(gdf), dplyr::group_size(gdf))
})

test_that("group_by_metadata retains dplyr .drop=FALSE empty groups", {
  skip_if_not_installed("dplyr")
  df <- data.frame(
    g = factor(c("a", "a", "b"), levels = c("a", "b", "z")),
    v = c(10, 20, 30)
  )
  gdf <- dplyr::group_by(df, g, .drop = FALSE)

  # dplyr keeps the empty "z" level as a zero-row group.
  expect_identical(group_metadata_keys(gdf), c("a", "b", "z"))
  expect_identical(group_metadata_sizes(gdf), dplyr::group_size(gdf))
  expect_identical(group_metadata_sizes(gdf), c(2L, 1L, 0L))
})
