# Typed `Sidecar<T>` fields (#1846, #1855).
#
# An `#[r_data] pub f: Sidecar<T>` field keeps its value in the external
# pointer's protection list, after the type ID and the user slot. The pointer
# roots the value while it is reachable, and `saveRDS()` writes it with the
# pointer. Rust reads and writes it through the typed accessors the derive
# generates (`self.f()` / `self.set_f(v)`); the struct holds only a
# back-reference, which the handle rewrites whenever it hands the struct out.
# Fixtures: `SidecarEnv` (`raw_slot: Sidecar<SEXP>`), `SidecarRawSexp`, the R6
# `SidecarSlotR6` and `SidecarNest` in `src/rust/rdata_sidecar_tests.rs`.

test_that("a sidecar value R no longer references survives a full GC", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1, flag = TRUE, name = "x")
  SidecarEnv_set_raw_slot(obj, data.frame(a = 1:3, b = c("x", "y", "z")))
  gc(full = TRUE)
  expect_identical(
    SidecarEnv_get_raw_slot(obj),
    data.frame(a = 1:3, b = c("x", "y", "z"))
  )
})

test_that("a Sidecar<SEXP> initialised with NULL reads NULL", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1, flag = TRUE, name = "x")
  expect_null(SidecarEnv_get_raw_slot(obj))
})

