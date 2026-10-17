all_equal_constructors <- function() {
  constructors <- list(
    s3 = new_equals3,
    env = EqualEnv$new,
    r6 = EqualR6$new,
    s4 = EqualS4,
    s7 = EqualS7
  )
  if (exists("equal_vctrs_pointer", mode = "function")) {
    constructors$vctrs <- function(estimate, iterations, auxiliary, cache) {
      vctrs::new_vctr(
        numeric(), .ptr = equal_vctrs_pointer(estimate, iterations, auxiliary, cache),
        class = "equal_vctrs"
      )
    }
  }
  lapply(constructors, function(new) {
    force(new)
    function(estimate = 1, iterations = 2, auxiliary = 3, cache = 0L) {
      new(estimate, iterations, auxiliary, cache)
    }
  })
}

all_equal_caught <- function(expr) tryCatch(expr, error = identity)

test_that("AllEqual compares values in every class system", {
  for (new in all_equal_constructors()) {
    x <- new()
    expect_true(all.equal(x, new()))
    expect_true(all.equal(x, x))
    expect_true(all.equal(x, new(cache = 99L)))
    differences <- all.equal(x, new(estimate = 4, iterations = 5, auxiliary = 6))
    expect_type(differences, "character")
    expect_length(differences, 3L)
    expect_match(differences[[1L]], "^Estimate: ")
    expect_match(differences[[2L]], "^Component .*iterations.*: ")
    expect_match(differences[[3L]], "^Auxiliary: ")
    expect_identical(all.equal(x, 1), sprintf("target is %s, current is numeric", data.class(x)))
  }
})

test_that("field, call and struct comparison settings have distinct precedence", {
  for (new in all_equal_constructors()) {
    x <- new()
    near <- new(estimate = 1 + 1e-7)
    expect_true(all.equal(x, near))
    expect_type(all.equal(x, near, tolerance = 1e-10), "character")
    expect_type(all.equal(x, new(iterations = 3), tolerance = 1e20), "character")
    expect_true(all.equal(x, new(estimate = 2), tolerance = 0.01, scale = 1000))
    expect_type(all.equal(x, new(estimate = 2), tolerance = 0.01, scale = 1), "character")
    expect_true(all.equal(x, new(), check.attributes = FALSE, check.names = FALSE))
  }
})

test_that("AllEqual refuses restored values even when every field is skipped", {
  for (new in all_equal_constructors()) {
    x <- new()
    saved <- unserialize(serialize(x, NULL))
    expect_s3_class(all_equal_caught(all.equal(saved, x)), "all_equal_saved")
    expect_s3_class(all_equal_caught(all.equal(x, saved)), "all_equal_saved")
  }
  skipped <- new_equalskipped(1L)
  expect_true(all.equal(skipped, new_equalskipped(9L)))
  expect_s3_class(
    all_equal_caught(all.equal(unserialize(serialize(skipped, NULL)), skipped)),
    "miniextendr_restored"
  )
  x <- new_equalunit()
  expect_true(all.equal(x, new_equalunit()))
  expect_s3_class(
    all_equal_caught(all.equal(x, unserialize(serialize(x, NULL)))),
    "miniextendr_restored"
  )
})

test_that("nested aliases and getters retain class dispatch and defaults", {
  x <- new_equalnested(1)
  near <- new_equalnested(1.001)
  expect_true(all.equal(x, near))
  differences <- all.equal(x, near, tolerance = 1e-6)
  expect_length(differences, 1L)
  expect_match(differences, "^Model: Leaf value: ")
  expect_true(all.equal(new_equalgetter("same", 1), new_equalgetter("same", 1.001)))
  differences <- all.equal(
    new_equalgetter("left", 1), new_equalgetter("right", 2), tolerance = 0
  )
  expect_length(differences, 2L)
  expect_match(differences[[1L]], "^Component .*value.*: ")
  expect_match(differences[[2L]], "^Computed model: Leaf value: ")
})

test_that("nested R6, S4 and S7 fields use their own comparison", {
  x <- new_equalnestedsystems(1)
  near <- new_equalnestedsystems(1 + 1e-7)
  expect_true(all.equal(x, near))
  differences <- all.equal(x, near, tolerance = 0)
  expect_length(differences, 3L)
  for (i in seq_along(differences)) {
    expect_match(differences[[i]], paste0("^Component .*", c("r6", "s4", "s7")[[i]], ".*: Estimate: "))
  }
})

test_that("sidecar slots and computed values are read without evaluating language", {
  x <- new_equalsidecar(quote(stop("must not execute")), 1)
  y <- new_equalsidecar(quote(stop("must not execute")), 1)
  expect_true(all.equal(x, y))
  expect_true(all.equal(
    new_equalsidecar(structure(1, names = "left"), 1),
    new_equalsidecar(structure(1, names = "right"), 1),
    check.attributes = FALSE
  ))
  y$value <- quote(stop("different"))
  expect_match(all.equal(x, y), "^Component .*value.*: ")
  expect_match(all.equal(x, new_equalsidecar(x$value, 2)), "^Component .*computed.*: ")
  expect_true(all.equal(x, x))
})

test_that("tuple fields use base list component prefixes", {
  expect_true(all.equal(new_equaltuple(1, "a"), new_equaltuple(1, "a")))
  differences <- all.equal(new_equaltuple(1, "a"), new_equaltuple(2, "b"))
  expect_length(differences, 2L)
  expect_match(differences[[1L]], "^Component 1: ")
  expect_match(differences[[2L]], "^Component 2: ")
})

test_that("testthat edition 2 uses AllEqual and edition 3 retains waldo semantics", {
  x <- new_equals3(1, 2, 3, 0L)
  equal <- new_equals3(1, 2, 3, 9L)
  different <- new_equals3(4, 2, 3, 0L)
  local({
    testthat::local_edition(2)
    expect_equal(x, equal)
    expect_failure(expect_equal(x, different))
  })
  local({
    testthat::local_edition(3)
    expect_equal(x, x)
    expect_failure(expect_equal(x, equal))
  })
})

test_that("AllEqual keeps converted fields rooted through later allocations", {
  skip_gc_stress_if_disabled()
  constructors <- all_equal_constructors()
  results <- local({
    gctorture(TRUE)
    on.exit(gctorture(FALSE), add = TRUE)
    gc_stress_all_equal()
    lapply(constructors, function(new) {
      list(equal = all.equal(new(), new()), unequal = all.equal(new(), new(estimate = 2)))
    })
  })
  for (result in results) {
    expect_true(result$equal)
    expect_match(result$unequal, "^Estimate: ")
  }
})

test_that("internal class comparison help stays internal", {
  skip_if_not(exists("equal_vctrs_pointer", mode = "function"))
  rd <- tools::Rd_db("miniextendr")[["equal_vctrs.Rd"]]
  keywords <- Filter(function(node) identical(attr(node, "Rd_tag"), "\\keyword"), rd)
  expect_true("internal" %in% vapply(keywords, function(node) trimws(paste(node, collapse = "")), character(1)))
})
