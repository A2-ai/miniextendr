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

# region: S3 / S4 / S7 / vctrs methods and trait methods

levels <- c("low", "mid", "high")

# The `plan*` fixtures report `route=<..>;level=<..>` for a match_arg
# `Either<Route, DataFrame>` and an omittable `choices(...)`
# `Missing<Either<String, f64>>`. `f` forwards its arguments through `...`,
# which keeps an omitted argument missing.
expect_route_level <- function(f) {
  expect_equal(f(), "route=Oral;level=absent")
  expect_equal(f("inf"), "route=Infusion;level=absent")
  expect_equal(f(factor("bolus")), "route=Bolus;level=absent")
  expect_equal(f(doses), "route=frame:2x2;level=absent")
  expect_equal(f(level = "mi"), "route=Oral;level=level:mid")
  expect_equal(f(level = factor("high")), "route=Oral;level=level:high")
  expect_equal(f(doses, level = 2.5), "route=frame:2x2;level=number:2.5")
  expect_choice_error(f("iv"), "route", routes)
  expect_choice_error(f(level = "max"), "level", levels)
  # NULL is not a choice: it goes to the `DataFrame` arm, which rejects it.
  expect_error(f(NULL), "route")
}

# The formals keep both choice vectors.
expect_route_level_formals <- function(fn) {
  fmls <- formals(fn)
  expect_equal(eval(fmls$route), routes)
  expect_equal(eval(fmls$level), levels)
}

test_that("S3 method with Either choice parameters", {
  p <- new_eitherroutes3()
  expect_route_level_formals(getS3method("plan_s3", "EitherRouteS3"))
  expect_route_level(function(...) plan_s3(p, ...))
})

test_that("S4 method with Either choice parameters", {
  h <- EitherRouteS4()
  expect_equal(names(formals(s4_plan)), c("x", "..."))
  expect_route_level_formals(
    methods::unRematchDefinition(methods::getMethod("s4_plan", "EitherRouteS4"))
  )
  expect_route_level(function(...) s4_plan(h, ...))
})

test_that("S7 constructor, method and shortcut with Either choice parameters", {
  expect_equal(eval(formals(EitherRouteS7)$start), routes)
  expect_equal(route_given(EitherRouteS7()), "absent")
  expect_equal(route_given(EitherRouteS7(start = "inf")), "Infusion")
  expect_equal(route_given(EitherRouteS7(start = factor("oral"))), "Oral")
  expect_equal(route_given(EitherRouteS7(start = doses)), "frame:2x2")
  expect_choice_error(EitherRouteS7(start = "iv"), "start", routes)

  h <- EitherRouteS7()
  expect_route_level_formals(S7::method(plan_s7, EitherRouteS7))
  expect_route_level_formals(EitherRouteS7_plan_s7)
  expect_route_level(function(...) plan_s7(h, ...))
  expect_route_level(function(...) EitherRouteS7_plan_s7(h, ...))
})

test_that("vctrs constructor and static method with Either choice parameters", {
  expect_equal(eval(formals(new_eitherroutevctrs)$route), routes)
  expect_equal(vctrs::vec_data(new_eitherroutevctrs()), 1)
  expect_equal(vctrs::vec_data(new_eitherroutevctrs("bo")), 2)
  expect_equal(vctrs::vec_data(new_eitherroutevctrs(factor("infusion"))), 3)
  expect_equal(vctrs::vec_data(new_eitherroutevctrs(doses)), c(0, 0))
  expect_choice_error(new_eitherroutevctrs("iv"), "route", routes)

  expect_route_level_formals(eitherroutevctrs_plan)
  expect_route_level(eitherroutevctrs_plan)
})

# The `EitherGrade` trait method reports `absent` or the grade (`level:<name>`
# / `number:<n>`) for an omittable `choices(...)` `Either<String, f64>`.
expect_either_grade <- function(grade) {
  expect_equal(grade(), "absent")
  expect_equal(grade(grade = "hi"), "level:high")
  expect_equal(grade(grade = 4), "number:4")
  expect_choice_error(grade(grade = "max"), "grade", levels)
}

test_that("S3 trait method with an omittable Either choice", {
  p <- new_eitherroutes3()
  expect_equal(eval(formals(getS3method("either_grade", "EitherRouteS3"))$grade), levels)
  expect_either_grade(function(...) either_grade(p, ...))
})

test_that("S7 trait method and its shortcut with an omittable Either choice", {
  h <- EitherRouteS7()
  expect_equal(
    eval(formals(S7::method(s7_trait_EitherGrade_either_grade, EitherRouteS7))$grade),
    levels
  )
  expect_equal(eval(formals(EitherRouteS7_either_grade)$grade), levels)
  expect_either_grade(function(...) s7_trait_EitherGrade_either_grade(h, ...))
  expect_either_grade(function(...) EitherRouteS7_either_grade(h, ...))
})

test_that("method Either choice params name the other kind in their @param line", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_text <- function(topic) {
    pages <- vapply(rd_db, function(rd) {
      gsub("\\s+", " ", paste(utils::capture.output(print(rd)), collapse = " "))
    }, character(1))
    page <- pages[grepl(paste0("\\alias{", topic, "}"), pages, fixed = TRUE)]
    expect_length(page, 1L)
    page[[1L]]
  }
  level_or_number <- "One of \"low\", \"mid\", \"high\", or a number; omitting the argument means no choice."
  for (topic in c("EitherRouteS3", "EitherRouteS7", "EitherRouteVctrs")) {
    expect_match(rd_text(topic), level_or_number, fixed = TRUE)
  }
  # The S7 constructor's `start` (inlined in `new_class()`).
  expect_match(
    rd_text("EitherRouteS7"),
    "\"infusion\", or a data frame; omitting the argument means no choice.",
    fixed = TRUE
  )
})

# endregion
