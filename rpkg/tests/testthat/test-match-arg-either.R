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

test_that("Either<T, List>: a choice goes Left, a list goes Right", {
  expect_equal(eval(formals(match_arg_either_route_or_list)$route), routes)
  expect_equal(match_arg_either_route_or_list(), "Oral")
  expect_equal(match_arg_either_route_or_list("bol"), "Bolus")
  expect_equal(match_arg_either_route_or_list(list(100, 50, 25)), "list:3")
  expect_choice_error(match_arg_either_route_or_list("iv"), "route", routes)
  # The list arm's own refusal reaches the argument error.
  expect_error(match_arg_either_route_or_list(list(a = 1, a = 2)), "duplicate name 'a'")
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

# region: `several_ok` choice lists with another kind of value (#1612)
#
# `Either<Vec<T>, R>` / `Either<Box<[T]>, R>`: character or factor input is
# matched element by element and reaches Rust as `Left(Vec<T>)`, an omitted
# argument selects every choice, and anything else (NULL included) goes to the
# `R` arm. The fixtures report routes as `"Oral,Bolus"` and a data frame as
# `"frame:<rows>x<cols>"`.

all_routes <- "Oral,Bolus,Infusion"

test_that("several_ok Either<Vec<T>, DataFrame>: choices formal, character and factor go Left, a data frame goes Right", {
  expect_equal(eval(formals(match_arg_either_routes)$routes), routes)
  expect_equal(match_arg_either_routes(), all_routes)
  expect_equal(match_arg_either_routes(routes), all_routes)
  expect_equal(match_arg_either_routes("inf"), "Infusion")
  expect_equal(match_arg_either_routes(c("inf", "or")), "Infusion,Oral")
  expect_equal(match_arg_either_routes(c("oral", "oral")), "Oral,Oral")
  expect_equal(match_arg_either_routes(factor(c("bolus", "oral"))), "Bolus,Oral")
  expect_equal(match_arg_either_routes(doses), "frame:2x2")
  expect_choice_error(match_arg_either_routes(c("oral", "iv")), "routes", routes)
  expect_match(
    conditionMessage(tryCatch(match_arg_either_routes(c("oral", "iv")), error = identity)),
    "element 2",
    fixed = TRUE
  )
})

test_that("several_ok Either<Vec<T>, DataFrame>: empty, NA, NULL and other input", {
  # An empty character vector is a choice list with no element, not the `R`
  # arm: the several-choice helper refuses it.
  e <- tryCatch(match_arg_either_routes(character(0)), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "routes")
  expect_match(conditionMessage(e), "length", fixed = TRUE)
  expect_choice_error(match_arg_either_routes(NA_character_), "routes", routes)
  # NULL is not a choice under `Either`: it goes to `R`, which rejects it.
  expect_error(match_arg_either_routes(NULL), "routes")
  expect_error(match_arg_either_routes(1:3), "routes")
})

test_that("several_ok Either<Box<[T]>, DataFrame>", {
  expect_equal(eval(formals(match_arg_either_routes_boxed)$route_set), routes)
  expect_equal(match_arg_either_routes_boxed(), all_routes)
  expect_equal(match_arg_either_routes_boxed(c("bo", "inf")), "Bolus,Infusion")
  expect_equal(match_arg_either_routes_boxed(factor("oral")), "Oral")
  expect_equal(match_arg_either_routes_boxed(doses), "frame:2x2")
  expect_choice_error(match_arg_either_routes_boxed("iv"), "route_set", routes)
})

test_that("several_ok Missing<Either<Vec<T>, R>> reports omission", {
  expect_equal(eval(formals(match_arg_either_routes_omitted)$route_list), routes)
  expect_equal(match_arg_either_routes_omitted(), "absent")
  expect_equal(match_arg_either_routes_omitted(c("inf", "or")), "Infusion,Oral")
  expect_equal(match_arg_either_routes_omitted(doses), "frame:2x2")
  expect_choice_error(match_arg_either_routes_omitted("iv"), "route_list", routes)
  # An explicit NULL is present and goes to `R`.
  expect_error(match_arg_either_routes_omitted(NULL), "route_list")
})

test_that("several_ok Option<Either<Vec<T>, R>>: NULL formal, NULL is None", {
  expect_null(formals(match_arg_either_routes_optional)$maybe_routes)
  expect_equal(match_arg_either_routes_optional(), "none")
  expect_equal(match_arg_either_routes_optional(NULL), "none")
  expect_equal(match_arg_either_routes_optional(c("bo", "inf")), "Bolus,Infusion")
  expect_equal(match_arg_either_routes_optional(doses), "frame:2x2")
  expect_choice_error(match_arg_either_routes_optional("iv"), "maybe_routes", routes)
})

test_that("several_ok Missing<Option<Either<Vec<T>, R>>>: omitted is Absent, NULL is Present(None)", {
  expect_equal(eval(formals(match_arg_either_routes_omitted_optional)$route_pick), routes)
  expect_equal(match_arg_either_routes_omitted_optional(), "absent")
  expect_equal(match_arg_either_routes_omitted_optional(NULL), "null")
  expect_equal(match_arg_either_routes_omitted_optional("inf"), "Infusion")
  expect_equal(match_arg_either_routes_omitted_optional(doses), "frame:2x2")
  expect_choice_error(match_arg_either_routes_omitted_optional("iv"), "route_pick", routes)
})

test_that("choices(), several_ok on Either<Vec<String>, f64>", {
  expect_equal(eval(formals(choices_either_tiers)$tiers), levels)
  expect_equal(choices_either_tiers(), "tiers:low,mid,high")
  expect_equal(choices_either_tiers(c("hi", "lo")), "tiers:high,low")
  expect_equal(choices_either_tiers(factor("mid")), "tiers:mid")
  expect_equal(choices_either_tiers(2.5), "number:2.5")
  expect_choice_error(choices_either_tiers("max"), "tiers", levels)
})

# The `plan*` fixtures below report `routes=<..>;tiers=<..>` for a
# `match_arg_several_ok` `Either<Vec<Route>, DataFrame>` and an omittable
# `choices_several_ok` `Missing<Either<Vec<String>, f64>>`. `f` forwards its
# arguments through `...`, which keeps an omitted argument missing.
expect_routes_tiers <- function(f) {
  expect_equal(f(), paste0("routes=", all_routes, ";tiers=absent"))
  expect_equal(f(c("inf", "or")), "routes=Infusion,Oral;tiers=absent")
  expect_equal(f(factor(c("bolus", "oral"))), "routes=Bolus,Oral;tiers=absent")
  expect_equal(f(doses), "routes=frame:2x2;tiers=absent")
  expect_equal(f(tiers = c("hi", "lo")), paste0("routes=", all_routes, ";tiers=tiers:high,low"))
  expect_equal(f(tiers = factor("mid")), paste0("routes=", all_routes, ";tiers=tiers:mid"))
  expect_equal(f(doses, tiers = 2.5), "routes=frame:2x2;tiers=number:2.5")
  expect_choice_error(f(c("oral", "iv")), "routes", routes)
  expect_choice_error(f(tiers = "max"), "tiers", levels)
  # NULL is not a choice: it goes to the `DataFrame` arm, which rejects it.
  expect_error(f(NULL), "routes")
}

# The formals keep both choice vectors.
expect_routes_tiers_formals <- function(fn) {
  fmls <- formals(fn)
  expect_equal(eval(fmls$routes), routes)
  expect_equal(eval(fmls$tiers), levels)
}

test_that("env method with several_ok Either choice lists", {
  e <- EitherRoutesEnv$new()
  expect_routes_tiers_formals(EitherRoutesEnv$plan)
  expect_routes_tiers(function(...) e$plan(...))
})

test_that("R6 method and trait method with several_ok Either choice lists", {
  p <- EitherRoutesR6$new()
  expect_routes_tiers_formals(p$plan)
  expect_routes_tiers(function(...) p$plan(...))

  tiers <- EitherRoutesR6$RouteTiers$tiers
  expect_equal(eval(formals(tiers)$tiers), levels)
  expect_equal(tiers(p), "tiers:low,mid,high")
  expect_equal(tiers(p, tiers = c("hi", "lo")), "tiers:high,low")
  expect_equal(tiers(p, tiers = 4), "number:4")
  expect_choice_error(tiers(p, tiers = "max"), "tiers", levels)
})

test_that("S3 method with several_ok Either choice lists", {
  p <- new_eitherroutess3()
  expect_routes_tiers_formals(getS3method("plan_routes_s3", "EitherRoutesS3"))
  expect_routes_tiers(function(...) plan_routes_s3(p, ...))
})

test_that("S4 method with several_ok Either choice lists", {
  h <- EitherRoutesS4()
  expect_equal(names(formals(s4_plan_routes)), c("x", "..."))
  expect_routes_tiers_formals(
    methods::unRematchDefinition(methods::getMethod("s4_plan_routes", "EitherRoutesS4"))
  )
  expect_routes_tiers(function(...) s4_plan_routes(h, ...))
})

test_that("S7 constructor, method and shortcut with several_ok Either choice lists", {
  expect_equal(eval(formals(EitherRoutesS7)$start), routes)
  expect_equal(routes_given(EitherRoutesS7()), "absent")
  expect_equal(routes_given(EitherRoutesS7(start = "inf")), "Infusion")
  expect_equal(routes_given(EitherRoutesS7(start = factor(c("oral", "bolus")))), "Oral,Bolus")
  expect_equal(routes_given(EitherRoutesS7(start = doses)), "frame:2x2")
  expect_choice_error(EitherRoutesS7(start = c("oral", "iv")), "start", routes)

  h <- EitherRoutesS7()
  expect_routes_tiers_formals(S7::method(plan_routes_s7, EitherRoutesS7))
  expect_routes_tiers_formals(EitherRoutesS7_plan_routes_s7)
  expect_routes_tiers(function(...) plan_routes_s7(h, ...))
  expect_routes_tiers(function(...) EitherRoutesS7_plan_routes_s7(h, ...))
})

test_that("vctrs constructor and static method with several_ok Either choice lists", {
  expect_equal(eval(formals(new_eitherroutesvctrs)$routes), routes)
  expect_equal(vctrs::vec_data(new_eitherroutesvctrs()), c(1, 2, 3))
  expect_equal(vctrs::vec_data(new_eitherroutesvctrs("bo")), 2)
  expect_equal(vctrs::vec_data(new_eitherroutesvctrs(c("inf", "or"))), c(3, 1))
  expect_equal(vctrs::vec_data(new_eitherroutesvctrs(factor("infusion"))), 3)
  expect_equal(vctrs::vec_data(new_eitherroutesvctrs(doses)), c(0, 0))
  expect_choice_error(new_eitherroutesvctrs("iv"), "routes", routes)

  expect_routes_tiers_formals(eitherroutesvctrs_plan_routes)
  expect_routes_tiers(eitherroutesvctrs_plan_routes)
})

# The `EitherGrades` trait method reports `absent` or the grades
# (`tiers:<a>,<b>` / `number:<n>`) for an omittable `choices_several_ok`
# `Either<Vec<String>, f64>`.
expect_either_grades <- function(grades) {
  expect_equal(grades(), "absent")
  expect_equal(grades(grades = c("hi", "lo")), "tiers:high,low")
  expect_equal(grades(grades = 4), "number:4")
  expect_choice_error(grades(grades = "max"), "grades", levels)
}

test_that("S3 trait method with an omittable several_ok Either choice list", {
  p <- new_eitherroutess3()
  expect_equal(eval(formals(getS3method("either_grades", "EitherRoutesS3"))$grades), levels)
  expect_either_grades(function(...) either_grades(p, ...))
})

test_that("S7 trait method and its shortcut with an omittable several_ok Either choice list", {
  h <- EitherRoutesS7()
  expect_equal(
    eval(formals(S7::method(s7_trait_EitherGrades_either_grades, EitherRoutesS7))$grades),
    levels
  )
  expect_equal(eval(formals(EitherRoutesS7_either_grades)$grades), levels)
  expect_either_grades(function(...) s7_trait_EitherGrades_either_grades(h, ...))
  expect_either_grades(function(...) EitherRoutesS7_either_grades(h, ...))
})

test_that("several_ok Either choice lists name the other kind in their @param line", {
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
  # The standalone fixtures share one page with the scalar ones; every
  # parameter name here is new, so each line survives.
  page <- rd_text("match_arg_either_routes")
  expect_match(page, "One or more of \"oral\", \"bolus\", \"infusion\", or a data frame.", fixed = TRUE)
  expect_match(page, "One or more of \"low\", \"mid\", \"high\", or a number.", fixed = TRUE)
  expect_match(
    page,
    "One or more of \"oral\", \"bolus\", \"infusion\", a data frame, or NULL for no choice.",
    fixed = TRUE
  )
  expect_match(
    page,
    "One or more of \"oral\", \"bolus\", \"infusion\", a data frame, or NULL; omitting the argument means no choice.",
    fixed = TRUE
  )
  tiers_or_number <- "One or more of \"low\", \"mid\", \"high\", or a number; omitting the argument means no choice."
  for (topic in c("EitherRoutesS3", "EitherRoutesS7", "EitherRoutesVctrs")) {
    expect_match(rd_text(topic), tiers_or_number, fixed = TRUE)
  }
  expect_match(
    rd_text("EitherRoutesR6"),
    "One or more of \"oral\", \"bolus\", \"infusion\", or a data frame.",
    fixed = TRUE
  )
  # The S7 constructor's `start` (inlined in `new_class()`).
  expect_match(
    rd_text("EitherRoutesS7"),
    "One or more of \"oral\", \"bolus\", \"infusion\", or a data frame; omitting the argument means no choice.",
    fixed = TRUE
  )
})

# endregion