test_that("sidecar values travel with saveRDS(), struct fields do not", {
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

test_that("a sidecar accessor refuses another type's external pointer", {
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

test_that("every sidecar field of a type keeps its own value", {
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

test_that("a plain constructor's initial values arrive in the protection list", {
  obj <- SidecarSlotR6$new(3L)
  expect_identical(obj$keys, 1:3)
  expect_identical(obj$n, 3L)
  expect_identical(obj$label, "fresh")
  expect_identical(obj$note, "")
  expect_identical(obj$key_count(), 3L)
  expect_identical(SidecarSlotR6_get_keys(obj$.__enclos_env__$private$.ptr), 1:3)
})

test_that("a Rust getter sees an R-side set, and an R getter a Rust-side set", {
  obj <- SidecarSlotR6$new(3L)

  obj$keys <- c(10L, 20L)
  gc(full = TRUE)
  expect_identical(obj$keys, c(10L, 20L))
  expect_identical(obj$key_count(), 2L)

  obj$push_key(30L)
  gc(full = TRUE)
  expect_identical(obj$keys, c(10L, 20L, 30L))
  expect_identical(obj$key_count(), 3L)
})

test_that("ref-only and mut-only fields keep both R accessors", {
  obj <- SidecarSlotR6$new(1L)
  # `label` is `ref` in Rust: R still writes it, Rust reads it.
  obj$label <- "renamed"
  expect_identical(obj$label_len(), 7L)
  expect_identical(obj$label, "renamed")
  # `note` is `mut` in Rust: Rust writes it, R reads (and writes) it.
  obj$write_note("from Rust")
  expect_identical(obj$note, "from Rust")
  obj$note <- "from R"
  expect_identical(obj$note, "from R")
})

test_that("the R setter validates the value as the field's type", {
  obj <- SidecarSlotR6$new(2L)
  expect_error(obj$keys <- "a", "'keys' must be integer", fixed = TRUE)
  expect_identical(obj$keys, 1:2)
  expect_error(
    SidecarSlotR6_set_label(obj$.__enclos_env__$private$.ptr, 1:3),
    "'label' must be",
    fixed = TRUE
  )
})

test_that("a nested edit through the active binding writes back into the slot", {
  obj <- SidecarSlotR6$new(2L)
  obj$keys <- c(1L, 2L)
  obj$keys[2] <- 5L
  expect_identical(obj$keys, c(1L, 5L))
})

test_that("a private Sidecar<Option<List>> reads NULL until set and keeps a value only Rust reaches", {
  obj <- SidecarSlotR6$new(1L)
  expect_null(obj$recall())
  obj$remember(list(x = 1:5))
  gc(full = TRUE)
  expect_identical(obj$recall(), list(x = 1:5))
  expect_false(exists("SidecarSlotR6_get_cache"))
  expect_false("cache" %in% names(obj))
})

test_that("replacing the value through DerefMut takes effect on both sides", {
  obj <- SidecarSlotR6$new(3L)
  obj$remember(list("old"))
  obj$reset(5L)
  expect_identical(obj$keys, 1:5)
  expect_identical(obj$key_count(), 5L)
  expect_identical(obj$n, 5L)
  expect_null(obj$recall())
})

test_that("a by-value self method writes the value and its sidecar values back", {
  obj <- SidecarSlotR6$new(3L)
  obj$remember(list(kept = TRUE))
  obj$label <- "before"
  obj$rebuilt(9L)
  gc(full = TRUE)
  expect_identical(obj$keys, c(1:3, 9L))
  expect_identical(obj$n, 4L)
  expect_identical(obj$label, "before")
  expect_identical(obj$recall(), list(kept = TRUE))
})

test_that("mem::swap between two pointers leaves the sidecar values with each pointer", {
  a <- SidecarSlotR6$new(2L)
  b <- SidecarSlotR6$new(4L)
  a$remember(list("a"))
  sidecar_swap(a, b)
  # The Rust fields swapped...
  expect_identical(a$n, 4L)
  expect_identical(b$n, 2L)
  # ...the sidecar values did not, on either side of the fence.
  expect_identical(a$keys, 1:2)
  expect_identical(b$keys, 1:4)
  expect_identical(a$key_count(), 2L)
  expect_identical(b$key_count(), 4L)
  expect_identical(a$recall(), list("a"))
  expect_null(b$recall())
})

test_that("into_inner, then wrapping again, keeps the values", {
  a <- SidecarSlotR6$new(3L)
  a$remember(list(1))
  a$label <- "moved"
  b <- sidecar_rewrap(a$.__enclos_env__$private$.ptr)
  expect_identical(SidecarSlotR6_get_keys(b), 1:3)
  expect_identical(SidecarSlotR6_get_label(b), "moved")
  expect_identical(sidecar_keys_via_deref(b), 1:3)
  # The moved-out pointer has no value any more.
  expect_error(a$n, "null external pointer", fixed = TRUE)
})

test_that("a clone copies the sidecar values and the pointers are independent", {
  obj <- SidecarSlotR6$new(4L)
  obj$remember(list("kept"))

  # The struct's own `Clone`.
  copy <- obj$duplicate()
  expect_identical(SidecarSlotR6_get_keys(copy), 1:4)
  expect_identical(SidecarSlotR6_get_n(copy), 4L)
  expect_identical(SidecarSlotR6_get_label(copy), "fresh")
  SidecarSlotR6_set_keys(copy, 99L)
  expect_identical(obj$keys, 1:4)
  obj$keys <- 7L
  expect_identical(SidecarSlotR6_get_keys(copy), 99L)

  # `ExternalPtr::clone`.
  twin <- sidecar_clone_handle(obj$.__enclos_env__$private$.ptr)
  expect_identical(SidecarSlotR6_get_keys(twin), 7L)
  SidecarSlotR6_set_keys(twin, 1:2)
  expect_identical(obj$keys, 7L)
  expect_identical(sidecar_keys_via_deref(twin), 1:2)
})

test_that("a struct nested inside another wrapped struct works on its own values", {
  nest <- SidecarNest$new(3L)
  expect_identical(nest$inner_keys(), 1:3)
  nest$set_inner_keys(7:9)
  gc(full = TRUE)
  expect_identical(nest$inner_keys(), 7:9)
  expect_identical(nest$tag, "outer")
  nest$tag <- "renamed"
  expect_identical(nest$tag, "renamed")
})

test_that("the trait-impl receiver path reads the sidecar fields", {
  obj <- SidecarSlotR6$new(2L)
  obj$label <- "shown"
  expect_identical(
    SidecarSlotR6$RDisplay$as_r_string(obj),
    "keys=[1, 2] label=shown"
  )
})

test_that("a sidecar accessor off R's main thread is a clear error", {
  obj <- SidecarSlotR6$new(2L)
  expect_match(obj$keys_off_thread(), "run on R's main thread only", fixed = TRUE)
  expect_match(obj$keys_off_thread(), "`keys` of `SidecarSlotR6`", fixed = TRUE)
})

test_that("sidecar values stay rooted under GC churn", {
  expect_null(miniextendr:::gc_stress_sidecar_fields())
})

test_that("sidecar values stay rooted under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    expect_null(miniextendr:::gc_stress_sidecar_fields())
  }
})
