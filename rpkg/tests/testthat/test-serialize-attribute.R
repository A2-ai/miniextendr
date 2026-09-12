test_that("serialize matches AsSerialize for serde-only records and early returns", {
  skip_if_missing_feature("serde")
  expected <- list(value = 4L, label = "value-4")
  expect_identical(serialized_record_attr(4L), expected)
  expect_identical(serialized_record_attr(4L), serialized_record_type(4L))
  expect_identical(serialized_record_early(-4L), expected)
  expect_identical(serialized_record_early(4L), expected)
})

test_that("serialize wraps the whole Result or Option just like the type spelling", {
  skip_if_missing_feature("serde")
  for (flag in c(FALSE, TRUE)) {
    expect_identical(serialized_result_attr(flag), serialized_result_type(flag))
    expect_identical(serialized_option_attr(flag), serialized_option_type(flag))
  }
  expect_identical(serialized_result_attr(TRUE), list(Err = "example failure"))
  expect_null(serialized_option_attr(FALSE))
  expect_error(serialized_result_payload(TRUE), "example failure")
  expect_identical(serialized_result_payload(FALSE),
                   list(value = 7L, label = "value-7"))
})

test_that("serialization composes with visibility and unit returns", {
  skip_if_missing_feature("serde")
  expect_identical(withVisible(serialized_invisible_attr()),
                   withVisible(serialized_invisible_type()))
  expect_false(withVisible(serialized_invisible_attr())$visible)
  expect_identical(withVisible(serialized_unit_attr()),
                   withVisible(serialized_unit_type()))
  expect_null(serialized_unit_attr())
})

test_that("serialized methods bypass class wrapping and preserve trait transport", {
  skip_if_missing_feature("serde")
  obj <- SerializeHost$new(5L)
  expect_identical(SerializeHost$snapshot(obj), SerializeHost$snapshot_type(obj))
  expect_identical(SerializeHost$self_data(obj), list(value = 5L))
  expect_identical(SerializeHost$SerializeValues$values(obj), c(5L, 6L))
  expect_identical(serialized_trait_view(obj), c(5L, 6L))
  expect_identical(SerializeHost$consume(obj), list(value = 5L))
  expect_error(SerializeHost$snapshot(obj), "released|NULL|null|consumed")
})

test_that("serializer errors stay classed Rust errors", {
  skip_if_missing_feature("serde")
  err <- tryCatch(serialized_failure_attr(), error = identity)
  expect_s3_class(err, "rust_error")
  expect_match(conditionMessage(err), "intentional serialization failure", fixed = TRUE)
})

test_that("serialize converts worker results on the R thread", {
  skip_if_missing_feature("serde")
  skip_if_missing_feature("worker-thread")
  expect_identical(serialized_record_worker(4L), serialized_record_type(4L))
})

test_that("the serialized return boundary survives GC stress", {
  skip_if_missing_feature("serde")
  skip_on_cran()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  expect_identical(miniextendr:::gc_stress_serialized_return(),
                   list(value = 41L, label = "value-41"))
})
