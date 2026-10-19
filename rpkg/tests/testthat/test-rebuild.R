# Rebuilding a pointer restored from a saved session (#2146):
# `#[externalptr(rebuild = path)]`. A save keeps the `Sidecar` values, not
# the Rust value; with a hook, the first use that needs the value rebuilds it
# from them and installs it, so later calls see a live pointer.
#
# Fixtures: src/rust/rebuild_tests.rs. `RebuildR6` rebuilds its total from
# its saved `values` and refuses a negative one; `RebuildPair` holds a
# `RebuildR6` pointer and converts it in its own hook.

reload <- function(x) unserialize(serialize(x, NULL))
reload_rds <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}
ptr_of <- function(obj) obj$.__enclos_env__$private$.ptr
rebuilds <- function() rebuild_counts(FALSE)[[1]]

test_that("the first use after a reload rebuilds the value, once", {
  for (back_of in list(reload, reload_rds)) {
    obj <- RebuildR6$new(c(1, 2, 3))
    back <- back_of(obj)
    before <- rebuilds()
    expect_identical(back$sum(), 6)
    expect_identical(back$sum(), 6)
    expect_identical(rebuilds() - before, 1L)
    back$scale(2)
    expect_identical(back$total, 12)
    # The saved slot stays: the hook's placeholder `Sidecar` is not written.
    expect_identical(back$values, c(1, 2, 3))
    expect_identical(obj$sum(), 6)
  }
})

test_that("every path that needs the value rebuilds it", {
  paths <- list(
    `&self method` = function(obj) obj$sum(),
    `&mut self method` = function(obj) {
      obj$scale(1)
      obj$sum()
    },
    `struct-field accessor` = function(obj) obj$total,
    `standalone struct-field getter` = function(obj) RebuildR6_get_total(ptr_of(obj)),
    `ExternalPtr<T> argument` = function(obj) rebuild_total_of(ptr_of(obj)),
    `ExternalPtr<T> argument, R6 handle` = function(obj) rebuild_total_of(obj)
  )
  for (path in names(paths)) {
    back <- reload(RebuildR6$new(c(2, 3)))
    before <- rebuilds()
    expect_identical(paths[[path]](back), 5, label = path)
    expect_identical(rebuilds() - before, 1L, label = path)
  }
})

test_that("a failed rebuild is refused with its cause as e$parent, and tried again", {
  back <- reload(RebuildR6$new(c(1, -2, -3)))
  before <- rebuilds()
  e <- tryCatch(back$sum(), error = identity)
  expect_s3_class(
    e,
    c("mx_rebuild_saved", "miniextendr_restored_no_value", "miniextendr_restored", "rust_error", "simpleError", "error", "condition"),
    exact = TRUE
  )
  expect_identical(
    conditionMessage(e),
    "this `RebuildR6` object was restored from a saved session and rebuilding it failed: the saved values hold 2 negative value(s)"
  )
  expect_s3_class(
    e$parent,
    c("mx_rebuild_error_negative", "mx_rebuild_error", "rust_error", "simpleError", "error", "condition"),
    exact = TRUE
  )
  expect_identical(conditionMessage(e$parent), "the saved values hold 2 negative value(s)")
  expect_identical(e$parent$n, 2L)
  # A handler for the cause's class does not catch the refusal.
  expect_error(
    tryCatch(back$sum(), mx_rebuild_error = function(e) "the cause's handler"),
    class = "miniextendr_restored_no_value"
  )
  # Nothing was installed: a fixed recipe rebuilds on the next use.
  back$values <- c(1, 2)
  expect_identical(back$sum(), 3)
  expect_identical(rebuilds() - before, 3L)
})

test_that("a rebuild that converts another restored pointer rebuilds that one in place", {
  back <- reload(RebuildPair$new(RebuildR6$new(c(4, 5))))
  before <- rebuilds()
  expect_identical(back$doubled(), 18)
  expect_identical(back$doubled(), 18)
  expect_identical(rebuilds() - before, 1L)

  # The inner refusal becomes the outer cause, with its own cause.
  back <- reload(RebuildPair$new(RebuildR6$new(-1)))
  e <- tryCatch(back$doubled(), error = identity)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_false(inherits(e, "mx_rebuild_error"))
  expect_s3_class(e$parent, c("mx_rebuild_saved", "miniextendr_restored_no_value"))
  expect_s3_class(e$parent$parent, "mx_rebuild_error_negative")
})

test_that("conditions the hook queues are signalled by the call that rebuilt", {
  back <- reload(RebuildR6$new(1))
  rebuild_counts(TRUE)
  on.exit(rebuild_counts(FALSE), add = TRUE)
  expect_warning(expect_identical(back$sum(), 1), "replayed the build", class = "mx_rebuild_replayed")
  expect_no_warning(back$sum())
})

test_that("a rebuilt value is dropped when its object is collected", {
  back <- reload(RebuildR6$new(c(1, 2)))
  expect_identical(back$sum(), 3)
  drops <- rebuild_counts(FALSE)[[2]]
  rm(back)
  gc(full = TRUE)
  gc(full = TRUE)
  expect_gt(rebuild_counts(FALSE)[[2]], drops)
})

test_that("a save by another version is refused as before; the hook does not run", {
  back <- reload(RebuildR6$new(1))
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  before <- rebuilds()
  expect_error(back$sum(), class = "miniextendr_restored_other_version")
  expect_identical(rebuilds(), before)
})

test_that("the rebuild survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_rebuild()
  gctorture(FALSE)
  succeed()
})
