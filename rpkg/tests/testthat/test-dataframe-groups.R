# DataFrame group-level iteration (group_by / GroupedDataFrame / group_rows).
#
# Parity target: R split() + lapply(). Documented deviation: NA keys form one
# trailing "NA" group instead of being dropped.

# split()'s sub-frames keep parent row.names; group_by_frames() resets them to
# compact 1:n. Normalize both sides before comparing.
reset_rownames <- function(frames) {
  lapply(frames, function(d) {
    rownames(d) <- NULL
    d
  })
}

test_that("group_by_frames matches split() on a factor key (level order, empty levels kept)", {
  df <- data.frame(
    g = factor(c("b", "a", "b", "a"), levels = c("b", "a", "z")),
    v = c(1, 2, 3, 4)
  )
  ours <- group_by_frames(df, "g")
  ref <- reset_rownames(split(df, df$g))

  expect_identical(names(ours), c("b", "a", "z"))
  expect_identical(names(ours), names(ref))
  expect_identical(reset_rownames(ours), ref)
  # empty level yields an empty frame with the full column set
  expect_identical(nrow(ours$z), 0L)
  expect_identical(names(ours$z), c("g", "v"))
  # factor column keeps class + levels in sub-frames
  expect_identical(levels(ours$b$g), c("b", "a", "z"))
})

test_that("group_by_frames matches split() on a character key (sorted order)", {
  df <- data.frame(
    g = c("banana", "apple", "banana", "cherry"),
    v = 1:4
  )
  ours <- group_by_frames(df, "g")
  ref <- reset_rownames(split(df, df$g))

  expect_identical(names(ours), c("apple", "banana", "cherry"))
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_frames matches split() on an integer key (numeric sort)", {
  df <- data.frame(g = c(10L, 2L, 10L, -1L), v = c("w", "x", "y", "z"))
  ours <- group_by_frames(df, "g")
  ref <- reset_rownames(split(df, df$g))

  expect_identical(names(ours), c("-1", "2", "10"))
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_frames orders logical keys FALSE, TRUE like split()", {
  df <- data.frame(g = c(TRUE, FALSE, TRUE), v = 1:3)
  ours <- group_by_frames(df, "g")
  ref <- reset_rownames(split(df, df$g))

  expect_identical(names(ours), c("FALSE", "TRUE"))
  expect_identical(reset_rownames(ours), ref)
})

test_that("NA keys form one trailing group (deviation from split(), which drops them)", {
  df <- data.frame(g = c("a", NA, "b", NA), v = 1:4)
  ours <- group_by_frames(df, "g")
  ref <- reset_rownames(split(df, df$g))

  # split() drops the NA rows; we append them as one trailing "NA" group
  expect_identical(names(ours), c(names(ref), "NA"))
  expect_identical(reset_rownames(ours[names(ref)]), ref)
  expect_identical(ours$`NA`$v, c(2L, 4L))
  expect_true(all(is.na(ours$`NA`$g)))

  # same trailing-NA placement for factor codes and integer keys
  fdf <- data.frame(g = factor(c("a", NA, "a")), v = 1:3)
  expect_identical(group_by_keys(fdf, "g"), c("a", "NA"))
  expect_identical(group_by_sizes(fdf, "g"), c(2L, 1L))
  idf <- data.frame(g = c(1L, NA, 1L), v = 1:3)
  expect_identical(group_by_keys(idf, "g"), c("1", "NA"))
  ldf <- data.frame(g = c(TRUE, NA, FALSE), v = 1:3)
  expect_identical(group_by_keys(ldf, "g"), c("FALSE", "TRUE", "NA"))
})

test_that("group_by_keys / group_by_sizes report group order and sizes", {
  df <- data.frame(g = c("b", "a", "b"), v = 1:3)
  expect_identical(group_by_keys(df, "g"), c("a", "b"))
  expect_identical(group_by_sizes(df, "g"), c(1L, 2L))
})

