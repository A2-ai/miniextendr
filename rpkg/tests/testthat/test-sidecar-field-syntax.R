# Field syntax for the `#[r_data]` fields of S3, S4 and env classes (#1848):
# `s3(r_data_accessors)`, `s4(r_data_accessors)`, `env(r_data_accessors)`,
# and the getters-only `r_data_accessors = "get"`.
#
# `$` / `[[` read a field and `$<-` / `[[<-` write it through the same
# accessors as `Type_get_f()` / `Type_set_f()`. A name that is not a field
# falls through to R's own `$` / `[[` on a list or an environment receiver,
# and raises `miniextendr_no_field` on a bare pointer or an S4 object. A
# condition reports the call as written: `x$data <- value`.
#
# Fixtures: `SidecarS3`, `SidecarS3Get`, `SidecarS4` and `SidecarEnv` in
# `src/rust/rdata_sidecar_tests.rs`.

caught <- function(expr) tryCatch(expr, error = identity)

# region: S3, the classed pointer

test_that("S3: `$` and `[[` read the fields", {
  x <- new_sidecars3(1.5)
  expect_identical(x$data, 1.5)
  expect_identical(x[["data"]], 1.5)
  expect_identical(x$tags, character())
  expect_identical(x[["tags"]], character())
})

test_that("S3: `$<-` and `[[<-` write the fields and keep the object", {
  x <- new_sidecars3(1.5)
  x$tags <- c("a", "b")
  expect_s3_class(x, "SidecarS3")
  expect_identical(x$tags, c("a", "b"))
  x[["tags"]] <- "c"
  expect_identical(x$tags, "c")
  x$data <- 2.5
  expect_identical(x$data, 2.5)
  x[["data"]] <- 3L
  expect_identical(x$data, 3)

  # The standalone accessors and the Rust methods see the same values.
  expect_identical(SidecarS3_get_tags(x), "c")
  expect_identical(tag_count(x), 1L)
  add_tag(x, "d")
  expect_identical(x$tags, c("c", "d"))
})

test_that("S3: a nested edit goes through the getter, then the setter", {
  x <- new_sidecars3(1.5)
  x$tags <- c("a", "b", "c")
  x$tags[2] <- "z"
  expect_identical(x$tags, c("a", "z", "c"))
  x[["tags"]][3] <- "y"
  expect_identical(x$tags, c("a", "z", "y"))
})

test_that("S3: a value that doesn't convert raises the setter's argument error with the assignment call", {
  x <- new_sidecars3(1.5)
  x$tags <- "kept"
  e <- caught(x$tags <- 1:3)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "'tags' must be", fixed = TRUE)
  expect_identical(e$param, "value")
  expect_equal(conditionCall(e), quote(x$tags <- value))
  e <- caught(x[["tags"]] <- 1:3)
  expect_equal(conditionCall(e), quote(x[["tags"]] <- value))
  # The scalar setters read with `as.numeric()` semantics, which warn first.
  e <- caught(suppressWarnings(x$data <- "a"))
  expect_match(conditionMessage(e), "'data' must be a number", fixed = TRUE)
  expect_equal(conditionCall(e), quote(x$data <- value))
  # A large value is never deparsed into the call.
  e <- caught(x$tags <- as.list(seq_len(1000)))
  expect_length(deparse(conditionCall(e)), 1L)
  # Nothing was written.
  expect_identical(x$tags, "kept")
  expect_identical(x$data, 1.5)
})

test_that("S3: a name that is not a field is refused on a bare pointer, naming the fields", {
  x <- new_sidecars3(1.5)
  e <- caught(x$nope)
  expect_s3_class(e, "miniextendr_no_field")
  expect_identical(
    conditionMessage(e),
    "`nope` is not a field of `SidecarS3`; its fields are `data`, `tags`"
  )
  expect_equal(conditionCall(e), quote(x$nope))
  e <- caught(x[["nope"]])
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x[["nope"]]))
  e <- caught(x$nope <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x$nope <- value))
  e <- caught(x[["nope"]] <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x[["nope"]] <- value))
  # No partial matching on a pointer.
  expect_s3_class(caught(x$ta), "miniextendr_no_field")
  # A numeric index goes to R's own `[[`, which can't subset a pointer.
  expect_error(x[[1]], "not subsettable")
})

