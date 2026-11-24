test_that("s4_is_s4 detects S4 objects", {
  setClass("TestS4Point", representation(x = "numeric", y = "numeric"))
  obj <- new("TestS4Point", x = 1, y = 2)
  expect_true(s4_is_s4(obj))
  expect_false(s4_is_s4(42))
  expect_false(s4_is_s4("hello"))
  removeClass("TestS4Point")
})

test_that("s4_has_slot_test checks slot existence", {
  setClass("TestS4Slots", representation(name = "character", value = "numeric"))
  obj <- new("TestS4Slots", name = "test", value = 42)
  expect_true(s4_has_slot_test(obj, "name"))
  expect_true(s4_has_slot_test(obj, "value"))
  expect_false(s4_has_slot_test(obj, "nonexistent"))
  removeClass("TestS4Slots")
})

test_that("s4_class_name_test returns correct class", {
  setClass("TestS4Named", representation(x = "numeric"))
  obj <- new("TestS4Named", x = 1)
  expect_equal(s4_class_name_test(obj), "TestS4Named")
  removeClass("TestS4Named")
})

test_that("s4_get_slot_test reads a slot and refuses a missing one", {
  setClass("TestS4Get", representation(x = "numeric"))
  on.exit(removeClass("TestS4Get"), add = TRUE)
  obj <- new("TestS4Get", x = 3)
  expect_identical(s4_get_slot_test(obj, "x"), 3)
  expect_error(s4_get_slot_test(obj, "nope"), "slot() failed: ", fixed = TRUE)
})

test_that("s4_with_slot_test returns a copy and leaves the argument unchanged", {
  setClass("TestS4Set", representation(label = "character"))
  setClass("TestS4NumBox", contains = "numeric", representation(label = "character"))
  on.exit({
    removeClass("TestS4NumBox")
    removeClass("TestS4Set")
  }, add = TRUE)
  obj <- new("TestS4Set", label = "a")
  out <- s4_with_slot_test(obj, "label", "z")
  expect_identical(out@label, "z")
  expect_identical(obj@label, "a")
  # `slot<-` returns a new object for `.Data` instead of changing its argument.
  box <- new("TestS4NumBox", c(1, 2), label = "b")
  out <- s4_with_slot_test(box, ".Data", c(9, 9, 9))
  expect_identical(out@.Data, c(9, 9, 9))
  expect_identical(out@label, "b")
  expect_identical(box@.Data, c(1, 2))
})

test_that("The caller's handlers see a warning from slot<-'s coercion (#1840)", {
  # `slot<-` coerces a value of a class that `setIs()` relates to the slot's
  # class with that relation's `coerce`, the user code it runs.
  setClass("TestS4Target", representation(v = "numeric"))
  setClass("TestS4Source", representation(n = "numeric"))
  setIs(
    "TestS4Source", "TestS4Target",
    coerce = function(from) {
      warning("coercing source to target")
      new("TestS4Target", v = from@n)
    },
    replace = function(from, value) {
      from@n <- value@v
      from
    }
  )
  setClass("TestS4Holder", representation(a = "TestS4Target"))
  on.exit({
    removeClass("TestS4Holder")
    removeClass("TestS4Source")
    removeClass("TestS4Target")
  }, add = TRUE)
  fresh <- function() new("TestS4Holder", a = new("TestS4Target", v = 1))
  src <- new("TestS4Source", n = 2)

  expect_no_warning(out <- suppressWarnings(s4_with_slot_test(fresh(), "a", src)))
  expect_identical(out@a@v, 2)
  expect_identical(
    tryCatch(s4_with_slot_test(fresh(), "a", src), warning = function(w) "caught"),
    "caught"
  )
  expect_warning(s4_with_slot_test(fresh(), "a", src), "coercing source to target", fixed = TRUE)
  # An invalid value is still R's error, raised again by the fixture.
  expect_error(s4_with_slot_test(fresh(), "a", "text"), "slot<-() failed: ", fixed = TRUE)
})
