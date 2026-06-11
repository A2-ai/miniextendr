# Non-scalar FromDataFrame readers (#782): column-expansion + struct-flatten.
# Each `*_roundtrip()` reads a data.frame into Vec<Row> with the generated reader,
# then rebuilds it with the writer. `roundtrip(input) == input` proves the reader
# reconstructs rows that re-serialise to the identical frame; the explicit column
# assertions pin the ground truth.

test_that("fixed-array [T; N] reader round-trips", {
  df <- data.frame(
    id = c(1L, 2L),
    coords_1 = c(1, 4),
    coords_2 = c(2, 5),
    coords_3 = c(3, 6)
  )
  out <- reader_fixed_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(nrow(out), 2L)
  expect_equal(out$id, df$id)
  expect_equal(out$coords_1, df$coords_1)
  expect_equal(out$coords_2, df$coords_2)
  expect_equal(out$coords_3, df$coords_3)
})

test_that("pinned-width Vec<f64> reader round-trips (trailing NA padding preserved)", {
  df <- data.frame(
    name = c("a", "b", "c"),
    scores_1 = c(1, 4, NA),
    scores_2 = c(2, NA, NA),
    scores_3 = c(3, NA, NA),
    stringsAsFactors = FALSE
  )
  out <- reader_pinned_roundtrip(df)
  expect_equal(out$name, df$name)
  expect_equal(out$scores_1, df$scores_1)
  expect_equal(out$scores_2, df$scores_2)
  expect_equal(out$scores_3, df$scores_3)
})

test_that("pinned-width Box<[f64]> reader round-trips (.into() conversion)", {
  df <- data.frame(
    k = c(10L, 20L),
    vals_1 = c(1, 3),
    vals_2 = c(2, NA)
  )
  out <- reader_box_pinned_roundtrip(df)
  expect_equal(out$k, df$k)
  expect_equal(out$vals_1, df$vals_1)
  expect_equal(out$vals_2, df$vals_2)
})

test_that("auto-expand Vec<f64> reader round-trips (ragged)", {
  # Row a = [1,2,3], b = [4], c = [5,6] → max width 3.
  df <- data.frame(
    name = c("a", "b", "c"),
    values_1 = c(1, 4, 5),
    values_2 = c(2, NA, 6),
    values_3 = c(3, NA, NA),
    stringsAsFactors = FALSE
  )
  out <- reader_auto_roundtrip(df)
  expect_equal(out$name, df$name)
  expect_equal(out$values_1, df$values_1)
  expect_equal(out$values_2, df$values_2)
  expect_equal(out$values_3, df$values_3)
})

test_that("auto-expand Box<[i32]> reader round-trips (.into() conversion)", {
  df <- data.frame(
    tag = c("x", "y"),
    xs_1 = c(7L, 8L),
    xs_2 = c(9L, NA),
    stringsAsFactors = FALSE
  )
  out <- reader_auto_box_roundtrip(df)
  expect_equal(out$tag, df$tag)
  expect_equal(out$xs_1, df$xs_1)
  expect_equal(out$xs_2, df$xs_2)
})

test_that("struct-flatten reader round-trips nested-DataFrameRow fields", {
  df <- data.frame(
    id = c(1L, 2L, 3L),
    origin_x = c(1, 3, 5),
    origin_y = c(2, 4, 6)
  )
  out <- reader_flatten_roundtrip(df)
  expect_equal(out$id, df$id)
  expect_equal(out$origin_x, df$origin_x)
  expect_equal(out$origin_y, df$origin_y)
})

test_that("struct-flatten reader handles a non-numeric inner column", {
  df <- data.frame(
    id = c(1L, 2L),
    owner_label = c("ada", "linus"),
    owner_age = c(30L, 50L),
    stringsAsFactors = FALSE
  )
  out <- reader_flatten_mixed_roundtrip(df)
  expect_equal(out$id, df$id)
  expect_equal(out$owner_label, df$owner_label)
  expect_equal(out$owner_age, df$owner_age)
})

test_that("struct-flatten reader recurses three levels", {
  df <- data.frame(
    id = c(1L, 2L),
    mid_a = c(10, 20),
    mid_leaf_z = c(100, 200)
  )
  out <- reader_flatten_nested_roundtrip(df)
  expect_equal(out$id, df$id)
  expect_equal(out$mid_a, df$mid_a)
  expect_equal(out$mid_leaf_z, df$mid_leaf_z)
})

test_that("readers handle a zero-row data.frame", {
  df <- data.frame(
    id = integer(0),
    coords_1 = numeric(0),
    coords_2 = numeric(0),
    coords_3 = numeric(0)
  )
  out <- reader_fixed_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(nrow(out), 0L)
})

test_that("reader gctorture fixtures drive the struct-flatten read path", {
  expect_no_error(gc_stress_reader_struct_flatten())
  expect_no_error(gc_stress_reader_nested_flatten())
})

# region: opaque list-column readers (#809) ------------------------------------

test_that("opaque Vec<f64> list-column reader round-trips (ragged)", {
  df <- data.frame(id = c(1L, 2L, 3L))
  df$data <- I(list(c(1, 2, 3), c(4), c(5, 6)))
  out <- reader_list_vec_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(out$id, df$id)
  expect_equal(unclass(out$data), list(c(1, 2, 3), c(4), c(5, 6)))
})