test_that("S3: a field write is shared by every copy of the object", {
  x <- new_sidecars3(1.5)
  y <- x
  y$tags <- c("p", "q")
  y$data <- 9
  expect_identical(x$tags, c("p", "q"))
  expect_identical(x$data, 9)
})

# endregion

# region: S3, a list carrying the handle in `.ptr`

new_s3_list <- function() {
  structure(
    list(.ptr = new_sidecars3(1.5), table = data.frame(a = 1:3), note = "plain"),
    class = "SidecarS3"
  )
}

test_that("S3 list: fields go to Rust, other names to the list", {
  x <- new_s3_list()
  expect_identical(x$data, 1.5)
  expect_identical(x[["data"]], 1.5)
  x$tags <- "t"
  expect_identical(x$tags, "t")
  expect_identical(SidecarS3_get_tags(x$.ptr), "t")
  expect_identical(SidecarS3_get_tags(x), "t")
  expect_identical(tag_count(x), 1L)

  expect_identical(x$table, data.frame(a = 1:3))
  expect_identical(x$tab, data.frame(a = 1:3)) # R's partial matching
  expect_identical(x$note, "plain")
  expect_identical(x[["note"]], "plain")
  expect_identical(x[[3]], "plain")
  expect_identical(typeof(x[[".ptr"]]), "externalptr")
  expect_null(x$nope)
  expect_null(x[["nope"]])

  x$table$a[1] <- 5L
  expect_identical(x$table$a, c(5L, 2L, 3L))
  x[["note"]] <- "edited"
  expect_identical(x$note, "edited")
  x$new <- 1
  expect_identical(x$new, 1)
  expect_s3_class(x, "SidecarS3")
  # The list elements don't hold the fields.
  expect_false(any(c("data", "tags") %in% names(x)))
})

test_that("S3 list: lapply(), str() and modifyList() work", {
  x <- new_s3_list()
  expect_identical(
    lapply(x, class),
    list(.ptr = "SidecarS3", table = "data.frame", note = "character")
  )
  expect_no_error(capture.output(str(x)))
  y <- modifyList(x, list(tags = c("m", "n"), note = "modified"))
  expect_identical(x$tags, c("m", "n")) # one pointer: the write is shared
  expect_identical(y$note, "modified")
  expect_identical(x$note, "plain")
})

test_that("S3 list: a field write is shared, a list element write is copied", {
  x <- new_s3_list()
  y <- x
  y$tags <- "shared"
  y$table <- NULL
  y$note <- "copied"
  expect_identical(x$tags, "shared")
  expect_identical(x$table, data.frame(a = 1:3))
  expect_identical(x$note, "plain")
  expect_null(y$table)
})

# endregion

# region: S3, an environment carrying the handle in `.ptr`

test_that("S3 environment: fields go to Rust, other bindings to the environment", {
  e <- new.env()
  e$.ptr <- new_sidecars3(1.5)
  e$plain <- "binding"
  class(e) <- "SidecarS3"

  expect_identical(e$data, 1.5)
  e$tags <- c("e", "f")
  expect_identical(e$tags, c("e", "f"))
  expect_identical(SidecarS3_get_tags(e$.ptr), c("e", "f"))
  expect_identical(e$plain, "binding")
  e$plain <- "rebound"
  expect_identical(get("plain", envir = e), "rebound")
  e[["other"]] <- 2
  expect_identical(get("other", envir = e), 2)
  expect_false(exists("tags", envir = e, inherits = FALSE))
})

# endregion

# region: S3, getters only

test_that("S3 getters only: `$` / `[[` read the fields, the class's own `$<-` writes", {
  x <- new_sidecars3get(2L)
  expect_identical(x$level, 2L)
  expect_identical(x[["level"]], 2L)
  expect_identical(x$notes, character())
  expect_s3_class(caught(x$nope), "miniextendr_no_field")

  # The class's copy-on-modify `$<-` returns a new object: a copy keeps its value.
  y <- x
  y$level <- 5L
  expect_identical(y$level, 5L)
  expect_identical(x$level, 2L)
  expect_s3_class(y, "SidecarS3Get")

  # No generated `[[<-`: R's default can't assign into a pointer.
  expect_false(exists("[[<-.SidecarS3Get", envir = asNamespace("miniextendr")))
  expect_error(suppressWarnings(x[["level"]] <- 3L))
})

