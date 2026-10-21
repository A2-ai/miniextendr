# Rebuilding a pointer restored from a saved session (#2146):
# `#[externalptr(rebuild = path)]`. A save keeps the `Sidecar` values, not
# the Rust value; with a hook, the first use that needs the value rebuilds it
# from them and installs it, so later calls see a live pointer.
#
# Fixtures: src/rust/rebuild_tests.rs. `RebuildR6` rebuilds its total from
# its saved `values` and refuses a negative one; `RebuildPair` holds a
# `RebuildR6` pointer and converts it in its own hook. `RebuildLayout`
# reads its save by field name and takes saves by another version
# (`any_version`, #2187); `rebuild_rewrite_layout()` lays a save out as
# another version might have. `RebuildRenamed`'s field is `r_rows` in Rust
# and `n.rows` in R, the name its save uses (#2215). The internal
# `Type_saved()` reads a save as stored, without rebuilding it (#2218). `RebuildWorded` and
# `RebuildMessage` word a failed rebuild with their package's
# `restored(rebuild_failed = ...)` and `restored(message = ...)` (#2218).

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
    paste0(
      "this `RebuildR6` object was restored from a saved session and rebuilding it failed: ",
      "the saved values hold 2 negative value(s); see `miniextendr:::RebuildR6_saved(x)` for what was saved"
    )
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

test_that("a save is read by field name: reordered, removed and added fields", {
  back <- reload(RebuildLayout$new("a", 3L))
  rebuild_rewrite_layout(ptr_of(back), c("doubled", "gone", "label"), TRUE)
  # A `Sidecar` read before the rebuild finds the slot by its name.
  expect_identical(back$label, "a")
  e <- tryCatch(back$added, error = identity)
  expect_s3_class(e, "miniextendr_restored_missing_slot")
  expect_identical(e$field, "added")
  # It names the slot asked for and lists the save's, without saying why
  # it is missing (#2218).
  expect_identical(
    conditionMessage(e),
    "the saved `RebuildLayout` has no slot named `added`; its slots are `doubled`, `gone`, `label`"
  )
  # The hook read `label` and `doubled` by name; `added` defaulted.
  expect_match(back$summary(), "^a/6/7/false/[0-9.]+$")
  # The installed object has this version's layout: `label` kept from the
  # save, `doubled` replaced by the hook (`Sidecar::replace`), `added` filled
  # from the hook's struct; the unknown slot is gone.
  expect_identical(back$label, "a")
  expect_identical(back$doubled, 6L)
  expect_identical(back$added, 7L)
  # It saves and reloads like any object.
  again <- reload(back)
  expect_match(again$summary(), "^a/12/7/true/[0-9.]+$")
})

test_that("a Sidecar write before the rebuild goes to the slot of its name", {
  back <- reload(RebuildLayout$new("a", 3L))
  rebuild_rewrite_layout(ptr_of(back), c("doubled", "label"), TRUE)
  back$label <- "b"
  expect_identical(back$label, "b")
  expect_error(back$added <- 1L, class = "miniextendr_restored_missing_slot")
  expect_match(back$summary(), "^b/6/7/false/")
})

test_that("a missing field read with the plain getter is the rebuild's cause", {
  back <- reload(RebuildLayout$new("a", 3L))
  rebuild_rewrite_layout(ptr_of(back), c("label", "doubled"), TRUE)
  rebuild_layout_strict(TRUE)
  on.exit(rebuild_layout_strict(FALSE), add = TRUE)
  e <- tryCatch(back$summary(), error = identity)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_s3_class(e$parent, "miniextendr_restored_missing_slot")
  expect_identical(e$parent$field, "added")
  expect_identical(
    conditionMessage(e),
    paste0(
      "this `RebuildLayout` object was restored from a saved session and rebuilding it failed: ",
      "the saved `RebuildLayout` has no slot named `added`; its slots are `label`, `doubled`; ",
      "see `miniextendr:::RebuildLayout_saved(x)` for what was saved"
    )
  )
})

test_that("any_version rebuilds a save by another version that carries slot names", {
  back <- reload(RebuildLayout$new("old", 2L))
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  # Any use rebuilds it, a `Sidecar` read included: its slots are re-laid out.
  expect_identical(back$doubled, 4L)
  expect_identical(back$summary(), "old/4/1/true/0.0.1")
})

test_that("any_version refuses another version's save without slot names", {
  back <- reload(RebuildLayout$new("old", 2L))
  rebuild_rewrite_layout(ptr_of(back), c("label", "doubled", "added"), FALSE)
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  expect_error(back$summary(), class = "miniextendr_restored_other_version")
})

