# Trait methods that take (and return) the reading markers:
# src/rust/trait_marker_tests.rs. From R the implementing method reads each
# argument the way the marker does; through the trait's View (the path another
# package takes) each marker crosses as its inner value and reads back
# unchanged.

caught <- function(expr) tryCatch(expr, error = identity)
caught_msg <- function(expr) conditionMessage(caught(expr))

reader <- MarkerReader$new()
marker_call <- function(method, ...) MarkerReader$MarkerArgs[[method]](reader, ...)

test_that("a trait method reads AsNumeric from text, a factor or a number", {
  expect_identical(marker_call("marker_number", value = " 2.5 "), 2.5)
  expect_identical(marker_call("marker_number", value = factor("10")), 10)
  expect_identical(marker_call("marker_number", value = 3L), 3)
  expect_identical(marker_call("marker_number", value = "NA"), NA_real_)
  expect_identical(marker_call("marker_missing", value = c("1", "NA", "", NA)), 3L)
})

test_that("a trait method keeps the marker's R gate", {
  expect_identical(
    caught_msg(marker_call("marker_number", value = list(1))),
    "'value' must be numeric, logical, character, or factor"
  )
})

test_that("a trait method reads AsCharacter, NULL and an omitted argument", {
  expect_identical(marker_call("marker_label", value = 101L), "101")
  expect_identical(marker_call("marker_label", value = "NA"), "NA")
  expect_identical(marker_call("marker_label", value = NA), "<NA>")
  expect_identical(marker_call("marker_labels", value = NULL), "null")
  expect_identical(marker_call("marker_labels", value = factor(c("a", NA))), "a,<NA>")
  expect_identical(marker_call("marker_maybe"), "absent")
  expect_identical(marker_call("marker_maybe", value = "4"), "4")
})

test_that("a trait method parses AsFromStr and returns a marker", {
  expect_identical(marker_call("marker_ip", value = "127.0.0.1"), "127.0.0.1")
  expect_identical(marker_call("marker_ips", value = c("127.0.0.1", "::1")), 2L)
  expect_identical(marker_call("marker_echo", value = c("1", "NA", "NaN")), c(1, NA, NaN))
})

test_that("no_na on a trait method checks what the marker reads as NA", {
  expect_identical(marker_call("marker_dose", value = " 2 "), 2)
  for (v in list("NA", factor(""))) {
    expect_identical(caught_msg(marker_call("marker_dose", value = v)), "'value' must not be NA")
  }
  # The Rust check after the conversion raises the R guard's condition.
  from_r <- caught(marker_call("marker_dose", value = NA))
  from_rust <- caught(marker_call("marker_dose", value = "NA"))
  expect_identical(class(from_rust), class(from_r))
  expect_identical(from_rust$kind, from_r$kind)
  expect_identical(from_rust$param, from_r$param)
  expect_identical(conditionMessage(from_rust), conditionMessage(from_r))
})

test_that("marker arguments cross the trait ABI as their inner value", {
  expect_identical(
    marker_args_through_view(),
    c(
      "2.5", "NA", "NaN", "2", "NA", "<NA>", "null", "a,<NA>", "absent", "1",
      "127.0.0.1", "2", "1,NA,NaN"
    )
  )
})