test_that("opaque Box<[i32]> list-column reader round-trips (.into() conversion)", {
  df <- data.frame(tag = c("a", "b"), stringsAsFactors = FALSE)
  df$xs <- I(list(c(7L, 8L), c(9L)))
  out <- reader_list_box_roundtrip(df)
  expect_equal(out$tag, df$tag)
  expect_equal(unclass(out$xs), list(c(7L, 8L), c(9L)))
})

test_that("opaque multi-list-column reader round-trips (Vec<i32> + Vec<String>)", {
  df <- data.frame(
    ids   = I(list(c(1L, 2L), c(3L), integer(0))),
    names = I(list(c("a", "b"), c("c"), character(0)))
  )
  out <- reader_list_multi_roundtrip(df)
  expect_equal(unclass(out$ids),   list(c(1L, 2L), c(3L), integer(0)))
  expect_equal(unclass(out$names), list(c("a", "b"), c("c"), character(0)))
})

test_that("opaque list-column reader handles empty per-row collections", {
  df <- data.frame(id = c(1L, 2L))
  df$data <- I(list(numeric(0), numeric(0)))
  out <- reader_list_vec_roundtrip(df)
  expect_equal(out$id, df$id)
  expect_equal(unclass(out$data), list(numeric(0), numeric(0)))
})

test_that("reader gctorture fixtures drive the list-column read path", {
  expect_no_error(gc_stress_reader_list_column())
})

# region: map-column readers (#764) --------------------------------------------

test_that("BTreeMap<String, i32> map-column reader round-trips", {
  df <- data.frame(id = c(1L, 2L, 3L))
  # Each element is a named list — the write shape of a struct-path map column.
  # BTreeMap writes back sorted by key, so keep inputs pre-sorted for equality.
  df$opts <- I(list(
    list(a = 1L, b = 2L),
    list(z = 9L),
    structure(list(), names = character(0))
  ))
  out <- reader_map_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(out$id, df$id)
  expect_equal(unclass(out$opts)[[1]], list(a = 1L, b = 2L))
  expect_equal(unclass(out$opts)[[2]], list(z = 9L))
  expect_length(unclass(out$opts)[[3]], 0L)
})

test_that("HashMap<String, f64> map-column reader round-trips (order-insensitive)", {
  df <- data.frame(label = c("x", "y"), stringsAsFactors = FALSE)
  df$weights <- I(list(
    list(w1 = 0.5, w2 = 1.5),
    list(solo = 2.5)
  ))
  out <- reader_hashmap_roundtrip(df)
  expect_equal(out$label, df$label)
  # HashMap iteration order is non-deterministic — compare sorted by name.
  sort_by_name <- function(x) x[order(names(x))]
  expect_equal(sort_by_name(unclass(out$weights)[[1]]), list(w1 = 0.5, w2 = 1.5))
  expect_equal(sort_by_name(unclass(out$weights)[[2]]), list(solo = 2.5))
})

test_that("map-column reader rejects named atomic vectors", {
  # Strict shape: each element must be a VECSXP named list; c(a = 1L) is INTSXP.
  df <- data.frame(id = 1L)
  df$opts <- I(list(c(a = 1L)))
  expect_error(reader_map_roundtrip(df), "opts")
})

test_that("map-column reader handles a zero-row data.frame", {
  df <- data.frame(id = integer(0))
  df$opts <- I(list())
  out <- reader_map_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(nrow(out), 0L)
})

test_that("reader gctorture fixture drives the map-column read path", {
  expect_no_error(gc_stress_reader_map_column())
})

test_that("parallel readers match the sequential result", {
  skip_if_missing_feature("rayon")

  fixed <- data.frame(
    id = c(1L, 2L),
    coords_1 = c(1, 4),
    coords_2 = c(2, 5),
    coords_3 = c(3, 6)
  )
  out_par <- reader_fixed_roundtrip_par(fixed)
  expect_equal(out_par$id, fixed$id)
  expect_equal(out_par$coords_1, fixed$coords_1)
  expect_equal(out_par$coords_3, fixed$coords_3)

  flat <- data.frame(
    id = c(1L, 2L, 3L),
    origin_x = c(1, 3, 5),
    origin_y = c(2, 4, 6)
  )
  out_flat_par <- reader_flatten_roundtrip_par(flat)
  expect_equal(out_flat_par$id, flat$id)
  expect_equal(out_flat_par$origin_x, flat$origin_x)
  expect_equal(out_flat_par$origin_y, flat$origin_y)

  # list-column _par: real off-thread index assembly (#809)
  lv <- data.frame(id = c(1L, 2L))
  lv$data <- I(list(c(1.1, 2.2), c(3.3)))
  out_lv_par <- reader_list_vec_roundtrip_par(lv)
  expect_equal(out_lv_par$id, lv$id)
  expect_equal(unclass(out_lv_par$data), list(c(1.1, 2.2), c(3.3)))

  # map-column _par: same Vec<map> pull, off-thread by-index assembly (#764)
  mp <- data.frame(id = c(1L, 2L))
  mp$opts <- I(list(list(a = 1L, b = 2L), list(c = 3L)))
  out_mp_seq <- reader_map_roundtrip(mp)
  out_mp_par <- reader_map_roundtrip_par(mp)
  expect_equal(out_mp_par$id, out_mp_seq$id)
  expect_equal(unclass(out_mp_par$opts), unclass(out_mp_seq$opts))
})
