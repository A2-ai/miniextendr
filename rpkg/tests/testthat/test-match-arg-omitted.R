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

# region: S3 / S4 / S7 / vctrs methods and trait methods

levels <- c("low", "mid", "high")

# The `omit_pick*` fixtures report `mode=<..>;modes=<..>;level=<..>`: what
# reached Rust for a `Missing<Option<ImplMode>>` match_arg, a
# `Missing<Vec<ImplMode>>` several_ok list and a `Missing<String>` inline
# choice. `f` forwards its arguments through `...`, which keeps an omitted
# argument missing.
expect_omitted_report <- function(f) {
  expect_equal(f(), "mode=absent;modes=absent;level=absent")
  expect_equal(
    f(mode = NULL, modes = NULL, level = NULL),
    "mode=null;modes=Fast,Safe,Debug;level=low"
  )
  expect_equal(
    f(mode = "Sa", modes = c("De", "Fa"), level = "hi"),
    "mode=Safe;modes=Debug,Fast;level=high"
  )
  expect_equal(f(mode = factor("Debug")), "mode=Debug;modes=absent;level=absent")
  expect_choice_error(f(mode = "nope"), "mode", modes)
  expect_choice_error(f(modes = c("Fast", "zzz")), "modes", modes)
  expect_choice_error(f(level = "max"), "level", levels)
}

# The formals keep every choice vector.
expect_omitted_formals <- function(fn) {
  fmls <- formals(fn)
  expect_equal(eval(fmls$mode), modes)
  expect_equal(eval(fmls$modes), modes)
  expect_equal(eval(fmls$level), levels)
}

test_that("S3 method with omittable choices", {
  p <- new_omitpicks3()
  expect_omitted_formals(getS3method("omit_pick_s3", "OmitPickS3"))
  expect_omitted_report(function(...) omit_pick_s3(p, ...))
})

test_that("S4 method with omittable choices", {
  h <- OmitPickS4()
  # The generic keeps `function(x, ...)`; the method's own formals (which R
  # wraps in `.local()`) carry the choice vectors.
  expect_equal(names(formals(s4_omit_pick)), c("x", "..."))
  expect_omitted_formals(
    methods::unRematchDefinition(methods::getMethod("s4_omit_pick", "OmitPickS4"))
  )
  expect_omitted_report(function(...) s4_omit_pick(h, ...))
})

test_that("S7 method and its fast-path shortcut with omittable choices", {
  h <- OmitPickS7()
  expect_omitted_formals(S7::method(omit_pick_s7, OmitPickS7))
  expect_omitted_formals(OmitPickS7_omit_pick_s7)
  expect_omitted_report(function(...) omit_pick_s7(h, ...))
  expect_omitted_report(function(...) OmitPickS7_omit_pick_s7(h, ...))
})

test_that("vctrs constructor, static method and format method with omittable choices", {
  expect_equal(eval(formals(new_vctrsomitscale)$mode), modes)
  expect_equal(vctrs::vec_data(new_vctrsomitscale()), 0)
  expect_equal(vctrs::vec_data(new_vctrsomitscale(NULL)), -1)
  expect_equal(vctrs::vec_data(new_vctrsomitscale("Sa")), 2)
  expect_equal(vctrs::vec_data(new_vctrsomitscale(factor("Debug"))), 3)
  expect_choice_error(new_vctrsomitscale("nope"), "mode", modes)

  expect_omitted_formals(vctrsomitscale_omit_pick)
  expect_omitted_report(vctrsomitscale_omit_pick)

  v <- new_vctrsomitscale("Sa")
  styles <- c("short", "long")
  expect_equal(eval(formals(getS3method("format", "VctrsOmitScale"))$style), styles)
  expect_equal(format(v), "2")
  expect_equal(format(v, style = "lo"), "long:2")
  expect_choice_error(format(v, style = "wide"), "style", styles)
})

# The `OmitGrade` trait methods report `absent`, `null` or the matched grade
# (`omit_grade`, `Missing<Option<String>>`), and `absent` or the matched grades
# (`omit_grades`, `Missing<Vec<String>>` several_ok). `grade` / `grades` call
# them with the receiver bound.
expect_omitted_grade_formals <- function(grade, grades) {
  expect_equal(eval(formals(grade)$grade), levels)
  expect_equal(eval(formals(grades)$grades), levels)
}
expect_omitted_grades <- function(grade, grades) {
  expect_equal(grade(), "absent")
  expect_equal(grade(grade = NULL), "null")
  expect_equal(grade(grade = "mi"), "mid")
  expect_choice_error(grade(grade = "max"), "grade", levels)
  expect_equal(grades(), "absent")
  expect_equal(grades(grades = NULL), "low,mid,high")
  expect_equal(grades(grades = c("hi", "lo")), "high,low")
  expect_choice_error(grades(grades = c("low", "max")), "grades", levels)
}

test_that("S3 trait methods with omittable inline choices", {
  p <- new_omitpicks3()
  expect_omitted_grade_formals(
    getS3method("omit_grade", "OmitPickS3"),
    getS3method("omit_grades", "OmitPickS3")
  )
  expect_omitted_grades(
    function(...) omit_grade(p, ...),
    function(...) omit_grades(p, ...)
  )
})

test_that("S7 trait methods and their shortcuts with omittable inline choices", {
  h <- OmitPickS7()
  expect_omitted_grade_formals(
    S7::method(s7_trait_OmitGrade_omit_grade, OmitPickS7),
    S7::method(s7_trait_OmitGrade_omit_grades, OmitPickS7)
  )
  expect_omitted_grade_formals(OmitPickS7_omit_grade, OmitPickS7_omit_grades)
  expect_omitted_grades(
    function(...) s7_trait_OmitGrade_omit_grade(h, ...),
    function(...) s7_trait_OmitGrade_omit_grades(h, ...)
  )
  expect_omitted_grades(
    function(...) OmitPickS7_omit_grade(h, ...),
    function(...) OmitPickS7_omit_grades(h, ...)
  )
})

test_that("S3 trait generics and vctrs static helpers are exported by name", {
  exports <- getNamespaceExports("miniextendr")
  # The S3 trait wrappers export the generic, not `generic.Class` as a plain
  # function.
  expect_true(all(c("omit_grade", "omit_grades", "checked_add") %in% exports))
  expect_false(any(c("omit_grade.OmitPickS3", "checked_add.S3TraitCounter") %in% exports))
  expect_true("vctrsomitscale_omit_pick" %in% exports)
})

test_that("method choice params get the choice text as their @param line", {
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
  omittable_levels <- "One of \"low\", \"mid\", \"high\"; omitting the argument means no choice."
  # `level` is a `choices(...)` param of the inherent S3 / S7 / vctrs methods
  # (an S4 method has no usage entry, so no argument docs);
  # `grade` / `grades` those of the S3 trait methods (documented on the S3
  # method's usage) and of the S7 trait shortcuts.
  for (topic in c("OmitPickS3", "OmitPickS7", "VctrsOmitScale")) {
    expect_match(rd_text(topic), omittable_levels, fixed = TRUE)
  }
  for (topic in c("OmitPickS3", "OmitPickS7")) {
    page <- rd_text(topic)
    expect_match(
      page,
      "One of \"low\", \"mid\", \"high\", or NULL; omitting the argument means no choice.",
      fixed = TRUE
    )
    expect_match(
      page,
      "One or more of \"low\", \"mid\", \"high\"; omitting the argument means no choice.",
      fixed = TRUE
    )
  }
})

# endregion
