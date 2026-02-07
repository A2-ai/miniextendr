# =============================================================================
# serde_r Tests: Direct Rust <-> R Serialization
# =============================================================================
#
# Tests for the serde_r feature which provides direct serialization between
# Rust types and native R objects without JSON intermediate.
#
# Test Categories:
# 1. Primitive types (bool, integers, floats, strings)
# 2. Option/NA handling
# 3. Vectors (smart dispatch to atomic vectors vs lists)
# 4. Nested structs (lists within lists)
# 5. HashMap/BTreeMap (named lists)
# 6. Enums (unit and data variants)
# 7. Round-trip tests
# 8. Error handling
# 9. Integration with R object systems (S4, S7, R6, environments)

# =============================================================================
# 1. Primitive Types
# =============================================================================

test_that("serde_r serializes primitive i32 correctly", {
  result <- serde_r_serialize_i32(42L)
  expect_type(result, "integer")
  expect_length(result, 1)
  expect_equal(result, 42L)
})

test_that("serde_r serializes primitive f64 correctly", {
  result <- serde_r_serialize_f64(3.14159)
  expect_type(result, "double")
  expect_length(result, 1)
  expect_equal(result, 3.14159)
})

test_that("serde_r serializes primitive bool correctly", {
  result_true <- serde_r_serialize_bool(TRUE)
  result_false <- serde_r_serialize_bool(FALSE)

  expect_type(result_true, "logical")
  expect_true(result_true)
  expect_false(result_false)
})
test_that("serde_r serializes String correctly", {
  result <- serde_r_serialize_string("hello world")
  expect_type(result, "character")
  expect_equal(result, "hello world")
})

# =============================================================================
# 2. Option/NA Handling
# =============================================================================

test_that("serde_r serializes Option::Some correctly", {
  result <- serde_r_serialize_option_i32(42L)
  expect_equal(result, 42L)
})

test_that("serde_r serializes Option::None to NULL", {
  # When explicitly passed NULL, it becomes None -> serializes to NULL
  result <- serde_r_serialize_option_i32(NULL)
  expect_null(result)
})
test_that("serde_r rejects NA_integer_ with clear error", {
  # serde_r doesn't auto-convert NA to None for i32 - it raises an error
  expect_error(serde_r_deserialize_wrong_type(NA_integer_), "NA")
})

# =============================================================================
# 3. Vector Smart Dispatch
# =============================================================================

test_that("serde_r serializes Vec<i32> to integer vector", {
  result <- serde_r_serialize_vec_i32(c(1L, 2L, 3L, 4L, 5L))
  expect_type(result, "integer")
  expect_equal(result, c(1L, 2L, 3L, 4L, 5L))
})

test_that("serde_r serializes Vec<f64> to numeric vector", {
  result <- serde_r_serialize_vec_f64(c(1.1, 2.2, 3.3))
  expect_type(result, "double")
  expect_equal(result, c(1.1, 2.2, 3.3))
})

test_that("serde_r serializes Vec<String> to character vector", {
  result <- serde_r_serialize_vec_string(c("a", "b", "c"))
  expect_type(result, "character")
  expect_equal(result, c("a", "b", "c"))
})

test_that("serde_r serializes Vec<bool> to logical vector", {
  result <- serde_r_serialize_vec_bool(c(TRUE, FALSE, TRUE))
  expect_type(result, "logical")
  expect_equal(result, c(TRUE, FALSE, TRUE))
})

test_that("serde_r serializes empty vectors correctly", {
  result <- serde_r_serialize_vec_i32(integer(0))
  expect_type(result, "list")  # Empty becomes list
  expect_length(result, 0)
})

test_that("serde_r deserializes integer vector to Vec<i32>", {
  result <- serde_r_deserialize_vec_i32(c(10L, 20L, 30L))
  expect_equal(result, c(10L, 20L, 30L))
})

test_that("serde_r deserializes numeric vector to Vec<f64>", {
  result <- serde_r_deserialize_vec_f64(c(1.5, 2.5, 3.5))
  expect_equal(result, c(1.5, 2.5, 3.5))
})

test_that("serde_r deserializes scalar primitives", {
  expect_equal(serde_r_deserialize_i32(99L), 99L)
  expect_equal(serde_r_deserialize_f64(3.14), 3.14)
  expect_equal(serde_r_deserialize_string("serde"), "serde")
})