test_that("a save without slot names by this version is read by position", {
  # `RebuildR6` has no `any_version`.
  back <- reload(RebuildR6$new(c(1, 2)))
  rebuild_rewrite_layout(ptr_of(back), "values", FALSE)
  expect_identical(back$values, c(1, 2))
  expect_identical(back$sum(), 3)
})

test_that("any_version refuses a save without slot names by this version, before the hook (#2218)", {
  back <- reload(RebuildLayout$new("pos", 5L))
  # Laid out by an older version, before slots were named: read by
  # position, `label` would be `doubled`'s value.
  rebuild_rewrite_layout(ptr_of(back), c("doubled", "label", "added"), FALSE)
  message <- paste0(
    "this `RebuildLayout` object was saved before saves named their fields, ",
    "so its saved values can't be matched to fields; re-create it, ",
    "and see `miniextendr:::RebuildLayout_saved(x)` for what was saved"
  )
  e <- tryCatch(back$summary(), error = identity)
  expect_s3_class(
    e,
    c("miniextendr_restored_unnamed_slots", "miniextendr_restored", "rust_error", "simpleError", "error", "condition"),
    exact = TRUE
  )
  expect_identical(conditionMessage(e), message)
  # A `Sidecar` read or write, which does not rebuild, is refused too.
  for (use in alist(back$label, back$added, back$doubled <- 1L)) {
    e <- tryCatch(eval(use), error = identity)
    expect_s3_class(e, "miniextendr_restored_unnamed_slots")
    expect_identical(conditionMessage(e), message, label = deparse(use))
  }
  # Nothing was installed: the next use is refused the same way.
  expect_error(back$summary(), class = "miniextendr_restored_unnamed_slots")
})

test_that("a failed rebuild is worded by the package's `rebuild_failed` (#2218)", {
  back <- reload(RebuildWorded$new(-1L))
  e <- tryCatch(back$value(), error = identity)
  expect_identical(
    conditionMessage(e),
    "this RebuildWorded could not be rebuilt (the saved values hold 1 negative value(s)); build a new one"
  )
  expect_identical(class(e)[1:3], c("mx_rebuild_worded", "miniextendr_restored_no_value", "miniextendr_restored"))
  expect_s3_class(e$parent, "mx_rebuild_error_negative")
  # Another restored refusal keeps the restored message.
  other <- reload(RebuildWorded$new(2L))
  sidecar_rewrite_saved_version(other, "0.0.1")
  e <- tryCatch(other$value(), error = identity)
  expect_s3_class(e, "miniextendr_restored_other_version")
  expect_identical(conditionMessage(e), "this RebuildWorded was saved; build a new one")
  # A good save rebuilds.
  expect_identical(reload(RebuildWorded$new(3L))$value(), 3L)
})

test_that("without `rebuild_failed`, a failed rebuild says the restored message, then the cause (#2218)", {
  back <- reload(RebuildMessage$new(-1L))
  e <- tryCatch(back$value(), error = identity)
  expect_identical(
    conditionMessage(e),
    paste0(
      "this RebuildMessage was saved; build a new one (rebuilding it failed: ",
      "the saved values hold 1 negative value(s); see `miniextendr:::RebuildMessage_saved(x)` for what was saved)"
    )
  )
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_identical(reload(RebuildMessage$new(4L))$value(), 4L)
})

test_that("a save names its slots by the fields' R names (#2215)", {
  back <- reload(RebuildRenamed$new(3L, "a"))
  # `r_rows` in Rust is `n.rows` in R, and in the save.
  expect_identical(rebuild_slot_names(ptr_of(back)), c("", "", "n.rows", "label"))
  # The R accessors read the saved slot by that name.
  expect_identical(back$n.rows, 3L)
  # So does the hook: `has("n.rows")`, the plain getter and `try_r_rows()`;
  # `has()` does not know the Rust name.
  expect_identical(back$summary(), "3/Some(3)/false")
  expect_identical(back$n.rows, 3L)
  again <- reload(back)
  expect_identical(rebuild_slot_names(ptr_of(again)), c("", "", "n.rows", "label"))
  expect_identical(again$n.rows, 3L)
})

