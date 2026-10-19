# A `default = "..."` on a `Missing<T>` parameter (#2161): the formal shows
# the default, an omitted argument reaches Rust as `Missing::Absent`, and a
# supplied value is always `Present`, even one identical to the default. The
# default is never evaluated.

test_that("Missing<f64> with a default: formal shows it, omitted is Absent", {
  expect_identical(formals(missing_default_tol)$tol, 1e-8)
  expect_equal(missing_default_tol(), "absent")
  expect_equal(missing_default_tol(0.5), "0.5")
  # The caller supplied it, so it is Present, as `missing()` says in R.
  expect_equal(missing_default_tol(1e-8), "0.00000001")
  expect_equal(missing_default_tol(tol = 1e-8), "0.00000001")
})

test_that("Missing<Option<T>> with a NULL default: omitted is Absent, NULL is Present(None)", {
  expect_null(formals(missing_default_weights)$weights)
  expect_true("weights" %in% names(formals(missing_default_weights)))
  expect_equal(missing_default_weights(), "absent")
  expect_equal(missing_default_weights(c(1, 2, 3)), "n=3")
  expect_equal(missing_default_weights(NULL), "null")
  expect_equal(missing_default_weights(weights = NULL), "null")
})

test_that("the default is never evaluated, nor checked", {
  expect_identical(
    formals(missing_default_unforced)$x,
    quote(stop("the default was evaluated"))
  )
  expect_equal(missing_default_unforced(), "absent")
  expect_equal(missing_default_unforced(2), "2")
  expect_error(missing_default_unforced(NA_real_), class = "rust_error")
})

test_that("a caller passes omission through, and its own default as a value", {
  forward <- function(tol) missing_default_tol(tol)
  expect_equal(forward(), "absent")
  expect_equal(forward(0.25), "0.25")
  via_dots <- function(...) missing_default_tol(...)
  expect_equal(via_dots(), "absent")
  # A caller's default is a value it hands over, even the wrapper's own.
  same_default <- function(tol = 1e-8) missing_default_tol(tol)
  expect_equal(same_default(), "0.00000001")
  forward_weights <- function(weights) missing_default_weights(weights)
  expect_equal(forward_weights(), "absent")
  expect_equal(forward_weights(NULL), "null")
})

test_that("S4 method with a Missing<f64> default", {
  h <- MissingDefaultS4()
  method <- methods::unRematchDefinition(methods::getMethod("s4_tol", "MissingDefaultS4"))
  expect_identical(formals(method)$tol, 1e-8)
  expect_equal(s4_tol(h), "absent")
  expect_equal(s4_tol(h, 0.5), "0.5")
  expect_equal(s4_tol(h, 1e-8), "0.00000001")
})

test_that("R6 method and R6 trait method with a Missing<f64> default", {
  obj <- MissingDefaultR6$new()
  expect_identical(formals(obj$tol)$tol, 1e-8)
  expect_equal(obj$tol(), "absent")
  expect_equal(obj$tol(0.5), "0.5")
  expect_equal(obj$tol(1e-8), "0.00000001")

  trait_tol <- MissingDefaultR6$MissingDefaultTol$trait_tol
  expect_identical(formals(trait_tol)$tol, 1e-8)
  expect_equal(trait_tol(obj), "absent")
  expect_equal(trait_tol(obj, 0.5), "0.5")
  expect_equal(trait_tol(obj, 1e-8), "0.00000001")
})

test_that("the usage line shows the default and the @param line the omission note", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  # The Rd page that documents `topic`.
  rd_text <- function(topic) {
    pages <- vapply(rd_db, function(rd) {
      gsub("\\s+", " ", paste(utils::capture.output(print(rd)), collapse = " "))
    }, character(1))
    page <- pages[grepl(paste0("\\alias{", topic, "}"), pages, fixed = TRUE)]
    expect_length(page, 1L)
    page[[1L]]
  }
  note <- "Omitting the argument means the function's own handling."

  tol_page <- rd_text("missing_default_tol")
  expect_match(tol_page, "missing_default_tol(tol = 1e-08)", fixed = TRUE)
  expect_match(tol_page, paste0("\\item{tol}{", note, "}"), fixed = TRUE)

  weights_page <- rd_text("missing_default_weights")
  expect_match(weights_page, "missing_default_weights(weights = NULL)", fixed = TRUE)
  expect_match(weights_page, paste0("\\item{weights}{", note, "}"), fixed = TRUE)

  # An author's @param text stands as written.
  unforced_page <- rd_text("missing_default_unforced")
  expect_match(
    unforced_page,
    "missing_default_unforced(x = stop(\"the default was evaluated\"))",
    fixed = TRUE
  )
  expect_match(unforced_page, "\\item{x}{A number, not NA.}", fixed = TRUE)
  expect_no_match(unforced_page, note, fixed = TRUE)

  # The R6 class page documents both methods' arguments.
  r6_page <- rd_text("MissingDefaultR6")
  expect_match(r6_page, "tol = 1e-08", fixed = TRUE)
  expect_match(r6_page, note, fixed = TRUE)
})
