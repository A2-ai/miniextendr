# Field syntax for the `#[r_data]` fields of S3, S4 and env classes (#1848):
# `s3(r_data_accessors)`, `s4(r_data_accessors)`, `env(r_data_accessors)`,
# and the getters-only `r_data_accessors = "get"`.
#
# `$` / `[[` read a field and `$<-` / `[[<-` write it through the same
# accessors as `Type_get_f()` / `Type_set_f()`. No reader matches a prefix
# of a name (#1922). On an S3 bare pointer `$` / `[[` take an exact field
# name, `[[` also a whole-number position, and an unknown name is refused
# with `miniextendr_no_field`, as on a write (#1921); `lookup = "list"`
# (`SidecarS3Get`) reads `NULL` for an unknown name and takes a logical or a
# fractional position, as a list does (#1891). The class also gets
# `names()`, `as.list()` (#1890) and `.DollarNames()` (#1885), the last also
# on S4 and env classes. A name that is not a field falls through, by the
# exact name, to a list or an environment receiver's own elements. A `[[`
# index that is neither one name nor one position is refused by the
# generated `[[` itself (#1920). A condition reports the call as written:
# `x$data <- value`.
#
# Fixtures: `SidecarS3`, `SidecarS3Get`, `SidecarComputed`, `SidecarHandmade`
# (classed by `new_handmade_rec()` in `R/sidecar_handmade.R`), `SidecarStrict`
# (`strict_rec`), `SidecarS4` and `SidecarEnv` in
# `src/rust/rdata_sidecar_tests.rs`.

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

