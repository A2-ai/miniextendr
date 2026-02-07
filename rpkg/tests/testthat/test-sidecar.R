# Tests for sidecar constructors and getters/setters

# =============================================================================
# SidecarEnv tests
# =============================================================================

test_that("SidecarEnv constructor works", {
  obj <- rdata_sidecar_env_new(count = 42L, score = 3.14, flag = TRUE, name = "test")
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarEnv getters return correct initial values", {
  obj <- rdata_sidecar_env_new(count = 10L, score = 2.5, flag = FALSE, name = "hello")

  expect_equal(SidecarEnv_get_count(obj), 10L)
  expect_equal(SidecarEnv_get_score(obj), 2.5)
  expect_equal(SidecarEnv_get_flag(obj), FALSE)
  expect_equal(SidecarEnv_get_name(obj), "hello")
})

test_that("SidecarEnv setters update values correctly", {
  obj <- rdata_sidecar_env_new(count = 0L, score = 0.0, flag = FALSE, name = "")

  SidecarEnv_set_count(obj, 99L)
  expect_equal(SidecarEnv_get_count(obj), 99L)

  SidecarEnv_set_score(obj, 1.618)
  expect_equal(SidecarEnv_get_score(obj), 1.618)

  SidecarEnv_set_flag(obj, TRUE)
  expect_equal(SidecarEnv_get_flag(obj), TRUE)

  SidecarEnv_set_name(obj, "updated")
  expect_equal(SidecarEnv_get_name(obj), "updated")
})

test_that("SidecarEnv raw_slot getter/setter works", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1.0, flag = TRUE, name = "x")

  # Set and get raw_slot
  raw_val <- as.raw(c(1, 2, 3))
  SidecarEnv_set_raw_slot(obj, raw_val)
  expect_equal(SidecarEnv_get_raw_slot(obj), raw_val)
})

# =============================================================================
# SidecarR6 tests
# =============================================================================

test_that("SidecarR6 constructor works", {
  obj <- rdata_sidecar_r6_new(value = 100L, label = "label1")
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarR6 getters return correct initial values", {
  obj <- rdata_sidecar_r6_new(value = 42L, label = "meaning")

  expect_equal(SidecarR6_get_value(obj), 42L)
  expect_equal(SidecarR6_get_label(obj), "meaning")
})

test_that("SidecarR6 setters update values correctly", {
  obj <- rdata_sidecar_r6_new(value = 0L, label = "old")

  SidecarR6_set_value(obj, 123L)
  expect_equal(SidecarR6_get_value(obj), 123L)

  SidecarR6_set_label(obj, "new")
  expect_equal(SidecarR6_get_label(obj), "new")
})

# =============================================================================
# SidecarR6 class-integrated access (r_data_accessors) tests
# =============================================================================

test_that("SidecarR6 active bindings work via R6 class", {
  # Create via R6 class constructor
  obj <- SidecarR6$new(value = 42L, label = "test")

  # Active bindings should provide getter access
  expect_equal(obj$value, 42L)
  expect_equal(obj$label, "test")

  # Active bindings should provide setter access
  obj$value <- 99L
  expect_equal(obj$value, 99L)

  obj$label <- "updated"
  expect_equal(obj$label, "updated")
})

# =============================================================================
# SidecarS3 tests
# =============================================================================

test_that("SidecarS3 constructor works", {
  # data is a scalar f64, not a vector
  obj <- rdata_sidecar_s3_new(data = 1.5)
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarS3 getter/setter works", {
  obj <- rdata_sidecar_s3_new(data = 1.5)

  expect_equal(SidecarS3_get_data(obj), 1.5)

  SidecarS3_set_data(obj, 4.5)
  expect_equal(SidecarS3_get_data(obj), 4.5)
})

# =============================================================================
# SidecarS4 tests
# =============================================================================

test_that("SidecarS4 constructor works", {
  obj <- rdata_sidecar_s4_new(slot_int = 1L, slot_real = 1.5, slot_str = "s4test")
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarS4 getters return correct initial values", {
  obj <- rdata_sidecar_s4_new(slot_int = 7L, slot_real = 2.718, slot_str = "euler")

  expect_equal(SidecarS4_get_slot_int(obj), 7L)
  expect_equal(SidecarS4_get_slot_real(obj), 2.718)
  expect_equal(SidecarS4_get_slot_str(obj), "euler")
})

test_that("SidecarS4 setters update values correctly", {
  obj <- rdata_sidecar_s4_new(slot_int = 0L, slot_real = 0.0, slot_str = "")

  SidecarS4_set_slot_int(obj, 42L)
  expect_equal(SidecarS4_get_slot_int(obj), 42L)

  SidecarS4_set_slot_real(obj, 3.14159)
  expect_equal(SidecarS4_get_slot_real(obj), 3.14159)

  SidecarS4_set_slot_str(obj, "pi")
  expect_equal(SidecarS4_get_slot_str(obj), "pi")
})

# =============================================================================
# SidecarS7 tests
# =============================================================================

