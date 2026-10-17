# Tests for #[miniextendr(as = "...")] attribute and R's as.<class>() coercion methods.
#
# These tests verify that the `as = "..."` attribute generates proper S3 methods
# for R's coercion generics like as.data.frame(), as.list(), as.character().

# region: RCoerceTestData tests - successful coercions

test_that("as.data.frame.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("Alice", "Bob", "Charlie"), c(1.0, 2.0, 3.0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(ncol(df), 2)
  expect_equal(names(df), c("name", "value"))
  expect_equal(df$name, c("Alice", "Bob", "Charlie"))
  expect_equal(df$value, c(1.0, 2.0, 3.0))
})

test_that("as.list.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("x", "y"), c(10.0, 20.0))

  lst <- as.list(obj)

  expect_type(lst, "list")
  expect_equal(length(lst), 2)
  expect_equal(names(lst), c("names", "values"))
  expect_equal(lst$names, c("x", "y"))
  expect_equal(lst$values, c(10.0, 20.0))
})

test_that("as.character.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))

  chr <- as.character(obj)

  expect_type(chr, "character")
  expect_length(chr, 1)
  expect_match(chr, "RCoerceTestData")
  expect_match(chr, "2 items")
})

test_that("as.numeric.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(1.5, 2.5, 3.5))

  num <- as.numeric(obj)

  expect_type(num, "double")
  expect_equal(num, c(1.5, 2.5, 3.5))
})

test_that("as.integer.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(10.9, 20.1, 30.5))

  int <- as.integer(obj)

  expect_type(int, "integer")
  expect_equal(int, c(10L, 20L, 30L))  # truncated, not rounded
})

# endregion

# region: RCoerceErrorTest tests - error handling

test_that("as.data.frame.RCoerceErrorTest returns error when empty", {
  obj <- RCoerceErrorTest$new(TRUE)  # is_empty = TRUE

  expect_error(
    as.data.frame(obj),
    "cannot create data.frame from empty data"
  )
})

test_that("as.data.frame.RCoerceErrorTest works when not empty", {
  obj <- RCoerceErrorTest$new(FALSE)  # is_empty = FALSE

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 1)
  expect_equal(df$value, 42.0)
})

test_that("as.list.RCoerceErrorTest returns NotSupported error", {
  obj <- RCoerceErrorTest$new(FALSE)

  expect_error(
    as.list(obj),
    "NotSupported"
  )
})

test_that("as.character.RCoerceErrorTest returns custom error", {
  obj <- RCoerceErrorTest$new(FALSE)

  expect_error(
    as.character(obj),
    "intentional error for testing"
  )
})

# endregion

# region: Edge cases

test_that("as.data.frame.RCoerceTestData works with empty data", {
  obj <- RCoerceTestData$new(character(0), numeric(0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 0)
  expect_equal(ncol(df), 2)
})

test_that("as.list.RCoerceTestData works with single element", {
  obj <- RCoerceTestData$new("only", 42.0)

  lst <- as.list(obj)

  expect_equal(lst$names, "only")
  expect_equal(lst$values, 42.0)
})

test_that("len method works via S3 generic", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(1, 2, 3))

  expect_equal(len(obj), 3L)
})

# endregion

# region: Exported snake_case surface (constructor aliases + underscore generics)

test_that("new_rcoercetestdata constructor alias builds a working object", {
  obj <- new_rcoercetestdata(c("a", "b"), c(1.0, 2.0))
  expect_true(inherits(obj, "RCoerceTestData"))
  expect_equal(len(obj), 2L)
})

test_that("new_rcoerceerrortest constructor alias builds a working object", {
  obj <- new_rcoerceerrortest(TRUE)
  expect_true(inherits(obj, "RCoerceErrorTest"))
  expect_error(as.data.frame(obj), "cannot create data.frame from empty data")
})

test_that("as_character generic dispatches like as.character", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))
  expect_equal(as_character(obj), as.character(obj))
  expect_error(
    as_character(RCoerceErrorTest$new(FALSE)),
    "intentional error for testing"
  )
})

test_that("as_integer generic truncates values like as.integer", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(10.9, 20.1, 30.5))
  expect_identical(as_integer(obj), c(10L, 20L, 30L))
})

test_that("as_numeric generic dispatches like as.numeric", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(1.5, 2.5, 3.5))
  expect_equal(as_numeric(obj), as.numeric(obj))
})

test_that("as_data_frame generic dispatches like as.data.frame", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))
  expect_equal(as_data_frame(obj), as.data.frame(obj))
})

# endregion

# region: Coercion argument handling (#1830)

