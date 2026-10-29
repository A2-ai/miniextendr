# tools/build-html-reference.R renders man/*.Rd into one HTML page with base R
# only. Exercised end to end through Rscript, the way a maintainer runs it.

# A two-topic fixture package with the template script in tools/.
write_refpkg <- function(pkg) {
  script <- system.file("templates", "rpkg", "tools", "build-html-reference.R",
                        package = "minirextendr", mustWork = TRUE)
  dir.create(file.path(pkg, "man"), recursive = TRUE)
  dir.create(file.path(pkg, "tools"))
  writeLines(c("Package: refpkg", "Version: 0.1.0", "Title: Reference Fixture",
               "Description: Exercises the HTML reference builder.",
               "License: MIT", "Encoding: UTF-8"),
             file.path(pkg, "DESCRIPTION"))
  writeLines(c("\\name{hello}", "\\alias{hello}", "\\alias{hello_alias}",
               "\\title{Say hello}",
               "\\description{Greets \\code{x}; see \\code{\\link{goodbye}} and",
               "  \\code{\\link[stats]{median}}; writes to \\code{\\link[base]{stderr}}.}",
               "\\usage{hello(x)}", "\\arguments{\\item{x}{A name.}}",
               "\\value{Invisibly, \\code{x}.}"),
             file.path(pkg, "man", "hello.Rd"))
  writeLines(c("\\name{goodbye}", "\\alias{goodbye}", "\\title{Say goodbye}",
               "\\description{The counterpart of \\code{\\link[refpkg]{hello_alias}}.}",
               "\\usage{goodbye()}", "\\value{NULL.}"),
             file.path(pkg, "man", "goodbye.Rd"))
  file.copy(script, file.path(pkg, "tools", "build-html-reference.R"))
  invisible(pkg)
}

test_that("build-html-reference.R renders a package's Rd sources to one page", {
  skip_if(getRversion() < "4.4.0", "tools::pkg2HTML() needs R >= 4.4.0")
  script <- system.file("templates", "rpkg", "tools", "build-html-reference.R",
                        package = "minirextendr", mustWork = TRUE)
  monorepo <- system.file("templates", "monorepo", "rpkg", "tools",
                          "build-html-reference.R",
                          package = "minirextendr", mustWork = TRUE)
  expect_identical(readLines(script), readLines(monorepo))

  root <- withr::local_tempdir()
  pkg <- write_refpkg(file.path(root, "refpkg"))

  run <- function(..., env = character()) {
    log <- tempfile("html-reference-", fileext = ".log")
    status <- system2(file.path(R.home("bin"), "Rscript"),
                      c(shQuote(file.path(pkg, "tools", "build-html-reference.R")),
                        vapply(list(...), shQuote, character(1))),
                      stdout = log, stderr = log, env = env)
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"))
  }

  # Default output directory, next to where rustdoc would land.
  res <- run()
  expect_identical(res$status, 0L, info = res$output)
  out <- file.path(pkg, "rust-target", "doc", "r")
  page <- file.path(out, "refpkg.html")
  expect_true(all(file.exists(page, file.path(out, "R-nav.css"), file.path(out, "Rlogo.svg"))))
  html <- paste(readLines(page, warn = FALSE), collapse = "\n")
  expect_match(html, "id='topic+hello'", fixed = TRUE)
  expect_match(html, 'href="#topic+goodbye"', fixed = TRUE)
  # Explicit \link[refpkg]{...} stays on the page; a base package's link goes
  # to the page of the Rd file that has the alias (stderr is documented in
  # showConnections.Rd), not to its index; no absolute machine path survives.
  expect_match(html, 'href="#topic+hello_alias"', fixed = TRUE)
  expect_match(html, '"https://search.r-project.org/R/refmans/stats/html/median.html"', fixed = TRUE)
  expect_match(html, '"https://search.r-project.org/R/refmans/base/html/showConnections.html"', fixed = TRUE)
  expect_false(grepl("00Index.html", html, fixed = TRUE))
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

  # A checkRd finding aborts the build instead of rendering around it...
  writeLines(c("\\name{broken}", "\\alias{broken}", "\\title{Broken}",
               "\\description{\\unknownmacro{oops}}"),
             file.path(pkg, "man", "broken.Rd"))
  res <- run(file.path(root, "never"))
  expect_false(identical(res$status, 0L))
  expect_match(res$output, "checkRd findings", fixed = TRUE)
  expect_false(file.exists(file.path(root, "never", "refpkg.html")))

  # ...unless the maintainer opts into a warn-only build.
  res <- run(file.path(root, "lenient"), env = "MINIEXTENDR_HTML_STRICT=0")
  expect_identical(res$status, 0L, info = res$output)
  expect_match(res$output, "building anyway", fixed = TRUE)
  expect_true(file.exists(file.path(root, "lenient", "refpkg.html")))
})

