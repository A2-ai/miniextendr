# Doc gating for `#[miniextendr(vctrs(...), noexport)]` / `@noRd` vctrs classes (#1180).
#
# The vctrs class generator historically never gated its doc-emitting blocks on
# `@noRd`/`internal`/`noexport` — only the `@export` line was suppressed — so a
# gated class still got a fully-aliased Rd page for every self-coercion method
# (`vec_ptype2` / `vec_cast` / `vec_ptype_abbr`) and static helper. It now folds
# `@noRd || (noexport && !internal)` into a `class_has_no_rd` gate like the other
# five class-system generators. S3method() dispatch registration is kept for the
# S3-method-shaped emissions (it is NAMESPACE plumbing, not export()/Rd surface;
# vctrs generics dispatch from the vctrs namespace and roxygen2 warns on
# recognized-but-unregistered S3 methods).
#
# Fixtures: `VctrsNoexportGated` / `VctrsNoRdGated` in
# rpkg/src/rust/export_control_tests.rs.

# Render every Rd page once per call; grepping the rendered text catches
# aliases, usage entries, and prose alike (same pattern as
# test-export-control.R / test-r6-noexport-field.R).
rd_db_texts <- function() {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  if (is.null(rd_db)) {
    return(NULL)
  }
  vapply(
    rd_db,
    function(rd) paste(capture.output(print(rd)), collapse = "\n"),
    character(1)
  )
}

# region: runtime behaviour — gated classes stay callable and functional

test_that("noexport vctrs class is constructible and callable via :::", {
  x <- miniextendr:::new_vctrsnoexportgated(c(1, 2, 3))
  expect_s3_class(x, "VctrsNoexportGated")
  expect_true(vctrs::vec_is(x))
  expect_equal(miniextendr:::vctrsnoexportgated_payload_sum(c(1, 2, 3)), 6)
})

test_that("noexport vctrs class self-coercion dispatch still works", {
  # S3method() registration must survive the doc gate — without it the class
  # is not a functioning vctr (vec_c dispatches vec_ptype2/vec_cast from the
  # vctrs namespace).
  x <- miniextendr:::new_vctrsnoexportgated(c(1, 2))
  y <- miniextendr:::new_vctrsnoexportgated(c(3))
  combined <- vctrs::vec_c(x, y)
  expect_s3_class(combined, "VctrsNoexportGated")
  expect_equal(length(combined), 3)
  expect_equal(vctrs::vec_ptype_abbr(x), "nxg")
})

test_that("@noRd vctrs class is constructible via :::", {
  x <- miniextendr:::new_vctrsnordgated(c(4, 5))
  expect_s3_class(x, "VctrsNoRdGated")
  expect_true(vctrs::vec_is(x))
})

# endregion

# region: NAMESPACE — no export(), but S3method() dispatch registration kept

test_that("noexport vctrs class has no export() entries but keeps S3method()", {
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(
    any(grepl("vctrsnoexportgated", export_lines, ignore.case = TRUE)),
    info = paste(
      "noexport vctrs class must have no export() entries. Found:",
      paste(grep("vctrsnoexportgated", export_lines, ignore.case = TRUE, value = TRUE),
        collapse = "; "
      )
    )
  )
  expect_true(
    "S3method(vec_ptype2,VctrsNoexportGated.VctrsNoexportGated)" %in% ns,
    info = "self-coercion S3method() registration must survive the doc gate"
  )
  expect_true("S3method(vec_cast,VctrsNoexportGated.VctrsNoexportGated)" %in% ns)
  expect_true("S3method(vec_ptype_abbr,VctrsNoexportGated)" %in% ns)
})

test_that("@noRd vctrs class has no export() entries but keeps S3method()", {
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(
    any(grepl("vctrsnordgated", export_lines, ignore.case = TRUE)),
    info = paste(
      "@noRd vctrs class must have no export() entries. Found:",
      paste(grep("vctrsnordgated", export_lines, ignore.case = TRUE, value = TRUE),
        collapse = "; "
      )
    )
  )
  expect_true("S3method(vec_ptype2,VctrsNoRdGated.VctrsNoRdGated)" %in% ns)
})

# endregion

# region: Rd pages — no alias/usage contribution anywhere for gated methods

test_that("noexport vctrs class methods appear on no Rd page", {
  rd_texts <- rd_db_texts()
  skip_if(is.null(rd_texts), "tools::Rd_db('miniextendr') unavailable — package not installed")

  # Method/helper names must not appear on ANY page — before the fix they
  # landed as aliases + \method usage entries on the class's shared page.
  for (needle in c(
    "vec_ptype_abbr.VctrsNoexportGated",
    "vec_ptype2.VctrsNoexportGated",
    "vec_cast.VctrsNoexportGated",
    "vctrsnoexportgated_payload_sum"
  )) {
    hits <- grepl(needle, rd_texts, fixed = TRUE)
    expect_false(
      any(hits),
      info = paste0(
        "`", needle, "` must not appear in any rendered Rd page. Found in: ",
        paste(names(rd_texts)[hits], collapse = ", ")
      )
    )
  }
})

test_that("@noRd vctrs class appears on no Rd page at all", {
  rd_texts <- rd_db_texts()
  skip_if(is.null(rd_texts), "tools::Rd_db('miniextendr') unavailable — package not installed")

  hits <- grepl("vctrsnordgated", rd_texts, ignore.case = TRUE)
  expect_false(
    any(hits),
    info = paste(
      "@noRd vctrs class (constructor included) must not appear in any rendered",
      "Rd page. Found in:", paste(names(rd_texts)[hits], collapse = ", ")
    )
  )
})

test_that("ungated vctrs class methods still land on its Rd page (positive control)", {
  # Guards the assertions above against passing vacuously: an exported vctrs
  # class (VctrsMatchArgScale, rpkg/src/rust/match_arg_impl_tests.rs) must keep
  # its self-coercion aliases.
  rd_texts <- rd_db_texts()
  skip_if(is.null(rd_texts), "tools::Rd_db('miniextendr') unavailable — package not installed")

  expect_true(
    any(grepl("vec_ptype2.VctrsMatchArgScale", rd_texts, fixed = TRUE)),
    info = "exported vctrs class must keep vec_ptype2 alias/usage on its Rd page"
  )
  expect_true(
    any(grepl("vec_cast.VctrsMatchArgScale", rd_texts, fixed = TRUE)),
    info = "exported vctrs class must keep vec_cast alias/usage on its Rd page"
  )
})

# endregion
