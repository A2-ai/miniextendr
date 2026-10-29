# A pointer to a serde type comes back after `readRDS()` (#2186).
#
# A type that derives `Serialize` and `Deserialize` keeps its value in the
# save: R calls the carrier in the pointer's protection list during
# `serialize()` (which encodes the value) and during `unserialize()` (which
# decodes it). The first access attaches the decoded value: a method, an
# argument, a field binding. A save that can't be decoded is refused there
# with `miniextendr_restored_no_value`, the decoding error as `e$parent`. A
# type that is not serde is refused as before.
#
# Fixtures: src/rust/saved_value_tests.rs. `SavedR6`, `SavedS7`, `SavedS4`,
# `SavedEnv` and `SavedS3` are serde, `PlainR6` is not. `SavedPositive`
# refuses to decode a negative count. `SavedRecord` leaves an empty note and a
# zero weight out of its save, as an older version would have saved it.

reload <- function(x) unserialize(serialize(x, NULL))
reload_rds <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}
ptr_of <- function(obj) obj$.__enclos_env__$private$.ptr

test_that("an R6 object comes back with its value and its sidecar values", {
  for (back_of in list(reload, reload_rds)) {
    obj <- SavedR6$new(3L)
    obj$tags <- c("a", "b")
    obj$bump()
    back <- back_of(obj)
    expect_identical(back$value(), 4L)
    expect_identical(back$n, 4L)
    expect_identical(back$tags, c("a", "b"))
    back$bump()
    back$tags <- "c"
    expect_identical(back$value(), 5L)
    expect_identical(back$tags, "c")
    # The original is its own object.
    expect_identical(obj$value(), 4L)
    expect_identical(obj$tags, c("a", "b"))
  }
})

test_that("every first access attaches the value", {
  paths <- list(
    `&self method` = function(obj) obj$value(),
    `&mut self method` = function(obj) {
      obj$bump()
      obj$value() - 1L
    },
    `struct-field binding` = function(obj) obj$n,
    `sidecar binding, then a method` = function(obj) {
      expect_identical(obj$tags, "t")
      obj$value()
    }
  )
  for (path in names(paths)) {
    obj <- SavedR6$new(2L)
    obj$tags <- "t"
    expect_identical(paths[[path]](reload(obj)), 2L, label = path)
  }
  back <- reload(SavedEnv$new(6L, "six"))
  expect_identical(saved_env_value_of(back), 6L)
  back <- reload(SavedEnv$new(6L, "six"))
  expect_identical(saved_env_handle_value(back), 6L)
})

test_that("each class system's object comes back", {
  expect_identical(saved_s7_value(reload_rds(SavedS7(3L))), 3L)
  expect_identical(s4_saved_s4_value(reload_rds(SavedS4(4L))), 4L)
  env <- reload_rds(SavedEnv$new(5L, "five"))
  expect_identical(env$value(), 5L)
  expect_identical(env$label(), "five")
  expect_identical(saved_s3_value(reload_rds(SavedS3$new(6L))), 6L)
})

test_that("a type that is not serde is refused, as before", {
  back <- reload(PlainR6$new(1L))
  expect_error(
    back$value(),
    "this `PlainR6` object was restored from a saved session and has no Rust value; re-create it",
    fixed = TRUE,
    class = "miniextendr_restored_no_value"
  )
  back <- reload(PlainR6$new(1L))
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  expect_error(back$value(), class = "miniextendr_restored_other_version")
})

test_that("a save read back and saved again before any access keeps its value", {
  back <- reload_rds(reload(SavedEnv$new(8L, "twice")))
  expect_identical(back$value(), 8L)
  # Attached, it saves its current value.
  back$bump()
  expect_identical(reload(back)$value(), 9L)
})

test_that("a save while a call holds the object mutably is a borrow conflict", {
  obj <- SavedEnv$new(1L, "held")
  expect_error(
    obj$bump_and_call(function() serialize(obj, NULL)),
    "object is in use by a call that is still running",
    fixed = TRUE,
    class = "miniextendr_borrow_conflict"
  )
  # The call ended, and with it the borrow: the object saves its value.
  expect_identical(reload(obj)$value(), 2L)
})

