# Revive hooks (#1854): a `Sidecar<T>` type reads saves by another version
# of the package through `#[externalptr(revive = path)]`.
#
# `readRDS()` gives a pointer without an address. The first access of a
# hooked type (a method receiver, an `ExternalPtr<T>` argument, an R
# accessor of a `Sidecar` field) runs the hook on the stored `prot` list and
# installs the rebuilt value in the same pointer, with a `prot` of the
# current layout. Fixture: `SidecarRevive` in
# `src/rust/sidecar_revive_tests.rs`; `sidecar_revive_stored()` builds the
# pointer `readRDS()` would give for a save by another version, so no second
# version of the package is needed.

type_id <- sidecar_revive_type_id()

# The current type ID with another version.
with_version <- function(version) {
  sub("@[^:]+::", sprintf("@%s::", version), type_id, perl = TRUE)
}

# The pointer `readRDS()` gives for a save: `slots` is a named list (its
# names are the layout record), or unnamed with `record = FALSE` for a save
# from before the record.
stored <- function(slots, version = NULL, user = NULL, record = TRUE,
                   type = NULL) {
  id <- if (!is.null(type)) {
    type
  } else if (is.null(version)) {
    type_id
  } else {
    with_version(version)
  }
  names <- if (record) as.character(names(slots)) else NULL
  miniextendr:::sidecar_revive_stored(id, user, unname(slots), names)
}

# A `SidecarRevive` R6 object over `ptr`, like `readRDS()` of a saved one.
with_pointer <- function(ptr) {
  obj <- SidecarRevive$new(0L)
  obj$.__enclos_env__$private$.ptr <- ptr
  obj
}

ptr_of <- function(obj) obj$.__enclos_env__$private$.ptr

is_live <- miniextendr:::sidecar_revive_is_live

test_that("the type ID names the crate, its version and the type", {
  expect_match(type_id, "^miniextendr@[0-9.]+::.*::SidecarRevive$")
  obj <- SidecarRevive$new(1L)
  expect_identical(miniextendr:::sidecar_stored_type_id(obj), type_id)
  expect_identical(miniextendr:::sidecar_stored_type_id(ptr_of(obj)), type_id)
  # `Option<String>`: `None` is `NA`.
  expect_identical(miniextendr:::sidecar_stored_type_id(42L), NA_character_)
  expect_identical(
    miniextendr:::sidecar_stored_type_id(new("externalptr")),
    NA_character_
  )
})

test_that("rule 1: a slot this version added keeps its default", {
  ptr <- stored(list(keys = 1:3, label = "old"))
  obj <- with_pointer(ptr)
  expect_false(is_live(ptr))
  expect_no_warning(expect_identical(obj$keys, 1:3))
  expect_true(is_live(ptr))
  expect_identical(obj$label, "old")
  expect_identical(obj$added, "absent")
  expect_identical(obj$n, 3L)
  expect_identical(obj$source, "revived (added missing)")
})

test_that("rule 2: a slot this version dropped is ignored", {
  ptr <- stored(list(keys = 1:2, label = "old", added = "kept", legacy = 1.5))
  obj <- with_pointer(ptr)
  expect_no_warning(expect_identical(obj$key_count(), 2L))
  expect_identical(obj$keys, 1:2)
  expect_identical(obj$added, "kept")
  expect_identical(obj$source, "revived")
})

test_that("rule 3: a stored value of another shape is rebuilt, with a warning", {
  ptr <- stored(list(keys = c("a", "b"), label = "old", added = "x"), user = 4L)
  obj <- with_pointer(ptr)
  # Signalled at the accessor call that revived the pointer.
  expect_warning(
    expect_identical(obj$keys, 1:4),
    "`keys` was rebuilt from the recipe",
    fixed = TRUE,
    class = "sidecar_revive_changed"
  )
  expect_true(is_live(ptr))
  expect_identical(obj$n, 4L)
  expect_identical(obj$label, "old")
  expect_identical(obj$source, "revived (keys rebuilt)")
  # Without a recipe the rebuilt field is empty.
  obj2 <- with_pointer(stored(list(keys = 1.5, label = "old", added = "x")))
  expect_warning(
    expect_identical(obj2$key_count(), 0L),
    class = "sidecar_revive_changed"
  )
  expect_identical(obj2$keys, integer())
})

