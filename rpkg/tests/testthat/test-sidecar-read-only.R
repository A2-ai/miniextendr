# Read-only `#[r_data]` fields (#2268): `#[r_data(read_only)]` per field
# under S3, R6 and S7, `Computed` fields under R6 and S7 (#1888), and the
# read-only field syntax `r_data_accessors = "read_only"` under S3, S4 and
# env. A refused write raises `miniextendr_read_only_field` with the type's
# `field_error` classes in front and its `read_only_hint` after the message;
# S7 refuses a property without a setter itself. Read-only applies to R
# only: Rust writes the fields.

caught <- function(expr) tryCatch(expr, error = identity)

# region: S3, read-only fields next to a writable one

test_that("an S3 read-only field reads, and its write is refused with the classes and hint", {
  x <- new_readonlys3(3L, "hi")
  expect_identical(x$id, 3L)
  expect_identical(x[["keys"]], integer(0))
  expect_identical(x$doubled, 6L)
  e <- caught(x$id <- 4L)
  expect_identical(
    class(e),
    c("readonly_error_field", "readonly_error", "miniextendr_read_only_field", "error", "condition")
  )
  expect_identical(
    conditionMessage(e),
    "`id` of `ReadOnlyS3` is read-only. Build a new object to change `id`."
  )
  expect_equal(conditionCall(e), quote(x$id <- value))
  e <- caught(x[["keys"]] <- 1L)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(
    conditionMessage(e),
    "`keys` of `ReadOnlyS3` is read-only. Build a new object to change `keys`."
  )
  expect_equal(conditionCall(e), quote(x[["keys"]] <- value))
  expect_identical(x$id, 3L)
  expect_identical(x$keys, integer(0))
})

test_that("a computed S3 field's refusal carries the package's classes and hint too", {
  x <- new_readonlys3(3L, "hi")
  e <- caught(x$doubled <- 1L)
  expect_identical(
    class(e),
    c("readonly_error_field", "readonly_error", "miniextendr_read_only_field", "error", "condition")
  )
  expect_identical(
    conditionMessage(e),
    paste(
      "`doubled` of `ReadOnlyS3` is computed from the Rust value and can't be assigned.",
      "Build a new object to change `doubled`."
    )
  )
})

test_that("a mixed S3 class writes its writable field, and Rust writes a read-only one", {
  x <- new_readonlys3(3L, "hi")
  x$note <- "there"
  expect_identical(x$note, "there")
  x[["note"]] <- "again"
  expect_identical(x$note, "again")
  add_key(x, 5L)
  expect_identical(x$keys, 5L)
  expect_identical(as.list(x), list(id = 3L, keys = 5L, doubled = 6L, note = "again"))
})

test_that("a read-only field has no standalone setter", {
  ns <- asNamespace("miniextendr")
  expect_false(exists("ReadOnlyS3_set_id", envir = ns, inherits = FALSE))
  expect_false(exists("ReadOnlyS3_set_keys", envir = ns, inherits = FALSE))
  expect_false(exists("ReadOnlyS3_set_doubled", envir = ns, inherits = FALSE))
  expect_true(exists("ReadOnlyS3_set_note", envir = ns, inherits = FALSE))
  expect_true(exists("ReadOnlyS3_get_id", envir = ns, inherits = FALSE))
})
# endregion

# region: every field read-only (`r_data_accessors = "read_only"`)

test_that("S3 `r_data_accessors = \"read_only\"` reads every field and refuses every write", {
  x <- new_frozens3(2L)
  expect_identical(x$size, 2L)
  expect_identical(x[["items"]], character(0))
  e <- caught(x$size <- 4L)
  expect_identical(class(e), c("frozen_error", "miniextendr_read_only_field", "error", "condition"))
  expect_identical(conditionMessage(e), "`size` of `FrozenS3` is read-only. A FrozenS3 is fixed once built.")
  expect_equal(conditionCall(e), quote(x$size <- value))
  e <- caught(x[["items"]] <- "a")
  expect_s3_class(e, "frozen_error")
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_equal(conditionCall(e), quote(x[["items"]] <- value))
  expect_identical(x$size, 2L)
  # A name that is not a field is still `miniextendr_no_field`.
  e <- caught(x$nope <- 1L)
  expect_s3_class(e, "miniextendr_no_field")
  expect_false(inherits(e, "miniextendr_read_only_field"))
  # Rust writes the fields.
  grow(x, "a")
  expect_identical(x$items, "a")
  expect_identical(item_count(x), 1L)
})