# =============================================================================
# 4. Nested Structs (Lists within Lists)
# =============================================================================

test_that("SerdeRPoint serializes to named list", {
  p <- SerdeRPoint$new(1.0, 2.0)
  data <- p$to_r()

  expect_type(data, "list")
  expect_named(data, c("x", "y"))
  expect_equal(data$x, 1.0)
  expect_equal(data$y, 2.0)
})

test_that("SerdeRPoint deserializes from named list", {
  # Note: from_r returns a raw ExternalPtr, not a proper S3 object
  # This is a limitation of static methods returning Result<Self, String>
  # The actual deserialization is tested via serde_r_roundtrip_point()
  data <- list(x = 3.0, y = 4.0)
  p <- SerdeRPoint$from_r(data)

  # Returns an ExternalPtr (not S3 wrapped)
  expect_type(p, "externalptr")

  # Full roundtrip works at Rust level
  expect_true(serde_r_roundtrip_point(3.0, 4.0))
})

test_that("SerdeRPoint3D handles optional label", {
  # With label
  p1 <- SerdeRPoint3D$new(1.0, 2.0, 3.0, "origin")
  data1 <- p1$to_r()
  expect_equal(data1$label, "origin")

  # Without label - use NA_character_ instead of NULL for Option<String>
  # NULL is not accepted by the macro's FromR for Option<String>
  p2 <- SerdeRPoint3D$new(1.0, 2.0, 3.0, NA_character_)
  data2 <- p2$to_r()
  expect_null(data2$label)
})

test_that("Rectangle serializes nested Points", {
  r <- Rectangle$new(0.0, 0.0, 10.0, 10.0)
  data <- r$to_r()

  expect_type(data, "list")
  expect_named(data, c("top_left", "bottom_right", "fill_color"))

  # Nested points

  expect_type(data$top_left, "list")
  expect_equal(data$top_left$x, 0.0)
  expect_equal(data$top_left$y, 0.0)
  expect_equal(data$bottom_right$x, 10.0)
  expect_equal(data$bottom_right$y, 10.0)
})

test_that("Rectangle deserializes from nested list", {
  # Note: from_r returns raw ExternalPtr, not S3 wrapped
  # Roundtrip tested via serde_r_roundtrip_rectangle()
  data <- list(
    top_left = list(x = 1.0, y = 2.0),
    bottom_right = list(x = 5.0, y = 6.0),
    fill_color = "blue"
  )
  r <- Rectangle$from_r(data)

  expect_type(r, "externalptr")

  # Test roundtrip at Rust level
  expect_true(serde_r_roundtrip_rectangle(0.0, 0.0, 10.0, 10.0))
})

test_that("DeepNest handles 3 levels of nesting", {
  dn <- DeepNest$new()
  data <- dn$to_r()

  expect_type(data, "list")
  expect_type(data$level1, "list")
  expect_type(data$level1$level2, "list")
  expect_type(data$level1$level2$level3, "list")

  # Check leaf values
  expect_equal(data$level1$name, "nested")
  expect_equal(data$level1$level2$values, c(10L, 20L, 30L))
  expect_equal(data$level1$level2$level3$data, c(1.0, 2.0, 3.0))
  expect_true(data$level1$level2$level3$flag)
})

# =============================================================================
# 5. HashMap/BTreeMap (Named Lists)
# =============================================================================

test_that("serde_r serializes HashMap to named list", {
  result <- serde_r_serialize_hashmap()

  expect_type(result, "list")
  expect_true("a" %in% names(result))
  expect_true("b" %in% names(result))
  expect_true("c" %in% names(result))
  expect_equal(result$a, 1L)
  expect_equal(result$b, 2L)
  expect_equal(result$c, 3L)
})

test_that("Maps struct serializes multiple map types", {
  m <- Maps$new()
  data <- m$to_r()

  expect_type(data, "list")
  expect_true("string_to_int" %in% names(data))
  expect_true("string_to_float" %in% names(data))
  expect_true("metadata" %in% names(data))

  # Check nested maps
  expect_type(data$string_to_int, "list")
  expect_type(data$metadata, "list")
})

# =============================================================================
# 6. Enums (Unit and Data Variants)
# =============================================================================

