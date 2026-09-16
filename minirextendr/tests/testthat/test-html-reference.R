# tools/build-html-reference.R renders man/*.Rd into one HTML page with base R
# only. Exercised end to end through Rscript, the way a maintainer runs it.
test_that("build-html-reference.R renders a package's Rd sources to one page", {
  skip_if(getRversion() < "4.4.0", "tools::pkg2HTML() needs R >= 4.4.0")
  script <- system.file("templates", "rpkg", "tools", "build-html-reference.R",
                        package = "minirextendr", mustWork = TRUE)
  monorepo <- system.file("templates", "monorepo", "rpkg", "tools",
                          "build-html-reference.R",
                          package = "minirextendr", mustWork = TRUE)
  expect_identical(readLines(script), readLines(monorepo))

  root <- withr::local_tempdir()
  pkg <- file.path(root, "refpkg")
  dir.create(file.path(pkg, "man"), recursive = TRUE)
  dir.create(file.path(pkg, "tools"))
  writeLines(c("Package: refpkg", "Version: 0.1.0", "Title: Reference Fixture",
               "Description: Exercises the HTML reference builder.",
               "License: MIT", "Encoding: UTF-8"),
             file.path(pkg, "DESCRIPTION"))
  writeLines(c("\\name{hello}", "\\alias{hello}", "\\alias{hello_alias}",
               "\\title{Say hello}",
               "\\description{Greets \\code{x}; see \\code{\\link{goodbye}} and",
               "  \\code{\\link[stats]{median}}.}",
               "\\usage{hello(x)}", "\\arguments{\\item{x}{A name.}}",
               "\\value{Invisibly, \\code{x}.}"),
             file.path(pkg, "man", "hello.Rd"))
  writeLines(c("\\name{goodbye}", "\\alias{goodbye}", "\\title{Say goodbye}",
               "\\description{The counterpart of \\code{\\link[refpkg]{hello_alias}}.}",
               "\\usage{goodbye()}", "\\value{NULL.}"),
             file.path(pkg, "man", "goodbye.Rd"))
  file.copy(script, file.path(pkg, "tools", "build-html-reference.R"))

  run <- function(...) {
    log <- tempfile("html-reference-", fileext = ".log")
    status <- system2(file.path(R.home("bin"), "Rscript"),
                      c(shQuote(file.path(pkg, "tools", "build-html-reference.R")),
                        vapply(list(...), shQuote, character(1))),
                      stdout = log, stderr = log)
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"))
  }

  # Default output directory, next to where rustdoc would land.
  res <- run()
  expect_identical(res$status, 0L, info = res$output)
  out <- file.path(pkg, "src", "rust", "target", "doc", "r")
  page <- file.path(out, "refpkg.html")
  expect_true(all(file.exists(page, file.path(out, "R-nav.css"), file.path(out, "Rlogo.svg"))))
  html <- paste(readLines(page, warn = FALSE), collapse = "\n")
  expect_match(html, "id='topic+hello'", fixed = TRUE)
  expect_match(html, 'href="#topic+goodbye"', fixed = TRUE)
  # Explicit \link[refpkg]{...} stays on the page; base packages go to their
  # hosted manuals; no absolute machine path survives.
  expect_match(html, 'href="#topic+hello_alias"', fixed = TRUE)
  expect_match(html, "search.r-project.org/R/refmans/stats/", fixed = TRUE)
  expect_false(grepl("refman/refpkg", html, fixed = TRUE))
  expect_false(grepl(R.home(), html, fixed = TRUE))
  expect_match(res$output, "wrote .*refpkg[.]html [(][0-9]+ topics[)]")

  # An explicit output directory beside a rustdoc crate index: the manual is
  # registered once, even when the script runs twice.
  docs <- file.path(root, "docs")
  dir.create(docs)
  writeLines(c("<html><head><title>Index of crates</title></head><body>",
               "<h1>List of all crates</h1>",
               '<ul class="all-items"><li><a href="mycrate/index.html">mycrate</a></li></ul>',
               "</body></html>"),
             file.path(docs, "index.html"))
  for (i in 1:2) {
    res <- run(file.path(docs, "r"))
    expect_identical(res$status, 0L, info = res$output)
  }
  expect_true(file.exists(file.path(docs, "r", "refpkg.html")))
  idx <- paste(readLines(file.path(docs, "index.html"), warn = FALSE), collapse = "\n")
  expect_identical(lengths(regmatches(idx, gregexpr('href="r/refpkg.html"', idx, fixed = TRUE))), 1L)
  expect_match(idx, 'href="mycrate/index.html"', fixed = TRUE)
  expect_match(idx, "<title>refpkg documentation</title>", fixed = TRUE)
  expect_match(res$output, "registered on", fixed = TRUE)

  # A checkRd finding aborts the build instead of rendering around it.
  writeLines(c("\\name{broken}", "\\alias{broken}", "\\title{Broken}",
               "\\description{\\unknownmacro{oops}}"),
             file.path(pkg, "man", "broken.Rd"))
  res <- run(file.path(root, "never"))
  expect_false(identical(res$status, 0L))
  expect_match(res$output, "checkRd findings", fixed = TRUE)
  expect_false(file.exists(file.path(root, "never", "refpkg.html")))
})
