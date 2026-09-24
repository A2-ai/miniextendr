# load_all() registers the source directory as the namespace path; it has
# generated man pages but no installed help database. Exercise both modes.
shared_pages_rd_db <- function() {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
}

# The `\arguments` items of a parsed Rd page as a named character vector
# (item name -> description text), in page order.
rd_argument_items <- function(rd) {
  is_tag <- function(x, tag) identical(attr(x, "Rd_tag"), tag)
  arguments <- Filter(function(x) is_tag(x, "\\arguments"), rd)
  if (length(arguments) == 0L) {
    return(character())
  }
  items <- Filter(function(x) is_tag(x, "\\item"), arguments[[1L]])
  text <- function(x) trimws(gsub("[[:space:]]+", " ", paste(unlist(x), collapse = "")))
  stats::setNames(
    vapply(items, function(item) text(item[[2L]]), character(1)),
    vapply(items, function(item) text(item[[1L]]), character(1))
  )
}

test_that("describeIn functions share the named help page in source order", {
  db <- shared_pages_rd_db()
  aliases <- unlist(lapply(db, function(page) {
    nodes <- page[vapply(page, function(x) identical(attr(x, "Rd_tag"), "\\alias"), logical(1))]
    unique(unlist(nodes))
  }), use.names = FALSE)
  expect_length(unique(aliases[duplicated(aliases)]), 0L)
  expect_true("doc_shared_topic.Rd" %in% names(db))
  rd <- db[["doc_shared_topic.Rd"]]
  section <- function(tag) {
    nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
    gsub("[[:space:]]+", " ", paste(unlist(nodes), collapse = ""))
  }
  expect_match(section("\\title"), "Shared documentation in source order.", fixed = TRUE)
  expect_match(section("\\section"), "Doubles each input value while retaining the input order.", fixed = TRUE)
  expect_match(section("\\section"), "Formats each input value with the shared documentation fixture.", fixed = TRUE)
  expect_match(section("\\keyword"), "utilities", fixed = TRUE)
  expect_match(section("\\keyword"), "methods", fixed = TRUE)
  expect_match(section("\\concept"), "shared documentation fixture", fixed = TRUE)
  usage <- section("\\usage")
  positions <- vapply(c("doc_shared_topic", "doc_shared_double", "doc_shared_vector"),
                     function(name) as.integer(regexpr(name, usage, fixed = TRUE)), integer(1))
  expect_true(all(positions > 0L))
  expect_true(all(diff(positions) > 0L))
  # Explicitly routed functions must not leak onto the automatic file-stem page.
  expect_false(any(grepl("doc_shared_", unlist(db[["doc_attr_tests.Rd"]]), fixed = TRUE)))
})

test_that("shared-page functions and the S3 method remain callable", {
  expect_identical(doc_shared_topic(c(1L, 3L)), c(1L, 3L))
  expect_identical(doc_shared_double(c(1L, 3L)), c(2L, 6L))
  expect_identical(format(structure(c(1L, 3L), class = "doc_shared_vector")), c("1", "3"))
})

test_that("a shared R topic keeps its argument docs when Rust functions join it (#1590)", {
  db <- shared_pages_rd_db()
  expect_true("range_summaries.Rd" %in% names(db))
  rd <- db[["range_summaries.Rd"]]
  args <- rd_argument_items(rd)
  # Every argument appears exactly once: the R block's entries (with the
  # grouped `lower, upper`) plus the one a Rust doc comment documents itself.
  expect_setequal(names(args), c("values", "lower, upper", "outside", "inclusive"))
  expect_false(anyDuplicated(names(args)) > 0L)
  expect_identical(args[["values"]], "A numeric vector.")
  expect_identical(args[["lower, upper"]], "Lower and upper bound of the range.")
  expect_match(args[["outside"]], "moves them onto the nearer bound", fixed = TRUE)
  expect_identical(args[["inclusive"]], "Whether a value equal to a bound counts as inside.")
  expect_false(any(grepl("no documentation available|undocumented|One of", args)))

  # The page is named and titled by the R block, and lists the describeIn member.
  is_tag <- function(x, tag) identical(attr(x, "Rd_tag"), tag)
  section <- function(tag) {
    paste(unlist(Filter(function(x) is_tag(x, tag), rd)), collapse = "")
  }
  expect_identical(section("\\name"), "range_summaries")
  expect_identical(section("\\title"), "Range summaries")
  expect_match(section("\\section"), "Midpoint between the smallest and the largest", fixed = TRUE)

  # `@inheritParams` fills the arguments of a function on its own page.
  inherited <- rd_argument_items(db[["shared_param_docs.Rd"]])
  expect_identical(inherited[["values"]], args[["values"]])
  expect_identical(inherited[["lower, upper"]], args[["lower, upper"]])
  expect_length(inherited, 2L)
})

test_that("the range-summary functions behave as documented", {
  x <- c(2, 5, 9)
  expect_identical(range_width(x), 7)
  expect_identical(range_within(x, 2, 9, inclusive = TRUE), c(TRUE, TRUE, TRUE))
  expect_identical(range_within(x, 2, 9, inclusive = FALSE), c(FALSE, TRUE, FALSE))
  expect_identical(range_clamp(x, 3, 8), c(3, 5, 8))
  expect_identical(range_clamp(x, 3, 8, outside = "drop"), 5)
  expect_identical(range_midpoint(x), 5.5)
  expect_equal(range_share_within(x, 3, 9), 2 / 3)
})