test_that("SidecarS7 constructor works", {
  skip_if_not("s7" %in% rpkg_enabled_features(), "S7 feature not enabled")
  obj <- rdata_sidecar_s7_new(prop_int = 5L, prop_flag = TRUE, prop_name = "s7obj")
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarS7 getters return correct initial values", {
  skip_if_not("s7" %in% rpkg_enabled_features(), "S7 feature not enabled")
  obj <- rdata_sidecar_s7_new(prop_int = 99L, prop_flag = FALSE, prop_name = "test")

  expect_equal(SidecarS7_get_prop_int(obj), 99L)
  expect_equal(SidecarS7_get_prop_flag(obj), FALSE)
  expect_equal(SidecarS7_get_prop_name(obj), "test")
})

test_that("SidecarS7 setters update values correctly", {
  skip_if_not("s7" %in% rpkg_enabled_features(), "S7 feature not enabled")
  obj <- rdata_sidecar_s7_new(prop_int = 0L, prop_flag = FALSE, prop_name = "")

  SidecarS7_set_prop_int(obj, 77L)
  expect_equal(SidecarS7_get_prop_int(obj), 77L)

  SidecarS7_set_prop_flag(obj, TRUE)
  expect_equal(SidecarS7_get_prop_flag(obj), TRUE)

  SidecarS7_set_prop_name(obj, "updated")
  expect_equal(SidecarS7_get_prop_name(obj), "updated")
})

# =============================================================================
# SidecarS7 class-integrated access (r_data_accessors) tests
# =============================================================================

test_that("SidecarS7 properties work via S7 class", {
  skip_if_not("s7" %in% rpkg_enabled_features(), "S7 feature not enabled")

  # Create via S7 class constructor
  obj <- SidecarS7(prop_int = 42L, prop_flag = TRUE, prop_name = "s7test")

  # S7 properties should provide getter access
  expect_equal(obj@prop_int, 42L)
  expect_equal(obj@prop_flag, TRUE)
  expect_equal(obj@prop_name, "s7test")

  # S7 properties should provide setter access
  obj@prop_int <- 99L
  expect_equal(obj@prop_int, 99L)

  obj@prop_flag <- FALSE
  expect_equal(obj@prop_flag, FALSE)

  obj@prop_name <- "updated"
  expect_equal(obj@prop_name, "updated")
})

# =============================================================================
# SidecarVctrs tests
# =============================================================================

test_that("SidecarVctrs constructor works", {
  skip_if_not("vctrs" %in% rpkg_enabled_features(), "vctrs feature not enabled")
  obj <- rdata_sidecar_vctrs_new(vec_data = c(1.0, 2.0), vec_label = "data")
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarVctrs getters return correct initial values", {
  skip_if_not("vctrs" %in% rpkg_enabled_features(), "vctrs feature not enabled")
  obj <- rdata_sidecar_vctrs_new(vec_data = c(1.5, 2.5, 3.5), vec_label = "test_vec")

  expect_equal(SidecarVctrs_get_vec_data(obj), c(1.5, 2.5, 3.5))
  expect_equal(SidecarVctrs_get_vec_label(obj), "test_vec")
})

test_that("SidecarVctrs setters update values correctly", {
  skip_if_not("vctrs" %in% rpkg_enabled_features(), "vctrs feature not enabled")
  obj <- rdata_sidecar_vctrs_new(vec_data = c(0.0), vec_label = "old")

  SidecarVctrs_set_vec_data(obj, c(10.0, 20.0, 30.0))
  expect_equal(SidecarVctrs_get_vec_data(obj), c(10.0, 20.0, 30.0))

  SidecarVctrs_set_vec_label(obj, "new_label")
  expect_equal(SidecarVctrs_get_vec_label(obj), "new_label")
})

# =============================================================================
# SidecarRaw tests
# =============================================================================

test_that("SidecarRaw constructor works", {
  obj <- rdata_sidecar_raw_new(byte_val = as.raw(42))
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarRaw getter/setter works", {
  obj <- rdata_sidecar_raw_new(byte_val = as.raw(10))

  expect_equal(SidecarRaw_get_byte_val(obj), as.raw(10))

  SidecarRaw_set_byte_val(obj, as.raw(255))
  expect_equal(SidecarRaw_get_byte_val(obj), as.raw(255))
})

# =============================================================================
# SidecarRawSexp tests
# =============================================================================

test_that("SidecarRawSexp constructor works", {
  obj <- rdata_sidecar_rawsexp_new()
  expect_true(inherits(obj, "externalptr"))
})

test_that("SidecarRawSexp int_vec getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  int_vec <- c(1L, 2L, 3L)
  SidecarRawSexp_set_int_vec(obj, int_vec)
  expect_equal(SidecarRawSexp_get_int_vec(obj), int_vec)
})

test_that("SidecarRawSexp real_vec getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  real_vec <- c(1.1, 2.2, 3.3)
  SidecarRawSexp_set_real_vec(obj, real_vec)
  expect_equal(SidecarRawSexp_get_real_vec(obj), real_vec)
})

