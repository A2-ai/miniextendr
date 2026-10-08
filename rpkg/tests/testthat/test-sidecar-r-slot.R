# RSlot sidecar fields (#1846).
#
# An `#[r_data] pub f: RSlot` field keeps its R value in the external
# pointer's protection list, after the type ID and the user slot. The pointer
# roots the value while it is reachable, and `saveRDS()` writes it with the
# pointer. Fixtures: `SidecarEnv` (`raw_slot`), `SidecarRawSexp` and the R6
# `SidecarSlotR6` in `src/rust/rdata_sidecar_tests.rs`.

test_that("an RSlot value R no longer references survives a full GC", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1, flag = TRUE, name = "x")
  SidecarEnv_set_raw_slot(obj, data.frame(a = 1:3, b = c("x", "y", "z")))
  gc(full = TRUE)
  expect_identical(
    SidecarEnv_get_raw_slot(obj),
    data.frame(a = 1:3, b = c("x", "y", "z"))
  )
})

test_that("an unset RSlot reads NULL", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1, flag = TRUE, name = "x")
  expect_null(SidecarEnv_get_raw_slot(obj))
})

test_that("RSlot values travel with saveRDS(), struct fields do not", {
  obj <- rdata_sidecar_env_new(count = 7L, score = 1, flag = TRUE, name = "x")
  SidecarEnv_set_raw_slot(obj, list(a = 1, b = "two"))
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(obj, path)
  back <- readRDS(path)

  expect_identical(SidecarEnv_get_raw_slot(back), list(a = 1, b = "two"))
  expect_error(
    SidecarEnv_get_count(back),
    "got a null external pointer",
    fixed = TRUE,
    class = "rust_error"
  )
  SidecarEnv_set_raw_slot(back, 1:2)
  expect_identical(SidecarEnv_get_raw_slot(back), 1:2)
})

test_that("an RSlot accessor refuses another type's external pointer", {
  wrong <- rdata_sidecar_s3_new(data = 1.5)
  expect_error(
    SidecarEnv_get_raw_slot(wrong),
    "expected ExternalPtr<SidecarEnv>",
    fixed = TRUE,
    class = "rust_error"
  )
  expect_error(
    SidecarEnv_set_raw_slot(wrong, 1L),
    "expected ExternalPtr<SidecarEnv>",
    fixed = TRUE,
    class = "rust_error"
  )
  expect_error(
    SidecarEnv_get_raw_slot(42L),
    "expected ExternalPtr<SidecarEnv>, got a non-external-pointer object",
    fixed = TRUE,
    class = "rust_error"
  )

  # Read back without its address, a pointer is checked by its stored type ID.
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(wrong, path)
  expect_error(
    SidecarEnv_get_raw_slot(readRDS(path)),
    "expected ExternalPtr<SidecarEnv>",
    fixed = TRUE,
    class = "rust_error"
  )
})

test_that("every RSlot of a type keeps its own value", {
  obj <- rdata_sidecar_rawsexp_new()
  f <- function(x) x + 1
  SidecarRawSexp_set_int_vec(obj, 1:3)
  SidecarRawSexp_set_func_val(obj, f)
  SidecarRawSexp_set_char_vec(obj, c("a", "b"))
  gc(full = TRUE)
  expect_identical(SidecarRawSexp_get_int_vec(obj), 1:3)
  expect_identical(SidecarRawSexp_get_func_val(obj), f)
  expect_identical(SidecarRawSexp_get_char_vec(obj), c("a", "b"))
  expect_null(SidecarRawSexp_get_list_val(obj))
})

test_that("an R6 constructor fills an RSlot from Rust and the binding reads it", {
  obj <- SidecarSlotR6$new(3L)
  expect_identical(obj$keys, 1:3)
  expect_identical(obj$n, 3L)
  expect_identical(obj$key_count(), 3L)

  obj$keys <- c(10L, 20L)
  gc(full = TRUE)
  expect_identical(obj$keys, c(10L, 20L))
  expect_identical(obj$key_count(), 2L)
})

test_that("a nested edit through the active binding writes back into the slot", {
  obj <- SidecarSlotR6$new(2L)
  obj$keys <- data.frame(id = 1:2)
  obj$keys$id[2] <- 5L
  expect_identical(obj$keys, data.frame(id = c(1L, 5L)))
})

test_that("a private RSlot keeps a value only Rust reaches", {
  obj <- SidecarSlotR6$new(1L)
  expect_null(obj$recall())
  obj$remember(list(x = 1:5))
  gc(full = TRUE)
  expect_identical(obj$recall(), list(x = 1:5))
  expect_false(exists("SidecarSlotR6_get_cache"))
})

test_that("a clone shares its original's RSlot values", {
  obj <- SidecarSlotR6$new(4L)
  obj$remember("kept")
  copy <- obj$duplicate()
  expect_identical(SidecarSlotR6_get_keys(copy), 1:4)
  expect_identical(SidecarSlotR6_get_n(copy), 4L)
})

test_that("RSlot values stay rooted under GC churn", {
  expect_null(miniextendr:::gc_stress_sidecar_r_slot())
})

test_that("RSlot values stay rooted under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    expect_null(miniextendr:::gc_stress_sidecar_r_slot())
  }
})
