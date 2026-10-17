partial_eq_constructors <- function() {
  constructors <- list(
    s3 = new_exacts3, env = ExactEnv$new, r6 = ExactR6$new,
    s4 = ExactS4, s7 = ExactS7
  )
  if (exists("exact_vctrs_pointer", mode = "function")) {
    constructors$vctrs <- function(value) {
      vctrs::new_vctr(numeric(), .ptr = exact_vctrs_pointer(value), class = "exact_vctrs")
    }
  }
  constructors
}

test_that("RPartialEq compares Rust values in every class system", {
  for (system in names(partial_eq_constructors())) {
    new <- partial_eq_constructors()[[system]]
    x <- new(1)
    expect_true(all.equal(x, new(1)))
    expect_true(all.equal(x, x))
    differences <- all.equal(x, new(2))
    expect_type(differences, "character")
    expect_match(differences[[1L]], "^Rust values of Exact.* differ$")
    if (system %in% c("env", "vctrs")) {
      expect_length(differences, 1L)
    } else {
      expect_length(differences, 2L)
      expect_match(differences[[2L]], "target: Exact")
      expect_match(differences[[2L]], "current: Exact")
    }
    expect_identical(all.equal(x, 1), sprintf("target is %s, current is numeric", data.class(x)))
    expect_type(all.equal(new(NaN), new(NaN)), "character")
    expect_true(all.equal(new(0), new(-0)))
  }
})

test_that("exact comparison accepts and ignores approximate settings without forcing them", {
  for (new in partial_eq_constructors()) {
    expect_type(all.equal(new(1), new(1 + 1e-7), tolerance = 1e9), "character")
    expect_true(all.equal(
      new(1), new(1), tolerance = stop("unused tolerance"),
      scale = stop("unused scale"), check.attributes = stop("unused setting")
    ))
  }
})

test_that("RPartialEq refuses restored target and current with the declared condition", {
  for (new in partial_eq_constructors()) {
    x <- new(1)
    saved <- unserialize(serialize(x, NULL))
    left <- tryCatch(all.equal(saved, x), error = identity)
    right <- tryCatch(all.equal(x, saved), error = identity)
    expect_s3_class(left, "partial_eq_saved")
    expect_s3_class(right, "partial_eq_saved")
    # S4's generated method runs in .local; keep the actual R call in either case.
    expect_true(as.character(conditionCall(left)[[1L]]) %in% c("all.equal", ".local"))
    expect_true(as.character(conditionCall(right)[[1L]]) %in% c("all.equal", ".local"))
  }
})

test_that("blanket metadata, trait aliases and type aliases preserve custom PartialEq", {
  expect_true(all.equal(new_exact_digit(1L), new_exact_digit(11L)))
  differences <- all.equal(new_exact_digit(1L), new_exact_digit(2L))
  expect_match(differences[[1L]], "^Rust values of .* differ$")
  expect_match(differences[[2L]], "ExactBlanket")
})

test_that("AllEqual dispatches nested exact comparisons without applying tolerance", {
  expect_true(all.equal(new_exactnested(1), new_exactnested(1)))
  differences <- all.equal(new_exactnested(1), new_exactnested(1 + 1e-7), tolerance = 1)
  expect_length(differences, 2L)
  expect_match(differences[[1L]], "^Exact model: Rust values of ExactS3 differ$")
  expect_match(differences[[2L]], "^Exact model: target:")
})

test_that("testthat editions retain their own S4 and R6 comparison behavior", {
  constructors <- partial_eq_constructors()
  for (system in names(constructors)) {
    new <- constructors[[system]]
    x <- new(1)
    local({
      testthat::local_edition(2)
      expect_equal(x, new(1))
      if (system == "s4") {
        # testthat imports base::all.equal, which bypasses the promoted S4 generic.
        expect_equal(x, new(2), tolerance = 1e9)
      } else {
        expect_failure(expect_equal(x, new(2), tolerance = 1e9))
      }
      expect_true(all.equal(x, new(1)))
      expect_type(all.equal(x, new(2)), "character")
    })
    local({
      testthat::local_edition(3)
      expect_equal(x, x)
      if (system == "r6") {
        # waldo's R6 proxy omits the private environment containing .ptr.
        expect_equal(x, new(1))
        expect_equal(x, new(2))
      } else {
        expect_failure(expect_equal(x, new(1)))
        expect_failure(expect_equal(x, new(2)))
      }
      expect_true(all.equal(x, new(1)))
      expect_type(all.equal(x, new(2)), "character")
    })
  }
})

test_that("exact comparison help describes ignored tolerance and the fieldwise alternative", {
  rd <- tools::Rd_db("miniextendr")[["ExactS3.Rd"]]
  text <- gsub("[[:space:]]+", " ", paste(capture.output(tools::Rd2txt(rd)), collapse = " "))
  expect_match(text, "ignored by this exact Rust PartialEq comparison", fixed = TRUE)
  expect_match(text, "#[derive(AllEqual)]", fixed = TRUE)
})

test_that("comparison help has no duplicated argument entries", {
  pages <- tools::Rd_db("miniextendr")
  tag_is <- function(node, name) {
    identical(attr(node, "Rd_tag"), paste0(intToUtf8(92L), name))
  }
  for (page in intersect(
    c("ExactEnv.Rd", "ExactNested.Rd", "ExactR6.Rd", "ExactS3.Rd", "ExactS4.Rd",
      "ExactS7.Rd", "exact_digit.Rd", "exact_vctrs.Rd"),
    names(pages)
  )) {
    arguments <- Filter(function(node) tag_is(node, "arguments"), pages[[page]])[[1L]]
    items <- Filter(function(node) tag_is(node, "item"), arguments)
    names <- trimws(unlist(strsplit(
      vapply(items, function(node) paste(node[[1L]], collapse = ""), character(1)), ","
    )))
    expect_identical(anyDuplicated(names), 0L, info = page)
  }
})