test_that("readRDS() loads the package in a session that has not, and the value comes back", {
  skip_on_os("windows")
  skip_if_not_installed("callr")
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(SavedEnv$new(7L, "fresh"), path)
  out <- callr::r(
    function(path) {
      before <- "miniextendr" %in% loadedNamespaces()
      x <- readRDS(path)
      list(
        before = before,
        after = "miniextendr" %in% loadedNamespaces(),
        value = x$value(),
        label = x$label()
      )
    },
    args = list(path = path)
  )
  expect_false(out$before)
  expect_true(out$after)
  expect_identical(out$value, 7L)
  expect_identical(out$label, "fresh")
})

test_that("a save that can't be decoded is refused, with the decoding error as e$parent", {
  back <- reload(SavedPositive$new(-2L))
  e <- tryCatch(back$value(), error = identity)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_s3_class(e, "miniextendr_restored")
  expect_identical(
    conditionMessage(e),
    paste0(
      "this `SavedPositive` object was restored from a saved session, but its saved value ",
      "could not be read: the count -2 is negative; re-create it"
    )
  )
  expect_s3_class(e$parent, "miniextendr_decode_error")
  expect_identical(conditionMessage(e$parent), "the count -2 is negative")
  # It stays refused.
  expect_error(back$value(), class = "miniextendr_restored_no_value")
  expect_identical(reload(SavedPositive$new(2L))$value(), 2L)
})

test_that("a save by another version is decoded: serde decides", {
  # The save leaves out the empty note, which decodes with its default.
  back <- reload(SavedRecord$new(1L, "", 2L))
  sidecar_rewrite_saved_version(back, "0.0.1")
  expect_identical(back$summary(), "1/added later/2")

  # The save leaves out the zero weight, which has no default.
  back <- reload(SavedRecord$new(1L, "kept", 0L))
  sidecar_rewrite_saved_version(back, "0.0.1")
  e <- tryCatch(back$summary(), error = identity)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_s3_class(e$parent, "miniextendr_decode_error")
  expect_match(conditionMessage(e$parent), "weight", fixed = TRUE)
})

test_that("a version-2 save carries no value and is refused", {
  back <- unserialize(serialize(SavedEnv$new(1L, "v2"), NULL, version = 2))
  expect_error(
    back$value(),
    "this `SavedEnv` object was restored from a saved session and has no Rust value; re-create it",
    fixed = TRUE,
    class = "miniextendr_restored_no_value"
  )
})

test_that("an object inside a list or an environment comes back", {
  l <- reload_rds(list(a = SavedEnv$new(1L, "a"), b = list(c = SavedS3$new(2L))))
  expect_identical(l$a$value(), 1L)
  expect_identical(saved_s3_value(l$b$c), 2L)

  e <- new.env()
  e$x <- SavedR6$new(5L)
  e$y <- e$x
  back <- reload_rds(e)
  expect_identical(back$x$value(), 5L)
  # One object, saved once: both bindings see the same value.
  back$x$bump()
  expect_identical(back$y$value(), 6L)
})

test_that("load() restores the good objects next to one that can't be decoded", {
  path <- tempfile(fileext = ".RData")
  on.exit(unlink(path), add = TRUE)
  good1 <- SavedEnv$new(1L, "one")
  bad <- SavedPositive$new(-1L)
  good2 <- SavedR6$new(2L)
  save(good1, bad, good2, file = path)
  env <- new.env()
  expect_no_error(load(path, envir = env))
  expect_identical(env$good1$value(), 1L)
  expect_identical(env$good2$value(), 2L)
  e <- tryCatch(env$bad$value(), error = identity)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_identical(conditionMessage(e$parent), "the count -1 is negative")
})

test_that("the saved-value round trip survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_saved_value()
  gctorture(FALSE)
  succeed()
})
