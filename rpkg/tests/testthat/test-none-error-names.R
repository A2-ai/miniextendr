skip_if_no_worker <- function() {
  skip_if_not(miniextendr_has_feature("worker-thread") ||
                miniextendr_has_feature("worker-default"), "worker feature not enabled")
}

test_that("standalone None errors use r_name and postfix", {
  expect_error(miniextendr:::none_named_main(), "`none_named_main()` returned no value", fixed = TRUE)
  expect_error(miniextendr:::none_postfix_main_entry(), "`none_postfix_main_entry()` returned no value", fixed = TRUE)
  # Renaming changes diagnostics only; scalar Option returns still use NA.
  expect_identical(miniextendr:::none_named_scalar(), NA_integer_)
})

test_that("standalone None errors use r_name and postfix on the worker", {
  skip_if_no_worker()
  expect_error(miniextendr:::none_named_worker(), "`none_named_worker()` returned no value", fixed = TRUE)
  expect_error(miniextendr:::none_postfix_worker_entry(), "`none_postfix_worker_entry()` returned no value", fixed = TRUE)
})

test_that("inherent None errors use the R member name", {
  lookup <- miniextendr:::NoneNameLookup$new()
  expect_error(lookup$find_value(), "`find_value()` returned no value", fixed = TRUE)
  expect_error(lookup$find_entry(), "`find_entry()` returned no value", fixed = TRUE)
})

test_that("inherent None errors use the R member name on the worker", {
  skip_if_no_worker()
  expect_error(miniextendr:::NoneNameLookup$load_value(), "`load_value()` returned no value", fixed = TRUE)
  expect_error(miniextendr:::NoneNameLookup$load_entry(), "`load_entry()` returned no value", fixed = TRUE)
})

for (system in c("env", "r6", "s3", "s4", "s7")) {
  test_that(paste(system, "None errors name the class entry"), {
    ns <- asNamespace("miniextendr")
    object <- switch(system,
      env = ns$NoneNameLookup$new(),
      r6 = ns$NoneNameR6$new(),
      s3 = ns$new_nonenames3(),
      s4 = ns$NoneNameS4(),
      s7 = ns$NoneNameS7()
    )
    entry <- switch(system, env = "find_value", r6 = "none_r6_value",
      s3 = "none_s3_value", s4 = "s4_none_s4_value", s7 = "none_s7_value")
    read <- if (system %in% c("env", "r6")) object[[entry]] else ns[[entry]]
    args <- if (system %in% c("env", "r6")) list() else list(object)
    expect_error(do.call(read, args), paste0("`", entry, "()` returned no value"), fixed = TRUE)
    if (system == "s7") {
      # Shortcuts share the generic's canonical diagnostic.
      expect_error(ns$NoneNameS7_none_s7_value(object), "`none_s7_value()` returned no value", fixed = TRUE)
      expect_error(ns$NoneNameS7_none_trait_value(object), "`s7_trait_NoneNameMethods_none_trait_value()` returned no value", fixed = TRUE)
    }
    if (system == "s4") {
      expect_error(ns$none_s4_lookup(object), "`none_s4_lookup()` returned no value", fixed = TRUE)
    }
    if (system != "env") {
      entry <- switch(system, r6 = "load_entry", s3 = "nonenames3_load_entry",
        s4 = "NoneNameS4_load_entry", s7 = "NoneNameS7_load_entry")
      load <- if (system == "r6") ns$NoneNameR6$load_entry else ns[[entry]]
      expect_error(load(), paste0("`", entry, "()` returned no value"), fixed = TRUE)
    }

    entry <- switch(system, s4 = "s4_trait_NoneNameMethods_none_trait_value",
      s7 = "s7_trait_NoneNameMethods_none_trait_value", "none_trait_value")
    read <- switch(system,
      env = ns$NoneNameLookup$NoneNameMethods$none_trait_value,
      r6 = ns$NoneNameR6$NoneNameMethods$none_trait_value,
      ns[[entry]])
    expect_error(read(object), paste0("`", entry, "()` returned no value"), fixed = TRUE)
    load <- switch(system,
      env = ns$NoneNameLookup$NoneNameMethods$none_trait_load,
      r6 = ns$NoneNameR6$NoneNameMethods$none_trait_load,
      s3 = ns$NoneNameS3$NoneNameMethods$none_trait_load,
      s4 = ns$NoneNameS4_NoneNameMethods_none_trait_load,
      s7 = attr(ns$NoneNameS7, "NoneNameMethods")$none_trait_load)
    entry <- if (system == "s4") "NoneNameS4_NoneNameMethods_none_trait_load" else "none_trait_load"
    expect_error(load(), paste0("`", entry, "()` returned no value"), fixed = TRUE)
  })
}

test_that("vctrs None errors name static helpers and protocol generics", {
  skip_if_not(miniextendr_has_feature("vctrs"), "vctrs feature not enabled")
  ns <- asNamespace("miniextendr")
  expect_error(ns$nonenamevctrs_load_entry(), "`nonenamevctrs_load_entry()` returned no value", fixed = TRUE)
  expect_error(format(ns$new_nonenamevctrs(1)), "`format()` returned no value", fixed = TRUE)
})
