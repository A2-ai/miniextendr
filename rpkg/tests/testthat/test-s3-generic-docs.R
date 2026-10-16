test_that("owned S3 generics have one generated help page across implementations", {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  aliases <- function(page) {
    unlist(Filter(function(x) identical(attr(x, "Rd_tag"), "\\alias"), page), use.names = FALSE)
  }
  topic <- "generic_doc_shared"
  pages <- Filter(function(page) topic %in% aliases(page), db)
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
    pages <- Filter(function(page) name %in% aliases(page), db)
    expect_identical(names(pages), paste0(name, ".Rd"))
  }
  for (name in c("print", "length", "summary")) {
    expect_false(paste0(name, ".Rd") %in% names(db))
  }
})
