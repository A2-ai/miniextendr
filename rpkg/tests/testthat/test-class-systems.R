test_that("S3Counter lifecycle works", {
  c <- new_s3counter(1L)
  expect_equal(s3_value(c), 1L)
  expect_equal(s3_inc(c), 2L)
  expect_equal(s3_add(c, 5L), 7L)
  expect_equal(s3_value(s3counter_default_counter()), 0L)
})

test_that("S3Counter S3 helpers exist through method names", {
  c <- new_s3counter(2L)
  expect_equal(s3_value.S3Counter(c), 2L)
  expect_equal(s3_inc.S3Counter(c), 3L)
  expect_equal(s3_add.S3Counter(c, 4L), 7L)
})

test_that("S4Counter lifecycle works", {
  c <- S4Counter(2L)
  expect_equal(s4_value(c), 2L)
  expect_equal(s4_inc(c), 3L)
  expect_equal(s4_add(c, 4L), 7L)
  expect_equal(s4_value(S4Counter_default_counter()), 0L)
})

test_that("S7Counter lifecycle works", {
  c <- S7Counter(3L)
  expect_equal(s7_value(c), 3L)
  expect_equal(s7_inc(c), 4L)
  expect_equal(s7_add(c, 6L), 10L)
  expect_equal(s7_value(S7Counter_default_counter()), 0L)
})

test_that("ReceiverCounter env-style methods work", {
  rc <- ReceiverCounter$new(5L)
  expect_equal(rc$value(), 5L)
  expect_equal(rc$inc(), 6L)
  expect_equal(rc$add(4L), 10L)
  expect_equal(ReceiverCounter$default_counter()$value(), 0L)
})

test_that("R6Counter class works", {
  c <- R6Counter$new(10L)
  expect_equal(c$value(), 10L)
  expect_equal(c$inc(), 11L)
  expect_equal(c$add(9L), 20L)
  expect_equal(R6Counter$default_counter()$value(), 0L)
})

test_that("R6Accumulator tracks totals", {
  acc <- R6Accumulator$new()
  expect_equal(acc$total(), 0)
  expect_equal(acc$count(), 0)
  expect_equal(acc$accumulate(1.5), 1.5)
  expect_equal(acc$accumulate(2.5), 4.0)
  expect_equal(acc$count(), 2)
  expect_equal(acc$average(), 2.0)
})

test_that("Calculator defaults and methods work", {
  calc <- Calculator$new()
  expect_equal(calc$get(), 0.0)
  expect_equal(calc$add(), 1.0)
  calc$set(10.5)
  expect_equal(calc$get(), 10.5)
  expect_equal(calc$add(0.5), 11.0)
})

test_that("r6_standalone_add sums", {
  expect_equal(r6_standalone_add(2L, 3L), 5L)
})

test_that("R6Temperature active binding getters work", {
  t <- R6Temperature$new(100)  # Boiling point in Celsius
  expect_equal(t$celsius, 100)
  expect_equal(t$fahrenheit, 212)  # 100 C = 212 F
})

test_that("R6Temperature active binding setters work", {
  t <- R6Temperature$new(0)  # Freezing point in Celsius
  expect_equal(t$celsius, 0)
  expect_equal(t$fahrenheit, 32)  # 0 C = 32 F

  # Set via Celsius setter
  t$celsius <- 100
  expect_equal(t$celsius, 100)
  expect_equal(t$fahrenheit, 212)

  # Set via Fahrenheit setter
  t$fahrenheit <- 32
  expect_equal(t$fahrenheit, 32)
  expect_equal(t$celsius, 0)  # Should be back to 0 C
})

test_that("S7Range computed property (length) works", {
  r <- S7Range(0, 10)
  # Computed property - read-only via @
  expect_equal(r@length, 10)

  # Regular methods still work
  expect_equal(s7_start(r), 0)
  expect_equal(s7_end(r), 10)
})

test_that("S7Range dynamic property (midpoint) read/write works", {
  r <- S7Range(0, 10)
  # Read the midpoint
  expect_equal(r@midpoint, 5)

  # Write the midpoint - this should adjust start and end
  r@midpoint <- 10
  # New range should be centered at 10 with same length (10)
  expect_equal(r@midpoint, 10)
  expect_equal(s7_start(r), 5)  # 10 - 5
  expect_equal(s7_end(r), 15)   # 10 + 5

  # Length should still be 10
  expect_equal(r@length, 10)
})

test_that("S7Range property helpers are not user-facing generics", {
  # get_midpoint and set_midpoint are internal to the property, accessed via @midpoint.
  # They should not be exported as user-facing functions in the package namespace.
  expect_false(exists("get_midpoint", mode = "function", envir = asNamespace("miniextendr")))
  expect_false(exists("set_midpoint", mode = "function", envir = asNamespace("miniextendr")))
})

