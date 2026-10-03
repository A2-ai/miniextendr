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

test_that("group_by_metadata rejects a double `.rows` column, as dplyr does", {
  # dplyr's validate_grouped_df() requires every `.rows` element to be an
  # integer vector; whole-number doubles are rejected too.
  df <- data.frame(g = c("a", "a", "b"), v = c(10, 20, 30))
  gdf <- make_grouped(
    df,
    keys_df = data.frame(g = c("a", "b"), stringsAsFactors = FALSE),
    rows = list(c(1, 2), 3) # doubles
  )
  msg <- "`.rows` column must be a list of one-based integer vectors, but the element for group 1 is double"
  expect_error(group_metadata_sizes(gdf), msg)
  expect_error(group_declaration_of(gdf), msg)

  skip_if_not_installed("dplyr")
  expect_error(dplyr::group_vars(gdf), "valid <grouped_df>")
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
    rows = list("1") # a character element
  )
  expect_error(
    group_metadata_keys(gdf),
    "list of one-based integer vectors, but the element for group 1 is character"
  )
})

test_that("group_by_metadata errors on a plain (non-grouped) frame", {
  plain <- data.frame(g = c("a", "b"), v = c(1, 2))
  expect_error(group_metadata_keys(plain), "not a grouped_df")
  # dplyr ignores a `groups` attribute on a frame without the grouped_df class.
  stray <- plain
  attr(stray, "groups") <- data.frame(g = c("a", "b"), .rows = I(list(1L, 2L)))
  expect_error(group_metadata_keys(stray), "not a grouped_df")
})

test_that("group_by_metadata reports a grouped class without a valid `groups` frame as corrupt", {
  # A grouped_df class without a `groups` data frame is corrupt, not
  # ungrouped: dplyr's group_vars() errors on it too.
  gdf <- structure(
    data.frame(g = c("a", "b"), v = c(1, 2)),
    class = c("grouped_df", "tbl_df", "tbl", "data.frame")
  )
  expect_error(group_metadata_keys(gdf), "the `groups` attribute must be a data frame")
  attr(gdf, "groups") <- data.frame(g = c("a", "b"))
  expect_error(
    group_metadata_keys(gdf),
    "the last column of the `groups` attribute must be called `.rows`"
  )
})

test_that("group_by_metadata reads double, Date and POSIXct key columns", {
  df <- data.frame(k = c(2.5, 1, 2.5, 1), v = 1:4)
  gdf <- make_grouped(df, data.frame(k = c(1, 2.5)), list(c(2L, 4L), c(1L, 3L)))
  expect_identical(group_metadata_keys(gdf), c("1", "2.5"))
  expect_identical(group_metadata_sizes(gdf), c(2L, 2L))
  expect_identical(group_metadata_frames(gdf)[["2.5"]]$v, c(1L, 3L))

  d <- as.Date("2024-01-01") + c(1, 0, 1, 0)
  gdf <- make_grouped(
    data.frame(k = d, v = 1:4),
    data.frame(k = as.Date(c("2024-01-01", "2024-01-02"))),
    list(c(2L, 4L), c(1L, 3L))
  )
  expect_identical(group_metadata_keys(gdf), c("2024-01-01", "2024-01-02"))

  p <- as.POSIXct("2024-01-01 10:00:00", tz = "UTC") + c(3600, 0)
  gdf <- make_grouped(
    data.frame(k = p, v = 1:2),
    data.frame(k = sort(p)),
    list(2L, 1L)
  )
  expect_identical(
    group_metadata_keys(gdf),
    c("2024-01-01 10:00:00", "2024-01-01 11:00:00")
  )
})

# --- stale `.rows` (the frame changed after grouping) -------------------------

# What `[` does to a grouped frame when dplyr is not loaded: the data.frame
# method subsets the rows and keeps every other attribute, the old `groups`
# included. (With dplyr loaded, its `[.grouped_df` regroups instead, so call
# the data.frame method directly.)
subset_without_dplyr <- function(x, i) base::`[.data.frame`(x, i, )