test_that("S3: `$` and `[[` on the pointer take an exact field name; any other name is refused (#1921)", {
  x <- new_sidecars3(1.5)
  x$tags <- c("a", "b")
  # A prefix is no name (#1922), and an unknown name refuses on read as on
  # assignment, naming the class and its fields.
  for (read in alist(x$nope, x[["nope"]], x$ta, x$d, x[["ta"]], x[[""]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "miniextendr_no_field")
    expect_match(
      conditionMessage(e),
      "is not a field of `SidecarS3`; its fields are `data`, `tags`",
      fixed = TRUE,
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  expect_identical(
    conditionMessage(caught(x$ta)),
    "`ta` is not a field of `SidecarS3`; its fields are `data`, `tags`"
  )
  # An exact name reads its field, even where it is a prefix of another.
  h <- new_handmade_rec(3L)
  expect_identical(h$id, 3L)
  expect_identical(h$ids, 1:3)
  expect_s3_class(caught(h$i), "miniextendr_no_field")
  expect_s3_class(caught(h$l), "miniextendr_no_field")
})

test_that("S3: no reader matches a prefix, on the pointer or on a list or environment receiver (#1922)", {
  withr::local_options(warnPartialMatchDollar = TRUE)
  x <- new_sidecars3(1.5)
  expect_s3_class(caught(x$ta), "miniextendr_no_field")
  # A list receiver's own elements go by the exact name: R's `$` on a list
  # would match `tab` to `table`.
  l <- new_s3_list()
  expect_no_warning(expect_null(l$tab))
  expect_null(l[["tab"]])
  expect_null(l[["tab", exact = FALSE]])
  expect_identical(l$table, data.frame(a = 1:3))
  e <- new.env()
  e$.ptr <- new_sidecars3(1.5)
  e$plain <- "binding"
  class(e) <- "SidecarS3"
  expect_null(e$pl)
  expect_null(e[["pl", exact = FALSE]])
  expect_identical(e$plain, "binding")
})

test_that("S3: `[[` on the pointer takes a whole-number position in the field order", {
  x <- new_sidecars3(1.5)
  x$tags <- "t"
  expect_identical(x[[1]], 1.5)
  expect_identical(x[[2L]], "t")
  expect_identical(x[[2.0]], "t")
  # A position that picks no field, `0` and a negative one included (#1899):
  # unlike a list, whose `[[-1]]` drops an element, they raise R's error.
  for (read in alist(x[[9]], x[[3L]], x[[0]], x[[-1]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "subscriptOutOfBoundsError")
    expect_identical(conditionMessage(e), "subscript out of bounds", label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # A logical, `NA`, a fraction and anything that is not one name or one
  # position are refused by the class's own `[[` (#1920, #1921).
  for (read in alist(
    x[[TRUE]], x[[FALSE]], x[[NA]], x[[NA_integer_]], x[[NA_real_]], x[[NA_character_]],
    x[[2.9]], x[[0.5]], x[[c(1, 2)]], x[[c("data", "tags")]], x[[list(1)]], x[[NULL]]
  )) {
    e <- caught(eval(read))
    expect_s3_class(e, "notSubsettableError")
    expect_identical(
      conditionMessage(e),
      "`[[` on a `SidecarS3` object takes one field name or one whole-number position",
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
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
  expect_null(x$tab) # the exact name only (#1922)
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
  # Getters only: no generated `$<-`; the standalone setter is internal and
  # still writes.
  expect_false(exists("$<-.handmade_rec", envir = asNamespace("miniextendr")))
  SidecarHandmade_set_extra(h, "kept")
  expect_identical(h$extra, "kept")
  expect_s3_class(caught(h$e), "miniextendr_no_field")
  # The pointer without the class has no methods.
  p <- rdata_sidecar_handmade_new(1L)
  expect_null(attr(p, "class"))
  expect_error(p$id, "not subsettable")
})

test_that("the readers' own errors carry the type's `field_error` classes in front (#1899)", {
  h <- new_handmade_rec(2L)
  classes <- c("handmade_error_field", "handmade_error")
  in_front <- function(e, also, label) {
    expect_identical(class(e)[seq_along(classes)], classes, label = label)
    expect_s3_class(e, also)
  }
  # A position that picks no field: R's own class stays behind them.
  for (read in alist(h[[5]], h[[0]], h[[-1]])) {
    e <- caught(eval(read))
    in_front(e, "subscriptOutOfBoundsError", deparse(read))
    expect_equal(conditionCall(e), read)
  }
  # A pointer of another type under the class: the message names the R
  # class the user sees, not the Rust type (#1920).
  other <- structure(rdata_sidecar_s3_new(1.5), class = "handmade_rec")
  e <- caught(other$id)
  in_front(e, "rust_error", "type mismatch")
  expect_identical(conditionMessage(e), "this `handmade_rec` object holds a pointer of another type")
  expect_equal(conditionCall(e), quote(other$id))
  # A pointer without an address that no `SidecarHandmade` left behind. A
  # fresh one: `new("externalptr")` is R's shared prototype, which setting a
  # class on would change for every later caller.
  null <- unserialize(serialize(methods::new("externalptr"), NULL))
  class(null) <- "handmade_rec"
  e <- caught(null$id)
  in_front(e, "rust_error", "null pointer")
  expect_identical(conditionMessage(e), "this `handmade_rec` object holds a null pointer")
  # A receiver of the class with no pointer, e.g. an older list format: the
  # restored classes too, the shared parent class after both specific ones.
  old <- structure(list(id = 1L), class = "handmade_rec")
  for (read in alist(old$id, old[["ids"]], old$label)) {
    e <- caught(eval(read))
    expect_identical(
      class(e)[1:4],
      c("handmade_error_saved", "handmade_error_field", "handmade_error", "miniextendr_no_handle"),
      label = deparse(read)
    )
    expect_identical(
      conditionMessage(e),
      "this `handmade_rec` object holds no pointer",
      label = deparse(read)
    )
  }
  # An atomic vector given the class was never saved: no restored class
  # (#1919). Reads that never reach a field stay R's.
  num <- structure(1, class = "handmade_rec")
  e <- caught(num$id)
  expect_identical(
    class(e)[1:3],
    c("handmade_error_field", "handmade_error", "miniextendr_no_handle")
  )
  expect_false(inherits(e, "handmade_error_saved"))
  expect_null(names(num))
  expect_identical(num[[1]], 1)
  # A type without the option keeps the plain errors, which name the class.
  e <- caught(new_sidecars3(1.5)[[9]])
  expect_identical(class(e)[1], "subscriptOutOfBoundsError")
  e <- caught(structure(list(data = 1), class = "SidecarS3")$data)
  expect_identical(class(e)[1], "miniextendr_no_handle")
  expect_identical(conditionMessage(e), "this `SidecarS3` object holds no pointer")
})

test_that("a `[[` index that is neither one name nor one position is refused by the class's own `[[` (#1920)", {
  h <- new_handmade_rec(2L)
  classes <- c("handmade_error_field", "handmade_error", "notSubsettableError")
  for (read in alist(h[[1:2]], h[[c("id", "ids")]], h[[list(1)]], h[[character()]], h[[NULL]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], classes, label = deparse(read))
    expect_identical(
      conditionMessage(e),
      "`[[` on a `handmade_rec` object takes one field name or one whole-number position",
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # A type without `field_error` gets R's class alone, on the user's call.
  x <- new_sidecars3(1.5)
  e <- caught(x[[1:2]])
  expect_identical(class(e)[1], "notSubsettableError")
  expect_identical(
    conditionMessage(e),
    "`[[` on a `SidecarS3` object takes one field name or one whole-number position"
  )
  expect_equal(conditionCall(e), quote(x[[1:2]]))
  # A list carrying the pointer keeps its own `[[` for such an index.
  l <- new_s3_list()
  expect_identical(l[[c(2, 1)]], 1:3)
})

test_that("a class with field syntax keeps its standalone accessors internal (#1900)", {
  exports <- getNamespaceExports("miniextendr")
  ns <- asNamespace("miniextendr")
  # Field syntax: `$` / `[[` are the R API, the accessors are internal.
  for (fn in c(
    "SidecarHandmade_get_id", "SidecarHandmade_set_extra",
    "SidecarS3_get_tags", "SidecarS3_set_tags",
    "SidecarS4_get_history", "SidecarEnv_get_name",
    "SidecarComputed_get_keys"
  )) {
    expect_false(fn %in% exports, label = fn)
    expect_true(exists(fn, envir = ns, inherits = FALSE), label = fn)
  }
  # No field syntax (R6, S7, plain S3): the accessors stay exported.
  for (fn in c("SidecarR6_get_value", "SidecarS7_get_prop_int", "SidecarVctrs_get_vec_label")) {
    expect_true(fn %in% exports, label = fn)
  }
})

# endregion

# region: S3, getters only and a list's lookup (`lookup = "list"`)

test_that("S3 getters only: `$` / `[[` read the fields, the class's own `$<-` writes", {
  x <- new_sidecars3get(2L)
  expect_identical(x$level, 2L)
  expect_identical(x[["level"]], 2L)
  expect_identical(x$notes, character())
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

test_that("lookup = \"list\": an unknown name reads NULL, and a prefix is no name (#1891, #1922)", {
  x <- new_sidecars3get(2L)
  withr::local_options(warnPartialMatchDollar = TRUE)
  expect_null(x$nope)
  expect_null(x[["nope"]])
  expect_null(x[[""]])
  expect_null(x[[NA_character_]])
  # No prefix match, and so no partial-match warning either.
  expect_no_warning(expect_null(x$lev))
  expect_null(x[["lev"]])
  # A list's lookup is still exact on a list or environment receiver: R's `$`
  # on the list would match `lev` to `levels_seen`.
  l <- structure(list(.ptr = new_sidecars3get(2L), levels_seen = 3L), class = "SidecarS3Get")
  expect_identical(l$level, 2L)
  expect_identical(l$levels_seen, 3L)
  expect_no_warning(expect_null(l$lev))
  expect_null(l[["lev"]])
  expect_null(l[["lev", exact = FALSE]])
  e <- new.env()
  e$.ptr <- new_sidecars3get(2L)
  e$levels_seen <- 3L
  class(e) <- "SidecarS3Get"
  expect_identical(e$level, 2L)
  expect_null(e$lev)
  expect_null(e[["lev", exact = FALSE]])
})

test_that("lookup = \"list\": `[[` takes a position as a list's does", {
  x <- new_sidecars3get(2L)
  expect_identical(x[[1]], 2L)
  expect_identical(x[[2L]], character())
  expect_identical(x[[2.9]], character()) # truncated, as for a list
  expect_identical(x[[TRUE]], 2L) # a list takes a logical as a position
  expect_null(x[[NA]])
  expect_null(x[[NA_integer_]])
  expect_null(x[[NA_real_]])
  for (read in alist(x[[9]], x[[0]], x[[-1]], x[[0.5]], x[[FALSE]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "subscriptOutOfBoundsError")
    expect_identical(conditionMessage(e), "subscript out of bounds", label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # Any other index is refused by the class's own `[[` (#1920).
  for (read in alist(x[[c(1, 2)]], x[[c("level", "notes")]], x[[list(1)]], x[[NULL]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "notSubsettableError")
    expect_identical(
      conditionMessage(e),
      "`[[` on a `SidecarS3Get` object takes one field name or one position",
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
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
  # The message names the receiver's R class (#1920).
  sub <- structure(list(.ptr = new_sidecarcomputed(4L)), class = c("computed_sub", "SidecarComputed"))
  expect_identical(
    conditionMessage(caught(sub$doubled <- 1L)),
    "`doubled` of `computed_sub` is computed from the Rust value and can't be assigned"
  )
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
  expect_s3_class(caught(x$r_keys), "miniextendr_no_field")
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
  # A read that never calls into Rust still answers: an unknown name is
  # refused as on a live object, the names and the completions answer.
  expect_s3_class(caught(back$nope), "miniextendr_no_field")
  expect_s3_class(caught(back[["nope"]]), "miniextendr_no_field")
  expect_identical(names(back), c("data", "tags"))
  expect_identical(utils::.DollarNames(back), c("data", "tags"))
  expect_s3_class(caught(back$nope <- 1), "miniextendr_no_field")
})

test_that("restored: under `refuse = \"every_read\"` names(), an unknown name and the completions refuse too (#1901)", {
  h <- new_handmade_rec(2L)
  back <- reload(h)
  message <- "this handmade_rec was saved; build a new one with new_handmade_rec()"
  classes <- c(
    "handmade_error_saved", "handmade_error",
    "miniextendr_restored_no_value", "miniextendr_restored"
  )
  for (read in alist(
    back$id, back[["ids"]], back[[1]], back$nope, back$i, back[["nope"]], back[[NA]],
    names(back), as.list(back)
  )) {
    e <- caught(eval(read))
    expect_identical(conditionMessage(e), message, label = deparse(read))
    expect_restored_classes(e, classes, deparse(read))
    expect_equal(conditionCall(e), read)
  }
  # The completions serve the console: they list nothing instead of raising.
  expect_identical(utils::.DollarNames(back), character(0))
  expect_identical(utils::.DollarNames(back, "^i"), character(0))
  # A live object answers as before.
  expect_s3_class(caught(h$nope), "miniextendr_no_field")
  expect_identical(names(h), c("id", "ids", "label", "extra"))
  expect_identical(utils::.DollarNames(h, "^i"), c("id", "ids"))
})

test_that("restored: under `refuse = \"every_read\"` the pointer is checked before the index (#1919)", {
  back <- reload(new_handmade_rec(2L))
  classes <- c(
    "handmade_error_saved", "handmade_error",
    "miniextendr_restored_no_value", "miniextendr_restored"
  )
  # A position past the last field and an index that is no name or
  # position: the saved pointer is refused first.
  for (read in alist(back[[9]], back[[0]], back[[1:2]], back[[list(1)]])) {
    e <- caught(eval(read))
    expect_restored_classes(e, classes, deparse(read))
    expect_equal(conditionCall(e), read)
  }
})

test_that("restored: under `refuse = \"every_read\"` a list or environment of the class without a pointer refuses every read (#1919)", {
  classes <- c("handmade_error_saved", "handmade_error_field", "handmade_error", "miniextendr_no_handle")
  # The shape an earlier release may have saved: a list of the class with
  # a pointer element of another name.
  old <- structure(list(ptr = methods::new("externalptr"), table = data.frame()), class = "handmade_rec")
  env <- structure(new.env(), class = "handmade_rec")
  for (read in alist(
    names(old), as.list(old), old$nope, old$table, old$id, old[[1]], old[["table"]], old[[1:2]],
    names(env), as.list(env), env$nope, env[["id"]]
  )) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:4], classes, label = deparse(read))
    expect_identical(conditionMessage(e), "this `handmade_rec` object holds no pointer", label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # A list carrying a live pointer keeps its own elements; one carrying a
  # saved pointer is refused like the pointer.
  live <- structure(list(.ptr = rdata_sidecar_handmade_new(2L), note = "n"), class = "handmade_rec")
  expect_identical(live$note, "n")
  expect_identical(live$id, 2L)
  expect_identical(names(live), c(".ptr", "note"))
  expect_identical(live[[2]], "n")
  # The completions never raise: a receiver the readers refuse completes to
  # nothing, a live one to the fields and its own names.
  expect_identical(utils::.DollarNames(old), character(0))
  expect_identical(utils::.DollarNames(env), character(0))
  expect_identical(utils::.DollarNames(live, "^[in]"), c("id", "ids", "note"))
  saved <- reload(live)
  expect_identical(utils::.DollarNames(saved), character(0))
  for (read in alist(names(saved), saved$note, saved[[2]], as.list(saved))) {
    e <- caught(eval(read))
    expect_restored_classes(
      e,
      c("handmade_error_saved", "handmade_error", "miniextendr_restored_no_value"),
      deparse(read)
    )
  }
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

# region: S3, the default lookup on a type with its own refusal message (#1920, #1921)

test_that("strict_rec: exact names and whole-number positions read their field", {
  s <- new_strict_rec(3L)
  expect_identical(s$id, 3L)
  expect_identical(s$steps, 1:3)
  expect_identical(s[["steps"]], 1:3)
  expect_identical(s[[1]], 3L)
  expect_identical(s[[2L]], 1:3)
  expect_identical(s[[2.0]], 1:3)
  expect_identical(names(s), c("id", "steps"))
  expect_identical(as.list(s), list(id = 3L, steps = 1:3))
})

test_that("strict_rec: an unknown name refuses on read as on assignment, with no partial match", {
  s <- new_strict_rec(3L)
  classes <- c("strict_error_field", "strict_error", "miniextendr_no_field")
  withr::local_options(warnPartialMatchDollar = TRUE)
  for (read in alist(s$st, s$nope, s[["st"]], s[["nope"]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], classes, label = deparse(read))
    expect_match(
      conditionMessage(e),
      "is not a field of `strict_rec`; its fields are `id`, `steps`",
      fixed = TRUE,
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("strict_rec: any other index is refused, not read as a list would", {
  s <- new_strict_rec(3L)
  bad <- c("strict_error_field", "strict_error", "notSubsettableError")
  for (read in alist(
    s[[TRUE]], s[[FALSE]], s[[NA]], s[[NA_real_]], s[[NA_character_]], s[[1.5]],
    s[[1:2]], s[[list(1)]], s[[character()]]
  )) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], bad, label = deparse(read))
    expect_identical(
      conditionMessage(e),
      "`[[` on a `strict_rec` object takes one field name or one whole-number position",
      label = deparse(read)
    )
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  oob <- c("strict_error_field", "strict_error", "subscriptOutOfBoundsError")
  for (read in alist(s[[0]], s[[-1]], s[[3]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], oob, label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("strict_rec: a list carrying the pointer keeps its own elements, by the exact name", {
  l <- structure(list(.ptr = new_strict_rec(2L), note = "n"), class = "strict_rec")
  expect_identical(l$steps, 1:2)
  expect_identical(l$note, "n")
  expect_null(l$nope)
  expect_null(l$no)
  expect_identical(l[[2]], "n")
})

test_that("strict_rec: the type's own message replaces the receiver refusals (#1920)", {
  message <- "not a strict_rec; build one with new_strict_rec()"
  other <- structure(rdata_sidecar_s3_new(1.5), class = "strict_rec")
  e <- caught(other$id)
  expect_identical(conditionMessage(e), message)
  expect_identical(class(e)[1:2], c("strict_error_field", "strict_error"))
  old <- structure(list(id = 1L), class = "strict_rec")
  e <- caught(names(old))
  expect_identical(conditionMessage(e), message)
  expect_identical(
    class(e)[1:4],
    c("strict_error_saved", "strict_error_field", "strict_error", "miniextendr_no_handle")
  )
})

test_that("strict_rec: a saved copy is refused before the name or index is looked at", {
  back <- reload(new_strict_rec(2L))
  # The default message names the R class, not the Rust type (#1920).
  expect_identical(
    conditionMessage(caught(back$id)),
    "this `strict_rec` object was restored from a saved session and has no Rust value; re-create it"
  )
  for (read in alist(back$st, back$steps, back[[TRUE]], back[[9]], names(back))) {
    e <- caught(eval(read))
    expect_restored_classes(
      e,
      c("strict_error_saved", "strict_error", "miniextendr_restored_no_value"),
      deparse(read)
    )
  }
})

# endregion
