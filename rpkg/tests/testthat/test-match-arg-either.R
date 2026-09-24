# A choice or another value: `match_arg` / `choices` on `Either<T, R>`. The
# formal is T's choice vector; character or factor input is matched and reaches
# Rust as `Left(T)`, anything else goes to the `R` arm unchanged.

skip_if_not(exists("match_arg_either_route"), "either feature not compiled in")

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

routes <- c("oral", "bolus", "infusion")
doses <- data.frame(time = c(0, 12), amt = c(100, 50))

test_that("Either<T, DataFrame>: choices formal, character and factor go Left, a data frame goes Right", {
  expect_equal(eval(formals(match_arg_either_route)$route), routes)
  expect_equal(match_arg_either_route(), "Oral")
  expect_equal(match_arg_either_route("inf"), "Infusion")
  expect_equal(match_arg_either_route(factor("bolus")), "Bolus")
  expect_equal(match_arg_either_route(doses), "frame:2x2")
  expect_choice_error(match_arg_either_route("iv"), "route", routes)
})

test_that("Either<T, DataFrame>: input that is neither fails in the R arm", {
  # NULL is not a choice: it goes to `R`, which rejects it.
  expect_error(match_arg_either_route(NULL), "route")
  expect_error(match_arg_either_route(1:3), "route")
})

test_that("Option<Either<T, R>>: NULL formal, NULL is None", {
  expect_null(formals(match_arg_either_route_optional)$maybe_route)
  expect_equal(match_arg_either_route_optional(), "none")
  expect_equal(match_arg_either_route_optional(NULL), "none")
  expect_equal(match_arg_either_route_optional("bo"), "Bolus")
  expect_equal(match_arg_either_route_optional(doses), "frame:2x2")
  expect_choice_error(match_arg_either_route_optional("iv"), "maybe_route", routes)
})

test_that("Missing<Either<T, R>> and Missing<Option<Either<T, R>>> report omission", {
  expect_equal(eval(formals(match_arg_either_route_omitted)$route), routes)
  expect_equal(match_arg_either_route_omitted(), "absent")
  expect_equal(match_arg_either_route_omitted("oral"), "Oral")
  expect_equal(match_arg_either_route_omitted(doses), "frame:2x2")
  expect_choice_error(match_arg_either_route_omitted("iv"), "route", routes)

  expect_equal(eval(formals(match_arg_either_route_omitted_optional)$route), routes)
  expect_equal(match_arg_either_route_omitted_optional(), "absent")
  expect_equal(match_arg_either_route_omitted_optional(NULL), "null")
  expect_equal(match_arg_either_route_omitted_optional("inf"), "Infusion")
  expect_equal(match_arg_either_route_omitted_optional(doses), "frame:2x2")
})

test_that("choices() on Either<String, f64>", {
  levels <- c("low", "mid", "high")
  expect_equal(eval(formals(choices_either_level)$level), levels)
  expect_equal(choices_either_level(), "level:low")
  expect_equal(choices_either_level("hi"), "level:high")
  expect_equal(choices_either_level(2.5), "number:2.5")
  expect_choice_error(choices_either_level("max"), "level", levels)
})

test_that("R6 method with an Either<T, R> match_arg parameter", {
  planner <- EitherRoutePlanner$new()
  expect_equal(eval(formals(planner$plan)$route), routes)
  expect_equal(planner$plan(), "Oral")
  expect_equal(planner$plan("bol"), "Bolus")
  expect_equal(planner$plan(doses), "frame:2x2")
  expect_choice_error(planner$plan("iv"), "route", routes)
})

test_that("trait method with choices() on Either<String, f64>", {
  planner <- EitherRoutePlanner$new()
  expect_equal(eval(formals(EitherRoutePlanner$RouteLevel$level)$level), c("low", "mid", "high"))
  expect_equal(EitherRoutePlanner$RouteLevel$level(planner), "level:low")
  expect_equal(EitherRoutePlanner$RouteLevel$level(planner, level = "mi"), "level:mid")
  expect_equal(EitherRoutePlanner$RouteLevel$level(planner, level = 4), "number:4")
  expect_choice_error(
    EitherRoutePlanner$RouteLevel$level(planner, level = "max"),
    "level",
    c("low", "mid", "high")
  )
})

test_that("the auto-generated @param line names the other kind", {
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
  page <- rd_text("match_arg_either_route")
  expect_match(page, "\"infusion\", a data frame, or NULL for no choice.", fixed = TRUE)
  expect_match(page, "\"high\", or a number.", fixed = TRUE)
  expect_match(rd_text("EitherRoutePlanner"), "\"infusion\", or a data frame.", fixed = TRUE)
})