test_that("group_by_metadata reports stale `.rows` after a subset", {
  df <- data.frame(k = c(1L, 1L, 2L, 2L), v = 1:4)
  gdf <- make_grouped(df, data.frame(k = 1:2), list(1:2, 3:4))

  # Fewer rows: group 2's cached indices point past the end.
  fewer <- subset_without_dplyr(gdf, c(1, 3))
  expect_error(
    group_metadata_sizes(fewer),
    "index 3 for group 2 is out of range.*metadata is stale.*without dplyr loaded"
  )
  # More rows (a duplicated row): the new row 5 is in no group.
  more <- subset_without_dplyr(gdf, c(1, 1, 2, 3, 4))
  expect_error(
    group_metadata_sizes(more),
    "do not cover row 5.*metadata is stale"
  )

  # The declaration reads no `.rows`, so it still works on both.
  expect_identical(group_declaration_of(fewer), list(vars = "k", drop = TRUE))
  expect_identical(group_declaration_of(more), list(vars = "k", drop = TRUE))
})

test_that("group_by_metadata reports `.rows` that claim a row twice", {
  df <- data.frame(k = c(1L, 1L, 2L, 2L), v = 1:4)
  across <- make_grouped(df, data.frame(k = 1:2), list(1:2, 2:4))
  expect_error(
    group_metadata_sizes(across),
    "put row 2 in both group 1 and group 2.*metadata is stale"
  )
  within <- make_grouped(df, data.frame(k = 1:2), list(c(1L, 1L, 2L), 3:4))
  expect_error(
    group_metadata_sizes(within),
    "list row 1 twice in group 1.*metadata is stale"
  )
})

test_that("a dplyr grouped tibble subset without dplyr's `[` is reported stale", {
  skip_if_not_installed("dplyr")
  gdf <- dplyr::group_by(data.frame(k = c(1L, 1L, 2L, 2L), v = 1:4), k)
  stale <- subset_without_dplyr(gdf, c(1, 3))
  expect_error(group_metadata_sizes(stale), "out of range.*metadata is stale")
  expect_identical(group_declaration_of(stale), list(vars = "k", drop = TRUE))
  # tibble's `[` (what a tibble dispatches to when only tibble is loaded)
  # keeps the old `groups` attribute too.
  stale_tbl <- getS3method("[", "tbl_df")(gdf, c(1, 3), )
  expect_error(group_metadata_sizes(stale_tbl), "out of range.*metadata is stale")
  expect_identical(group_declaration_of(stale_tbl), list(vars = "k", drop = TRUE))
  # dplyr's own `[` regroups the subset, which then reads cleanly.
  expect_identical(group_metadata_sizes(gdf[c(1, 3), ]), c(1L, 1L))
})

# --- the grouping declaration (group_vars + .drop) ----------------------------

test_that("group_declaration reads the grouping variables and .drop policy", {
  df <- data.frame(g = c("a", "b"), k = 1:2, v = c(1, 2))
  gdf <- make_grouped(df, data.frame(g = c("a", "b"), k = 1:2), list(1L, 2L))
  # No `.drop` attribute: dplyr's default is to drop.
  expect_identical(group_declaration_of(gdf), list(vars = c("g", "k"), drop = TRUE))

  with_drop <- function(value) {
    groups <- attr(gdf, "groups")
    attr(groups, ".drop") <- value
    out <- gdf
    attr(out, "groups") <- groups
    out
  }
  expect_false(group_declaration_of(with_drop(FALSE))$drop)
  # Only an exact `FALSE` keeps empty groups: identical(.drop, FALSE).
  not_false <- list(TRUE, NA, 0L, c(FALSE, FALSE), structure(FALSE, x = 1), "FALSE")
  for (value in not_false) {
    expect_true(group_declaration_of(with_drop(value))$drop, info = deparse(value))
  }

  # Not grouped: a plain frame, or a stray `groups` attribute on one.
  expect_null(group_declaration_of(df))
  stray <- df
  attr(stray, "groups") <- attr(gdf, "groups")
  expect_null(group_declaration_of(stray))

  # Key types that group_by_metadata rejects do not matter here.
  listy <- make_grouped(df, data.frame(g = c("a", "b")), list(1L, 2L))
  groups <- attr(listy, "groups")
  groups$g <- list(1, 2)
  attr(listy, "groups") <- groups
  expect_identical(group_declaration_of(listy), list(vars = "g", drop = TRUE))
})

test_that("group_declaration matches dplyr::group_vars() and group_by_drop_default()", {
  skip_if_not_installed("dplyr")
  df <- data.frame(
    g = factor(c("a", "b"), levels = c("a", "b", "z")),
    k = c(1.5, 2.5),
    v = 1:2
  )
  grouped <- list(
    dplyr::group_by(df, g),
    dplyr::group_by(df, g, k, .drop = FALSE),
    dplyr::group_by(df, k, .drop = TRUE),
    dplyr::rowwise(df),
    dplyr::rowwise(df, g)
  )
  for (gdf in grouped) {
    expect_identical(
      group_declaration_of(gdf),
      list(vars = dplyr::group_vars(gdf), drop = dplyr::group_by_drop_default(gdf))
    )
  }
  expect_null(group_declaration_of(dplyr::ungroup(dplyr::group_by(df, g))))
})

