# Field syntax for the `#[r_data]` fields of S3, S4 and env classes (#1848):
# `s3(r_data_accessors)`, `s4(r_data_accessors)`, `env(r_data_accessors)`,
# and the getters-only `r_data_accessors = "get"`.
#
# `$` / `[[` read a field and `$<-` / `[[<-` write it through the same
# accessors as `Type_get_f()` / `Type_set_f()`. On an S3 bare pointer the
# reads look the fields up as a list's `$` / `[[` do (#1891): a unique prefix
# through `$`, a position through `[[`, `NULL` for an unknown name; the class
# also gets `names()`, `as.list()` (#1890) and `.DollarNames()` (#1885), the
# last also on S4 and env classes. A name that is not a field falls through
# to R's own `$` / `[[` on a list or an environment receiver; a write to one
# raises `miniextendr_no_field` on a bare pointer, as `$` does on an S4
# object. A condition reports the call as written: `x$data <- value`.
#
# Fixtures: `SidecarS3`, `SidecarS3Get`, `SidecarComputed`, `SidecarHandmade`
# (classed by `new_handmade_rec()` in `R/sidecar_handmade.R`), `SidecarS4`
# and `SidecarEnv` in `src/rust/rdata_sidecar_tests.rs`.

caught <- function(expr) tryCatch(expr, error = identity)

# An S3 list carrying the handle in `.ptr`, next to elements of its own.
new_s3_list <- function() {
  structure(
    list(.ptr = new_sidecars3(1.5), table = data.frame(a = 1:3), note = "plain"),
    class = "SidecarS3"
  )
}

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

test_that("S3: `$` on the pointer reads as a list's does: an exact name, a unique prefix, else NULL", {
  x <- new_sidecars3(1.5)
  x$tags <- c("a", "b")
  expect_null(x$nope)
  expect_null(x[["nope"]])
  # A unique prefix reads the field, as for a list.
  expect_identical(x$ta, c("a", "b"))
  expect_identical(x$d, 1.5)
  # `[[` takes an exact name only.
  expect_null(x[["ta"]])
  expect_null(x[[""]])
  expect_null(x[[NA_character_]])
  # An exact name beats a prefix; an ambiguous prefix is NULL.
  h <- new_handmade_rec(3L)
  expect_identical(h$id, 3L)
  expect_identical(h$ids, 1:3)
  expect_null(h$i)
  expect_identical(h$l, "rec-3")
})

test_that("S3: a partial match through `$` warns under warnPartialMatchDollar, with the call", {
  x <- new_sidecars3(1.5)
  withr::local_options(warnPartialMatchDollar = TRUE)
  w <- tryCatch(x$ta, warning = identity)
  expect_s3_class(w, "simpleWarning")
  expect_identical(conditionMessage(w), "partial match of 'ta' to 'tags'")
  expect_equal(conditionCall(w), quote(x$ta))
  expect_identical(suppressWarnings(x$ta), character())
  # An exact name never warns.
  expect_no_warning(x$tags)
  # A list receiver warns through R's own `$`.
  l <- new_s3_list()
  expect_warning(l$tab, "partial match of 'tab' to 'table'")
})

test_that("S3: `[[` on the pointer takes a position in the field order, as a list's does", {
  x <- new_sidecars3(1.5)
  x$tags <- "t"
  expect_identical(x[[1]], 1.5)
  expect_identical(x[[2L]], "t")
  expect_identical(x[[2.9]], "t") # truncated, as for a list
  expect_identical(x[[TRUE]], 1.5) # a list takes a logical as a position
  expect_null(x[[NA]])
  expect_null(x[[NA_integer_]])
  expect_null(x[[NA_real_]])
  e <- caught(x[[9]])
  expect_s3_class(e, "subscriptOutOfBoundsError")
  expect_identical(conditionMessage(e), "subscript out of bounds")
  expect_equal(conditionCall(e), quote(x[[9]]))
  expect_equal(conditionCall(caught(x[[3L]])), quote(x[[3L]]))
  # `0` and a negative position raise as they do on a list.
  e <- caught(x[[0]])
  expect_match(conditionMessage(e), "attempt to select less than one element", fixed = TRUE)
  expect_equal(conditionCall(e), quote(x[[0]]))
  expect_identical(x[[-1]], "t") # a two-element list's `[[-1]]` is its second element
  h <- new_handmade_rec(1L)
  e <- caught(h[[-1]])
  expect_match(conditionMessage(e), "invalid negative subscript", fixed = TRUE)
  expect_equal(conditionCall(e), quote(h[[-1]]))
  # Any other index goes to R's own `[[`, which can't subset a pointer.
  expect_error(x[[c(1, 2)]], "not subsettable")
  expect_error(x[[c("data", "tags")]], "not subsettable")
  expect_error(x[[list(1)]], "not subsettable")
  expect_error(x[[NULL]], "not subsettable")
})

