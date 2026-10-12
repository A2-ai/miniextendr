# A type's methods split across two `#[miniextendr]` impl blocks: the class is
# defined once and the methods of both blocks attach to it (#1776). Fixtures in
# src/rust/split_impl_tests.rs; the first block of each type has the
# constructor, the second adds methods, among them `from_half`, which returns
# `Self` and stays a static method; both blocks have doc comments.

test_that("an env class keeps the methods of both impl blocks", {
  obj <- SplitEnv$new(5L)
  expect_identical(obj$value(), 5L)
  obj$add(2L)
  expect_identical(obj$value(), 7L)
  expect_identical(obj$doubled(), 14L)
  expect_identical(SplitEnv$origin(), 0L)
  # `new` is the constructor, so the second block's `from_half` is static.
  expect_identical(SplitEnv$from_half(3L)$value(), 6L)
})

test_that("an R6 class keeps the members of both impl blocks", {
  obj <- SplitR6$new(5L)
  expect_identical(obj$value(), 5L)
  obj$add(2L)
  expect_identical(obj$value(), 7L)
  expect_identical(obj$doubled, 14L)
  expect_identical(obj$.__enclos_env__$private$predecessor(), 6L)
  expect_true(is.function(obj$.__enclos_env__$private$finalize))
  expect_identical(SplitR6$origin(), 0L)
  expect_identical(SplitR6$from_half(3L)$value(), 6L)
})

test_that("an R6 deep_clone from a second impl block runs on a deep clone", {
  obj <- SplitR6Clone$new(1L)
  deep <- obj$clone(deep = TRUE)
  deep$set(5L)
  # The handler gave the deep clone its own Rust value.
  expect_identical(obj$value(), 1L)
  expect_identical(deep$value(), 5L)
  # A shallow clone shares the pointer: the handler did not run.
  shallow <- obj$clone()
  shallow$set(9L)
  expect_identical(obj$value(), 9L)
})

test_that("an S3 class keeps the methods of both impl blocks", {
  obj <- SplitS3$new(5L)
  expect_s3_class(obj, "SplitS3")
  expect_identical(split3_value(obj), 5L)
  expect_identical(split3_doubled(obj), 10L)
  expect_identical(splits3_origin(), 0L)
  expect_identical(split3_value(splits3_from_half(3L)), 6L)
})

test_that("an S4 class keeps the methods of both impl blocks", {
  obj <- SplitS4(5L)
  expect_true(methods::is(obj, "SplitS4"))
  expect_identical(s4_split4_value(obj), 5L)
  expect_identical(s4_split4_doubled(obj), 10L)
  expect_identical(SplitS4_origin(), 0L)
  expect_identical(s4_split4_value(SplitS4_from_half(3L)), 6L)
})

test_that("an S7 class keeps the methods of both impl blocks", {
  obj <- SplitS7(5L)
  expect_true(S7::S7_inherits(obj, SplitS7))
  expect_identical(split7_value(obj), 5L)
  expect_identical(split7_doubled(obj), 10L)
  expect_identical(SplitS7_split7_doubled(obj), 10L)
  expect_identical(SplitS7_origin(), 0L)
  expect_identical(split7_value(SplitS7_from_half(3L)), 6L)
})

test_that("a vctrs class keeps the functions of both impl blocks", {
  v <- new_splitvctrs(c(1, 2.5))
  expect_s3_class(v, "SplitVctrs")
  expect_identical(vctrs::vec_ptype_abbr(v), "split")
  expect_identical(vctrs::vec_c(v, v), new_splitvctrs(c(1, 2.5, 1, 2.5)))
  expect_identical(splitvctrs_total(c(1, 2.5)), 3.5)
})

# load_all() registers the source directory as the namespace path; it has
# generated man pages but no installed help database.
split_rd_db <- function() {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
}

split_rd_section <- function(rd, tag) {
  nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
  trimws(gsub("[[:space:]]+", " ", paste(unlist(nodes), collapse = "")))
}

test_that("every export of the split types has a help page", {
  db <- split_rd_db()
  aliases <- unlist(lapply(db, function(rd) unlist(rd[vapply(
    rd, function(x) identical(attr(x, "Rd_tag"), "\\alias"), logical(1)
  )])), use.names = FALSE)
  exports <- grep(
    "^(Split|new_split|split[0-9]_|splits3_|splitvctrs_)",
    getNamespaceExports("miniextendr"),
    value = TRUE
  )
  expect_gt(length(exports), 0L)
  expect_identical(setdiff(exports, aliases), character())
})

# The class page has the doc comments of both impl blocks, the first block's
# first, and one title: the first block's, or the second's when the first has
# no doc comment (SplitEnv).
test_that("the class page has the doc comments of both impl blocks", {
  db <- split_rd_db()
  pages <- list(
    SplitEnv = list(
      title = "Env Class Split Over Two Impl Blocks",
      first = NULL,
      second = "Its second impl block documents the class and adds the arithmetic."
    ),
    SplitR6 = list(
      title = "SplitR6 R6 Class",
      first = "R6 class with methods in two impl blocks.",
      second = "Its second impl block adds an active binding, a private method, a finalizer and a static method."
    ),
    SplitR6Clone = list(
      title = "SplitR6Clone R6 Class",
      first = "Cloneable R6 class with its deep-clone handler in a second impl block.",
      second = "Its second impl block has the deep-clone handler."
    ),
    SplitS3 = list(
      title = "S3 Class Split Over Two Impl Blocks",
      first = "S3 class with methods in two impl blocks.",
      second = "Its second impl block adds a generic and a static method."
    ),
    SplitS4 = list(
      title = "SplitS4 S4 Class",
      first = "S4 class with methods in two impl blocks.",
      second = "Its second impl block adds a generic and a static method."
    ),
    SplitS7 = list(
      title = "SplitS7 S7 Class",
      first = "S7 class with methods in two impl blocks.",
      second = "Its second impl block adds a generic and a static method."
    ),
    SplitVctrs = list(
      title = "SplitVctrs vctrs S3 Class",
      first = "vctrs class with functions in two impl blocks.",
      second = "Its second impl block adds a static function."
    )
  )
  for (class in names(pages)) {
    page <- pages[[class]]
    rd <- db[[paste0(class, ".Rd")]]
    expect_false(is.null(rd), info = class)
    expect_identical(split_rd_section(rd, "\\title"), page$title, info = class)
    description <- split_rd_section(rd, "\\description")
    second <- regexpr(page$second, description, fixed = TRUE)
    expect_gt(second, 0L, label = paste(class, "second block's paragraph"))
    if (!is.null(page$first)) {
      first <- regexpr(page$first, description, fixed = TRUE)
      expect_gt(first, 0L, label = paste(class, "first block's paragraph"))
      expect_lt(first, second, label = paste(class, "first block's paragraph position"))
    }
  }
})