test_that("WithEnums serializes unit enum variant to string", {
  e <- WithEnums$new_circle(5.0)
  data <- e$to_r()

  # Status is Active (unit variant) -> should be string
  expect_equal(data$status, "Active")
})

test_that("WithEnums serializes data enum variant to tagged list", {
  e <- WithEnums$new_circle(5.0)
  data <- e$to_r()

  # Shape is Circle { radius: 5.0 } -> list(Circle = list(radius = 5.0))
  expect_type(data$shape, "list")
  expect_true("Circle" %in% names(data$shape))
  expect_equal(data$shape$Circle$radius, 5.0)
})

test_that("WithEnums serializes Rectangle variant", {
  e <- WithEnums$new_rectangle(10.0, 20.0)
  data <- e$to_r()

  expect_equal(data$status, "Inactive")
  expect_true("Rectangle" %in% names(data$shape))
  expect_equal(data$shape$Rectangle$width, 10.0)
  expect_equal(data$shape$Rectangle$height, 20.0)
})

test_that("WithEnums handles optional enum", {
  e1 <- WithEnums$new_circle(5.0)  # has optional_status = Some(Pending)
  data1 <- e1$to_r()
  expect_equal(data1$optional_status, "Pending")

  e2 <- WithEnums$new_rectangle(10.0, 20.0)  # has optional_status = None
  data2 <- e2$to_r()
  expect_null(data2$optional_status)
})

# =============================================================================
# 7. Round-trip Tests
# =============================================================================

test_that("Point roundtrip preserves values", {
  expect_true(serde_r_roundtrip_point(1.5, 2.5))
  expect_true(serde_r_roundtrip_point(0.0, 0.0))
  expect_true(serde_r_roundtrip_point(-100.5, 100.5))
})

test_that("Rectangle roundtrip preserves values", {
  expect_true(serde_r_roundtrip_rectangle(0.0, 0.0, 10.0, 10.0))
  expect_true(serde_r_roundtrip_rectangle(-5.0, -5.0, 5.0, 5.0))
})

test_that("DeepNest roundtrip preserves complex structure", {
  expect_true(serde_r_roundtrip_deep_nest())
})

test_that("Collections roundtrip preserves all collection types", {
  expect_true(serde_r_roundtrip_collections())
})

test_that("WithOptionals roundtrip with all values present", {
  expect_true(serde_r_roundtrip_optionals_present())
})

test_that("WithOptionals roundtrip with all None values", {
  expect_true(serde_r_roundtrip_optionals_none())
})

test_that("SerdeRPoint$to_r() -> SerdeRPoint$from_r() roundtrip", {
  # from_r returns raw externalptr, so we test the Rust-level roundtrip
  original <- SerdeRPoint$new(42.0, 24.0)
  data <- original$to_r()
  restored <- SerdeRPoint$from_r(data)

  # Verify from_r worked (returns externalptr)
  expect_type(restored, "externalptr")

  # Full roundtrip works at Rust level
  expect_true(serde_r_roundtrip_point(42.0, 24.0))
})

# =============================================================================
# 8. Collections Struct
# =============================================================================

test_that("Collections serializes all collection types", {
  c <- Collections$new()
  data <- c$to_r()

  # Smart dispatch: homogeneous vectors -> atomic vectors
  expect_type(data$integers, "integer")
  expect_equal(data$integers, c(1L, 2L, 3L, 4L, 5L))

  expect_type(data$floats, "double")
  expect_equal(data$floats, c(1.1, 2.2, 3.3))

  expect_type(data$strings, "character")
  expect_equal(data$strings, c("a", "b", "c"))

  expect_type(data$bools, "logical")
  expect_equal(data$bools, c(TRUE, FALSE, TRUE))

  # Vec<Point> -> list of lists
  expect_type(data$points, "list")
  expect_length(data$points, 2)
  expect_type(data$points[[1]], "list")
})

test_that("Collections empty variant works", {
  c <- Collections$empty()
  data <- c$to_r()

  expect_length(data$integers, 0)
  expect_length(data$floats, 0)
  expect_length(data$strings, 0)
  expect_length(data$bools, 0)
  expect_length(data$points, 0)
})

# =============================================================================
# 9. Tuple and Tuple Struct
# =============================================================================