test_that("S3: a write to a name that is not a field is refused on a bare pointer, naming the fields", {
  x <- new_sidecars3(1.5)
  e <- caught(x$nope <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_identical(
    conditionMessage(e),
    "`nope` is not a field of `SidecarS3`; its fields are `data`, `tags`"
  )
  expect_equal(conditionCall(e), quote(x$nope <- value))
  e <- caught(x[["nope"]] <- 1)
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x[["nope"]] <- value))
  # No partial matching, and no positions, on a write.
  expect_s3_class(caught(x$ta <- "a"), "miniextendr_no_field")
  expect_error(x[[1]] <- 2, "not subsettable")
  expect_identical(x$tags, character())
})

test_that("S3: names() and as.list() on the pointer give the fields in declared order", {
  x <- new_sidecars3(1.5)
  expect_identical(names(x), c("data", "tags"))
  expect_identical(as.list(x), list(data = 1.5, tags = character()))
  x$tags <- c("a", "b")
  expect_identical(as.list(x), list(data = 1.5, tags = c("a", "b")))
  # `lapply()` and `vapply()` iterate `as.list()`; `length()` stays a pointer's.
  expect_identical(lapply(x, class), list(data = "numeric", tags = "character"))
  expect_identical(vapply(x, length, 1L), c(data = 1L, tags = 2L))
  expect_length(x, 1L)
  # Computed fields and R names take their declared place.
  expect_identical(
    names(new_sidecarcomputed(1L)),
    c("doubled", "keys", "nothing", "base", "boom", "table")
  )
  h <- new_handmade_rec(2L)
  expect_identical(names(h), c("id", "ids", "label", "extra"))
  # A NULL field is kept.
  expect_identical(as.list(h), list(id = 2L, ids = 1:2, label = "rec-2", extra = NULL))
  SidecarHandmade_set_extra(h, "set")
  expect_identical(as.list(h)$extra, "set")
  # A computed getter's error reaches `as.list()` with the call.
  x <- new_sidecarcomputed(4L)
  e <- caught(as.list(x))
  expect_s3_class(e, "sidecar_computed_boom")
  expect_equal(conditionCall(e), quote(as.list(x)))
})

