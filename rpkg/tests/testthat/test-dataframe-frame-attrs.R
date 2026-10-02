# Frame-level attributes through the DataFrame producers: `select_rows`, and
# the column producers `drop` / `select` / `prepend_column` / `with_column`.
#
# Every frame attribute except `names` and `row.names` survives, the rule of
# `dplyr_reconstruct()`, `vctrs::vec_slice()` and tibble's `[`. dplyr's
# `groups` is the exception: it indexes the rows and names the key columns, so
# a producer that changes the rows (or removes or replaces a grouping column)
# returns the ungrouped frame instead of a grouped one with stale groups.

frame_with_attrs <- function() {
  df <- data.frame(a = 1:4, b = c("w", "x", "y", "z"))
  attr(df, "meta") <- list(source = "lab", version = 2L)
  attr(df, "units") <- c(a = "mg/L")
  df
}

# A grouped_df built by hand, exactly as dplyr lays one out, so the grouping
# tests run without dplyr installed. `extra` adds non-grouping columns.
hand_grouped <- function(extra = list()) {
  structure(
    c(list(g = c("a", "a", "b", "b"), v = c(1, 2, 3, 4)), extra),
    class = c("grouped_df", "tbl_df", "tbl", "data.frame"),
    row.names = c(NA_integer_, -4L),
    groups = structure(
      list(g = c("a", "b"), .rows = list(1:2, 3:4)),
      class = c("tbl_df", "tbl", "data.frame"),
      row.names = c(NA_integer_, -2L),
      .drop = TRUE
    ),
    meta = "m"
  )
}

tibble_class <- c("tbl_df", "tbl", "data.frame")

# --- select_rows ---------------------------------------------------------------

test_that("select_rows keeps the frame's own attributes, with fresh names and row.names", {
  df <- frame_with_attrs()
  idx <- c(4L, 2L)
  out <- miniextendr:::dataframe_select_rows(df, idx)

  expect_identical(attr(out, "meta"), list(source = "lab", version = 2L))
  expect_identical(attr(out, "units"), c(a = "mg/L"))
  expect_identical(class(out), "data.frame")
  expect_identical(names(out), c("a", "b"))
  # Compact row names for the selected row count, not the source's four.
  expect_identical(.row_names_info(out, 0L), c(NA_integer_, -2L))
  expect_identical(out$a, c(4L, 2L))
  expect_identical(out$b, c("z", "x"))

  # vctrs::vec_slice() keeps every attribute but names and row.names too.
  expect_identical(out, vctrs::vec_slice(df, idx))
  # Base `[` keeps them as well; it only differs in keeping the source rows'
  # names (`4`, `2`).
  expect_equal(out, df[idx, , drop = FALSE], ignore_attr = "row.names")
})

test_that("select_rows matches dplyr_row_slice() and tibble's `[` on an attributed frame", {
  skip_if_not_installed("dplyr")
  df <- frame_with_attrs()
  idx <- c(3L, 1L, 3L)
  expect_identical(
    miniextendr:::dataframe_select_rows(df, idx),
    dplyr::dplyr_row_slice(df, idx)
  )
  expect_identical(
    miniextendr:::dataframe_select_rows(df, idx),
    dplyr::slice(df, idx)
  )

  tb <- tibble::as_tibble(df)
  attr(tb, "meta") <- "tibble meta"
  out <- miniextendr:::dataframe_select_rows(tb, idx)
  expect_identical(out, tb[idx, ])
  expect_identical(attr(out, "meta"), "tibble meta")
})

test_that("select_rows keeps frame and column attributes together", {
  df <- data.frame(
    t = as.POSIXct(c("2026-01-01 08:00", "2026-01-02 09:00", "2026-01-03 10:00"), tz = "UTC"),
    d = as.difftime(c(1, 2, 3), units = "hours"),
    f = factor(c("b", "a", "b"), levels = c("b", "a"))
  )
  attr(df, "units") <- c(d = "h")
  out <- miniextendr:::dataframe_select_rows(df, c(3L, 1L))

  expect_identical(attr(out, "units"), c(d = "h"))
  expect_identical(attr(out$t, "tzone"), "UTC")
  expect_identical(attr(out$d, "units"), "hours")
  expect_identical(levels(out$f), c("b", "a"))
  expect_identical(out, vctrs::vec_slice(df, c(3L, 1L)))
})

test_that("select_rows gives a frame with character row names fresh compact ones", {
  df <- frame_with_attrs()
  rownames(df) <- c("r1", "r2", "r3", "r4")
  out <- miniextendr:::dataframe_select_rows(df, c(3L, 1L, 2L))
  expect_identical(.row_names_info(out, 0L), c(NA_integer_, -3L))
  expect_identical(nrow(out), 3L)
  expect_identical(attr(out, "meta"), attr(df, "meta"))
})