test_that("build-html-reference.R leaves out internal topics nothing on the page links to", {
  skip_if(getRversion() < "4.4.0", "tools::pkg2HTML() needs R >= 4.4.0")
  root <- withr::local_tempdir()
  pkg <- write_refpkg(file.path(root, "refpkg"))
  # An internal helper nothing links to, one that goodbye links to, and the
  # package overview, which roxygen2's "_PACKAGE" marks internal too.
  writeLines(c("\\name{hidden_helper}", "\\alias{hidden_helper}", "\\title{Hidden helper}",
               "\\description{Not part of the API.}", "\\keyword{internal}"),
             file.path(pkg, "man", "hidden_helper.Rd"))
  writeLines(c("\\name{linked_helper}", "\\alias{linked_helper}", "\\title{Linked helper}",
               "\\description{Internal, but goodbye links here.}", "\\keyword{internal}"),
             file.path(pkg, "man", "linked_helper.Rd"))
  writeLines(c("\\docType{package}", "\\name{refpkg-package}", "\\alias{refpkg}",
               "\\alias{refpkg-package}", "\\title{refpkg: Reference Fixture}",
               "\\description{The package overview.}", "\\keyword{internal}"),
             file.path(pkg, "man", "refpkg-package.Rd"))
  writeLines(c("\\name{goodbye}", "\\alias{goodbye}", "\\title{Say goodbye}",
               "\\description{Built on \\code{\\link{linked_helper}}.}",
               "\\usage{goodbye()}", "\\value{NULL.}"),
             file.path(pkg, "man", "goodbye.Rd"))

  run <- function(out, env = character()) {
    log <- tempfile("html-reference-", fileext = ".log")
    status <- system2(file.path(R.home("bin"), "Rscript"),
                      c(shQuote(file.path(pkg, "tools", "build-html-reference.R")), shQuote(out)),
                      stdout = log, stderr = log, env = env)
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"),
         html = if (file.exists(file.path(out, "refpkg.html"))) {
           paste(readLines(file.path(out, "refpkg.html"), warn = FALSE), collapse = "\n")
         })
  }

  res <- run(file.path(root, "default"))
  expect_identical(res$status, 0L, info = res$output)
  expect_false(grepl("id='topic+hidden_helper'", res$html, fixed = TRUE))
  expect_match(res$html, "id='topic+linked_helper'", fixed = TRUE)
  expect_match(res$html, 'href="#topic+linked_helper"', fixed = TRUE)
  expect_match(res$html, "id='topic+refpkg-package'", fixed = TRUE)
  expect_match(res$html, "id='topic+hello'", fixed = TRUE)
  expect_match(res$output, "1 internal left out", fixed = TRUE)
  # The package directory is left as it was.
  expect_true(file.exists(file.path(pkg, "man", "hidden_helper.Rd")))

  res <- run(file.path(root, "all"), env = "MINIEXTENDR_HTML_INTERNAL=1")
  expect_identical(res$status, 0L, info = res$output)
  expect_match(res$html, "id='topic+hidden_helper'", fixed = TRUE)
  expect_false(grepl("internal left out", res$output, fixed = TRUE))
})

test_that("miniextendr_html_reference() runs the package's script from R", {
  skip_if(getRversion() < "4.4.0", "tools::pkg2HTML() needs R >= 4.4.0")
  root <- withr::local_tempdir()
  pkg <- write_refpkg(file.path(root, "refpkg"))

  page <- miniextendr_html_reference(pkg, out = file.path(root, "out"))
  expect_identical(basename(page), "refpkg.html")
  expect_true(file.exists(page))

  # Default output directory and the strict/lenient switch.
  writeLines(c("\\name{broken}", "\\alias{broken}", "\\title{Broken}",
               "\\description{\\unknownmacro{oops}}"),
             file.path(pkg, "man", "broken.Rd"))
  expect_error(suppressWarnings(miniextendr_html_reference(pkg)), "exit status")
  page <- suppressWarnings(miniextendr_html_reference(pkg, strict = FALSE))
  expect_identical(normalizePath(dirname(page)),
                   normalizePath(file.path(pkg, "rust-target", "doc", "r")))
  expect_identical(Sys.getenv("MINIEXTENDR_HTML_STRICT", unset = "unset"), "unset")

  # internal = TRUE renders internal topics and restores the environment.
  writeLines(c("\\name{hidden_helper}", "\\alias{hidden_helper}", "\\title{Hidden helper}",
               "\\description{Not part of the API.}", "\\keyword{internal}"),
             file.path(pkg, "man", "hidden_helper.Rd"))
  unlink(file.path(pkg, "man", "broken.Rd"))
  page <- miniextendr_html_reference(pkg, out = file.path(root, "internal"), internal = TRUE)
  expect_match(paste(readLines(page, warn = FALSE), collapse = "\n"), "id='topic+hidden_helper'", fixed = TRUE)
  page <- miniextendr_html_reference(pkg, out = file.path(root, "public"))
  expect_false(grepl("id='topic+hidden_helper'", paste(readLines(page, warn = FALSE), collapse = "\n"), fixed = TRUE))
  expect_identical(Sys.getenv("MINIEXTENDR_HTML_INTERNAL", unset = "unset"), "unset")

  # No script: point at the helper that adds it.
  bare <- file.path(root, "bare")
  dir.create(bare)
  expect_error(miniextendr_html_reference(bare), "use_miniextendr_html_reference")
})