test_that("coercion aliases preserve choice formals and omitted arguments", {
  obj <- new_rcoerceargstest()
  method <- getS3method("as.list", "RCoerceArgsTest")
  expect_identical(names(formals(method)), c("x", "mode", "label", "value", "..."))
  choices <- eval(formals(method)$mode)
  expect_identical(choices, c("Fast", "Safe", "Debug"))
  expect_null(formals(method)$label)
  expect_identical(as.list(obj), list(mode = "absent", label = "null", value = "1"))
  expect_identical(as.list(obj, mode = NULL)$mode, "null")
  expect_identical(as.list(obj, mode = "Sa", label = "h"),
                   list(mode = "Safe", label = "high", value = "1"))
  expect_identical(as.list(obj, mode = choices)$mode, "absent")
  expect_identical(as.list(obj, mode = factor("Safe"))$mode, "Safe")
  expect_identical(as.list(obj, mode = "Sa", label = "h"),
                   coerce_selected(obj, mode = "Sa", label = "h"))
  expect_error(as.list(obj, label = "invalid"), class = "rust_error")
  expect_error(as.list(obj, label = c("low", "high", "low")), class = "rust_error")
})

test_that("coercion checks report the generic call and honor explicit guards", {
  obj <- new_rcoerceargstest()
  bad <- structure("wrong", class = "coerce_value")
  err <- tryCatch(as.list(obj, value = bad), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$param, "value")
  expect_match(conditionMessage(err), "double")
  expect_identical(conditionCall(err), quote(as.list(obj, value = bad)))
  expect_error(as.list(obj, value = structure(NA_real_, class = "coerce_value")),
               "coercion value must not be missing", class = "rust_error")
  expect_error(as.list(obj, value = 2), "must inherit", class = "rust_error")
  expect_error(as.list(obj, value = structure(2, class = c("coerce_value", "coerce_forbidden"))),
               "must not inherit", class = "rust_error")
  expect_identical(as.list(obj, value = structure(2, class = "coerce_value"))$value, "2")
})

test_that("coercion wrappers run the ordinary method hooks", {
  obj <- new_rcoerceargstest()
  expect_message(value <- as.character(obj, amount = 1.5), "coercion cleanup")
  expect_identical(value, "2")
  expect_message(err <- tryCatch(as.character(obj, amount = NA_integer_), error = identity),
                 "coercion cleanup")
  expect_s3_class(err, "rust_error")
  expect_identical(err$param, "amount")
  expect_null(conditionCall(err))
})

test_that("coercion wrappers retain one explicit dots formal and no_default", {
  obj <- new_rcoerceargstest()
  method <- getS3method("as.integer", "RCoerceArgsTest")
  expect_identical(names(formals(method)), c("x", "...", "mode"))
  expect_identical(formals(method)["mode"], alist(mode = ))
  expect_identical(as.integer(obj, a = 10, b = 20, mode = "Sa"), c(2L, 2L))
  expect_error(as.integer(obj), 'argument "mode" is missing')
  direct <- tryCatch(coerce_dots(obj), error = identity)
  alias <- tryCatch(as.integer(obj), error = identity)
  expect_identical(class(alias), class(direct))
})

# endregion


# region: Coercion visibility (#1821)

test_that("noexport coercions stay callable directly without S3 registration", {
  ns <- asNamespace("miniextendr")
  obj <- miniextendr:::new_coercehidden()
  method <- get("as.list.CoerceHidden", envir = ns, inherits = FALSE)
  expect_identical(method(obj), list(42L))
  expect_null(getS3method("as.list", "CoerceHidden", optional = TRUE, envir = baseenv()))
  methods <- getNamespaceInfo(ns, "S3methods")
  expect_false(any(methods[, 2L] == "CoerceHidden"))
  expect_false("new_coercehidden" %in% getNamespaceExports(ns))
  expect_false("CoerceHidden" %in% getNamespaceExports(ns))
})

test_that("internal and noRd coercions retain S3 dispatch without constructor exports", {
  ns <- asNamespace("miniextendr")
  for (type in c("CoerceInternal", "CoerceNoRd")) {
    ctor <- paste0("new_", tolower(type))
    obj <- get(ctor, envir = ns, inherits = FALSE)()
    expect_identical(as.list(obj), list(42L))
    expect_true(is.function(getS3method("as.list", type, envir = baseenv())))
    expect_false(ctor %in% getNamespaceExports(ns))
    expect_false(type %in% getNamespaceExports(ns))
  }
})

test_that("coercion aliases follow the class's documentation visibility", {
  rd <- tools::Rd_db("miniextendr")
  aliases <- unlist(lapply(rd, function(page) {
    tags <- Filter(function(tag) identical(attr(tag, "Rd_tag"), "\\alias"), page)
    vapply(tags, paste, character(1), collapse = "")
  }), use.names = FALSE)
  expect_false(any(grepl("CoerceHidden|CoerceNoRd", aliases, ignore.case = TRUE)))
  expect_true("as.list.CoerceInternal" %in% aliases)
  expect_true("coerce_visibility_list.CoerceInternal" %in% aliases)
  expect_true("CoerceInternal.Rd" %in% names(rd))
  keywords <- Filter(function(tag) identical(attr(tag, "Rd_tag"), "\\keyword"),
                     rd[["CoerceInternal.Rd"]])
  expect_true("internal" %in% vapply(keywords, paste, character(1), collapse = ""))
})

# endregion
