# test-dataframe-readers-enum.R — R-side assertions for enum FromDataFrame readers (#807).
#
# Each test builds an input data.frame by calling the Rust align entrypoint (which uses
# the writer), then calls *_roundtrip(df) which reads it back with the new reader and
# rebuilds with the writer.  roundtrip(make()) == make() proves the reader reconstructs
# rows that re-serialise to the identical frame.  Column values are asserted individually
# so failures pinpoint the broken field.
#
# Round-trip caveat: absent-variant cells are NA in the writer output.  The hand-built
# frames in this file must use the exact NA-fill pattern the writer produces — safest way
# is to let the Rust align entrypoint produce the canonical reference.

test_that("re_scalar: scalar tagged-union enum round-trips via reader", {
  ref_df <- re_scalar_align()
  rt <- re_scalar_roundtrip(ref_df)
  expect_equal(rt[["_type"]], ref_df[["_type"]])
  expect_equal(rt[["id"]],    ref_df[["id"]])
  expect_equal(rt[["x"]],     ref_df[["x"]])
  expect_equal(rt[["name"]],  ref_df[["name"]])
  # Full frame equality
  expect_equal(rt, ref_df)
})

test_that("re_scalar: zero-row frame round-trips", {
  # Build a zero-row frame by subsetting the reference.
  ref_df <- re_scalar_align()
  zero_df <- ref_df[integer(0), ]
  row.names(zero_df) <- integer(0)
  rt <- re_scalar_roundtrip_zero(zero_df)
  expect_equal(nrow(rt), 0L)
  expect_equal(names(rt), names(ref_df))
})

test_that("re_expand: column-expansion enum round-trips via reader", {
  ref_df <- re_expand_align()
  rt <- re_expand_roundtrip(ref_df)
  expect_equal(rt[["_type"]], ref_df[["_type"]])
  expect_equal(rt[["id"]],    ref_df[["id"]])
  expect_equal(rt[["c_1"]],   ref_df[["c_1"]])
  expect_equal(rt[["c_2"]],   ref_df[["c_2"]])
  expect_equal(rt[["s_1"]],   ref_df[["s_1"]])
  expect_equal(rt[["s_2"]],   ref_df[["s_2"]])
  expect_equal(rt[["s_3"]],   ref_df[["s_3"]])
  expect_equal(rt, ref_df)
})

test_that("re_move: as_factor nested-enum round-trips via reader", {
  ref_df <- re_move_align()
  rt <- re_move_roundtrip(ref_df)
  expect_equal(rt[["_type"]], ref_df[["_type"]])
  expect_equal(rt[["id"]],    ref_df[["id"]])
  # dir column must be a factor with the right levels and values
  expect_true(is.factor(rt[["dir"]]))
  expect_equal(levels(rt[["dir"]]), c("North", "South", "East", "West"))
  expect_equal(as.character(rt[["dir"]]), as.character(ref_df[["dir"]]))
  expect_equal(rt, ref_df)
})

test_that("re_tracked: nested payload-bearing enum flatten round-trips via reader", {
  ref_df <- re_tracked_align()
  rt <- re_tracked_roundtrip(ref_df)
  expect_equal(rt[["_type"]],          ref_df[["_type"]])
  expect_equal(rt[["id"]],             ref_df[["id"]])
  expect_equal(rt[["status_variant"]], ref_df[["status_variant"]])
  expect_equal(rt[["status_code"]],    ref_df[["status_code"]])
  expect_equal(rt, ref_df)
})

test_that("re_loc: struct-flatten variant field round-trips via reader", {
  ref_df <- re_loc_align()
  rt <- re_loc_roundtrip(ref_df)
  expect_equal(rt[["_type"]], ref_df[["_type"]])
  expect_equal(rt[["id"]],    ref_df[["id"]])
  expect_equal(rt[["p_x"]],  ref_df[["p_x"]])
  expect_equal(rt[["p_y"]],  ref_df[["p_y"]])
  expect_equal(rt, ref_df)
})

test_that("gc stress: enum flatten reader runs clean", {
  expect_no_error(gc_stress_reader_enum_flatten())
})

test_that("gc stress: enum factor reader runs clean", {
  expect_no_error(gc_stress_reader_enum_factor())
})

test_that("re_scalar: parallel round-trip matches sequential", {
  skip_if_missing_feature("rayon")
  ref_df <- re_scalar_align()
  seq_rt  <- re_scalar_roundtrip(ref_df)
  par_rt  <- re_scalar_roundtrip_par(ref_df)
  expect_equal(par_rt, seq_rt)
})

test_that("re_expand: parallel round-trip matches sequential", {
  skip_if_missing_feature("rayon")
  ref_df  <- re_expand_align()
  seq_rt  <- re_expand_roundtrip(ref_df)
  par_rt  <- re_expand_roundtrip_par(ref_df)
  expect_equal(par_rt, seq_rt)
})

test_that("re_tracked: parallel round-trip (delegates to sequential) matches sequential", {
  skip_if_missing_feature("rayon")
  ref_df  <- re_tracked_align()
  seq_rt  <- re_tracked_roundtrip(ref_df)
  par_rt  <- re_tracked_roundtrip_par(ref_df)
  expect_equal(par_rt, seq_rt)
})

test_that("re_loc: parallel round-trip (delegates to sequential) matches sequential", {
  skip_if_missing_feature("rayon")
  ref_df  <- re_loc_align()
  seq_rt  <- re_loc_roundtrip(ref_df)
  par_rt  <- re_loc_roundtrip_par(ref_df)
  expect_equal(par_rt, seq_rt)
})
