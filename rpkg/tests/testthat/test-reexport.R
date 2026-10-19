# reexport!(pkg::name) (#1910), from rpkg/src/rust/reexport_tests.rs:
# vctrs's vec_ptype_abbr(), the generic of `vec_ptype_abbr.percent`
# (vctrs_class_example.rs), declared by two invocations, and vctrs's `%0%`,
# a name written as a string literal.

reexported <- c("vec_ptype_abbr", "%0%")

test_that("re-exported names are the functions of the other package", {
  expect_true(all(reexported %in% getNamespaceExports("miniextendr")))
  expect_identical(miniextendr::vec_ptype_abbr, vctrs::vec_ptype_abbr)
  expect_identical(miniextendr::`%0%`, vctrs::`%0%`)
})

test_that("NAMESPACE imports and exports each re-exported name once", {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  ns <- parseNamespaceFile(basename(pkg_path), dirname(pkg_path))
  for (name in reexported) {
    expect_identical(sum(ns$exports == name), 1L, info = name)
  }
  vctrs_imports <- unlist(lapply(ns$imports, function(import) {
    if (is.list(import) && identical(import[[1L]], "vctrs")) import[[2L]]
  }))
  expect_true(all(reexported %in% vctrs_imports))
})

test_that("man/reexports.Rd lists the re-exported names", {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  rd <- db[["reexports.Rd"]]
  expect_false(is.null(rd))
  is_alias <- vapply(rd, function(x) identical(attr(x, "Rd_tag"), "\\alias"), logical(1))
  expect_true(all(reexported %in% unlist(rd[is_alias])))
})

test_that("the installed package documents the re-exported names", {
  skip_if_not_installed("callr")
  undocumented <- callr::r(function() {
    unlist(unclass(tools::undoc(package = "miniextendr")), use.names = FALSE)
  }, libpath = .libPaths())
  expect_false(any(reexported %in% undocumented))
})

# A fresh session: the test session may have vctrs attached or the namespace
# loaded some other way.
test_that("with only the package attached, the bare generic dispatches to its method", {
  skip_if_not_installed("callr")
  res <- callr::r(function() {
    library(miniextendr)
    list(
      vctrs_attached = "package:vctrs" %in% search(),
      found = find("vec_ptype_abbr"),
      abbr = vec_ptype_abbr(new_percent(0.5)),
      operator = NULL %0% "fallback"
    )
  }, libpath = .libPaths())
  expect_false(res$vctrs_attached)
  expect_identical(res$found, "package:miniextendr")
  expect_identical(res$abbr, "%")
  expect_identical(res$operator, "fallback")
})
