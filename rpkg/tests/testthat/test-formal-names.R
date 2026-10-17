test_that("explicit formals control named calls, defaults, choice matching and omission", {
  expect_identical(names(formals(formal_arguments)),
                   c("conf.int", "conf.level", "optional.value", "pick.mode", "literal.mode"))
  expect_identical(formal_arguments(), c("false", "0.95", "missing", "safe", "fast"))
  expect_identical(formal_arguments(conf.int = TRUE, conf.level = 0.9, optional.value = 7L,
                                   pick.mode = "f", literal.mode = "s"),
                   c("true", "0.9", "7", "fast", "safe"))
  forward <- function(optional.value) formal_arguments(optional.value = optional.value)
  expect_identical(forward()[[3L]], "missing")
  expect_identical(forward(8L)[[3L]], "8")
  expect_true(formal_choice_symbols(conf.int = "fast", conf_int = "safe"))
  expect_identical(formal_coerce(numeric.value = 7L), 7)
})

test_that("R guards and native conversion conditions name the actual formal", {
  check_param <- function(expr, name) {
    error <- tryCatch(force(expr), error = identity)
    expect_s3_class(error, "rust_error")
    expect_identical(error$param, name)
    expect_match(conditionMessage(error), name, fixed = TRUE)
  }
  check_param(formal_arguments(conf.int = 1L), "conf.int")
  check_param(formal_arguments(conf.level = NA_real_), "conf.level")
  check_param(formal_arguments(pick.mode = "bad"), "pick.mode")
  check_param(formal_arguments(literal.mode = "bad"), "literal.mode")
  check_param(formal_native(exact.value = "bad"), "exact.value")
  expect_error(formal_native(), "exact.value", class = "missingArgError")
})

test_that("dotted base calls cannot be replaced by argument functions", {
  fail <- function(...) stop("shadowed")
  expect_identical(formal_shadow(is.character = fail, parent.frame = fail, .Call = fail,
                                quoted.expr = stop("never forced")),
                   quote(stop("never forced")))
})

formal_cases <- list(
  env = list(class = "FormalEnv", add = "fe_add", capture = "fe_capture"),
  r6 = list(class = "FormalR6", add = "fr_add", capture = "fr_capture"),
  s3 = list(class = "new_formals3", add = "fs3_add", capture = "fs3_capture"),
  s4 = list(class = "FormalS4", add = "s4_fs4_add", capture = "s4_fs4_capture"),
  s7 = list(class = "FormalS7", add = "fs7_add", capture = "fs7_capture")
)
for (system in names(formal_cases)) local({
  system <- system
  case <- formal_cases[[system]]
  test_that(paste(system, "constructors and methods use explicit names"), {
    ns <- asNamespace("miniextendr")
    class <- get(case$class, ns)
    constructor <- if (system %in% c("env", "r6")) class$new else class
    object <- constructor(start.at = 10L)
    method <- if (system %in% c("env", "r6")) object[[case$add]] else get(case$add, ns)
    args <- if (system %in% c("env", "r6")) list(add.by = 3L) else list(object, add.by = 3L)
    expect_identical(do.call(method, args), 13L)
    bad <- args
    bad$add.by <- "bad"
    error <- tryCatch(do.call(method, bad), error = identity)
    expect_s3_class(error, "rust_error")
    expect_identical(error$param, "add.by")
    frame <- new.env(parent = ns)
    frame$object <- object
    capture <- if (system %in% c("env", "r6")) {
      call("$", quote(object), as.name(case$capture))
    } else as.name(case$capture)
    capture_args <- if (system %in% c("env", "r6")) list() else list(quote(object))
    expect_null(eval(as.call(c(list(capture), capture_args)), frame))
    capture_args$quoted.expr <- quote(stop("never forced"))
    expect_identical(eval(as.call(c(list(capture), capture_args)), frame),
                     quote(stop("never forced")))

    trait <- if (system %in% c("env", "r6")) class$FormalTrait$named else {
      get(switch(system, s3 = "named", s4 = "s4_trait_FormalTrait_named",
                 s7 = "s7_trait_FormalTrait_named"), ns)
    }
    expect_identical(trait(object, trait.value = 8L), 9L)
    expect_identical(trait(object), 6L)
    error <- tryCatch(trait(object, trait.value = "bad"), error = identity)
    expect_identical(error$param, "trait.value")
    generic <- if (system %in% c("env", "r6")) class$FormalGeneric$generic_named else {
      get(switch(system, s3 = "generic_named", s4 = "s4_trait_FormalGeneric_generic_named",
                 s7 = "s7_trait_FormalGeneric_generic_named"), ns)
    }
    expect_identical(generic(object, generic.value = 8L), 8L)
    static <- if (system %in% c("env", "r6")) class$named_static else {
      get(paste0(switch(system, s3 = "formals3", s4 = "FormalS4", s7 = "FormalS7"),
                 "_named_static"), ns)
    }
    expect_identical(static(static.value = 11L), 11L)
  })
})

test_that("vctrs exposes constructor and static formal names", {
  x <- new_formalvctrs(.data = c(1, 2))
  expect_identical(vctrs::vec_data(x), c(1, 2))
  expect_false(formalvctrs_named_static())
  expect_true(formalvctrs_named_static(na.rm = TRUE))
})

test_that("generated help uses explicit formals without Rust-name fillers", {
  db <- tools::Rd_db("miniextendr")
  page <- paste(capture.output(tools::Rd2txt(db[["formal_arguments.Rd"]])), collapse = "\n")
  expect_match(page, "conf.int", fixed = TRUE)
  expect_match(page, "conf.level", fixed = TRUE)
  expect_false(grepl("conf_int", page, fixed = TRUE))
})

test_that("dotted predicates, S3 methods and property adapters forward named values", {
  error <- tryCatch(formal_shadow_predicate(is.character = function(...) stop("shadowed")),
                    error = identity)
  expect_s3_class(error, "rust_error")
  expect_identical(error$param, "is.character")
  expect_identical(formal_shadow_predicate(is.character = "ok"), "ok")
  expect_identical(format.formal_result(NULL, conf.int = TRUE), "true")
  expect_true("conf.int" %in% names(formals(format.formal_result)))
  expect_identical(format(structure(list(), class = "formal_result"), conf.int = TRUE), "true")
  r6 <- FormalProperties$new()
  r6$reading <- 12L
  expect_identical(r6$reading, 12L)
  error <- tryCatch({r6$reading <- "bad"}, error = identity)
  expect_identical(error$param, "new.value")
  s7 <- FormalS7Properties()
  s7@reading <- 13L
  expect_identical(s7@reading, 13L)
  expect_identical(formal_dispatch(s7, other.value = 2L), 15L)
})
