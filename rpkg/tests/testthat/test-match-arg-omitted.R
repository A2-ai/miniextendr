# Omitted choice: a `Missing<..>` choice parameter keeps the choice vector as
# its formal and reaches Rust as `Missing::Absent` when omitted (#1551).

# A choice failure is the argument error of #1591 (`rust_error`,
# `kind = "conversion"`, `e$param`), its message names the argument and lists
# the choices. Only those stable parts are asserted; the exact wording belongs
# to the R-side helpers.
expect_choice_error <- function(expr, param, choices) {
  e <- tryCatch(expr, error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, param)
  msg <- conditionMessage(e)
  expect_true(grepl(param, msg, fixed = TRUE), info = msg)
  for (choice in choices) {
    expect_true(grepl(choice, msg, fixed = TRUE), info = msg)
  }
}

modes <- c("Fast", "Safe", "Debug")

test_that("Missing<Option<T>> match_arg: choices formal, omitted is Absent, NULL is Present(None)", {
  expect_equal(eval(formals(match_arg_omitted_mode)$mode), modes)
  expect_equal(match_arg_omitted_mode(), "absent")
  expect_equal(match_arg_omitted_mode(NULL), "null")
  expect_equal(match_arg_omitted_mode("Sa"), "Safe")
  expect_equal(match_arg_omitted_mode("Debug"), "Debug")
  expect_equal(match_arg_omitted_mode(factor("Fast")), "Fast")
  # The formal default is documentation only: passing it is a supplied value.
  expect_equal(match_arg_omitted_mode(modes), "Fast")
  expect_choice_error(match_arg_omitted_mode("nope"), "mode", modes)
})

test_that("Missing<T> match_arg: NULL still selects the first choice", {
  expect_equal(eval(formals(match_arg_omitted_plain)$mode), modes)
  expect_equal(match_arg_omitted_plain(), "absent")
  expect_equal(match_arg_omitted_plain(NULL), "Fast")
  expect_equal(match_arg_omitted_plain("Sa"), "Safe")
  expect_choice_error(match_arg_omitted_plain("nope"), "mode", modes)
})

test_that("Missing<Vec<T>> several_ok: omitted is Absent, NULL selects every choice", {
  expect_equal(eval(formals(match_arg_omitted_modes)$modes), modes)
  expect_equal(match_arg_omitted_modes(), "absent")
  expect_equal(match_arg_omitted_modes(NULL), "Fast, Safe, Debug")
  expect_equal(match_arg_omitted_modes(c("Fa", "De")), "Fast, Debug")
  expect_choice_error(match_arg_omitted_modes(c("Fast", "zzz")), "modes", modes)
  expect_equal(match_arg_omitted_modes_boxed(), "absent")
  expect_equal(match_arg_omitted_modes_boxed(NULL), "3 modes")
  expect_equal(match_arg_omitted_modes_boxed("Safe"), "1 modes")
})

test_that("choices() on Missing<Option<String>> / Missing<Vec<String>>", {
  colors <- c("red", "green", "blue")
  expect_equal(eval(formals(choices_omitted_color)$color), colors)
  expect_equal(choices_omitted_color(), "absent")
  expect_equal(choices_omitted_color(NULL), "null")
  expect_equal(choices_omitted_color("gr"), "green")
  expect_choice_error(choices_omitted_color("purple"), "color", colors)
  expect_equal(eval(formals(choices_omitted_colors)$colors), colors)
  expect_equal(choices_omitted_colors(), "absent")
  expect_equal(choices_omitted_colors(NULL), "red, green, blue")
  expect_equal(choices_omitted_colors(c("re", "bl")), "red, blue")
})

test_that("omission passes through a caller only without a default", {
  forward <- function(mode) match_arg_omitted_mode(mode)
  expect_equal(forward(), "absent")
  expect_equal(forward("Sa"), "Safe")
  via_dots <- function(...) match_arg_omitted_mode(...)
  expect_equal(via_dots(), "absent")
  expect_equal(via_dots(mode = NULL), "null")
  # A caller formal with a default hands the default over as a value.
  with_default <- function(mode = c("Fast", "Safe", "Debug")) match_arg_omitted_mode(mode)
  expect_equal(with_default(), "Fast")
})

test_that("R6 method with Missing<Option<T>> match_arg", {
  obj <- R6MatchArgCounter$new("Safe")
  expect_equal(eval(formals(obj$peek)$mode), modes)
  expect_equal(obj$peek(), "current:Safe")
  expect_equal(obj$peek(NULL), "null")
  expect_equal(obj$peek("De"), "Debug")
  expect_choice_error(obj$peek("nope"), "mode", modes)
})

test_that("call = caller: the omission guard keeps the caller attribution (#1548)", {
  expect_equal(miniextendr:::call_attr_omitted(), "absent")
  expect_equal(miniextendr:::call_attr_omitted(NULL), "null")
  expect_equal(miniextendr:::call_attr_omitted("Sa"), "Safe")
  e <- tryCatch(miniextendr:::call_attr_omitted(mode = "bogus"), error = identity)
  expect_s3_class(e, "error")
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_omitted(mode = "bogus")))
  expect_match(conditionMessage(e), "mode", fixed = TRUE)
})

test_that("the auto-generated @param line says that omitting means no choice", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  # The Rd page that documents `topic` (fixtures share a page per source file).
  rd_text <- function(topic) {
    pages <- vapply(rd_db, function(rd) {
      gsub("\\s+", " ", paste(utils::capture.output(print(rd)), collapse = " "))
    }, character(1))
    page <- pages[grepl(paste0("\\alias{", topic, "}"), pages, fixed = TRUE)]
    expect_length(page, 1L)
    page[[1L]]
  }
  # The standalone fixtures share one page, where each parameter name is
  # listed once; the R6 method has its own argument list.
  page <- rd_text("match_arg_omitted_plain")
  expect_match(page, "\"Debug\"; omitting the argument means no choice.", fixed = TRUE)
  expect_match(
    page,
    "One or more of \"Fast\", \"Safe\", \"Debug\"; omitting the argument means no choice.",
    fixed = TRUE
  )
  expect_match(page, "\"blue\", or NULL; omitting the argument means no choice.", fixed = TRUE)
  expect_match(
    rd_text("R6MatchArgCounter"),
    "\"Debug\", or NULL; omitting the argument means no choice.",
    fixed = TRUE
  )
})