test_that("select_rows keeps a packed data.frame column's own attributes", {
  df <- data.frame(id = 1:3)
  inner <- data.frame(p = c(10, 20, 30))
  attr(inner, "note") <- "packed"
  df$inner <- inner
  out <- miniextendr:::dataframe_select_rows(df, c(2L, 3L))
  expect_identical(attr(out$inner, "note"), "packed")
  expect_identical(out$inner$p, c(20, 30))
  expect_identical(.row_names_info(out$inner, 0L), c(NA_integer_, -2L))
})

test_that("select_rows keeps a stray `groups` attribute on an ungrouped frame", {
  # dplyr only treats `groups` as grouping on a grouped_df / rowwise_df.
  df <- data.frame(a = 1:3)
  attr(df, "groups") <- "not dplyr's"
  out <- miniextendr:::dataframe_select_rows(df, 2L)
  expect_identical(attr(out, "groups"), "not dplyr's")
  expect_identical(class(out), "data.frame")
})

test_that("select_rows returns a hand-built grouped_df ungrouped, without stale groups", {
  gdf <- hand_grouped()
  out <- miniextendr:::dataframe_select_rows(gdf, c(3L, 1L))

  expect_null(attr(out, "groups"))
  expect_identical(class(out), tibble_class)
  expect_null(group_declaration_of(out))
  expect_identical(attr(out, "meta"), "m")
  expect_identical(out$g, c("b", "a"))
  expect_identical(out$v, c(3, 1))
  expect_identical(.row_names_info(out, 0L), c(NA_integer_, -2L))

  # Only the grouped_df entry goes; the rest of the class stays.
  plain <- gdf
  class(plain) <- c("grouped_df", "data.frame")
  expect_identical(class(miniextendr:::dataframe_select_rows(plain, 1L)), "data.frame")
})

test_that("select_rows returns a dplyr grouped_df ungrouped, regroupable as documented", {
  skip_if_not_installed("dplyr")
  gdf <- dplyr::group_by(data.frame(g = c(1, 1, 2, 2), v = 1:4), g)
  attr(gdf, "meta") <- "m"
  idx <- c(4L, 1L, 2L)
  out <- miniextendr:::dataframe_select_rows(gdf, idx)

  expect_null(attr(out, "groups"))
  expect_identical(class(out), tibble_class)
  expect_null(group_declaration_of(out))
  expect_identical(attr(out, "meta"), "m")
  # A valid tibble: dplyr verbs and printing take it.
  expect_no_error(format(out))
  expect_identical(dplyr::mutate(out, w = v * 2L)$w, c(8L, 2L, 4L))

  # Same rows as dplyr's row slice, which regroups; ungroup() also drops
  # `meta`, so compare the rest.
  ref <- dplyr::dplyr_row_slice(gdf, idx)
  expect_equal(out, dplyr::ungroup(ref), ignore_attr = "meta")

  # Regrouping by the input's grouping variables gives dplyr's groups back.
  regrouped <- dplyr::grouped_df(out, dplyr::group_vars(gdf))
  expect_identical(dplyr::group_data(regrouped), dplyr::group_data(ref))
})

test_that("select_rows returns a dplyr rowwise_df ungrouped", {
  skip_if_not_installed("dplyr")
  rdf <- dplyr::rowwise(data.frame(id = 1:3, v = 4:6), id)
  out <- miniextendr:::dataframe_select_rows(rdf, c(3L, 1L))
  expect_null(attr(out, "groups"))
  expect_identical(class(out), tibble_class)
  expect_equal(out, dplyr::ungroup(dplyr::dplyr_row_slice(rdf, c(3L, 1L))))
})

test_that("group sub-frames of a grouped_df come back as valid ungrouped frames", {
  frames <- group_metadata_frames(hand_grouped())
  for (sub in frames) {
    expect_identical(class(sub), tibble_class)
    expect_null(attr(sub, "groups"))
    expect_identical(attr(sub, "meta"), "m")
  }
  expect_identical(frames[["b"]]$v, c(3, 4))
})

# --- column producers ----------------------------------------------------------

test_that("column producers keep the frame's own attributes and row names", {
  df <- frame_with_attrs()
  rownames(df) <- c("r1", "r2", "r3", "r4")
  outs <- list(
    drop = miniextendr:::dataframe_drop_column(df, "b"),
    select = miniextendr:::dataframe_select_columns(df, c("b", "a")),
    prepend = miniextendr:::dataframe_prepend_column(df, "id", 4:1),
    append = miniextendr:::dataframe_with_column(df, "c", c(TRUE, FALSE, TRUE, FALSE))
  )
  for (nm in names(outs)) {
    out <- outs[[nm]]
    expect_identical(attr(out, "meta"), attr(df, "meta"), info = nm)
    expect_identical(attr(out, "units"), c(a = "mg/L"), info = nm)
    expect_identical(class(out), "data.frame", info = nm)
    expect_identical(rownames(out), c("r1", "r2", "r3", "r4"), info = nm)
  }
  expect_identical(names(outs$drop), "a")
  expect_identical(names(outs$select), c("b", "a"))
  expect_identical(names(outs$prepend), c("id", "a", "b"))
  expect_identical(names(outs$append), c("a", "b", "c"))
  # The input is untouched.
  expect_identical(df, {
    ref <- frame_with_attrs()
    rownames(ref) <- c("r1", "r2", "r3", "r4")
    ref
  })
})