test_that("SidecarRawSexp char_vec getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  char_vec <- c("a", "b", "c")
  SidecarRawSexp_set_char_vec(obj, char_vec)
  expect_equal(SidecarRawSexp_get_char_vec(obj), char_vec)
})

test_that("SidecarRawSexp list_val getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  list_val <- list(x = 1L, y = "test")
  SidecarRawSexp_set_list_val(obj, list_val)
  expect_equal(SidecarRawSexp_get_list_val(obj), list_val)
})

test_that("SidecarRawSexp env_val getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  env_val <- new.env()
  env_val$x <- 42L
  SidecarRawSexp_set_env_val(obj, env_val)
  result <- SidecarRawSexp_get_env_val(obj)
  expect_true(is.environment(result))
  expect_equal(result$x, 42L)
})

test_that("SidecarRawSexp func_val getter/setter works", {
  obj <- rdata_sidecar_rawsexp_new()

  func_val <- function(x) x + 1
  SidecarRawSexp_set_func_val(obj, func_val)
  result <- SidecarRawSexp_get_func_val(obj)
  expect_true(is.function(result))
  expect_equal(result(5), 6)
})

# =============================================================================
# Setter returns invisible(x) tests
# =============================================================================

test_that("SidecarEnv setters return invisible(x)", {
  obj <- rdata_sidecar_env_new(count = 0L, score = 0.0, flag = FALSE, name = "")

  # Setter should return the same object (invisibly)
  result <- SidecarEnv_set_count(obj, 42L)
  expect_identical(result, obj)

  # Verify it's actually invisible (withVisible returns vis=FALSE)
  vis_result <- withVisible(SidecarEnv_set_score(obj, 1.5))
  expect_identical(vis_result$value, obj)
  expect_false(vis_result$visible)
})

test_that("SidecarR6 setters return invisible(x)", {
  obj <- rdata_sidecar_r6_new(value = 0L, label = "")

  result <- SidecarR6_set_value(obj, 99L)
  expect_identical(result, obj)

  vis_result <- withVisible(SidecarR6_set_label(obj, "test"))
  expect_identical(vis_result$value, obj)
  expect_false(vis_result$visible)
})

test_that("SidecarS4 setters return invisible(x)", {
  obj <- rdata_sidecar_s4_new(slot_int = 0L, slot_real = 0.0, slot_str = "")

  result <- SidecarS4_set_slot_int(obj, 7L)
  expect_identical(result, obj)

  vis_result <- withVisible(SidecarS4_set_slot_real(obj, 2.5))
  expect_identical(vis_result$value, obj)
  expect_false(vis_result$visible)
})

# =============================================================================
# Raw SEXP identity tests (getter returns exact stored SEXP)
# =============================================================================

test_that("SidecarRawSexp getter returns identical SEXP", {
  obj <- rdata_sidecar_rawsexp_new()

  # Create unique identifiable objects
  env <- new.env()
  env$marker <- "unique_marker_12345"

  SidecarRawSexp_set_env_val(obj, env)
  result <- SidecarRawSexp_get_env_val(obj)

  # Not just equal - should be the exact same object
  expect_identical(result, env)

  # Modify through one reference, visible through the other
  result$new_field <- 42L
  expect_equal(env$new_field, 42L)
})

test_that("SidecarEnv raw_slot preserves SEXP identity", {
  obj <- rdata_sidecar_env_new(count = 1L, score = 1.0, flag = TRUE, name = "x")

  # Store a list with a reference
  lst <- list(nested = list(deep = 1:10))

  SidecarEnv_set_raw_slot(obj, lst)
  result <- SidecarEnv_get_raw_slot(obj)

  # Should be identical (same SEXP, not a copy)
  expect_identical(result, lst)
})

# =============================================================================
# SidecarVctrs vctrs S3 dispatch tests
# =============================================================================

test_that("vec_ptype2.SidecarVctrs.SidecarVctrs returns correct ptype", {
  skip_if_not("vctrs" %in% rpkg_enabled_features(), "vctrs feature not enabled")

  obj1 <- rdata_sidecar_vctrs_new(vec_data = c(1.0, 2.0), vec_label = "a")
  obj2 <- rdata_sidecar_vctrs_new(vec_data = c(3.0), vec_label = "b")

  ptype <- vec_ptype2.SidecarVctrs.SidecarVctrs(obj1, obj2)

  expect_true(inherits(ptype, "SidecarVctrs"))
  expect_equal(length(ptype), 0L)
})

test_that("vec_cast.SidecarVctrs.SidecarVctrs returns identity", {
  skip_if_not("vctrs" %in% rpkg_enabled_features(), "vctrs feature not enabled")

  obj <- rdata_sidecar_vctrs_new(vec_data = c(1.5, 2.5), vec_label = "test")
  ptype <- rdata_sidecar_vctrs_new(vec_data = numeric(0), vec_label = "")

  result <- vec_cast.SidecarVctrs.SidecarVctrs(obj, ptype)

  expect_identical(result, obj)
})