# endregion

# region: S4

test_that("S4: `$` and `$<-` reach the fields", {
  o <- SidecarS4(1L, 2.5, "s")
  expect_identical(o$slot_int, 1L)
  expect_identical(o$slot_str, "s")
  expect_identical(o$history, numeric())
  o$history <- c(1, 2)
  expect_identical(o$history, c(1, 2))
  expect_s4_class(o, "SidecarS4")
  o$slot_int <- 7L
  expect_identical(o$slot_int, 7L)
  o$history[3] <- 3
  expect_identical(o$history, c(1, 2, 3))
  expect_identical(s4_history_len(o), 3L)
  s4_record(o, 4)
  expect_identical(o$history, c(1, 2, 3, 4))
})

test_that("S4: a name that is not a field, and a failed conversion, report the call", {
  x <- SidecarS4(1L, 2.5, "s")
  e <- caught(x$nope)
  expect_s3_class(e, "miniextendr_no_field")
  expect_identical(
    conditionMessage(e),
    paste0(
      "`nope` is not a field of `SidecarS4`; its fields are ",
      "`slot_int`, `slot_real`, `slot_str`, `history`"
    )
  )
  expect_equal(conditionCall(e), quote(x$nope))
  # An assignment reports the method's first formal, `x`, for R's `*tmp*`.
  e <- caught(x$nope <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x$nope <- value))
  e <- caught(suppressWarnings(x$slot_int <- "a"))
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(x$slot_int <- value))
  expect_identical(x$slot_int, 1L)
})

test_that("S4: a field write is shared by every copy of the object", {
  o <- SidecarS4(1L, 2.5, "s")
  p <- o
  p$slot_str <- "shared"
  expect_identical(o$slot_str, "shared")
})

# endregion

# region: env

test_that("env: fields and methods both go through `$`", {
  obj <- SidecarEnv$new(3L, 1.5, TRUE, "n")
  expect_identical(obj$count, 3L)
  expect_identical(obj[["count"]], 3L)
  expect_identical(obj$name, "n")
  obj$double_count()
  expect_identical(obj$count, 6L)
  obj$count <- 10L
  expect_identical(obj$count, 10L)
  obj[["name"]] <- "m"
  expect_identical(obj$name, "m")
  expect_identical(SidecarEnv_get_name(obj), "m")
  obj$raw_slot <- list(1, "a")
  expect_identical(obj$raw_slot, list(1, "a"))
  expect_true(is.function(obj$double_count))
  # A name that is neither a field nor a method reads as NULL, as before.
  expect_null(obj$nope)
})

test_that("env: a refused write reports the call; an unknown name is refused", {
  obj <- SidecarEnv$new(3L, 1.5, TRUE, "n")
  # An assignment reports the method's first formal, `x`, for R's `*tmp*`.
  e <- caught(suppressWarnings(obj$count <- "oops"))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'count' must be a number: got \"oops\"")
  expect_equal(conditionCall(e), quote(x$count <- value))
  e <- caught(obj$nope <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_match(conditionMessage(e), "`nope` is not a field of `SidecarEnv`", fixed = TRUE)
  expect_equal(conditionCall(e), quote(x$nope <- value))
  expect_s3_class(caught(obj[["nope"]] <- 1), "miniextendr_no_field")
  expect_identical(obj$count, 3L)
})

# endregion

# region: a saved and restored object

test_that("restored: `$` reads a `Sidecar` field; a struct field raises the restored error", {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  x <- new_sidecars3(1.5)
  x$tags <- c("saved", "tags")
  saveRDS(x, path)
  back <- readRDS(path)
  expect_identical(back$tags, c("saved", "tags"))
  back$tags <- "edited"
  expect_identical(back$tags, "edited")
  expect_error(back$data, class = "miniextendr_restored_no_value")
})

# endregion