test_that("S3: .DollarNames() completes the fields", {
  x <- new_sidecars3(1.5)
  # The completion engine passes an anchored pattern (`x$ta<Tab>` is `^ta`).
  expect_identical(utils::.DollarNames(x, "^ta"), "tags")
  expect_identical(utils::.DollarNames(x, "ta"), c("data", "tags"))
  expect_identical(utils::.DollarNames(x), c("data", "tags"))
  expect_identical(utils::.DollarNames(x, "^nope"), character())
  expect_identical(
    utils::.DollarNames(new_sidecarcomputed(1L), "^b"),
    c("base", "boom")
  )
  # Registered under the qualified generic, so the namespace loads with only
  # base attached.
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_true("S3method(utils::.DollarNames,SidecarS3)" %in% ns)
  expect_true("S3method(utils::.DollarNames,handmade_rec)" %in% ns)
  expect_false(any(grepl("^S3method\\(.DollarNames,", ns)))
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

test_that("S3 list: names() and as.list() are the list's own; .DollarNames() adds its names", {
  x <- new_s3_list()
  expect_identical(names(x), c(".ptr", "table", "note"))
  expect_identical(as.list(x), x)
  expect_identical(x[[2]], data.frame(a = 1:3))
  # The fields first, then the list's own names.
  expect_identical(
    utils::.DollarNames(x),
    c("data", "tags", ".ptr", "table", "note")
  )
  expect_identical(utils::.DollarNames(x, "^ta"), c("tags", "table"))
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
  # `names()` and `as.list()` are the environment's own (`as.list()` through
  # `as.list.environment()`, which the class attribute would otherwise keep
  # from a classed environment); `.DollarNames()` lists the fields, then its
  # bindings.
  expect_setequal(names(e), c(".ptr", "plain", "other"))
  expect_setequal(names(as.list(e)), c("plain", "other"))
  expect_identical(as.list(e)[c("plain", "other")], list(plain = "rebound", other = 2))
  expect_setequal(names(as.list(e, all.names = TRUE)), c(".ptr", "plain", "other"))
  expect_setequal(utils::.DollarNames(e), c("data", "tags", ".ptr", "plain", "other"))
  expect_identical(utils::.DollarNames(e, "^p"), "plain")
})

# endregion

# region: S3, a class the package builds itself (#1891)

test_that("an S3 impl with `class = ...` and no constructor gets the readers for a pointer classed by hand", {
  h <- new_handmade_rec(3L)
  expect_s3_class(h, "handmade_rec")
  expect_identical(typeof(h), "externalptr")
  expect_identical(h$id, 3L)
  expect_identical(h[["ids"]], 1:3)
  expect_identical(h[[3]], "rec-3")
  expect_null(h$extra)
  expect_identical(names(h), c("id", "ids", "label", "extra"))
  expect_identical(utils::.DollarNames(h, "^i"), c("id", "ids"))
  # Getters only: no generated `$<-`, and the standalone setters still write.
  expect_false(exists("$<-.handmade_rec", envir = asNamespace("miniextendr")))
  SidecarHandmade_set_extra(h, "kept")
  expect_identical(h$extra, "kept")
  expect_identical(h$e, "kept")
  # The pointer without the class has no methods.
  p <- rdata_sidecar_handmade_new(1L)
  expect_null(attr(p, "class"))
  expect_error(p$id, "not subsettable")
})

# endregion

# region: S3, getters only

test_that("S3 getters only: `$` / `[[` read the fields, the class's own `$<-` writes", {
  x <- new_sidecars3get(2L)
  expect_identical(x$level, 2L)
  expect_identical(x[["level"]], 2L)
  expect_identical(x$notes, character())
  expect_null(x$nope)
  expect_identical(x$lev, 2L)
  expect_identical(x[[2]], character())
  expect_identical(names(x), c("level", "notes"))
  expect_identical(as.list(x), list(level = 2L, notes = character()))
  expect_identical(utils::.DollarNames(x, "^n"), "notes")

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

test_that("S4: .DollarNames() completes the fields through the S3 method utils dispatches", {
  o <- SidecarS4(1L, 2.5, "s")
  expect_identical(utils::.DollarNames(o, "^slot"), c("slot_int", "slot_real", "slot_str"))
  expect_identical(utils::.DollarNames(o), c("slot_int", "slot_real", "slot_str", "history"))
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_true("S3method(utils::.DollarNames,SidecarS4)" %in% ns)
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

test_that("env: .DollarNames() completes the fields, then the methods", {
  obj <- SidecarEnv$new(3L, 1.5, TRUE, "n")
  completions <- utils::.DollarNames(obj)
  expect_identical(completions[1:5], c("count", "score", "flag", "name", "raw_slot"))
  expect_true(all(c("new", "double_count") %in% completions))
  expect_identical(utils::.DollarNames(obj, "^dou"), "double_count")
  expect_identical(utils::.DollarNames(obj, "^na"), "name")
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_true("S3method(utils::.DollarNames,SidecarEnv)" %in% ns)
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

# region: computed fields and R names (#1883, #1891)
#
# Fixture: `SidecarComputed` in `src/rust/rdata_sidecar_tests.rs`, an S3 class
# whose `Computed` fields (`doubled`, `nothing`, `boom`) interleave with a
# `Sidecar` field named `keys` in R (`r_keys` in Rust), a struct field `base`
# and a `Sidecar<SEXP>` named `table` (`r_table`).

test_that("computed fields read through `$`, `[[` and the standalone getter, at their declared place", {
  x <- new_sidecarcomputed(4L)
  expect_identical(x$doubled, 8L)
  expect_identical(x[["doubled"]], 8L)
  expect_identical(SidecarComputed_get_doubled(x), 8L)
  expect_null(x$nothing)
  expect_null(x[["nothing"]])
  expect_null(SidecarComputed_get_nothing(x))
  # Declared order, computed fields interleaved with the `Sidecar` and struct
  # fields, under the R names.
  expect_identical(
    miniextendr:::.rdata_fields_SidecarComputed,
    c("doubled", "keys", "nothing", "base", "boom", "table")
  )
  # A computed field follows the Rust value.
  x$base <- 10L
  expect_identical(x$doubled, 20L)
  expect_identical(x$base, 10L)
  # No setter exists for a computed field.
  expect_false(exists("SidecarComputed_set_doubled", envir = asNamespace("miniextendr")))
  expect_true(exists("SidecarComputed_set_keys", envir = asNamespace("miniextendr")))
})

test_that("a computed getter's own error reaches R with its class and the call", {
  x <- new_sidecarcomputed(4L)
  e <- caught(x$boom)
  expect_s3_class(e, "sidecar_computed_boom")
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "boom: base is 4")
  expect_equal(conditionCall(e), quote(x$boom))
  e <- caught(x[["boom"]])
  expect_s3_class(e, "sidecar_computed_boom")
  expect_equal(conditionCall(e), quote(x[["boom"]]))
  e <- caught(SidecarComputed_get_boom(x))
  expect_s3_class(e, "sidecar_computed_boom")
  expect_equal(conditionCall(e), quote(SidecarComputed_get_boom(x)))
})

test_that("assigning a computed field raises miniextendr_read_only_field with the assignment call", {
  x <- new_sidecarcomputed(4L)
  e <- caught(x$doubled <- 1L)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(
    conditionMessage(e),
    "`doubled` of `SidecarComputed` is computed from the Rust value and can't be assigned"
  )
  expect_equal(conditionCall(e), quote(x$doubled <- value))
  e <- caught(x[["nothing"]] <- 1L)
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_equal(conditionCall(e), quote(x[["nothing"]] <- value))
  expect_identical(x$doubled, 8L)
})

test_that("`name =` gives a field its R name everywhere R sees it", {
  x <- new_sidecarcomputed(3L)
  expect_identical(x$keys, 1:3)
  expect_identical(x[["keys"]], 1:3)
  expect_identical(SidecarComputed_get_keys(x), 1:3)
  x$keys <- 7L
  expect_identical(SidecarComputed_get_keys(x), 7L)
  SidecarComputed_set_keys(x, 8:9)
  expect_identical(x$keys, 8:9)
  # The Rust side reads the same slot under its Rust name.
  expect_identical(sidecar_computed_keys(x), 8:9)
  expect_identical(key_count(x), 2L)
  expect_null(x$table)
  x$table <- data.frame(a = 1)
  expect_identical(x$table, data.frame(a = 1))
  expect_identical(SidecarComputed_get_table(x), data.frame(a = 1))
  # The Rust identifier is no R name.
  expect_null(x$r_keys)
  e <- caught(x$r_keys <- 1L)
  expect_s3_class(e, "miniextendr_no_field")
  expect_match(
    conditionMessage(e),
    "its fields are `doubled`, `keys`, `nothing`, `base`, `boom`, `table`",
    fixed = TRUE
  )
  expect_false(exists("SidecarComputed_get_r_keys", envir = asNamespace("miniextendr")))
  # The setter's message names the R field.
  e <- caught(x$keys <- "a")
  expect_match(conditionMessage(e), "'keys' must be", fixed = TRUE)
  expect_identical(x$keys, 8:9)
})

# endregion

# region: a saved and restored object (#1891)

reload <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}

# The classes a restored refusal carries, in order, before `rust_error`.
expect_restored_classes <- function(e, classes, label) {
  at <- match(classes, class(e))
  expect_false(anyNA(at), label = paste(label, "carries", paste(classes, collapse = ", ")))
  expect_identical(at, sort(at), label = paste(label, "class order"))
  expect_true(at[length(at)] < match("rust_error", class(e)), label = paste(label, "before rust_error"))
}

test_that("restored: the field methods refuse the pointer; the standalone accessors still read the save", {
  x <- new_sidecars3(1.5)
  x$tags <- c("saved", "tags")
  back <- reload(x)
  # The `Sidecar` value is there: the standalone accessors read and write it.
  expect_identical(SidecarS3_get_tags(back), c("saved", "tags"))
  SidecarS3_set_tags(back, "edited")
  expect_identical(SidecarS3_get_tags(back), "edited")
  # The field methods refuse a restored pointer, `Sidecar` field or not.
  for (read in alist(back$tags, back[["tags"]], back$data, back[["data"]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "miniextendr_restored_no_value")
    expect_s3_class(e, "miniextendr_restored")
    expect_identical(
      conditionMessage(e),
      "this `SidecarS3` object was restored from a saved session and has no Rust value; re-create it",
      label = deparse(read)
    )
  }
  e <- caught(back$tags <- "again")
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_equal(conditionCall(e), quote(x$tags <- value))
  e <- caught(back[["tags"]] <- "again")
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_identical(SidecarS3_get_tags(back), "edited")
  # `as.list()` reads every field, so it refuses too.
  e <- caught(as.list(back))
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_equal(conditionCall(e), quote(as.list(back)))
  # A read that never calls into Rust still answers: an unknown name, the
  # names and the completions.
  expect_null(back$nope)
  expect_null(back[["nope"]])
  expect_identical(names(back), c("data", "tags"))
  expect_identical(utils::.DollarNames(back), c("data", "tags"))
  expect_s3_class(caught(back$nope <- 1), "miniextendr_no_field")
})

test_that("restored: the S4 and env field methods refuse too", {
  o <- SidecarS4(1L, 2.5, "s")
  o$history <- c(1, 2)
  back <- reload(o)
  expect_identical(SidecarS4_get_history(back@ptr), c(1, 2))
  expect_s3_class(caught(back$history), "miniextendr_restored_no_value")
  expect_s3_class(caught(back$history <- 3), "miniextendr_restored_no_value")
  expect_identical(SidecarS4_get_history(back@ptr), c(1, 2))

  obj <- SidecarEnv$new(3L, 1.5, TRUE, "n")
  obj$raw_slot <- list(1)
  back <- reload(obj)
  expect_identical(SidecarEnv_get_raw_slot(back), list(1))
  expect_s3_class(caught(back$raw_slot), "miniextendr_restored_no_value")
  expect_s3_class(caught(back[["raw_slot"]]), "miniextendr_restored_no_value")
  expect_s3_class(caught(back$raw_slot <- list(2)), "miniextendr_restored_no_value")
  expect_identical(SidecarEnv_get_raw_slot(back), list(1))
})

test_that("the package's restored classes and message apply to every restored refusal of the type", {
  back <- reload(new_sidecarcomputed(2L))
  message <- "this SidecarComputed was saved; build a new one with new_sidecarcomputed()"
  classes <- c(
    "sidecar_computed_saved", "sidecar_computed_error",
    "miniextendr_restored_no_value", "miniextendr_restored"
  )
  paths <- list(
    `$ on a Sidecar field` = function() back$keys,
    `$ on a struct field` = function() back$base,
    `$ on a computed field` = function() back$doubled,
    `[[` = function() back[["keys"]],
    `$<-` = function() back$keys <- 1L,
    `standalone struct-field getter` = function() SidecarComputed_get_base(back),
    `standalone computed getter` = function() SidecarComputed_get_doubled(back),
    `method taking the handle` = function() key_count(back),
    `ExternalPtr<T> argument` = function() sidecar_computed_keys(back)
  )
  for (path in names(paths)) {
    e <- caught(paths[[path]]())
    expect_match(conditionMessage(e), message, fixed = TRUE, label = path)
    expect_restored_classes(e, classes, path)
  }
  # The standalone `Sidecar` accessors still read and write the save.
  expect_identical(SidecarComputed_get_keys(back), 1:2)
  SidecarComputed_set_keys(back, 5L)
  expect_identical(SidecarComputed_get_keys(back), 5L)
  # A type without the option keeps miniextendr's message and classes.
  e <- caught(reload(new_sidecars3(1))$data)
  expect_identical(class(e)[1:2], c("miniextendr_restored_no_value", "miniextendr_restored"))
})

test_that("another version's save keeps the package's classes and message, with the version fields", {
  back <- reload(new_sidecarcomputed(2L))
  miniextendr:::sidecar_rewrite_saved_version(back, "0.0.1")
  current <- as.character(utils::packageVersion("miniextendr"))
  classes <- c(
    "sidecar_computed_saved", "sidecar_computed_error",
    "miniextendr_restored_other_version", "miniextendr_restored"
  )
  for (expr in alist(
    back$keys, back$doubled, SidecarComputed_get_keys(back), key_count(back), sidecar_computed_keys(back)
  )) {
    e <- caught(eval(expr))
    expect_match(
      conditionMessage(e),
      "this SidecarComputed was saved; build a new one with new_sidecarcomputed()",
      fixed = TRUE,
      label = deparse(expr)
    )
    expect_restored_classes(e, classes, deparse(expr))
    expect_identical(e$saved_version, "0.0.1", label = deparse(expr))
    expect_identical(e$current_version, current, label = deparse(expr))
  }
})

# endregion
