# Gated-off members must be absent from the native table and from R (#2107).
for (system in c("env", "r6", "s3", "s4", "s7", "vctrs")) {
  test_that(paste(system, "methods follow cfg and cfg_attr"), {
    feature_enabled <- "vctrs" %in% miniextendr_enabled_features()
    if (system == "vctrs") {
      skip_if_not(feature_enabled, "vctrs feature is disabled")
    }
    ns <- asNamespace("miniextendr")
    object <- switch(system,
      env = ns$GatedMethodEnv$new(10L),
      r6 = ns$GatedMethodR6$new(10L),
      s3 = ns$new_gatedmethods3(10L),
      s4 = ns$GatedMethodS4(10L),
      s7 = ns$GatedMethodS7(10L),
      vctrs = ns$new_gatedmethodvctrs(10)
    )
    call <- function(name) {
      switch(system,
        env = object[[name]](),
        r6 = object[[name]](),
        s3 = ns[[name]](object),
        s4 = ns[[paste0("s4_", name)]](object),
        s7 = ns[[name]](object),
        vctrs = ns[[paste0("gatedmethodvctrs_", name)]](object)
      )
    }
    expect_equal(call("cfg_value"), 10)
    expect_equal(call("cfg_renamed"), 11)
    expect_identical(call("cfg_feature"), feature_enabled)
    if (system %in% c("env", "r6")) {
      expect_null(object$cfg_disabled)
      expect_null(object$cfg_nested_disabled)
      expect_null(object$original_name)
    }
    if (system == "s7") {
      expect_identical(S7::prop(object, "current"), 10L)
      S7::prop(object, "current") <- 20L
      expect_identical(call("cfg_value"), 20L)
    }
  })
}

test_that("gated-off methods leave no native registrations or R functions", {
  native <- names(getDLLRegisteredRoutines("miniextendr")$.Call)
  gated <- native[grepl("_GatedMethod", native)]
  expect_true(any(grepl("__cfg_value$", gated)))
  expect_false(any(grepl("__cfg_(disabled|nested_disabled)$", gated)))
  ns <- asNamespace("miniextendr")
  expect_false(any(grepl("cfg_(disabled|nested_disabled)", ls(ns, all.names = TRUE))))
})

test_that("a gated-off parameter has no formal", {
  ns <- asNamespace("miniextendr")
  expect_identical(names(formals(ns$cfg_gate_params)), "kept")
  expect_identical(ns$cfg_gate_params(3L), 3L)
})

test_that("gated-off enum variants are neither levels nor choices", {
  ns <- asNamespace("miniextendr")
  expect_identical(levels(ns$cfg_gate_level_high()), c("Low", "High"))
  expect_identical(eval(formals(ns$cfg_gate_mode)$mode), c("Fast", "Slow"))
  expect_identical(ns$cfg_gate_mode("Slow"), "Slow")
  expect_error(ns$cfg_gate_mode("Hidden"))
})

test_that("trait gates select methods and implementations", {
  ns <- asNamespace("miniextendr")
  square <- ns$CfgGateSquare$new(3)
  expect_identical(square$CfgGateShape$area(), 9)
  expect_null(square$CfgGateShape$hidden)
  native <- names(getDLLRegisteredRoutines("miniextendr")$.Call)
  expect_true(any(grepl("CfgGateShape.*area", native)))
  expect_false(any(grepl("CfgGateShape.*hidden", native)))
})