test_that("a version that renamed the Rust field reads an earlier save (#2215)", {
  # The version that saved it had the field after `label`, under another
  # Rust name with the same R name, `n.rows`: its save is laid out
  # `label`, `n.rows`.
  back <- reload(RebuildRenamed$new(5L, "old"))
  rebuild_rewrite_layout(ptr_of(back), c("label", "n.rows"), TRUE)
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  expect_identical(back$summary(), "5/Some(5)/false")
  expect_identical(back$n.rows, 5L)
  expect_identical(back$label, "old")
})

test_that("a save under another R name has no slot for the field (#2215)", {
  back <- reload(RebuildRenamed$new(5L, "old"))
  rebuild_rewrite_layout(ptr_of(back), c("r_rows", "label"), TRUE)
  e <- tryCatch(back$n.rows, error = identity)
  expect_s3_class(e, "miniextendr_restored_missing_slot")
  expect_identical(e$field, "n.rows")
  expect_identical(back$summary(), "-1/None/false")
  expect_identical(back$n.rows, -1L)
  expect_identical(back$label, "old")
})

test_that("Type_saved() reads a save as stored, without rebuilding it (#2218)", {
  back <- reload(RebuildR6$new(c(1, 2, 3)))
  before <- rebuilds()
  expect_identical(RebuildR6_saved(back), list(values = c(1, 2, 3)))
  expect_identical(RebuildR6_saved(ptr_of(back)), list(values = c(1, 2, 3)))
  expect_identical(rebuilds(), before)
  # An env class's object, whose saved slot holds another object's pointer:
  # the pointer as stored, not rebuilt.
  pair <- reload(RebuildPair$new(RebuildR6$new(c(4, 5))))
  inner <- RebuildPair_saved(pair)$inner
  expect_type(inner, "externalptr")
  expect_identical(rebuilds(), before)
  # A live object gives its current values, which a save of it would hold.
  live <- RebuildLayout$new("a", 3L)
  live$label <- "b"
  expect_identical(RebuildLayout_saved(live), list(label = "b", doubled = 3L, added = 1L))
})

test_that("Type_saved() reads another version's save under any_version, without rebuilding it", {
  back <- reload(RebuildLayout$new("old", 2L))
  # Laid out by that version, which had no `added`; edited before it is
  # marked as another version's, since a `Sidecar` read of that rebuilds.
  rebuild_rewrite_layout(ptr_of(back), c("doubled", "label"), TRUE)
  sidecar_rewrite_saved_version(ptr_of(back), "0.0.1")
  expect_identical(RebuildLayout_saved(back), list(doubled = 2L, label = "old"))
  # The first use rebuilds it: this version's layout, `doubled` recomputed,
  # `added` from the hook.
  expect_identical(back$summary(), "old/4/7/false/0.0.1")
  expect_identical(RebuildLayout_saved(back), list(label = "old", doubled = 4L, added = 7L))
})

test_that("after a failed rebuild, Type_saved() shows what was saved (#2218)", {
  back <- reload(RebuildR6$new(c(-1, 2)))
  expect_error(back$sum(), class = "miniextendr_restored_no_value")
  expect_identical(RebuildR6_saved(back), list(values = c(-1, 2)))
  # A `rebuild_failed` template stays as the package wrote it.
  worded <- reload(RebuildWorded$new(-1L))
  expect_error(worded$value(), "build a new one$")
  expect_identical(RebuildWorded_saved(worded), list(n = -1L))
})

test_that("Type_saved() reads a save without slot names as an unnamed list", {
  back <- reload(RebuildLayout$new("pos", 5L))
  rebuild_rewrite_layout(ptr_of(back), c("doubled", "label", "added"), FALSE)
  # Under `any_version` the object is refused; its values can still be seen.
  expect_error(back$summary(), class = "miniextendr_restored_unnamed_slots")
  expect_identical(RebuildLayout_saved(back), list(5L, "pos", 1L))
})

test_that("Type_saved() refuses an object of another type", {
  e <- tryCatch(RebuildLayout_saved(RebuildR6$new(1)), error = identity)
  expect_s3_class(e, "rust_error")
  # A type without `field_error` classes refuses with a plain panic, whose
  # message ends with its source location.
  expect_match(conditionMessage(e), "this `RebuildR6` object holds a pointer of another type", fixed = TRUE)
  e <- tryCatch(RebuildLayout_saved(reload(RebuildR6$new(1))), error = identity)
  expect_match(conditionMessage(e), "this `RebuildR6` object holds a null pointer", fixed = TRUE)
})

test_that("Type_saved() survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_rebuild_saved()
  gctorture(FALSE)
  succeed()
})

test_that("the rebuild survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_rebuild()
  gctorture(FALSE)
  succeed()
})
