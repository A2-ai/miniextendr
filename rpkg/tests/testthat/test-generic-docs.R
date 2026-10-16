generic_help_db <- function() {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
}

generic_help_aliases <- function(page) {
  unlist(Filter(function(x) identical(attr(x, "Rd_tag"), "\\alias"), page), use.names = FALSE)
}

test_that("owned S3 generics have one generated help page across implementations", {
  db <- generic_help_db()
  topic <- "generic_doc_shared"
  pages <- Filter(function(page) topic %in% generic_help_aliases(page), db)
  expect_identical(names(pages), paste0(topic, ".Rd"))
  text <- paste(unlist(pages[[1L]]), collapse = "")
  expect_match(text, "an S3 generic", fixed = TRUE)
  expect_match(text, "GenericDocInherent", fixed = TRUE)
  expect_match(text, "GenericDocTrait", fixed = TRUE)
  expect_true(topic %in% getNamespaceExports("miniextendr"))
  expect_identical(generic_doc_shared(GenericDocInherent$new()), 11L)
  expect_identical(generic_doc_shared(GenericDocTrait$new()), 22L)

  # Existing inherent generics need no handwritten aliases either.
  for (name in c("split3_value", "split3_doubled", "box_width", "box_midpoint")) {
    pages <- Filter(function(page) name %in% generic_help_aliases(page), db)
    expect_identical(names(pages), paste0(name, ".Rd"))
  }
  for (name in c("print", "length", "summary")) {
    expect_false(paste0(name, ".Rd") %in% names(db))
  }
})

test_that("explicitly owned S4 generics are documented and retain both methods", {
  topic <- "generic_doc_s4_shared"
  pages <- Filter(function(page) topic %in% generic_help_aliases(page), generic_help_db())
  expect_identical(names(pages), paste0(topic, ".Rd"))
  text <- paste(unlist(pages[[1L]]), collapse = "")
  expect_match(text, "an S4 generic", fixed = TRUE)
  expect_match(text, "GenericDocS4Inherent", fixed = TRUE)
  expect_match(text, "GenericDocS4Trait", fixed = TRUE)
  args <- Filter(function(x) identical(attr(x, "Rd_tag"), "\\arguments"), pages[[1L]])
  items <- Filter(function(x) identical(attr(x, "Rd_tag"), "\\item"), args[[1L]])
  expect_identical(
    vapply(items, function(x) paste(x[[1L]], collapse = ""), character(1)),
    c("object", "...")
  )

  ns <- asNamespace("miniextendr")
  expect_true(topic %in% getNamespaceExports(ns))
  generic <- methods::getGeneric(topic, where = ns)
  expect_identical(generic@package, "miniextendr")
  expect_identical(names(formals(generic)), c("object", "..."))
  expect_identical(generic_doc_s4_shared(GenericDocS4Inherent()), 31L)
  expect_identical(generic_doc_s4_shared(GenericDocS4Trait()), 32L)
})
