# S7 multi-dispatch and Ops operator methods. Fixtures:
# src/rust/s7_dispatch_tests.rs. Before these were supported, the generated
# `S7::method(generic, Class)` registrations stopped the package from loading
# ("`signature` must be a list for multidispatch generics").

test_that("s7(dispatch = \"x, other\") builds a two-argument generic", {
  expect_true(inherits(mx_s7_pair, "S7_generic"))
  expect_equal(mx_s7_pair@dispatch_args, c("x", "other"))
  expect_true("mx_s7_pair" %in% getNamespaceExports("miniextendr"))

  left <- MxS7Left("a")
  expect_equal(mx_s7_pair(left, 1), "left a:1")
  expect_equal(mx_s7_pair(left, 3, sep = "="), "left a=3")
  expect_equal(mx_s7_pair(x = left, other = 4), "left a:4")
  # `other` is registered as S7::class_any, so a value of the wrong type is
  # rejected by the method's own argument check, which names the parameter,
  # rather than by S7 dispatch.
  expect_error(mx_s7_pair(left, "z"), "other")
})

test_that("the receiver's class selects the method", {
  expect_equal(mx_s7_pair(MxS7Left("a"), 1), "left a:1")
  expect_equal(mx_s7_pair(MxS7Right("b"), 1), "right b:1")
  expect_error(mx_s7_pair(1, 2), "Can't find method")
  # Shortcuts keep the Rust parameter names.
  expect_equal(MxS7Left_mx_s7_pair(MxS7Left("a"), 5), "left a:5")
  expect_equal(MxS7Right_mx_s7_pair(MxS7Right("b"), 6, sep = "-"), "right b-6")
})

test_that("the generic dispatches on the second argument too", {
  # A more specific method for (MxS7Left, MxS7Right) wins over the generated
  # list(MxS7Left, S7::class_any) method, which still handles the rest.
  S7::method(mx_s7_pair, list(MxS7Left, MxS7Right)) <- function(x, other, ...) "left-right"
  expect_equal(mx_s7_pair(MxS7Left("a"), MxS7Right("b")), "left-right")
  expect_equal(mx_s7_pair(MxS7Left("a"), 7), "left a:7")
  expect_equal(mx_s7_pair(MxS7Right("b"), 7), "right b:7")
})

test_that("Ops methods dispatch through R's operator syntax", {
  a <- MxS7Money(2)
  b <- MxS7Money(3)

  total <- a + b
  expect_true(S7::S7_inherits(total, MxS7Money))
  expect_equal(total@amount, 5)
  expect_equal((a * 4)@amount, 8)
  expect_true(a == MxS7Money(2))
  expect_false(a == b)
  expect_true(a < 2.5)
  expect_false(b < 2.5)

  # The right operand goes through the Rust conversion.
  expect_error(a + 5, "e2")
  # Only (MxS7Money, any) is registered: the reversed and unary forms are not.
  expect_error(5 + a)
  expect_error(-a)
})

test_that("Ops methods attach to base's operator, not a package generic", {
  exports <- getNamespaceExports("miniextendr")
  for (op in c("+", "*", "==", "<")) {
    expect_false(op %in% exports, info = op)
  }
  # `s7(generic = "+")` keeps the shortcut under the Rust name; `r_name = "*"`
  # has none.
  expect_equal(MxS7Money_add(MxS7Money(1), MxS7Money(2))@amount, 3)
  expect_true(MxS7Money_equals(MxS7Money(1), MxS7Money(1)))
  ns <- asNamespace("miniextendr")
  expect_false(exists("MxS7Money_*", envir = ns, inherits = FALSE))
  expect_false(exists("MxS7Money_times", envir = ns, inherits = FALSE))
})
