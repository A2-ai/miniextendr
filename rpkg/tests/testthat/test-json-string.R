# Tests for AsJson / FromJson JSON string adapters

# region: AsJson — compact JSON output

test_that("AsJson serializes struct to compact JSON string", {
  result <- test_json_point()
  expect_type(result, "character")
  expect_length(result, 1)
  parsed <- jsonlite::fromJSON(result)
  expect_equal(parsed$x, 1.5)
  expect_equal(parsed$y, 2.5)
})

test_that("AsJson serializes config to JSON string", {
  result <- test_json_config()
  parsed <- jsonlite::fromJSON(result)
  expect_equal(parsed$max_threads, 4)
  expect_equal(parsed$name, "test")
})

# endregion

# region: AsJsonPretty — pretty-printed JSON

test_that("AsJsonPretty produces indented JSON", {
  result <- test_json_pretty_point()
  expect_type(result, "character")
  # Pretty JSON has newlines
  expect_true(grepl("\n", result))
  parsed <- jsonlite::fromJSON(result)
  expect_equal(parsed$x, 1)
  expect_equal(parsed$y, 2)
})

# endregion

# region: FromJson — parse JSON from R character

test_that("FromJson parses JSON config", {
  result <- test_fromjson_config('{"max_threads": 8, "name": "prod"}')
  expect_equal(result, 8L)
})

test_that("FromJson parses JSON point", {
  result <- test_fromjson_point_sum('{"x": 10.5, "y": 20.5}')
  expect_equal(result, 31.0)
})

test_that("FromJson errors on invalid JSON", {
  expect_error(test_fromjson_bad("not json at all"))
})

test_that("FromJson errors on wrong schema", {
  expect_error(test_fromjson_bad('{"wrong_field": 1}'))
})

# endregion

# region: AsJsonVec — Vec<T> → character vector of JSON strings

test_that("AsJsonVec produces character vector of JSON strings", {
  result <- test_json_vec_points()
  expect_type(result, "character")
  expect_length(result, 2)

  p1 <- jsonlite::fromJSON(result[1])
  expect_equal(p1$x, 1)
  expect_equal(p1$y, 2)

  p2 <- jsonlite::fromJSON(result[2])
  expect_equal(p2$x, 3)
  expect_equal(p2$y, 4)
})

# endregion