test_that("the read-only field syntax writes no standalone setter", {
  ns <- asNamespace("miniextendr")
  for (setter in c("FrozenS3_set_size", "FrozenS3_set_items", "FrozenS4_set_depth", "FrozenS4_set_label", "FrozenEnv_set_count")) {
    expect_false(exists(setter, envir = ns, inherits = FALSE), label = setter)
  }
  for (getter in c("FrozenS3_get_size", "FrozenS3_get_items", "FrozenS4_get_depth", "FrozenEnv_get_count")) {
    expect_true(exists(getter, envir = ns, inherits = FALSE), label = getter)
  }
})

test_that("S4 `r_data_accessors = \"read_only\"` reads every field and refuses every write", {
  o <- FrozenS4(1L, "s")
  expect_identical(o$depth, 1L)
  expect_identical(o$label, "s")
  e <- caught(o$depth <- 4L)
  expect_identical(class(e), c("miniextendr_read_only_field", "error", "condition"))
  expect_identical(conditionMessage(e), "`depth` of `FrozenS4` is read-only")
  expect_equal(conditionCall(e), quote(x$depth <- value))
  expect_s3_class(caught(o$nope <- 4L), "miniextendr_no_field")
  expect_identical(o$depth, 1L)
  expect_true(methods::existsMethod("$<-", "FrozenS4"))
})

test_that("env `r_data_accessors = \"read_only\"` reads the field and refuses its write", {
  x <- FrozenEnv$new(1L)
  expect_identical(x$count, 1L)
  e <- caught(x$count <- 4L)
  expect_identical(class(e), c("miniextendr_read_only_field", "error", "condition"))
  expect_identical(conditionMessage(e), "`count` of `FrozenEnv` is read-only")
  expect_equal(conditionCall(e), quote(x$count <- value))
  e <- caught(x[["count"]] <- 4L)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_equal(conditionCall(e), quote(x[["count"]] <- value))
  expect_identical(x$count, 1L)
  x$bump()
  expect_identical(x$count, 2L)
})
# endregion

# region: R6

test_that("an R6 read-only or computed active binding reads, and refuses a value with the classes and hint", {
  r <- ReadOnlyR6$new(3L, 1L)
  expect_identical(r$id, 3L)
  expect_identical(r$doubled, 6L)
  e <- caught(r$id <- 4L)
  expect_identical(
    class(e),
    c("readonly_r6_error", "miniextendr_read_only_field", "error", "condition")
  )
  expect_identical(
    conditionMessage(e),
    "`id` of `ReadOnlyR6` is read-only. Build a new ReadOnlyR6 to change `id`."
  )
  expect_null(conditionCall(e))
  e <- caught(r$doubled <- 4L)
  expect_s3_class(e, "readonly_r6_error")
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(
    conditionMessage(e),
    paste(
      "`doubled` of `ReadOnlyR6` is computed from the Rust value and can't be assigned.",
      "Build a new ReadOnlyR6 to change `doubled`."
    )
  )
  expect_identical(r$id, 3L)
})

test_that("a mixed R6 class writes its writable binding, and Rust writes a read-only one", {
  r <- ReadOnlyR6$new(3L, 1L)
  r$level <- 9L
  expect_identical(r$level, 9L)
  r$next_id()
  expect_identical(r$id, 4L)
  expect_identical(r$doubled, 8L)
  ns <- asNamespace("miniextendr")
  expect_false(exists("ReadOnlyR6_set_id", envir = ns, inherits = FALSE))
  expect_false(exists("ReadOnlyR6_set_doubled", envir = ns, inherits = FALSE))
  expect_true(exists("ReadOnlyR6_set_level", envir = ns, inherits = FALSE))
})
# endregion

# region: S7

test_that("an S7 read-only or computed property reads and has no setter", {
  s <- ReadOnlyS7(3L, 1L)
  expect_identical(s@id, 3L)
  expect_identical(s@doubled, 6L)
  # S7 refuses the write itself, with its own error.
  expect_error(s@id <- 4L, "Can't set read-only property", fixed = TRUE)
  expect_error(s@doubled <- 4L, "Can't set read-only property", fixed = TRUE)
  expect_identical(s@id, 3L)
  props <- S7::S7_class(s)@properties
  expect_null(props$id$setter)
  expect_null(props$doubled$setter)
  expect_false(is.null(props$level$setter))
})

test_that("a mixed S7 class writes its writable property", {
  s <- ReadOnlyS7(3L, 1L)
  s@level <- 9L
  expect_identical(s@level, 9L)
  ns <- asNamespace("miniextendr")
  expect_false(exists("ReadOnlyS7_set_id", envir = ns, inherits = FALSE))
  expect_false(exists("ReadOnlyS7_set_doubled", envir = ns, inherits = FALSE))
  expect_true(exists("ReadOnlyS7_set_level", envir = ns, inherits = FALSE))
})
# endregion
