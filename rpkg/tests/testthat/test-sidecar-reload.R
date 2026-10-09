# A saved and restored object (#1872).
#
# `saveRDS()` / `serialize()` write an external pointer's tag and `prot`
# list, not its address: the `Sidecar<T>` values and the user slot come back,
# the Rust value does not. The R accessors of `Sidecar` fields read and
# write the restored pointer; everything that needs the Rust value (a
# method, an `ExternalPtr<T>` argument, the accessor of a struct field)
# raises a classed error: `miniextendr_restored_no_value` for a save by this
# version of the package, `miniextendr_restored_other_version` for a save by
# another (simulated here by `sidecar_rewrite_saved_version()`, which
# rewrites the version in the stored type ID). Both carry
# `miniextendr_restored`. A pointer not built by miniextendr keeps the
# plain `expected ExternalPtr<T>` messages.
#
# Fixture: `SidecarSlotR6` in `src/rust/rdata_sidecar_tests.rs`.

ptr_of <- function(obj) obj$.__enclos_env__$private$.ptr

reload_rds <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}

reload_serialize <- function(x) unserialize(serialize(x, NULL))

# The paths that need the Rust value of a `SidecarSlotR6` R6 object.
rust_value_paths <- list(
  `&self method` = function(obj) obj$key_count(),
  `&mut self method` = function(obj) obj$push_key(9L),
  `self method` = function(obj) obj$rebuilt(9L),
  `&mut ExternalPtr<Self> method` = function(obj) obj$reset(1L),
  `trait-impl method` = function(obj) SidecarSlotR6$RDisplay$as_r_string(obj),
  `struct-field accessor` = function(obj) obj$n,
  `standalone struct-field getter` = function(obj) SidecarSlotR6_get_n(ptr_of(obj)),
  `standalone struct-field setter` = function(obj) SidecarSlotR6_set_n(ptr_of(obj), 2L),
  `ExternalPtr<T> argument` = function(obj) sidecar_keys_via_handle(ptr_of(obj)),
  `ExternalPtr<T> argument, R6 handle` = function(obj) sidecar_keys_via_handle(obj)
)

for (reload_name in c("saveRDS / readRDS", "serialize / unserialize")) {
  reload <- if (reload_name == "saveRDS / readRDS") reload_rds else reload_serialize

  test_that(sprintf("%s: the sidecar values come back and their R accessors work", reload_name), {
    obj <- SidecarSlotR6$new(3L)
    obj$label <- "saved"
    obj$remember(list(a = 1))
    back <- reload(obj)

    expect_identical(back$keys, 1:3)
    expect_identical(back$label, "saved")
    expect_identical(back$note, "")
    expect_identical(SidecarSlotR6_get_keys(ptr_of(back)), 1:3)
    back$keys <- 4:6
    back$label <- "edited"
    SidecarSlotR6_set_note(ptr_of(back), "noted")
    gc(full = TRUE)
    expect_identical(back$keys, 4:6)
    expect_identical(back$label, "edited")
    expect_identical(back$note, "noted")
    # The original is untouched.
    expect_identical(obj$keys, 1:3)
    expect_identical(obj$label, "saved")
    expect_identical(obj$recall(), list(a = 1))
  })

  test_that(sprintf("%s: the Rust value is gone, and every path says so", reload_name), {
    obj <- SidecarSlotR6$new(3L)
    back <- reload(obj)
    for (path in names(rust_value_paths)) {
      expect_error(
        rust_value_paths[[path]](back),
        "this `SidecarSlotR6` object was restored from a saved session and has no Rust value; re-create it",
        fixed = TRUE,
        class = "miniextendr_restored_no_value",
        label = path
      )
      expect_error(
        rust_value_paths[[path]](back),
        class = "miniextendr_restored",
        label = path
      )
    }
    # The sidecar values are still there afterwards.
    expect_identical(back$keys, 1:3)
    expect_identical(obj$key_count(), 3L)
  })
}

test_that("a save by another version is refused, naming both versions", {
  obj <- SidecarSlotR6$new(2L)
  back <- reload_rds(obj)
  miniextendr:::sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  current <- as.character(utils::packageVersion("miniextendr"))
  message <- sprintf(
    "this `SidecarSlotR6` object was saved by miniextendr 0.0.1 and can't be read by miniextendr %s; re-create it",
    current
  )

  # The R accessors of the sidecar fields.
  err <- expect_error(back$keys, message, fixed = TRUE, class = "miniextendr_restored_other_version")
  expect_s3_class(err, "miniextendr_restored")
  expect_identical(err$saved_version, "0.0.1")
  expect_identical(err$current_version, current)
  expect_error(back$keys <- 1L, message, fixed = TRUE, class = "miniextendr_restored_other_version")
  expect_error(
    SidecarSlotR6_get_keys(ptr_of(back)),
    message,
    fixed = TRUE,
    class = "miniextendr_restored_other_version"
  )
  expect_error(
    SidecarSlotR6_set_label(ptr_of(back), "x"),
    message,
    fixed = TRUE,
    class = "miniextendr_restored_other_version"
  )

  # Everything that needs the Rust value.
  for (path in names(rust_value_paths)) {
    err <- expect_error(
      rust_value_paths[[path]](back),
      message,
      fixed = TRUE,
      class = "miniextendr_restored_other_version",
      label = path
    )
    expect_identical(err$saved_version, "0.0.1", label = path)
  }
})

test_that("a pointer miniextendr did not build keeps the plain messages", {
  foreign <- new("externalptr")
  expect_error(
    SidecarSlotR6_get_keys(foreign),
    "expected ExternalPtr<SidecarSlotR6>",
    fixed = TRUE,
    class = "rust_error"
  )
  expect_error(
    SidecarSlotR6_get_n(foreign),
    "expected ExternalPtr<SidecarSlotR6>, got a null external pointer",
    fixed = TRUE,
    class = "rust_error"
  )
  obj <- SidecarSlotR6$new(1L)
  obj$.__enclos_env__$private$.ptr <- foreign
  err <- expect_error(obj$key_count(), "expected ExternalPtr<SidecarSlotR6>, found `<unknown>`", fixed = TRUE)
  expect_false(inherits(err, "miniextendr_restored"))
  expect_error(sidecar_keys_via_handle(foreign), "external pointer is null", fixed = TRUE)

  # A restored pointer of another type is a type mismatch, not a restored
  # `SidecarSlotR6`.
  other <- reload_rds(rdata_sidecar_s3_new(data = 1.5))
  expect_error(SidecarSlotR6_get_keys(other), "expected ExternalPtr<SidecarSlotR6>", fixed = TRUE)
  obj$.__enclos_env__$private$.ptr <- other
  err <- expect_error(obj$key_count(), "expected ExternalPtr<SidecarSlotR6>, found `", fixed = TRUE)
  expect_false(inherits(err, "miniextendr_restored"))
  expect_error(sidecar_keys_via_handle(other), "type mismatch: expected `SidecarSlotR6`", fixed = TRUE)
})

test_that("a copy of the R object shares the one Rust value", {
  x <- SidecarSlotR6$new(2L)
  y <- x
  y$label <- "through y"
  y$push_key(3L)
  expect_identical(x$label, "through y")
  expect_identical(x$keys, 1:3)
  expect_identical(x$key_count(), 3L)
  # An explicit copy is independent.
  z <- x$duplicate()
  SidecarSlotR6_set_label(z, "copy")
  expect_identical(x$label, "through y")
})