# A grouped_df's `groups` attribute must pass dplyr's validate_grouped_df()
# structural checks, which group_vars() runs: it is a data frame, its last
# column is `.rows`, and `.rows` is a list of integer vectors. Row bounds are
# not checked (check_bounds = FALSE).

# A `groups` frame built without dplyr: the columns in `columns` (a named
# list), in order, with `n` rows.
groups_df <- function(columns, n) {
  structure(columns, class = "data.frame", row.names = c(NA, -n))
}
corrupt_groups <- function() {
  list(
    missing = list(groups = NULL, error = "the `groups` attribute must be a data frame"),
    `not a data frame` = list(
      groups = list(ID = 1:2, .rows = list(1:2, 3L)),
      error = "the `groups` attribute must be a data frame"
    ),
    `no columns` = list(
      groups = data.frame(),
      error = "the `groups` attribute must be a data frame"
    ),
    `no .rows` = list(
      groups = groups_df(list(ID = 1:2), 2L),
      error = "the last column of the `groups` attribute must be called `.rows`"
    ),
    `.rows not last` = list(
      groups = groups_df(list(.rows = list(1:2, 3L), ID = 1:2), 2L),
      error = "the last column of the `groups` attribute must be called `.rows`"
    ),
    `.rows not a list` = list(
      groups = groups_df(list(ID = 1:2, .rows = c(1L, 3L)), 2L),
      error = "the `.rows` column must be a list of one-based integer vectors, not integer"
    ),
    `.rows of doubles` = list(
      groups = groups_df(list(ID = 1:2, .rows = list(c(1, 2), 3)), 2L),
      error = "but the element for group 1 is double"
    ),
    `.rows with NULL` = list(
      groups = groups_df(list(ID = 1:2, .rows = list(1:2, NULL)), 2L),
      error = "but the element for group 2 is NULL"
    )
  )
}
grouped_obs <- function(groups) {
  structure(
    data.frame(ID = c(1L, 1L, 2L), x = 1:3),
    class = c("grouped_df", "tbl_df", "tbl", "data.frame"),
    groups = groups
  )
}

test_that("group_declaration refuses a grouped_df with a corrupt `groups` attribute", {
  for (case in names(corrupt_groups())) {
    spec <- corrupt_groups()[[case]]
    expect_error(group_declaration_of(grouped_obs(spec$groups)), spec$error, info = case)
  }

  # Out-of-bounds `.rows` are not checked: the declaration still reads.
  oob <- grouped_obs(groups_df(list(ID = 1:2, .rows = list(1:2, 9L)), 2L))
  expect_identical(group_declaration_of(oob), list(vars = "ID", drop = TRUE))
})

test_that("group_declaration errors exactly where dplyr::group_vars() does", {
  skip_if_not_installed("dplyr")
  for (case in names(corrupt_groups())) {
    gdf <- grouped_obs(corrupt_groups()[[case]]$groups)
    expect_error(dplyr::group_vars(gdf), "valid <grouped_df>", info = case)
    expect_error(group_declaration_of(gdf), "not a valid grouped_df", info = case)
  }

  # Where dplyr reads the declaration, the same vars and drop.
  ok <- list(
    `out of bounds` = tibble::new_tibble(list(ID = 1:2, .rows = list(1:2, 9L)), nrow = 2L),
    valid = tibble::new_tibble(list(ID = 1:2, .rows = list(1:2, 3L)), nrow = 2L),
    `.drop = FALSE` = tibble::new_tibble(
      list(ID = 1:2, .rows = list(1:2, 3L)),
      nrow = 2L,
      .drop = FALSE
    ),
    `list_of .rows` = tibble::new_tibble(
      list(ID = 1:2, .rows = vctrs::list_of(1:2, 3L)),
      nrow = 2L
    )
  )
  for (case in names(ok)) {
    gdf <- grouped_obs(ok[[case]])
    expect_identical(
      group_declaration_of(gdf),
      list(vars = dplyr::group_vars(gdf), drop = dplyr::group_by_drop_default(gdf)),
      info = case
    )
  }
})

