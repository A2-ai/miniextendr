# Doc comments carry their blank lines and indentation into R help
# (rpkg/src/rust/roxygen_carry_tests.rs). load_all() has the generated man
# pages but no installed help database, so read whichever exists.
roxygen_carry_rd <- function(page = "roxygen_carry_tests.Rd") {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  db[[page]]
}

# The text of one section of a parsed Rd page, as written.
roxygen_carry_section <- function(rd, tag) {
  nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
  paste(unlist(nodes), collapse = "")
}

test_that("a multi-line tag keeps its paragraph breaks", {
  rd <- roxygen_carry_rd()
  expect_false(is.null(rd))
  details <- roxygen_carry_section(rd, "\\details")
  expect_match(
    details,
    "The first paragraph of the details.\n\nThe second paragraph of the details.",
    fixed = TRUE
  )
  value <- roxygen_carry_section(rd, "\\value")
  expect_match(
    value,
    "The number 3.\n\nThe second paragraph of the return value, indented.",
    fixed = TRUE
  )
})

test_that("a markdown list in the leading prose stays a list", {
  rd <- roxygen_carry_rd()
  is_tag <- function(x, tag) identical(attr(x, "Rd_tag"), tag)
  description <- Filter(function(x) is_tag(x, "\\description"), rd)[[1L]]
  lists <- Filter(function(x) is_tag(x, "\\itemize"), description)
  expect_length(lists, 1L)
  items <- Filter(function(x) is_tag(x, "\\item"), lists[[1L]])
  expect_length(items, 2L)
  expect_match(
    paste(unlist(lists), collapse = ""),
    "first: an item\ncontinued on an indented line",
    fixed = TRUE
  )
})

test_that("an example keeps its indentation and runs", {
  rd <- roxygen_carry_rd()
  examples <- roxygen_carry_section(rd, "\\examples")
  expect_match(examples, "x <- c(1, 2) |>\n  sum()\n", fixed = TRUE)
  env <- new.env(parent = asNamespace("miniextendr"))
  eval(parse(text = examples), envir = env)
  expect_equal(env$x, 3)
})

# rpkg keeps `roxygen_prose_links` at its default, "strip"
# (rpkg/src/rust/roxygen_prose_links_tests.rs): leading prose loses its link
# brackets, an explicit tag keeps its \link{}.
test_that("leading prose drops its links and an explicit tag keeps them", {
  rd <- roxygen_carry_rd("roxygen_prose_links_tests.Rd")
  expect_false(is.null(rd))
  # Every Rd macro used inside a node, nested ones included.
  rd_tags <- function(x) {
    c(attr(x, "Rd_tag"), if (is.list(x)) unlist(lapply(x, rd_tags)))
  }
  section <- function(tag) {
    Filter(function(x) identical(attr(x, "Rd_tag"), tag), rd)[[1L]]
  }
  description <- roxygen_carry_section(rd, "\\description")
  expect_match(description, "roxygen_prose_links_demo()", fixed = TRUE)
  expect_no_match(description, "[", fixed = TRUE)
  expect_false("\\link" %in% rd_tags(section("\\description")))
  expect_true("\\link" %in% rd_tags(section("\\details")))
  expect_match(roxygen_carry_section(rd, "\\details"), "roxygen_carry_demo()", fixed = TRUE)
})

# A link no R package can resolve (`crate::` root, a `pkg::` part that is not
# an R package name) loses its brackets in explicit tag text too
# (rpkg/src/rust/roxygen_rustdoc_links_tests.rs, #1739); an R link there stays.
test_that("an explicit tag drops its rustdoc-only links and keeps its R links", {
  rd <- roxygen_carry_rd("roxygen_rustdoc_links_tests.Rd")
  expect_false(is.null(rd))
  details <- Filter(function(x) identical(attr(x, "Rd_tag"), "\\details"), rd)[[1L]]
  rd_tags <- function(x) {
    c(attr(x, "Rd_tag"), if (is.list(x)) unlist(lapply(x, rd_tags)))
  }
  expect_equal(sum(rd_tags(details) == "\\link"), 1L)
  text <- roxygen_carry_section(rd, "\\details")
  expect_match(text, "see\nroxygen_rustdoc_links_demo,", fixed = TRUE)
  expect_match(text, "roxygen_rustdoc_links_tests::roxygen_rustdoc_links_demo and", fixed = TRUE)
  expect_match(text, "crate::roxygen_carry_tests. An R link stays: roxygen_prose_links_demo()", fixed = TRUE)
  expect_no_match(text, "[", fixed = TRUE)
})
