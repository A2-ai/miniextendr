# Test RNative derive macro for newtype wrappers

test_that("RNative derive works for tuple struct (UserId)", {
  # UserId(i32) newtype should work with Coerce
  expect_equal(test_rnative_newtype(42L), 42L)
  expect_equal(test_rnative_newtype(0L), 0L)
  expect_equal(test_rnative_newtype(-100L), -100L)
})

test_that("RNative derive works for named-field struct (Temperature)", {
  # Temperature { celsius: f64 } newtype should work with Coerce
  expect_equal(test_rnative_named_field(98.6), 98.6)
  expect_equal(test_rnative_named_field(0.0), 0.0)
  expect_equal(test_rnative_named_field(-40.0), -40.0)
})
