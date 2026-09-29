# Doc comments carry their blank lines and indentation into R help
# (rpkg/src/rust/roxygen_carry_tests.rs). load_all() has the generated man
# pages but no installed help database, so read whichever exists.
roxygen_carry_rd <- function() {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  db[["roxygen_carry_tests.Rd"]]
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
