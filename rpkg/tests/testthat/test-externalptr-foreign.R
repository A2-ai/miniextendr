test_that("live native-symbol pointers are rejected before reading Rust storage", {
  routines <- getDLLRegisteredRoutines(getLoadedDLLs()[["miniextendr"]])
  foreign <- routines[[".Call"]][[1L]][["address"]]
  expect_type(foreign, "externalptr")

  for (access in list(
    function(x) SidecarSlotR6_get_n(x),
    function(x) SidecarSlotR6_set_n(x, 2L),
    function(x) SidecarSlotR6_get_keys(x),
    function(x) SidecarSlotR6_set_keys(x, 1:2),
    function(x) SidecarSlotR6_set_label(x, "foreign")
  )) {
    expect_error(access(foreign), "pointer of another type", fixed = TRUE, class = "rust_error")
  }
  expect_error(sidecar_keys_via_handle(foreign), "external pointer has no valid type id", fixed = TRUE)

  obj <- SidecarSlotR6$new(1L)
  obj$.__enclos_env__$private$.ptr <- foreign
  for (access in list(
    function(x) x$key_count(),
    function(x) x$push_key(9L),
    function(x) x$rebuilt(9L),
    function(x) SidecarSlotR6$RDisplay$as_r_string(x)
  )) {
    err <- expect_error(access(obj), "expected ExternalPtr<SidecarSlotR6>", fixed = TRUE, class = "rust_error")
    expect_false(inherits(err, "miniextendr_restored"))
  }
  expect_error(obj$n, "pointer of another type", fixed = TRUE, class = "rust_error")
  expect_error(sidecar_keys_via_handle(obj), "external pointer has no valid type id", fixed = TRUE)

  # Refusing the native address must not damage the registered routine.
  live <- SidecarSlotR6$new(2L)
  expect_identical(live$key_count(), 2L)
  expect_identical(sidecar_keys_via_handle(live), 1:2)
})
