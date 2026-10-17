test_that("qualified S3 methods have only lazy namespace registrations", {
  ns <- asNamespace("miniextendr")
  namespace <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  targets <- list(
    "tibble::as_tibble" = c("mx_qualified_rows", "mx_qualified_export", "qualified_rows"),
    "tibble::tbl_sum" = "qualified_rows",
    "vctrs::vec_ptype2" = "mx_qualified_vec.mx_qualified_vec"
  )
  for (target in names(targets)) {
    generic <- sub("^.*::", "", target)
    for (class in targets[[target]]) {
      method <- paste(generic, class, sep = ".")
      expect_true(exists(method, envir = ns, inherits = FALSE))
      expect_false(method %in% getNamespaceExports(ns))
      expect_true(paste0("S3method(", target, ",", class, ")") %in% namespace)
      expect_false(paste0("S3method(", generic, ",", class, ")") %in% namespace)
      expect_false(exists(paste(target, class, sep = "."), envir = ns, inherits = FALSE))
    }
  }
})

test_that("qualified S3 methods dispatch in either namespace load order", {
  skip_if_not_installed("callr")
  skip_if_not_installed("tibble")
  for (generic_first in c(FALSE, TRUE)) {
    result <- callr::r(function(generic_first) {
      if (generic_first) loadNamespace("tibble")
      library(miniextendr)
      loaded_on_attach <- "tibble" %in% loadedNamespaces()
      columns <- list(a = 1:2, b = c("u", "v"))
      rows <- tibble::as_tibble(structure(columns, class = "mx_qualified_rows"))
      explicit <- tibble::as_tibble(structure(columns, class = "mx_qualified_export"))
      pointer <- new_qualified_rows(7L)
      list(
        loaded = loaded_on_attach,
        rows = rows,
        explicit = explicit,
        pointer = tibble::as_tibble(pointer),
        trait = tibble::tbl_sum(pointer),
        bare = format(pointer),
        registered = identical(
          getS3method("as_tibble", "qualified_rows", envir = asNamespace("tibble")),
          get("as_tibble.qualified_rows", asNamespace("miniextendr"))
        )
      )
    }, args = list(generic_first), libpath = .libPaths())
    expect_identical(result$loaded, generic_first)
    expect_identical(result$rows, data.frame(a = 1:2, b = c("u", "v")))
    expect_identical(result$explicit, result$rows)
    expect_identical(result$pointer, data.frame(value = 7L))
    expect_identical(result$trait, "value=7")
    expect_identical(result$bare, "qualified=7")
    expect_true(result$registered)
  }
})

test_that("qualified vctrs double dispatch retains both class names", {
  skip_if_not_installed("vctrs")
  x <- structure(integer(), class = "mx_qualified_vec")
  expect_identical(vctrs::vec_ptype2(x, x), x)
  expect_identical(
    getS3method("vec_ptype2", "mx_qualified_vec.mx_qualified_vec", envir = asNamespace("vctrs")),
    get("vec_ptype2.mx_qualified_vec.mx_qualified_vec", asNamespace("miniextendr"))
  )
})