test_that("empty frames and single groups are handled", {
  # empty frame, character key: no groups
  edf <- data.frame(g = character(0), v = integer(0))
  expect_identical(group_by_keys(edf, "g"), character(0))
  expect_identical(length(group_by_frames(edf, "g")), 0L)

  # empty frame, factor key: one (empty) group per level, like split()
  efdf <- data.frame(g = factor(character(0), levels = c("a", "b")), v = integer(0))
  expect_identical(group_by_keys(efdf, "g"), c("a", "b"))
  expect_identical(group_by_sizes(efdf, "g"), c(0L, 0L))

  # single group: everything in it
  sdf <- data.frame(g = c("x", "x", "x"), v = 1:3)
  frames <- group_by_frames(sdf, "g")
  expect_identical(names(frames), "x")
  expect_identical(frames$x$v, 1:3)
})

test_that("double key columns and missing columns are errors", {
  df <- data.frame(g = c(1.5, 2.5), v = 1:2)
  expect_error(group_by_keys(df, "g"), "cannot group by column")
  expect_error(group_by_keys(df, "nope"), "no such column")
})

test_that("group_by_extract_sums partitions one typed extraction by group", {
  df <- data.frame(
    g = c("b", "a", "b", "a", "c"),
    v = c(1, 2, 3, 4, 5)
  )
  out <- group_by_extract_sums(df)
  expect_s3_class(out, "data.frame")
  expect_identical(out$key, c("a", "b", "c"))
  ref <- tapply(df$v, df$g, sum)
  expect_identical(out$sum, as.numeric(ref[out$key]))
  expect_identical(out$n, c(2L, 2L, 1L))
})

test_that("group_rows_summary groups typed rows Rust-side (Option key, None first)", {
  out <- group_rows_summary()
  expect_s3_class(out, "data.frame")
  # v = 0..11; i %% 4 -> a: 0+4+8, b: 1+5+9, c: 2+6+10, None: 3+7+11
  expect_identical(out$key, c("NA", "a", "b", "c"))
  expect_identical(out$sum, c(21, 12, 15, 18))
  expect_identical(out$n, rep(3L, 4))
})

test_that("gc_stress_group_by smoke-runs and returns the expected partitions", {
  out <- miniextendr:::gc_stress_group_by()
  expect_identical(names(out), c("a", "b", "c", "NA"))
  expect_identical(sum(vapply(out, nrow, integer(1))), 60L)
})

# --- group_by_multi: composite keys ------------------------------------------
#
# Parity target for non-NA groups: split(df, interaction(..., drop = TRUE)).
# The R output IS the spec — these tests compute the reference from base R
# rather than hard-coding it. Documented deviation: NA-containing tuples form
# trailing per-tuple groups (first-encounter order) instead of being dropped.

test_that("group_by_multi_frames matches split(interaction()) on two character keys", {
  df <- data.frame(
    a = c("x", "y", "x", "y", "x"),
    b = c("p", "p", "q", "q", "p"),
    v = 1:5,
    stringsAsFactors = FALSE
  )
  ours <- group_by_multi_frames(df, c("a", "b"))
  ref <- reset_rownames(split(df, interaction(df$a, df$b, drop = TRUE)))

  expect_identical(names(ours), names(ref)) # first column varies fastest
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_multi honours factor level order (not label order)", {
  df <- data.frame(
    a = factor(c("lo", "hi", "lo", "hi"), levels = c("lo", "hi")),
    b = factor(c("A", "A", "B", "B"), levels = c("B", "A")),
    v = 1:4
  )
  ours <- group_by_multi_frames(df, c("a", "b"))
  ref <- reset_rownames(split(df, interaction(df$a, df$b, drop = TRUE)))

  expect_identical(names(ours), names(ref)) # lo.B, hi.B, lo.A, hi.A
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_multi matches split(interaction()) on integer x character keys", {
  df <- data.frame(
    a = c(10L, 2L, 10L, 2L, -1L),
    b = c("z", "z", "y", "y", "z"),
    v = 1:5,
    stringsAsFactors = FALSE
  )
  ours <- group_by_multi_frames(df, c("a", "b"))
  ref <- reset_rownames(split(df, interaction(df$a, df$b, drop = TRUE)))

  expect_identical(names(ours), names(ref)) # 2.y, 10.y, -1.z, 2.z, 10.z
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_multi matches split(interaction()) on three keys", {
  df <- data.frame(
    a = c("x", "x", "y", "y", "x", "y"),
    b = c(TRUE, FALSE, TRUE, FALSE, TRUE, TRUE),
    c = c(1L, 1L, 2L, 2L, 2L, 1L),
    v = 1:6,
    stringsAsFactors = FALSE
  )
  ours <- group_by_multi_frames(df, c("a", "b", "c"))
  ref <- reset_rownames(split(df, interaction(df$a, df$b, df$c, drop = TRUE)))

  expect_identical(names(ours), names(ref))
  expect_identical(reset_rownames(ours), ref)
})

