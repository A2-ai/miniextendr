# `#[r_data]` fields that read `NULL` for `None` (#2270), under S3, R6 and
# S7: `#[r_data(null_if_none)]` on an `Option<T>` field (`label`, and the
# read-only `code`) and a `Nullable<T>` field (`tag`). `plain` is an
# `Option<String>` field without the option, which keeps the default: `NA`
# for `None`. Writing `NULL` or `NA` stores `None` in each of them. S7
# admits the `NULL`: the generated properties have no class (`class_any`).
# `SavedNull` is a serde type whose save keeps both forms (#2186).
#
# Fixtures: src/rust/rdata_null_if_none_tests.rs.

caught <- function(expr) tryCatch(expr, error = identity)
reload <- function(x) unserialize(serialize(x, NULL))
reload_rds <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}

# region: S3

test_that("S3: None reads NULL, Some reads the value, and a plain Option<String> reads NA", {
  x <- new_nulls3(NULL, NULL, NULL, NULL)
  expect_null(x$label)
  expect_null(x[["label"]])
  expect_null(x$code)
  expect_null(x$tag)
  expect_null(x[["tag"]])
  expect_identical(x$plain, NA_character_)
  expect_identical(
    as.list(x),
    list(label = NULL, code = NULL, tag = NULL, plain = NA_character_)
  )
  y <- new_nulls3("a", 3L, "t", "p")
  expect_identical(y$label, "a")
  expect_identical(y$code, 3L)
  expect_identical(y$tag, "t")
  expect_identical(y$plain, "p")
  expect_identical(as.list(y), list(label = "a", code = 3L, tag = "t", plain = "p"))
})

test_that("S3: writing NULL or NA stores None", {
  x <- new_nulls3("a", 3L, "t", "p")
  x$label <- NULL
  expect_null(x$label)
  x$label <- "b"
  expect_identical(x$label, "b")
  x[["label"]] <- NA_character_
  expect_null(x$label)
  x$tag <- NULL
  expect_null(x$tag)
  x[["tag"]] <- "u"
  expect_identical(x$tag, "u")
  x$tag <- NA_character_
  expect_null(x$tag)
  # Without the option, `None` reads `NA`.
  x$plain <- NULL
  expect_identical(x$plain, NA_character_)
})

test_that("S3: a read-only null_if_none field refuses a write, and a bad value names its field", {
  x <- new_nulls3(NULL, 3L, NULL, NULL)
  e <- caught(x$code <- NULL)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(conditionMessage(e), "`code` of `NullS3` is read-only")
  expect_identical(x$code, 3L)
  # A `Nullable<String>` field is described as an `Option<String>` one.
  expect_identical(
    conditionMessage(caught(x$label <- 1)),
    "'label' must be NULL or a single string: got numeric"
  )
  expect_identical(
    conditionMessage(caught(x$tag <- 1)),
    "'tag' must be NULL or a single string: got numeric"
  )
})
# endregion

# region: R6

test_that("R6: None reads NULL, Some reads the value, and writing NULL or NA stores None", {
  r <- NullR6$new(NULL, NULL, NULL, NULL)
  expect_null(r$label)
  expect_null(r$code)
  expect_null(r$tag)
  expect_identical(r$plain, NA_character_)
  r$label <- "a"
  r$tag <- "t"
  expect_identical(r$label, "a")
  expect_identical(r$tag, "t")
  r$label <- NULL
  r$tag <- NA_character_
  expect_null(r$label)
  expect_null(r$tag)
  r$plain <- NULL
  expect_identical(r$plain, NA_character_)

  r <- NullR6$new("a", 3L, "t", "p")
  expect_identical(r$label, "a")
  expect_identical(r$code, 3L)
  expect_identical(r$tag, "t")
  expect_identical(r$plain, "p")
  e <- caught(r$code <- NULL)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(r$code, 3L)
})
# endregion

# region: S7

test_that("S7: the properties admit NULL: None reads NULL, and writing NULL or NA stores None", {
  s <- NullS7(NULL, NULL, NULL, NULL)
  expect_null(s@label)
  expect_null(s@code)
  expect_null(s@tag)
  expect_identical(s@plain, NA_character_)
  expect_no_error(S7::validate(s))
  s@label <- "a"
  s@tag <- "t"
  expect_identical(s@label, "a")
  expect_identical(s@tag, "t")
  s@label <- NULL
  s@tag <- NA_character_
  expect_null(s@label)
  expect_null(s@tag)
  expect_no_error(S7::validate(s))
  props <- S7::S7_class(s)@properties
  for (name in c("label", "code", "tag", "plain")) {
    expect_identical(props[[name]]$class, S7::class_any, label = name)
  }

  s <- NullS7("a", 3L, "t", "p")
  expect_identical(s@label, "a")
  expect_identical(s@code, 3L)
  expect_identical(s@tag, "t")
  expect_identical(s@plain, "p")
  expect_error(s@code <- NULL, "Can't set read-only property", fixed = TRUE)
  expect_identical(s@code, 3L)
})
# endregion

# region: serde, a save keeps both forms

test_that("a saved Nullable field and a null_if_none field come back with None and Some", {
  for (back_of in list(reload, reload_rds)) {
    back <- back_of(SavedNull$new(NULL, NULL))
    expect_null(back$tag)
    expect_null(back$label)
    back <- back_of(SavedNull$new("t", "l"))
    expect_identical(back$tag, "t")
    expect_identical(back$label, "l")
    obj <- SavedNull$new("t", "l")
    obj$tag <- NULL
    obj$label <- NA_character_
    back <- back_of(obj)
    expect_null(back$tag)
    expect_null(back$label)
  }
})
# endregion
