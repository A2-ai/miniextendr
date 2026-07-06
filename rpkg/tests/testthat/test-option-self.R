# Option<Self> static returns wrap as class objects (#1164).
#
# `OptionSelfLookup$try_find()` returns `Option<Self>` from Rust — the
# lookup-shaped fallible constructor. `Some(Self)` must come back as a usable
# class object exactly like `$new()` (not a bare ExternalPtr), and `None`
# raises an R error via the normal Option error path. These assertions mirror
# the `Result<Self, String>` coverage in test-serde_r.R (audit A4).

test_that("try_find(Some) wraps its Option<Self> exactly like $new() does", {
  found <- OptionSelfLookup$try_find(1L)

  expect_s3_class(found, "OptionSelfLookup")
  expect_equal(found$id(), 1L)
  expect_equal(found$label(), "one")

  # Identical in shape to a directly constructed object.
  made <- OptionSelfLookup$new(1L, "one")
  expect_s3_class(made, "OptionSelfLookup")
  expect_equal(class(found), class(made))
  expect_equal(found$id(), made$id())
  expect_equal(found$label(), made$label())
})

test_that("try_find returns a usable class object for every known id", {
  for (id in 1:3) {
    entry <- OptionSelfLookup$try_find(id)
    expect_s3_class(entry, "OptionSelfLookup")
    expect_equal(entry$id(), id)
  }
  expect_equal(OptionSelfLookup$try_find(2L)$label(), "two")
  expect_equal(OptionSelfLookup$try_find(3L)$label(), "three")
})

test_that("try_find(None) raises an error, not a value", {
  expect_error(OptionSelfLookup$try_find(42L), "returned no value")
  expect_error(OptionSelfLookup$try_find(-1L), "returned no value")
})