test_that("group_declaration reads rowwise frames with and without key columns", {
  rowwise_obs <- function(groups) {
    structure(
      data.frame(ID = c(1L, 1L, 2L), x = 1:3),
      class = c("rowwise_df", "tbl_df", "tbl", "data.frame"),
      groups = groups
    )
  }
  no_keys <- groups_df(list(.rows = list(1L, 2L, 3L)), 3L)
  keyed <- groups_df(list(ID = c(1L, 1L, 2L), .rows = list(1L, 2L, 3L)), 3L)
  expect_identical(group_declaration_of(rowwise_obs(no_keys)), list(vars = character(), drop = TRUE))
  expect_identical(group_declaration_of(rowwise_obs(keyed)), list(vars = "ID", drop = TRUE))
  # group_by_drop_default() has no rowwise_df method, so `.drop` is ignored.
  attr(keyed, ".drop") <- FALSE
  expect_identical(group_declaration_of(rowwise_obs(keyed)), list(vars = "ID", drop = TRUE))

  # The structural checks apply to a rowwise_df too. Here dplyr is looser:
  # group_data.rowwise_df() returns the attribute without validating it.
  expect_error(
    group_declaration_of(rowwise_obs(NULL)),
    "the `groups` attribute must be a data frame"
  )

  skip_if_not_installed("dplyr")
  df <- data.frame(g = c("a", "b", "b"), v = 1:3)
  rowwise <- list(dplyr::rowwise(df), dplyr::rowwise(df, g))
  dropless <- dplyr::rowwise(df, g)
  groups <- attr(dropless, "groups")
  attr(groups, ".drop") <- FALSE
  attr(dropless, "groups") <- groups
  rowwise <- c(rowwise, list(dropless))
  for (rw in rowwise) {
    expect_identical(
      group_declaration_of(rw),
      list(vars = dplyr::group_vars(rw), drop = dplyr::group_by_drop_default(rw))
    )
  }
  # A rowwise frame subset without dplyr keeps its old `groups`; like
  # group_vars(), the declaration does not check that `.rows` fit the rows.
  stale <- subset_without_dplyr(dplyr::rowwise(df, g), 1:2)
  expect_identical(group_declaration_of(stale), list(vars = dplyr::group_vars(stale), drop = TRUE))
})

test_that("a plain data.frame with a stray `groups` attribute is not grouped", {
  stray <- data.frame(ID = c(1L, 1L, 2L), x = 1:3)
  attr(stray, "groups") <- groups_df(list(ID = 1:2, .rows = list(1:2, 3L)), 2L)
  expect_null(group_declaration_of(stray))
  attr(stray, "groups") <- "not a frame"
  expect_null(group_declaration_of(stray))

  skip_if_not_installed("dplyr")
  # dplyr reads it as ungrouped too: no grouping variables, the default drop.
  expect_identical(dplyr::group_vars(stray), character())
  expect_true(dplyr::group_by_drop_default(stray))
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
  expect_identical(group_declaration_of(gdf), list(vars = "g", drop = FALSE))
})

test_that("group_by_metadata reads every dplyr key type", {
  skip_if_not_installed("dplyr")
  keys <- list(
    integer = c(1L, 2L, 1L, 2L),
    double = c(2.5, 1, 2.5, 1),
    Date = as.Date("2024-01-01") + c(1, 0, 1, 0),
    POSIXct = as.POSIXct("2024-01-01 10:00:00", tz = "UTC") + c(3600, 0, 3600, 0)
  )
  for (type in names(keys)) {
    gdf <- dplyr::group_by(data.frame(k = keys[[type]], v = 1:4), k)
    expect_identical(group_metadata_sizes(gdf), c(2L, 2L), info = type)
    expect_identical(
      group_metadata_keys(gdf),
      as.character(dplyr::group_keys(gdf)$k),
      info = type
    )
    expect_identical(
      group_declaration_of(gdf),
      list(vars = "k", drop = TRUE),
      info = type
    )
  }

  # A factor with an unused level under .drop = FALSE keeps a zero-row group.
  f <- factor(c("a", "a", "b", "b"), levels = c("a", "b", "z"))
  gdf <- dplyr::group_by(data.frame(k = f, v = 1:4), k, .drop = FALSE)
  expect_identical(group_metadata_sizes(gdf), c(2L, 2L, 0L))
  expect_identical(group_declaration_of(gdf), list(vars = "k", drop = FALSE))
})