test_that("column producers match dplyr::select() and dplyr::mutate() on an attributed frame", {
  skip_if_not_installed("dplyr")
  df <- frame_with_attrs()
  expect_identical(miniextendr:::dataframe_select_columns(df, c("b", "a")), dplyr::select(df, b, a))
  expect_identical(miniextendr:::dataframe_drop_column(df, "a"), dplyr::select(df, -a))
  expect_identical(
    miniextendr:::dataframe_with_column(df, "c", 4:1),
    dplyr::mutate(df, c = 4:1)
  )
})

test_that("column producers keep a grouping whose columns all stay", {
  gdf <- hand_grouped(list(k = c(10, 20, 30, 40)))
  outs <- list(
    drop = miniextendr:::dataframe_drop_column(gdf, "k"),
    select = miniextendr:::dataframe_select_columns(gdf, c("v", "g")),
    prepend = miniextendr:::dataframe_prepend_column(gdf, "id", 4:1),
    append = miniextendr:::dataframe_with_column(gdf, "w", c(0.5, 1.5, 2.5, 3.5))
  )
  for (nm in names(outs)) {
    out <- outs[[nm]]
    expect_identical(class(out), class(gdf), info = nm)
    expect_identical(attr(out, "groups"), attr(gdf, "groups"), info = nm)
    expect_identical(group_metadata_sizes(out), c(2L, 2L), info = nm)
    expect_identical(attr(out, "meta"), "m", info = nm)
  }
})

test_that("column producers ungroup when a grouping column is removed or replaced", {
  gdf <- hand_grouped()
  outs <- list(
    drop = miniextendr:::dataframe_drop_column(gdf, "g"),
    select = miniextendr:::dataframe_select_columns(gdf, "v"),
    prepend = miniextendr:::dataframe_prepend_column(gdf, "g", c("z", "z", "z", "z"))
  )
  for (nm in names(outs)) {
    out <- outs[[nm]]
    expect_identical(class(out), tibble_class, info = nm)
    expect_null(attr(out, "groups"), info = nm)
    expect_null(group_declaration_of(out), info = nm)
    expect_identical(attr(out, "meta"), "m", info = nm)
  }
  expect_identical(outs$prepend$g, c("z", "z", "z", "z"))
})

test_that("column producers on dplyr grouped and rowwise frames give valid frames", {
  skip_if_not_installed("dplyr")
  gdf <- dplyr::group_by(data.frame(g = c(1, 1, 2), k = c("x", "y", "x"), v = 1:3), g)
  kept <- miniextendr:::dataframe_select_columns(gdf, c("g", "v"))
  expect_identical(kept, dplyr::select(gdf, g, v))
  expect_identical(dplyr::group_data(kept), dplyr::group_data(gdf))

  dropped <- miniextendr:::dataframe_drop_column(gdf, "g")
  expect_identical(class(dropped), tibble_class)
  expect_identical(dplyr::mutate(dropped, z = v + 1L)$z, 2:4)

  # A rowwise_df without id columns: its groups are one row each, still valid
  # while the rows are.
  rdf <- dplyr::rowwise(data.frame(v = 1:3, w = 4:6))
  out <- miniextendr:::dataframe_drop_column(rdf, "w")
  expect_s3_class(out, "rowwise_df")
  expect_identical(dplyr::group_data(out), dplyr::group_data(rdf))
  expect_identical(dplyr::mutate(out, s = sum(v))$s, 1:3)
})

test_that("frame-attribute copy and ungrouping survive gctorture", {
  skip_gc_stress_if_disabled()
  gdf <- hand_grouped()
  df <- frame_with_attrs()
  run <- function() {
    list(
      miniextendr:::dataframe_select_rows(gdf, c(4L, 1L)),
      miniextendr:::dataframe_select_rows(df, c(2L, 2L, 3L)),
      miniextendr:::dataframe_drop_column(gdf, "g"),
      miniextendr:::dataframe_select_columns(gdf, c("v", "g")),
      miniextendr:::dataframe_prepend_column(gdf, "id", 4:1)
    )
  }
  ref <- run()
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)
  out <- run()
  gctorture(old)
  expect_identical(out, ref)
})
