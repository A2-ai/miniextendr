# Console-output prelude items: r_print!, r_println!, r_str!, r_warning
# (fixtures in rpkg/src/rust/console_output_tests.rs, audit A7)

test_that("r_print! writes to the R console without a newline", {
  expect_output(console_r_print("hello-from-rust"), "hello-from-rust")
})

test_that("r_print! interpolates format arguments", {
  expect_output(console_r_print_formatted("count", 7L), "count=7")
})

test_that("r_println! writes message plus newline", {
  out <- capture.output(console_r_println("lined-output"))
  expect_identical(out, "lined-output")
})

test_that("r_println!() with no arguments emits just a newline", {
  out <- capture.output(console_r_println_empty())
  expect_identical(out, "")
})

test_that("r_warning raises an R warning and returns normally", {
  expect_warning(console_r_warning("low-level-warning"), "low-level-warning")
  expect_identical(suppressWarnings(console_r_warning("x")), 42L)
})

test_that("r_str! evaluates dynamically built R code", {
  expect_identical(console_r_str_sum_seq(10L), 55L)
  expect_identical(console_r_str_sum_seq(1L), 1L)
})

test_that("r_str! surfaces parse errors as R conditions, not crashes", {
  expect_error(console_r_str_parse_error())
})
