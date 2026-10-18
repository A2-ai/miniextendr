# How a class gets its constructor. Fixtures in src/rust/constructor_tests.rs:
# `Infer*` types have one receiver-less method returning `Self`, which is the
# constructor; `Marked*` types have two and mark `with_value` as the
# constructor; `NoCtor*` types have none (their factories return
# `Result<Self, String>` and `Option<Self>`) and are made by those factories;
# `SplitInfer*` types have their one candidate in a second impl block. A
# constructor not named `new` keeps its own name as a static method too.
# `noctor_rec` is an S3 class the package names and builds with its factories,
# and `NoCtorVctrs` a vctrs class without a constructor.

# R6's `$new()` and S7's constructor, called without the `.ptr` the factories
# pass, refuse to build an object holding no pointer.
expect_no_constructor <- function(expr, class) {
  expect_error(expr, sprintf("`%s` has no constructor", class), fixed = TRUE)
}

# region: inferred constructor

test_that("an env class's one method returning Self is its constructor", {
  obj <- InferEnv$new(5L)
  expect_s3_class(obj, "InferEnv")
  expect_identical(obj$value(), 5L)
  # The constructor keeps its own name.
  own <- InferEnv$from_value(6L)
  expect_s3_class(own, "InferEnv")
  expect_identical(own$value(), 6L)
  # `Result<Self, E>` does not make a method a candidate: `parse` is static.
  expect_identical(InferEnv$parse("7")$value(), 7L)
  expect_error(InferEnv$parse("x"), "cannot parse")
})

test_that("an R6 class's one method returning Self is its constructor", {
  obj <- InferR6$new(5L)
  expect_true(R6::is.R6(obj))
  expect_identical(obj$value(), 5L)
  own <- InferR6$from_value(6L)
  expect_true(inherits(own, "InferR6"))
  expect_identical(own$value(), 6L)
  expect_identical(InferR6$parse("7")$value(), 7L)
})

test_that("an S3 class's one method returning Self is its constructor", {
  obj <- new_infers3(5L)
  expect_s3_class(obj, "InferS3")
  expect_identical(infer3_value(obj), 5L)
  expect_identical(infer3_value(InferS3$new(6L)), 6L)
  own <- infers3_from_value(7L)
  expect_s3_class(own, "InferS3")
  expect_identical(infer3_value(own), 7L)
  expect_identical(infer3_value(infers3_parse("8")), 8L)
})

test_that("an S4 class's one method returning Self is its constructor", {
  obj <- InferS4(5L)
  expect_true(methods::is(obj, "InferS4"))
  expect_identical(s4_infer4_value(obj), 5L)
  own <- InferS4_from_value(6L)
  expect_true(methods::is(own, "InferS4"))
  expect_identical(s4_infer4_value(own), 6L)
  expect_identical(s4_infer4_value(InferS4_parse("7")), 7L)
})

test_that("an S7 class's one method returning Self is its constructor", {
  obj <- InferS7(5L)
  expect_true(S7::S7_inherits(obj, InferS7))
  expect_identical(infer7_value(obj), 5L)
  own <- InferS7_from_value(6L)
  expect_true(S7::S7_inherits(own, InferS7))
  expect_identical(infer7_value(own), 6L)
  expect_identical(infer7_value(InferS7_parse("7")), 7L)
})

# endregion

# region: marked constructor

test_that("an env class's marked method is its constructor", {
  expect_identical(MarkedEnv$new(5L)$value(), 5L)
  own <- MarkedEnv$with_value(6L)
  expect_s3_class(own, "MarkedEnv")
  expect_identical(own$value(), 6L)
  # The other method returning `Self` is static.
  expect_identical(MarkedEnv$zero()$value(), 0L)
})

test_that("an R6 class's marked method is its constructor", {
  expect_identical(MarkedR6$new(5L)$value(), 5L)
  own <- MarkedR6$with_value(6L)
  expect_true(inherits(own, "MarkedR6"))
  expect_identical(own$value(), 6L)
  expect_identical(MarkedR6$zero()$value(), 0L)
})

test_that("an S3 class's marked method is its constructor", {
  expect_identical(marked3_value(new_markeds3(5L)), 5L)
  own <- markeds3_with_value(6L)
  expect_s3_class(own, "MarkedS3")
  expect_identical(marked3_value(own), 6L)
  expect_identical(marked3_value(markeds3_zero()), 0L)
})

