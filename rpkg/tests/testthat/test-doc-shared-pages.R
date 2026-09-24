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

# The text of every `tag` node of a parsed Rd page, whitespace collapsed.
rd_section_text <- function(rd, tag) {
  nodes <- Filter(function(x) identical(attr(x, "Rd_tag"), tag), rd)
  trimws(gsub("[[:space:]]+", " ", paste(unlist(nodes), collapse = "")))
}

# The `\alias` values of a parsed Rd page.
rd_aliases <- function(rd) {
  unlist(Filter(function(x) identical(attr(x, "Rd_tag"), "\\alias"), rd), use.names = FALSE)
}

test_that("a shared R topic keeps its argument docs when Rust functions join it (#1590)", {
  db <- shared_pages_rd_db()
  expect_true("range_summaries.Rd" %in% names(db))
  rd <- db[["range_summaries.Rd"]]
  args <- rd_argument_items(rd)
  # Every argument appears exactly once: the R block's entries (with the
  # grouped `lower, upper`, and `x` / `...` for the RangeBox S3 methods) plus
  # the one a Rust doc comment documents itself.
  expect_setequal(names(args), c("values", "lower, upper", "outside", "x", "...", "inclusive"))
  expect_false(anyDuplicated(names(args)) > 0L)
  expect_identical(args[["values"]], "A numeric vector.")
  expect_identical(args[["lower, upper"]], "Lower and upper bound of the range.")
  expect_match(args[["outside"]], "moves them onto the nearer bound", fixed = TRUE)
  expect_identical(args[["inclusive"]], "Whether a value equal to a bound counts as inside.")
  # The S3 methods' structural `@param x An object.` / `@param ...` lines
  # would win over the family's entries; the joining blocks leave them out.
  expect_identical(args[["x"]], "A RangeBox object.")
  expect_identical(args[["..."]], "Unused; accepted for S3 method compatibility.")
  expect_false(any(grepl("no documentation available|undocumented|One of|An object|Additional arguments", args)))

  # The page is named and titled by the R block although R/range_summaries.R
  # sorts after R/miniextendr-wrappers.R: the generated blocks that join the
  # page carry `@order NaN` and so are read after it.
  expect_identical(rd_section_text(rd, "\\name"), "range_summaries")
  expect_identical(rd_section_text(rd, "\\title"), "Range summaries")
  expect_match(rd_section_text(rd, "\\section"), "Midpoint between the smallest and the largest", fixed = TRUE)

  # `@inheritParams` fills the arguments of a function on its own page.
  inherited <- rd_argument_items(db[["shared_param_docs.Rd"]])
  expect_identical(inherited[["values"]], args[["values"]])
  expect_identical(inherited[["lower, upper"]], args[["lower, upper"]])
  expect_length(inherited, 2L)
})

test_that("method-level @describeIn lists S3 methods on the destination page (#1590)", {
  db <- shared_pages_rd_db()
  rd <- db[["range_summaries.Rd"]]
  functions <- rd_section_text(rd, "\\section")
  # roxygen2 names an S3 method entry `generic(Class)` and a function `name()`.
  expect_match(functions, "box_width(RangeBox): Width of a range box.", fixed = TRUE)
  expect_match(functions, "rangebox_from_values(): The smallest range box holding every value.", fixed = TRUE)
  usage <- rd_section_text(rd, "\\usage")
  for (call in c("box_width", "box_covers", "rangebox_from_values(values)")) {
    expect_match(usage, call, fixed = TRUE)
  }
  # The methods (and the S3 generic blocks named after them) moved off the
  # class page; `@rdname` and `@describeIn` members share the family page.
  expect_true(all(c("box_width.RangeBox", "box_covers.RangeBox", "rangebox_from_values") %in% rd_aliases(rd)))
  class_page <- db[["RangeBox.Rd"]]
  expect_false(any(grepl("box_width|box_covers|rangebox_from_values", rd_aliases(class_page))))
  expect_identical(names(rd_argument_items(class_page)), c("lower", "upper"))
})

test_that("a method split onto its own page documents the S3 method arguments itself (#1590)", {
  args <- rd_argument_items(shared_pages_rd_db()[["greeting_build.Rd"]])
  expect_identical(args[["x"]], "A GreetingBuilder.")
  expect_identical(args[["..."]], "Unused; accepted for S3 method compatibility.")
  expect_length(args, 2L)
})

test_that("a file-stem page keeps generated @param lines only where no function documents the argument (#1590)", {
  db <- shared_pages_rd_db()
  expect_true("stem_page_docs.Rd" %in% names(db))
  rd <- db[["stem_page_docs.Rd"]]
  expect_identical(rd_section_text(rd, "\\name"), "stem_page_docs")
  expect_identical(rd_section_text(rd, "\\title"), "Values transformed in place")
  expect_true(all(c("stem_scale", "stem_shift", "stem_floor") %in% rd_aliases(rd)))
  args <- rd_argument_items(rd)
  expect_false(anyDuplicated(names(args)) > 0L)
  expect_setequal(names(args), c("values", "factor", "offset", "direction", "floor"))
  # `values` is documented by the first function only; the generated lines of
  # the other two (written later, so they would win) are dropped.
  expect_identical(args[["values"]], "Numbers to transform.")
  # No function documents `direction`: its generated choice list stays.
  expect_identical(args[["direction"]], "One of \"up\", \"down\".")
  expect_false(any(grepl("no documentation available", args, fixed = TRUE)))
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
  box <- rangebox_from_values(x)
  expect_s3_class(box, "RangeBox")
  expect_identical(box_width(box), 7)
  expect_identical(box_covers(box, c(1, 2, 9, 10)), c(FALSE, TRUE, TRUE, FALSE))
  expect_identical(box_width(new_rangebox(1, 4)), 3)
})

test_that("the file-stem page functions behave as documented", {
  x <- c(1, 5)
  expect_identical(stem_scale(x, 2), c(2, 10))
  expect_identical(stem_shift(x, 1), c(2, 6))
  expect_identical(stem_shift(x, 1, direction = "down"), c(0, 4))
  expect_identical(stem_floor(x, 3), c(3, 5))
})