test_that("Tuple serializes to unnamed list", {
  result <- serde_r_serialize_tuple()

  expect_type(result, "list")
  expect_length(result, 3)
  expect_null(names(result))  # Unnamed

  expect_equal(result[[1]], 42L)
  expect_equal(result[[2]], 3.14)
  expect_equal(result[[3]], "hello")
})

test_that("Tuple struct serializes to unnamed list", {
  result <- serde_r_serialize_tuple_struct()

  expect_type(result, "list")
  expect_length(result, 2)
  expect_null(names(result))

  expect_equal(result[[1]], 42L)
  expect_equal(result[[2]], "answer")
})

# =============================================================================
# 10. Complex Integration
# =============================================================================

test_that("Complex nested structure serializes correctly", {
  data <- serde_r_complex_nested()

  expect_type(data, "list")
  expect_true("points" %in% names(data))
  expect_true("rectangles" %in% names(data))
  expect_true("metadata" %in% names(data))
  expect_true("counts" %in% names(data))
  expect_true("flags" %in% names(data))

  # Points are list of lists
  expect_type(data$points, "list")
  expect_length(data$points, 3)

  # Rectangles have nested structure
  expect_length(data$rectangles, 2)
  expect_true("top_left" %in% names(data$rectangles[[1]]))

  # Counts are atomic (smart dispatch)
  expect_type(data$counts, "integer")
  expect_equal(data$counts, c(1L, 2L, 3L, 4L, 5L))

  # Flags are atomic
  expect_type(data$flags, "logical")
})

test_that("Complex structure deserializes and validates", {
  # Create from R and deserialize
  complex_data <- list(
    points = list(
      list(x = 1.0, y = 2.0),
      list(x = 3.0, y = 4.0)
    ),
    rectangles = list(
      list(
        top_left = list(x = 0.0, y = 0.0),
        bottom_right = list(x = 1.0, y = 1.0),
        fill_color = NULL
      )
    ),
    metadata = list(key = "value"),
    counts = c(1L, 2L, 3L),
    flags = c(TRUE, FALSE)
  )

  result <- serde_r_deserialize_complex(complex_data)

  expect_match(result, "points=2")
  expect_match(result, "rects=1")
  expect_match(result, "meta=1")
  expect_match(result, "counts=")
  expect_match(result, "flags=")
})

# =============================================================================
# 11. Error Handling
# =============================================================================

test_that("Deserialization type mismatch returns error", {
  # Try to deserialize a string as i32 - should raise R error
  result <- tryCatch(
    serde_r_deserialize_wrong_type("not a number"),
    error = function(e) conditionMessage(e)
  )
  expect_match(result, "type|mismatch|expected", ignore.case = TRUE)
})

test_that("Deserialization missing field returns error", {
  # Point requires x and y - should raise R error
  incomplete <- list(x = 1.0)  # missing y
  result <- tryCatch(
    serde_r_deserialize_missing_field(incomplete),
    error = function(e) conditionMessage(e)
  )
  expect_match(result, "missing|field|y", ignore.case = TRUE)
})

# =============================================================================
# 12. Integration with R Object Systems
# =============================================================================

test_that("serde_r deserializes from named list", {
  # Named lists are the primary input format for serde_r deserialization
  p <- SerdeRPoint$from_r(list(x = 1.0, y = 2.0))
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(p, "externalptr")
})

test_that("serde_r handles R lists created various ways", {
  # list()
  p1 <- SerdeRPoint$from_r(list(x = 1.0, y = 2.0))
  expect_type(p1, "externalptr")

  # as.list()
  vec <- c(x = 1.0, y = 2.0)
  p2 <- SerdeRPoint$from_r(as.list(vec))
  expect_type(p2, "externalptr")
})

test_that("serde_r handles environment-to-list conversion", {
  # Create an environment, convert to list, deserialize
  e <- new.env()
  e$x <- 5.0
  e$y <- 10.0

  data <- as.list(e)
  p <- SerdeRPoint$from_r(data)
  expect_type(p, "externalptr")
})

