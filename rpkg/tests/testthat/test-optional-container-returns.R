# Both presence states go through each generated R class system (#2088).
optional_container_fixture <- function(system) {
  ns <- asNamespace("miniextendr")
  object <- switch(system,
    env = ns$OptionalContainerEnv$new(),
    r6 = ns$OptionalContainerR6$new(),
    s3 = ns$new_optionalcontainers3(),
    s4 = ns$OptionalContainerS4(),
    s7 = ns$OptionalContainerS7(),
    vctrs = ns$new_optionalcontainervctrs(1)
  )
  function(kind, present) {
    name <- paste0("oc_", kind)
    switch(system,
      env = object[[name]](present),
      r6 = object[[name]](present),
      s3 = ns[[name]](object, present),
      s4 = ns[[paste0("s4_", name)]](object, present),
      s7 = ns[[name]](object, present),
      vctrs = ns[[paste0("optionalcontainervctrs_", name)]](object, present)
    )
  }
}

for (system in c("env", "r6", "s3", "s4", "s7", "vctrs")) {
  test_that(paste(system, "optional container/reference methods return NULL for None"), {
    if (system == "vctrs") {
      skip_if_not("vctrs" %in% miniextendr_enabled_features(), "vctrs feature is disabled")
    }
    call <- optional_container_fixture(system)
    values <- list(integers = c(1L, 2L), strings = c("a", "b"),
                   hash_map = list(a = 1L), tree_map = list(a = c(1L, 2L)),
                   hash_set = 42L, tree_set = c("a", "b"), reference = 42L,
                   empty = integer(), boxed = c(1L, 2L), slice = c(1L, 2L),
                   array = c(1L, 2L), tuple = list(1L, "a"))
    for (kind in names(values)) {
      expect_identical(call(kind, TRUE), values[[kind]], info = kind)
      expect_null(call(kind, FALSE), info = kind)
    }
  })
}

test_that("trait methods preserve optional container/reference absence", {
  object <- miniextendr:::OptionalContainerEnv$new()
  methods <- object$OptionalContainerReturns
  values <- list(integers = c(1L, 2L), map = list(a = c(1L, 2L)),
                 set = c("a", "b"), reference = 42L)
  for (kind in names(values)) {
    call <- methods[[paste0("oct_", kind)]]
    expect_identical(call(TRUE), values[[kind]], info = kind)
    expect_null(call(FALSE), info = kind)
  }
})
