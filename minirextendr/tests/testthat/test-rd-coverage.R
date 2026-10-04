# Every function in R/ has an Rd page, exported or not (#1727).
#
# A helper's contract (arguments, return value) is only checked once its
# roxygen block becomes an Rd page: the Rd checks then compare its \usage
# with its \arguments. Internal helpers get `@keywords internal`, not `@noRd`.
#
# Both tests read the package source (R/*.R and man/*.Rd), so they skip when
# it is not available, e.g. when testing an installed package.

# The minirextendr source directory: the package these tests belong to
# (devtools::test(), testthat::test_local()), else the monorepo checkout
# named by MINIEXTENDR_LOCAL_PATH. NULL when neither has R/ and man/.
minirextendr_source_dir <- function() {
  repo <- find_miniextendr_repo()
  candidates <- c(
    file.path(testthat::test_path(), "..", ".."),
    if (!is.null(repo)) file.path(repo, "minirextendr")
  )
  for (dir in candidates) {
    desc <- file.path(dir, "DESCRIPTION")
    if (!file.exists(desc)) next
    package <- unname(read.dcf(desc, fields = "Package")[1L, 1L])
    if (!identical(package, "minirextendr")) next
    has_r <- length(list.files(file.path(dir, "R"), pattern = "\\.[Rr]$")) > 0L
    has_man <- length(list.files(file.path(dir, "man"), pattern = "\\.Rd$")) > 0L
    if (has_r && has_man) return(normalizePath(dir))
  }
  NULL
}

# Top-level `name <- function(...)` / `name = function(...)` definitions,
# one row per function, with the file that defines it.
top_level_functions <- function(r_dir) {
  files <- sort(list.files(r_dir, pattern = "\\.[Rr]$", full.names = TRUE))
  found <- lapply(files, function(file) {
    exprs <- parse(file, keep.source = FALSE, encoding = "UTF-8")
    is_fun <- vapply(exprs, function(e) {
      is.call(e) && length(e) == 3L &&
        (identical(e[[1L]], as.name("<-")) || identical(e[[1L]], as.name("="))) &&
        is.call(e[[3L]]) && identical(e[[3L]][[1L]], as.name("function"))
    }, logical(1))
    fn_names <- vapply(exprs[is_fun], function(e) as.character(e[[2L]]), character(1))
    data.frame(name = fn_names, file = rep(basename(file), length(fn_names)))
  })
  do.call(rbind, found)
}

# Every \alias{} of the Rd files in `man_dir`, unescaped (`\%` becomes `%`).
rd_aliases <- function(man_dir) {
  files <- list.files(man_dir, pattern = "\\.Rd$", full.names = TRUE)
  unlist(lapply(files, function(file) {
    rd <- tools::parse_Rd(file, encoding = "UTF-8")
    tags <- vapply(rd, attr, character(1), "Rd_tag")
    vapply(rd[tags == "\\alias"], function(x) paste(unlist(x), collapse = ""), character(1))
  }))
}

test_that("every top-level function in R/ has an Rd page", {
  pkg_dir <- minirextendr_source_dir()
  skip_if(is.null(pkg_dir), "minirextendr source tree (R/ and man/) not found")

  funs <- top_level_functions(file.path(pkg_dir, "R"))
  expect_gt(nrow(funs), 0L)

  missing <- funs[!funs$name %in% rd_aliases(file.path(pkg_dir, "man")), ]
  if (nrow(missing) > 0L) {
    fail(paste0(
      nrow(missing), " function(s) without an \\alias{} in man/*.Rd. ",
      "Give each a roxygen block (`@keywords internal` for internal ones, ",
      "never `@noRd`) and run devtools::document():\n",
      paste0("  ", missing$name, "() in R/", missing$file, collapse = "\n")
    ))
  } else {
    succeed()
  }
})

test_that("every Rd page documents all of its arguments, internal pages too", {
  pkg_dir <- minirextendr_source_dir()
  skip_if(is.null(pkg_dir), "minirextendr source tree (R/ and man/) not found")

  # R CMD check skips undocumented arguments on `\keyword{internal}` pages
  # unless _R_CHECK_RD_INTERNAL_TOO_ is set; chkInternal = TRUE checks them.
  problems <- tools::checkDocFiles(dir = pkg_dir, chkInternal = TRUE)
  expect_identical(format(problems), character())
})