test_that("rule 4: a save by another version is read, and named", {
  ptr <- stored(list(keys = 1:3, label = "later", added = "x"), version = "9.9.9")
  obj <- with_pointer(ptr)
  expect_identical(miniextendr:::sidecar_stored_type_id(ptr), with_version("9.9.9"))
  expect_warning(
    expect_identical(obj$key_count(), 3L),
    "rebuilt from a save by version 9.9.9",
    fixed = TRUE,
    class = "sidecar_revive_version"
  )
  expect_identical(obj$keys, 1:3)
  expect_identical(obj$label, "later")
  expect_identical(obj$source, "revived (version 9.9.9)")
  # The pointer now carries the current type ID.
  expect_identical(miniextendr:::sidecar_stored_type_id(ptr), type_id)
  expect_identical(miniextendr:::sidecar_stored_type_id(obj), type_id)

  # A type ID without a version is read too.
  bare <- stored(
    list(keys = 1L, label = "l", added = "a"),
    type = "miniextendr::SidecarRevive"
  )
  obj <- with_pointer(bare)
  expect_no_warning(expect_identical(obj$keys, 1L))
  expect_identical(obj$source, "revived (no version)")
})

test_that("rule 5: a save from before the layout record is read by position", {
  ptr <- stored(list(1:5, "old"), record = FALSE)
  obj <- with_pointer(ptr)
  expect_no_warning(expect_identical(obj$keys, 1:5))
  expect_identical(obj$label, "old")
  expect_identical(obj$added, "absent")
  expect_identical(obj$n, 5L)
  expect_identical(obj$source, "revived (no record)")

  # Short, and from another version: the recipe fills in.
  ptr <- stored(list(), record = FALSE, version = "0.0.1", user = 2L)
  obj <- with_pointer(ptr)
  expect_warning(
    expect_identical(obj$keys, 1:2),
    class = "sidecar_revive_version"
  )
  expect_identical(obj$label, "unlabelled")
  expect_identical(obj$source, "revived (version 0.0.1, no record)")
})

test_that("the hook's error is raised with its own class, and the pointer stays unrevived", {
  ptr <- stored(list(keys = 1:3, label = "l", added = "a"), user = "refuse")
  obj <- with_pointer(ptr)
  expect_error(
    obj$keys,
    "refused to rebuild: the recipe says so",
    fixed = TRUE,
    class = "sidecar_revive_refused"
  )
  expect_false(is_live(ptr))
  expect_error(obj$key_count(), class = "sidecar_revive_error")
  expect_error(SidecarRevive_get_keys(ptr), class = "sidecar_revive_refused")
  expect_error(
    miniextendr:::sidecar_revive_keys_via_ptr(ptr),
    class = "sidecar_revive_refused"
  )
  expect_false(is_live(ptr))
})

test_that("a pointer of another type is refused, not rebuilt", {
  other <- stored(list(keys = 1:3), type = "miniextendr@0.1.0::miniextendr::Other")
  expect_error(
    SidecarRevive_get_keys(other),
    "expected ExternalPtr<SidecarRevive>",
    fixed = TRUE,
    class = "rust_error"
  )
  expect_error(
    with_pointer(other)$key_count(),
    "expected ExternalPtr<SidecarRevive>",
    fixed = TRUE
  )
  expect_false(is_live(other))

  # A real save of an unhooked type.
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(rdata_sidecar_s3_new(data = 1.5), path)
  back <- readRDS(path)
  expect_error(
    SidecarRevive_get_keys(back),
    "expected ExternalPtr<SidecarRevive>",
    fixed = TRUE,
    class = "rust_error"
  )
  expect_false(is_live(back))

  # A fresh pointer is a null external pointer of no type.
  expect_error(
    SidecarRevive_get_keys(new("externalptr")),
    "expected ExternalPtr<SidecarRevive>",
    fixed = TRUE
  )
})

