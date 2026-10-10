# Typed `Sidecar<T>` fields (#1846, #1855, #1856).
#
# An `#[r_data] pub f: Sidecar<T>` field keeps its value in the external
# pointer's protection list, after the type ID and the user slot. The pointer
# roots the value while it is reachable, and `saveRDS()` writes it with the
# pointer. Rust reads and writes it through the typed accessors the derive
# generates, associated functions taking the handle (`Type::f(&ptr)` /
# `Type::set_f(&mut ptr, v)`); a method that needs the value takes the handle
# as its receiver. The struct holds nothing that points back at its pointer.
# Fixtures: `SidecarEnv` (`raw_slot: Sidecar<SEXP>`), `SidecarRawSexp`, the R6
# `SidecarSlotR6` and `SidecarNest`, and the S3 / S4 / S7 `SidecarS3` /
# `SidecarS4` / `SidecarS7` in `src/rust/rdata_sidecar_tests.rs`.

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
  # A struct field needs the Rust value: see test-sidecar-reload.R.
  expect_error(
    SidecarEnv_get_count(back),
    "restored from a saved session and has no Rust value",
    fixed = TRUE,
    class = "miniextendr_restored_no_value"
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

test_that("a field's R name is the one R sees; the Rust identifier stays in Rust (#1891)", {
  # `SidecarSlotR6`'s `r_label` is `label` in R: the active binding and the
  # standalone accessors; `SidecarS7`'s `r_scores` is the property `scores`.
  obj <- SidecarSlotR6$new(1L)
  expect_identical(obj$label, "fresh")
  expect_identical(SidecarSlotR6_get_label(obj$.__enclos_env__$private$.ptr), "fresh")
  SidecarSlotR6_set_label(obj$.__enclos_env__$private$.ptr, "set")
  expect_identical(obj$label, "set")
  expect_identical(obj$label_len(), 3L)
  expect_false("r_label" %in% names(obj))
  expect_false(exists("SidecarSlotR6_get_r_label", envir = asNamespace("miniextendr")))
  expect_identical(miniextendr:::.rdata_fields_SidecarSlotR6, c("keys", "n", "label", "note"))

  s7 <- SidecarS7(prop_int = 1L, prop_flag = TRUE, prop_name = "s")
  expect_identical(s7@scores, numeric())
  s7@scores <- c(1, 2)
  expect_identical(SidecarS7_get_scores(s7), c(1, 2))
  expect_identical(score_total(s7), 3)
  expect_false("r_scores" %in% S7::prop_names(s7))
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

test_that("a by-value self method leaves the slots in the pointer and flushes what it set", {
  obj <- SidecarSlotR6$new(3L)
  obj$remember(list(kept = TRUE))
  obj$label <- "before"
  obj$rebuilt(9L)
  gc(full = TRUE)
  # `..self` kept the fields: their slots were never touched.
  expect_identical(obj$keys, 1:3)
  expect_identical(obj$label, "before")
  # The struct field it changed, and the field it filled with `Sidecar::new`.
  expect_identical(obj$n, 12L)
  expect_identical(obj$recall(), list(9L))
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

test_that("into_inner right after a swap takes the pointer's own values along", {
  a <- SidecarSlotR6$new(2L)
  b <- SidecarSlotR6$new(4L)
  sidecar_swap(a, b)
  # Nothing in between: the struct `a` holds came from `b`, the values are `a`'s.
  moved <- sidecar_rewrap(a$.__enclos_env__$private$.ptr)
  expect_identical(SidecarSlotR6_get_keys(moved), 1:2)
  expect_identical(SidecarSlotR6_get_n(moved), 4L)
  expect_identical(b$keys, 1:4)
  expect_identical(b$n, 2L)
})

test_that("into_inner, then wrapping again, keeps the values", {
  a <- SidecarSlotR6$new(3L)
  a$remember(list(1))
  a$label <- "moved"
  b <- sidecar_rewrap(a$.__enclos_env__$private$.ptr)
  expect_identical(SidecarSlotR6_get_keys(b), 1:3)
  expect_identical(SidecarSlotR6_get_label(b), "moved")
  expect_identical(sidecar_keys_via_handle(b), 1:3)
  # The moved-out pointer has no value any more.
  expect_error(a$n, "null external pointer", fixed = TRUE)
})

test_that("a clone shares the sidecar values and a write replaces one side's slot only", {
  obj <- SidecarSlotR6$new(4L)
  obj$remember(list("kept"))

  # `ExternalPtr::clone` through a handle-receiver method.
  copy <- obj$duplicate()
  expect_identical(SidecarSlotR6_get_keys(copy), 1:4)
  expect_identical(SidecarSlotR6_get_n(copy), 4L)
  expect_identical(SidecarSlotR6_get_label(copy), "fresh")
  SidecarSlotR6_set_keys(copy, 99L)
  expect_identical(obj$keys, 1:4)
  obj$keys <- 7L
  expect_identical(SidecarSlotR6_get_keys(copy), 99L)

  # `ExternalPtr::clone` on an argument.
  twin <- sidecar_clone_handle(obj$.__enclos_env__$private$.ptr)
  expect_identical(SidecarSlotR6_get_keys(twin), 7L)
  SidecarSlotR6_set_keys(twin, 1:2)
  expect_identical(obj$keys, 7L)
  expect_identical(sidecar_keys_via_handle(twin), 1:2)

  # `ExternalPtr::clone_from`: the target takes the source's values in place.
  target <- SidecarSlotR6$new(1L)
  target$remember(list("mine"))
  sidecar_clone_from(target$.__enclos_env__$private$.ptr, obj$.__enclos_env__$private$.ptr)
  gc(full = TRUE)
  expect_identical(target$keys, 7L)
  expect_identical(target$n, 4L)
  expect_identical(target$recall(), list("kept"))
  target$keys <- 5L
  expect_identical(obj$keys, 7L)
  expect_identical(target$keys, 5L)
})

test_that("a struct nested by its handle inside another wrapped struct works on its own values", {
  nest <- SidecarNest$new(3L)
  expect_identical(nest$inner_keys(), 1:3)
  nest$set_inner_keys(7:9)
  gc(full = TRUE)
  expect_identical(nest$inner_keys(), 7:9)
  expect_identical(nest$tag, "outer")
  nest$tag <- "renamed"
  expect_identical(nest$tag, "renamed")
})

test_that("a trait-impl method sees the struct fields only", {
  obj <- SidecarSlotR6$new(2L)
  obj$label <- "shown"
  expect_identical(SidecarSlotR6$RDisplay$as_r_string(obj), "n=2")
})

test_that("an S3 method that takes the handle reads and writes a sidecar value", {
  obj <- SidecarS3$new(1.5)
  expect_identical(tag_count(obj), 0L)
  add_tag(obj, "a")
  add_tag(obj, "b")
  gc(full = TRUE)
  expect_identical(tag_count(obj), 2L)
  expect_identical(SidecarS3_get_tags(obj), c("a", "b"))
  SidecarS3_set_tags(obj, "z")
  expect_identical(tag_count(obj), 1L)
  expect_identical(SidecarS3_get_data(obj), 1.5)
})

test_that("an S4 method that takes the handle reads and writes a sidecar value", {
  obj <- SidecarS4(slot_int = 1L, slot_real = 2.5, slot_str = "s")
  expect_identical(s4_history_len(obj), 0L)
  s4_record(obj, 1.5)
  s4_record(obj, 2.5)
  gc(full = TRUE)
  expect_identical(s4_history_len(obj), 2L)
  expect_identical(SidecarS4_get_history(obj@ptr), c(1.5, 2.5))
  SidecarS4_set_history(obj@ptr, 9)
  expect_identical(s4_history_len(obj), 1L)
  expect_identical(SidecarS4_get_slot_str(obj@ptr), "s")
})

test_that("an S7 method that takes the handle reads and writes a sidecar property", {
  obj <- SidecarS7(prop_int = 1L, prop_flag = TRUE, prop_name = "s")
  expect_identical(score_total(obj), 0)
  add_score(obj, 1.5)
  add_score(obj, 2.5)
  gc(full = TRUE)
  expect_identical(score_total(obj), 4)
  expect_identical(obj@scores, c(1.5, 2.5))
  obj@scores <- 10
  expect_identical(SidecarS7_score_total(obj), 10)
  expect_identical(obj@prop_name, "s")
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