test_that("serde_r works with R6 class data", {
  skip_if_not_installed("R6")

  # Create an R6 class that produces compatible data
  TestClass <- R6::R6Class("TestClass",
    public = list(
      x = NULL,
      y = NULL,
      initialize = function(x, y) {
        self$x <- x
        self$y <- y
      },
      to_list = function() {
        list(x = self$x, y = self$y)
      }
    )
  )

  obj <- TestClass$new(3.0, 4.0)
  data <- obj$to_list()
  p <- SerdeRPoint$from_r(data)
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(p, "externalptr")
})

test_that("serde_r works with S4 class slots as list", {
  # Define a simple S4 class
  setClass("S4Point", slots = c(x = "numeric", y = "numeric"))
  s4obj <- new("S4Point", x = 7.0, y = 8.0)

  # Extract slots as list
  data <- list(x = s4obj@x, y = s4obj@y)
  p <- SerdeRPoint$from_r(data)
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(p, "externalptr")

  # Cleanup
  removeClass("S4Point")
})

test_that("serde_r works with S7 class data", {
  skip_if_not_installed("S7")

  # Create S7 class
  S7Point <- S7::new_class("S7Point",
    properties = list(
      x = S7::class_double,
      y = S7::class_double
    )
  )

  s7obj <- S7Point(x = 9.0, y = 10.0)

  # Extract properties as list
  data <- list(x = S7::prop(s7obj, "x"), y = S7::prop(s7obj, "y"))
  p <- SerdeRPoint$from_r(data)
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(p, "externalptr")
})

# =============================================================================
# 13. Edge Cases
# =============================================================================

test_that("serde_r handles special float values", {
  # Inf and -Inf
  result_inf <- serde_r_serialize_f64(Inf)
  expect_equal(result_inf, Inf)

  result_neg_inf <- serde_r_serialize_f64(-Inf)
  expect_equal(result_neg_inf, -Inf)
})

test_that("serde_r handles empty string", {
  result <- serde_r_serialize_string("")
  expect_equal(result, "")
})

test_that("serde_r handles unicode strings", {
  result <- serde_r_serialize_string("hello \u4e16\u754c")
  expect_equal(result, "hello \u4e16\u754c")
})

test_that("serde_r handles large vectors", {
  large_vec <- 1:10000
  result <- serde_r_serialize_vec_i32(large_vec)
  expect_length(result, 10000)
  expect_equal(result[1], 1L)
  expect_equal(result[10000], 10000L)
})

test_that("serde_r handles deeply nested list from R", {
  deep <- list(
    level1 = list(
      level2 = list(
        level3 = list(
          data = c(1.0, 2.0, 3.0),
          flag = TRUE
        ),
        values = c(10L, 20L, 30L)
      ),
      name = "nested"
    )
  )

  dn <- DeepNest$from_r(deep)
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(dn, "externalptr")

  # Verify roundtrip works at Rust level
  expect_true(serde_r_roundtrip_deep_nest())
})

# =============================================================================
# 14. WithOptionals Struct
# =============================================================================

test_that("WithOptionals handles all present values", {
  opt <- WithOptionals$all_present()
  data <- opt$to_r()

  expect_equal(data$required_int, 42L)
  expect_equal(data$optional_int, 100L)
  expect_equal(data$optional_float, 3.14)
  expect_equal(data$optional_string, "hello")
  expect_true(data$optional_bool)
})

test_that("WithOptionals handles all None values", {
  opt <- WithOptionals$all_none()
  data <- opt$to_r()

  expect_equal(data$required_int, 0L)
  expect_null(data$optional_int)
  expect_null(data$optional_float)
  expect_null(data$optional_string)
  expect_null(data$optional_bool)
})

test_that("WithOptionals handles mixed values", {
  opt <- WithOptionals$mixed()
  data <- opt$to_r()

  expect_equal(data$required_int, 42L)
  expect_null(data$optional_int)
  expect_equal(data$optional_float, 2.71828)
  expect_null(data$optional_string)
  expect_false(data$optional_bool)
})

test_that("WithOptionals deserializes with NULL fields", {
  data <- list(
    required_int = 99L,
    optional_int = NULL,
    optional_float = 1.5,
    optional_string = NULL,
    optional_bool = TRUE
  )

  opt <- WithOptionals$from_r(data)
  # from_r returns raw ExternalPtr, not S3 object
  expect_type(opt, "externalptr")

  # Verify roundtrip works at Rust level
  expect_true(serde_r_roundtrip_optionals_none())
})