test_that("an S4 class's marked method is its constructor", {
  expect_identical(s4_marked4_value(MarkedS4(5L)), 5L)
  own <- MarkedS4_with_value(6L)
  expect_true(methods::is(own, "MarkedS4"))
  expect_identical(s4_marked4_value(own), 6L)
  expect_identical(s4_marked4_value(MarkedS4_zero()), 0L)
})

test_that("an S7 class's marked method is its constructor", {
  expect_identical(marked7_value(MarkedS7(5L)), 5L)
  own <- MarkedS7_with_value(6L)
  expect_true(S7::S7_inherits(own, MarkedS7))
  expect_identical(marked7_value(own), 6L)
  expect_identical(marked7_value(MarkedS7_zero()), 0L)
})

# endregion

# region: no constructor

test_that("an env class without a constructor is made by its factories", {
  expect_null(NoCtorEnv$new)
  obj <- NoCtorEnv$of(3L)
  expect_s3_class(obj, "NoCtorEnv")
  expect_identical(obj$value(), 3L)
  expect_identical(NoCtorEnv$parse("0")$value(), 0L)
  expect_error(NoCtorEnv$of(-1L), "negative value")
  expect_null(NoCtorEnv$parse("x"))
})

test_that("an R6 class without a constructor is made by its factories", {
  expect_no_constructor(NoCtorR6$new(), "NoCtorR6")
  obj <- NoCtorR6$of(3L)
  expect_true(R6::is.R6(obj))
  expect_identical(obj$value(), 3L)
  expect_identical(NoCtorR6$parse("0")$value(), 0L)
  expect_error(NoCtorR6$of(-1L), "negative value")
})

test_that("an S3 class without a constructor is made by its factories", {
  expect_null(NoCtorS3$new)
  # No `new_<class>()` a package's own could clash with (#2069).
  expect_false(exists("new_noctors3", envir = asNamespace("miniextendr"), inherits = FALSE))
  obj <- noctors3_of(3L)
  expect_s3_class(obj, "NoCtorS3")
  expect_identical(noctor3_value(obj), 3L)
  expect_identical(noctor3_value(noctors3_parse("0")), 0L)
  expect_error(noctors3_of(-1L), "negative value")
})

test_that("an S4 class without a constructor is made by its factories", {
  expect_false(exists("NoCtorS4", envir = asNamespace("miniextendr"), inherits = FALSE))
  obj <- NoCtorS4_of(3L)
  expect_true(methods::is(obj, "NoCtorS4"))
  expect_identical(s4_noctor4_value(obj), 3L)
  expect_identical(s4_noctor4_value(NoCtorS4_parse("0")), 0L)
  expect_error(NoCtorS4_of(-1L), "negative value")
})

test_that("an S7 class without a constructor is made by its factories", {
  expect_no_constructor(NoCtorS7(), "NoCtorS7")
  obj <- NoCtorS7_of(3L)
  expect_true(S7::S7_inherits(obj, NoCtorS7))
  expect_identical(noctor7_value(obj), 3L)
  expect_identical(noctor7_value(NoCtorS7_parse("0")), 0L)
  expect_error(NoCtorS7_of(-1L), "negative value")
})

test_that("an S3 class the package names is made by its factories", {
  obj <- noctor_rec_of(3L)
  expect_s3_class(obj, "noctor_rec")
  expect_identical(noctor_rec_value(obj), 3L)
  expect_identical(noctor_rec_value(noctor_rec_parse("0")), 0L)
  expect_error(noctor_rec_of(-1L), "negative value")
  # No constructor and no class environment (#1918).
  ns <- asNamespace("miniextendr")
  expect_false(exists("new_noctor_rec", envir = ns, inherits = FALSE))
  expect_false(exists("noctor_rec", envir = ns, inherits = FALSE))
})

test_that("a vctrs class without a constructor is made with vctrs::new_vctr()", {
  expect_false(exists("new_noctorvctrs", envir = asNamespace("miniextendr"), inherits = FALSE))
  v <- vctrs::new_vctr(c(1, 2.5), class = "NoCtorVctrs")
  expect_s3_class(v, "NoCtorVctrs")
  expect_identical(vctrs::vec_ptype_abbr(v), "noctor")
  expect_identical(noctorvctrs_total(c(1, 2.5)), 3.5)
})

