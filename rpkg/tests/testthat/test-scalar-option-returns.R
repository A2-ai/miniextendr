# Scalar Option returns through the generated R class wrappers (#1415, ported
# from #1490 into #2121): None is the type's NA in every class system, as a
# standalone function's is. Option<()> still raises; Option<Self> gives NULL.

scalar_option_fixture <- function(system) {
  ns <- asNamespace("miniextendr")
  object <- switch(system,
    env = ns$ScalarOptionEnv$new(),
    r6 = ns$ScalarOptionR6$new(),
    s3 = ns$new_scalaroptions3(),
    s4 = ns$ScalarOptionS4(),
    s7 = ns$ScalarOptionS7(),
    vctrs = ns$new_scalaroptionvctrs(1)
  )
  function(kind, present) {
    name <- paste0("optional_", kind)
    switch(system,
      env = do.call("$", list(object, name))(present),
      r6 = do.call("$", list(object, name))(present),
      s3 = ns[[name]](object, present),
      s4 = ns[[paste0("s4_", name)]](object, present),
      s7 = ns[[name]](object, present),
      vctrs = ns[[paste0("scalaroptionvctrs_", name)]](object, present)
    )
  }
}

for (system in c("env", "r6", "s3", "s4", "s7", "vctrs")) {
  test_that(paste(system, "scalar Option methods preserve Some values and typed NA"), {
    if (system == "vctrs") {
      skip_if_not("vctrs" %in% miniextendr_enabled_features(), "vctrs feature is disabled")
    }
    call <- scalar_option_fixture(system)
    values <- list(integer = 42L, real = 2.5, logical = TRUE,
                   string = "owned", str = "borrowed")
    missing <- list(integer = NA_integer_, real = NA_real_, logical = NA,
                    string = NA_character_, str = NA_character_)
    for (kind in names(values)) {
      expect_identical(call(kind, TRUE), values[[kind]], info = kind)
      expect_identical(call(kind, FALSE), missing[[kind]], info = kind)
    }
    expect_null(call("unit", TRUE))
    expect_error(call("unit", FALSE), "returned no value")
  })
}

test_that("Option<Self> statics give a class object or NULL in every class system", {
  ns <- asNamespace("miniextendr")
  # The static's function, keyed by the class its Some is wrapped in.
  statics <- list(
    ScalarOptionEnv = ns$ScalarOptionEnv$optional_self,
    ScalarOptionR6 = ns$ScalarOptionR6$optional_self,
    ScalarOptionS3 = ns$scalaroptions3_optional_self,
    ScalarOptionS4 = ns$ScalarOptionS4_optional_self,
    `miniextendr::ScalarOptionS7` = ns$ScalarOptionS7_optional_self
  )
  for (class in names(statics)) {
    expect_true(inherits(statics[[class]](TRUE), class), info = class)
    expect_null(statics[[class]](FALSE), info = class)
  }
})

test_that("coerced, R-native, and path scalar Options retain their NA type", {
  object <- miniextendr:::ScalarOptionEnv$new()
  values <- list(i8 = 42L, i16 = 42L, u16 = 42L, u32 = 42L, f32 = 2.5,
                 i64 = 42L, u64 = 42L, isize = 42L, usize = 42L,
                 rboolean = TRUE, rlogical = TRUE, complex = 2+3i,
                 path = "path", os_string = "os")
  for (kind in names(values)) {
    call <- do.call("$", list(object, paste0("optional_", kind)))
    expect_identical(call(TRUE), values[[kind]], info = kind)
    expect_identical(call(FALSE), values[[kind]][NA_integer_], info = kind)
  }
})

test_that("strict scalar Option methods preserve conversion checks", {
  class <- miniextendr:::ScalarOptionEnv
  call <- class$optional_strict
  expect_identical(call(TRUE, FALSE), 42L)
  expect_identical(call(FALSE, FALSE), NA_integer_)
  expect_identical(call(FALSE, TRUE), NA_integer_)
  expect_error(call(TRUE, TRUE))
  expect_s3_class(class$optional_self(TRUE), "ScalarOptionEnv")
  expect_null(class$optional_self(FALSE))
})