# =============================================================================
# S7 Phase 2: Property patterns (defaults, required, deprecated)
# =============================================================================

test_that("S7Config property with default value works", {
  # Create config with explicit score
  config <- S7Config("test", 75.0, 1L)
  expect_equal(config@score, 75.0)

  # Modify via setter
  config@score <- 90.0
  expect_equal(config@score, 90.0)
})

test_that("S7Config required property is accessible", {
  # The 'name' property is marked as required (no default).
  # Construction error cannot be tested from R because Rust always provides the value.
  config <- S7Config("test", 50.0, 1L)
  expect_equal(config@name, "test")
})

test_that("S7Config deprecated property emits warning", {
  config <- S7Config("test", 50.0, 42L)

  # Accessing deprecated property should emit a warning
  expect_warning(
    result <- config@old_version,
    "deprecated"
  )
  expect_equal(result, 42L)

  # Regular accessor doesn't warn
  expect_equal(get_version(config), 42L)
})

# =============================================================================
# S7 Phase 3: Generic dispatch control
# =============================================================================

test_that("S7Strict no_dots generic works", {
  strict <- S7Strict(42L)

  # strict_length should work normally
  expect_equal(strict_length(strict), 42L)

  # The generic signature should be function(x), not function(x, ...)
  # This means extra arguments would cause an error (in strict S7)
})

test_that("S7Strict describe_any method works", {
  strict <- S7Strict(123L)
  expect_equal(describe_any(strict), "S7Strict with value 123")
})

test_that("S7 fallback does not fail with slot-access error on ordinary objects", {
  # describe_any is registered for class_any (fallback). Calling it on a

  # non-S7Strict object should produce a type-conversion error from Rust,
  # NOT a raw slot-access failure like "no applicable method for `@`".
  msg <- tryCatch(
    { describe_any(1L); NA_character_ },
    error = function(e) conditionMessage(e)
  )

  # Must NOT be a slot-access error
  expect_false(grepl("no applicable method for `@`", msg, fixed = TRUE))
})

# =============================================================================
# S7 Phase 4: convert() methods - type coercion
# =============================================================================

test_that("S7 convert_from works (Celsius to Fahrenheit)", {
  c <- S7Celsius(100.0)  # Boiling point
  expect_equal(value(c), 100.0)

  # Convert using S7::convert() - uses convert_from on S7Fahrenheit
  f <- S7::convert(c, S7Fahrenheit)
  expect_equal(value(f), 212.0)  # 100C = 212F
})

test_that("S7 convert_to works (Fahrenheit to Celsius)", {
  f <- S7Fahrenheit(32.0)  # Freezing point

  # Convert using S7::convert() - uses convert_to on S7Fahrenheit
  c <- S7::convert(f, S7Celsius)
  expect_equal(value(c), 0.0)  # 32F = 0C
})

test_that("S7 bidirectional conversion works", {
  # Start with Celsius
  c1 <- S7Celsius(25.0)

  # Convert to Fahrenheit (uses convert_from on S7Fahrenheit)
  f <- S7::convert(c1, S7Fahrenheit)
  expect_equal(value(f), 77.0)  # 25C = 77F

  # Convert back to Celsius (uses convert_to on S7Fahrenheit)
  c2 <- S7::convert(f, S7Celsius)
  expect_equal(value(c2), 25.0, tolerance = 1e-10)
})

# =============================================================================
# R6 cloneable tests
# =============================================================================

test_that("R6Cloneable supports $clone()", {
  obj <- R6Cloneable$new(42L)
  expect_equal(obj$get_value(), 42L)

  # Clone the object (shallow clone - shares the same ExternalPtr)
  cloned <- obj$clone()
  expect_equal(cloned$get_value(), 42L)

  # Shallow clone shares the underlying pointer, so mutations are visible in both
  cloned$set_value(99L)
  expect_equal(cloned$get_value(), 99L)
  expect_equal(obj$get_value(), 99L)  # Same pointer, same value
})

test_that("R6Cloneable has lock_class = TRUE", {
  # With lock_class = TRUE, we cannot add new methods/fields to the class
  expect_error(R6Cloneable$set("public", "extra", function() "nope"))
})

# =============================================================================
# S7 abstract/parent tests
# =============================================================================

test_that("S7Shape is abstract and cannot be instantiated", {
  expect_error(S7Shape())
})

test_that("S7Circle has parent S7Shape", {
  c <- S7Circle(3.0)
  # Check area computation
  expect_equal(circle_area(c), pi * 9.0, tolerance = 1e-10)
})