# endregion

# region: help pages of classes without a constructor

# load_all() registers the source directory as the namespace path; it has
# generated man pages but no installed help database.
ctor_rd_db <- function() {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
}

ctor_rd_section <- function(rd, tag) {
  nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
  trimws(gsub("[[:space:]]+", " ", paste(unlist(nodes), collapse = "")))
}

ctor_rd_aliases <- function(rd) {
  unlist(rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), "\\alias"), logical(1))])
}

# Each page has its title and the impl docs.
test_that("the impl docs of a class without a constructor reach its page", {
  db <- ctor_rd_db()
  pages <- list(
    NoCtorEnv = c("NoCtorEnv Class", "Env class whose objects come from its factories."),
    NoCtorR6 = c("NoCtorR6 R6 Class", "R6 class whose objects come from its factories."),
    NoCtorS3 = c("NoCtorS3 S3 Class", "S3 class whose objects come from its factories."),
    NoCtorS4 = c("NoCtorS4 S4 Class", "S4 class whose objects come from its factories."),
    NoCtorS7 = c("NoCtorS7 S7 Class", "S7 class whose objects come from its factories."),
    NoCtorVctrs = c(
      "NoCtorVctrs vctrs S3 Class",
      "vctrs class whose vectors the package builds with vctrs::new_vctr()."
    )
  )
  for (class in names(pages)) {
    rd <- db[[paste0(class, ".Rd")]]
    expect_false(is.null(rd), info = class)
    expect_identical(ctor_rd_section(rd, "\\title"), pages[[class]][[1]], info = class)
    expect_match(
      ctor_rd_section(rd, "\\description"), pages[[class]][[2]],
      fixed = TRUE, info = class
    )
  }
})

test_that("a class the package names gets a titled page named after it", {
  rd <- ctor_rd_db()[["noctor_rec.Rd"]]
  expect_false(is.null(rd))
  expect_identical(ctor_rd_section(rd, "\\title"), "noctor_rec S3 Class")
  expect_true("noctor_rec" %in% ctor_rd_aliases(rd))
  expect_match(
    ctor_rd_section(rd, "\\description"),
    "Records of one value, built by noctor_rec_of() and noctor_rec_parse().",
    fixed = TRUE
  )
})

test_that("impl docs with @rdname join the package's own page", {
  rd <- ctor_rd_db()[["handmade_rec.Rd"]]
  expect_false(is.null(rd))
  expect_identical(
    ctor_rd_section(rd, "\\title"),
    "A list-like S3 class the package builds itself"
  )
  expect_match(
    ctor_rd_section(rd, "\\description"), "The readers of handmade_rec",
    fixed = TRUE
  )
  expect_false("handmade_rec" %in% ctor_rd_aliases(rd))
})

test_that("every export has a help page", {
  aliases <- unlist(lapply(ctor_rd_db(), ctor_rd_aliases), use.names = FALSE)
  # S4 method tables (`.__T__generic:pkg`) are exported but not documented,
  # as `R CMD check` knows.
  exports <- grep("^[.]__[TC]__", getNamespaceExports("miniextendr"), value = TRUE, invert = TRUE)
  expect_identical(setdiff(exports, aliases), character())
})

# endregion

# region: decided across impl blocks

test_that("a type's one candidate is its constructor whichever impl block has it", {
  expect_identical(SplitInferEnv$new(4L)$value(), 4L)
  expect_identical(SplitInferR6$new(4L)$value(), 4L)
  expect_identical(splitinfer3_value(new_splitinfers3(4L)), 4L)
  expect_identical(s4_splitinfer4_value(SplitInferS4(4L)), 4L)
  expect_identical(splitinfer7_value(SplitInferS7(4L)), 4L)
})

test_that("a constructor from a second impl block keeps its own name", {
  expect_identical(SplitInferEnv$from_value(5L)$value(), 5L)
  expect_identical(SplitInferR6$from_value(5L)$value(), 5L)
  expect_identical(splitinfer3_value(splitinfers3_from_value(5L)), 5L)
  expect_identical(s4_splitinfer4_value(SplitInferS4_from_value(5L)), 5L)
  expect_identical(splitinfer7_value(SplitInferS7_from_value(5L)), 5L)
})

# endregion