test_that("group_by_multi keys/sizes report labels joined with '.' in group order", {
  df <- data.frame(
    a = c("x", "y", "x", "y", "x"),
    b = c("p", "p", "q", "q", "p"),
    v = 1:5,
    stringsAsFactors = FALSE
  )
  expect_identical(group_by_multi_keys(df, c("a", "b")), c("x.p", "y.p", "x.q", "y.q"))
  expect_identical(group_by_multi_sizes(df, c("a", "b")), c(2L, 1L, 1L, 1L))
})

test_that("NA-containing tuples form trailing per-tuple groups (first-encounter order)", {
  df <- data.frame(
    a = c("x", NA, "y", "x", NA),
    b = c("p", "p", NA, "q", NA),
    v = 1:5,
    stringsAsFactors = FALSE
  )
  ours <- group_by_multi_frames(df, c("a", "b"))
  ref <- reset_rownames(split(df, interaction(df$a, df$b, drop = TRUE)))

  # non-NA subset matches split(interaction()); split() drops the NA rows
  expect_identical(reset_rownames(ours[names(ref)]), ref)

  na_names <- setdiff(names(ours), names(ref))
  expect_identical(na_names, c("NA.p", "y.NA", "NA.NA")) # first-encounter order
  # ...and they trail the non-NA groups
  expect_identical(tail(names(ours), length(na_names)), na_names)
  expect_identical(ours[["NA.p"]]$v, 2L)
  expect_identical(ours[["y.NA"]]$v, 3L)
  expect_identical(ours[["NA.NA"]]$v, 5L)
})

test_that("single-column slice delegates to the scalar group_by path (no 1-tuples)", {
  df <- data.frame(g = c("b", "a", "b"), v = 1:3)
  expect_identical(group_by_multi_keys(df, "g"), group_by_keys(df, "g"))
  expect_identical(group_by_multi_sizes(df, "g"), group_by_sizes(df, "g"))
  # scalar keys, not "a"/"b" wrapped as 1-tuples
  expect_identical(group_by_multi_keys(df, "g"), c("a", "b"))
})

test_that("group_by_multi validates its column slice and unsupported types", {
  df <- data.frame(a = c("x", "y"), b = 1:2, d = c(1.5, 2.5))
  expect_error(group_by_multi_keys(df, character(0)), "at least one column")
  expect_error(group_by_multi_keys(df, c("a", "nope")), "no such column")
  expect_error(group_by_multi_keys(df, c("a", "d")), "cannot group by column")
})

test_that("gc_stress_group_by_multi smoke-runs and preserves all rows", {
  out <- miniextendr:::gc_stress_group_by_multi()
  expect_identical(sum(vapply(out, nrow, integer(1))), 60L)
  # NA-containing tuples (g == NA) trail the non-NA groups
  na_names <- grep("^NA[.]|[.]NA$", names(out), value = TRUE)
  expect_true(length(na_names) > 0)
  expect_identical(tail(names(out), length(na_names)), na_names)
})
