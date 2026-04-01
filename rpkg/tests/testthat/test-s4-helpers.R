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