test_that("every path revives a reloaded pointer", {
  reload <- function() {
    stored(list(keys = 1:3, label = "old", added = "x"), version = "0.0.9")
  }
  paths <- list(
    `active binding` = function(obj, ptr) expect_identical(obj$keys, 1:3),
    `standalone accessor` = function(obj, ptr) {
      expect_identical(SidecarRevive_get_keys(ptr), 1:3)
    },
    `&self` = function(obj, ptr) expect_identical(obj$key_count(), 3L),
    `&mut self` = function(obj, ptr) {
      obj$push_key(9L)
      expect_identical(obj$keys, c(1:3, 9L))
      expect_identical(obj$n, 4L)
    },
    `self` = function(obj, ptr) {
      obj$relabel("renamed")
      expect_identical(obj$label, "renamed")
      expect_identical(obj$keys, 1:3)
    },
    `ExternalPtr<T> argument` = function(obj, ptr) {
      expect_identical(miniextendr:::sidecar_revive_keys_via_ptr(ptr), 1:3)
    },
    `trait-ABI receiver` = function(obj, ptr) {
      expect_identical(
        SidecarRevive$RDisplay$as_r_string(obj),
        "keys=[1, 2, 3] label=old added=x"
      )
    },
    `R setter` = function(obj, ptr) {
      obj$label <- "set"
      expect_identical(obj$label, "set")
      expect_identical(obj$keys, 1:3)
    }
  )
  for (name in names(paths)) {
    ptr <- reload()
    obj <- with_pointer(ptr)
    expect_false(is_live(ptr), label = name)
    expect_warning(paths[[name]](obj, ptr), class = "sidecar_revive_version")
    expect_true(is_live(ptr), label = name)
    expect_identical(miniextendr:::sidecar_stored_type_id(ptr), type_id, label = name)
    # Live now: no second rebuild, no second warning.
    expect_no_warning(obj$source)
    expect_match(obj$source, "^revived \\(version 0.0.9\\)$", label = name)
  }
})

test_that("every R object sharing the pointer sees the rebuilt value", {
  ptr <- stored(list(keys = 1:3, label = "shared", added = "x"))
  a <- with_pointer(ptr)
  b <- with_pointer(ptr)
  expect_identical(a$keys, 1:3)
  expect_true(is_live(ptr))
  # `n` is a struct field: readable only from a live value.
  expect_no_warning(expect_identical(b$n, 3L))
  b$push_key(4L)
  expect_identical(a$keys, 1:4)
  expect_identical(a$n, 4L)
})

test_that("a saveRDS() / readRDS() round trip of a hooked R6 object revives", {
  obj <- SidecarRevive$new(3L)
  obj$label <- "saved"
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(obj, path)
  back <- readRDS(path)

  expect_false(is_live(ptr_of(back)))
  expect_identical(miniextendr:::sidecar_stored_type_id(back), type_id)
  expect_no_warning(expect_identical(back$keys, 1:3))
  expect_true(is_live(ptr_of(back)))
  expect_identical(back$label, "saved")
  expect_identical(back$added, "present")
  expect_identical(back$n, 3L)
  expect_identical(back$source, "revived")
  expect_identical(miniextendr:::sidecar_stored_type_id(back), type_id)

  # Alive after a GC, and saved again as a current object.
  gc(full = TRUE)
  back$push_key(4L)
  saveRDS(back, path)
  again <- readRDS(path)
  expect_identical(again$keys, 1:4)
  expect_identical(again$n, 4L)
  expect_identical(again$source, "revived")

  # The original is untouched.
  expect_identical(obj$keys, 1:3)
  expect_identical(obj$source, "new")
})

test_that("an unhooked type keeps today's refusal after a reload", {
  obj <- SidecarSlotR6$new(2L)
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(obj, path)
  back <- readRDS(path)
  expect_identical(back$keys, 1:2)
  expect_error(back$n, "got a null external pointer", fixed = TRUE, class = "rust_error")
  expect_error(back$key_count(), "got a null external pointer", fixed = TRUE)
  expect_false(is_live(ptr_of(back)))
})

test_that("the recipe in the user slot survives a round trip and feeds the hook", {
  p <- miniextendr:::sidecar_revive_with_recipe(2L, 5L)
  expect_identical(miniextendr:::sidecar_revive_recipe(p), 5L)
  expect_identical(SidecarRevive_get_keys(p), 1:2)
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(p, path)
  back <- readRDS(path)
  expect_false(is_live(back))
  # Revived through the `ExternalPtr<T>` argument; the recipe is in the
  # rewritten prot.
  expect_identical(miniextendr:::sidecar_revive_recipe(back), 5L)
  expect_true(is_live(back))
  expect_identical(SidecarRevive_get_keys(back), 1:2)

  refusing <- miniextendr:::sidecar_revive_with_recipe(1L, "refuse")
  expect_identical(SidecarRevive_get_keys(refusing), 1L)
  saveRDS(refusing, path)
  back <- readRDS(path)
  expect_error(SidecarRevive_get_keys(back), class = "sidecar_revive_refused")
  expect_false(is_live(back))
})

test_that("revive keeps everything rooted under GC churn", {
  expect_null(suppressWarnings(miniextendr:::gc_stress_sidecar_revive()))
})

test_that("revive keeps everything rooted under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    expect_null(suppressWarnings(miniextendr:::gc_stress_sidecar_revive()))
  }
})
