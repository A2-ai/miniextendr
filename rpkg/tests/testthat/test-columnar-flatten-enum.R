# Tests for vec_to_dataframe_flatten_enums (#1056): a nested enum field is
# flattened into a `<field>_variant` tag column plus prefixed
# `<field>_<subfield>` payload columns, NA-filled where a variant lacks a field.

test_that("externally-tagged enum field flattens to variant tag + union payload", {
  df <- flatten_enum_field_fixture()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)

  # Scalar fields first, then the flattened enum columns.
  expect_true(all(c("id", "user") %in% names(df)))
  expect_true("action_variant" %in% names(df))
  expect_true(all(c("action_file", "action_weight", "action_path") %in% names(df)))

  # Tag values track the variant per row.
  expect_equal(df$action_variant, c("Add", "Init", "Add"))

  # Add rows carry file/weight; Init row NA-fills them. Init carries path;
  # Add rows NA-fill it.
  expect_equal(df$action_file, c(10.0, NA, 30.0))
  expect_equal(df$action_weight, c(2.5, NA, 4.0))
  expect_equal(df$action_path, c(NA, "/tmp", NA))

  # Scalar columns are untouched.
  expect_equal(df$id, c(1L, 2L, 3L))
  expect_equal(df$user, c("alice", "bob", "carol"))
})

test_that("unit variant gets the tag with NA payload", {
  df <- flatten_enum_unit_variant_fixture()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_true("event_variant" %in% names(df))
  expect_true("event_value" %in% names(df))

  expect_equal(df$event_variant, c("Set", "Reset"))
  # Set row has the value; the unit Reset row is NA.
  expect_equal(df$event_value, c(7.0, NA))
})

test_that("Option<Enum> None row NA-fills tag and payload", {
  df <- flatten_enum_option_none_fixture()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_true("action_variant" %in% names(df))

  # The present row carries the variant + payload; the None row is all-NA.
  expect_equal(df$action_variant, c("Add", NA))
  expect_equal(df$action_file, c(5.0, NA))
  expect_equal(df$action_weight, c(1.0, NA))
})
