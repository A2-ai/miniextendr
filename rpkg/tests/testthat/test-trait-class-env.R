# Class environments that only a trait impl fills (#2289). Fixtures in
# src/rust/trait_class_env_tests.rs: `trait_env_rec` is an S3 class the
# package names and builds with its factory, `TraitEnvVctrs` a vctrs class
# with a constructor and `trait_env_vec` a vctrs class the package names,
# without one. Each has a `ClassEnvStatics` trait impl with the static
# method `label()` and the const `CODE`, which land in
# `<class>$ClassEnvStatics`.

trait_env_rd <- function(page) {
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  db[[paste0(page, ".Rd")]]
}

trait_env_rd_tags <- function(rd, tag) {
  nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
  trimws(gsub("[[:space:]]+", " ", unlist(nodes)))
}

expect_trait_env <- function(class, label, code) {
  env <- get(class, envir = asNamespace("miniextendr"), inherits = FALSE)
  expect_true(is.environment(env), info = class)
  expect_true(class %in% getNamespaceExports("miniextendr"), info = class)
  expect_null(env$new, info = class)
  expect_identical(env$ClassEnvStatics$label(), label, info = class)
  expect_identical(env$ClassEnvStatics$CODE(), code, info = class)
}

test_that("an S3 class the package names keeps the environment a trait impl uses", {
  expect_trait_env("trait_env_rec", "renamed S3", 21L)
  expect_identical(trait_env_rec$ClassEnvStatics$label(), "renamed S3")
  # The Rust name binds nothing, and there is no constructor.
  ns <- asNamespace("miniextendr")
  expect_false(exists("TraitEnvRecS3", envir = ns, inherits = FALSE))
  expect_false(exists("new_trait_env_rec", envir = ns, inherits = FALSE))
  expect_s3_class(trait_env_rec_of(3L), "trait_env_rec")
  expect_error(trait_env_rec_of(-1L), "negative value")
})

test_that("a vctrs class keeps the environment a trait impl uses", {
  expect_trait_env("TraitEnvVctrs", "vctrs", 32L)
  expect_identical(TraitEnvVctrs$ClassEnvStatics$CODE(), 32L)
  v <- new_traitenvvctrs(c(1, 2.5))
  expect_s3_class(v, "TraitEnvVctrs")
  expect_identical(vctrs::vec_data(v), c(1, 2.5))
})

test_that("a vctrs class the package names keeps the environment a trait impl uses", {
  expect_trait_env("trait_env_vec", "renamed vctrs", 43L)
  ns <- asNamespace("miniextendr")
  expect_false(exists("TraitEnvVecRec", envir = ns, inherits = FALSE))
  expect_false(exists("new_trait_env_vec", envir = ns, inherits = FALSE))
  expect_identical(trait_env_vec_total(c(1, 2.5)), 3.5)
})

test_that("a vctrs class without a trait namespace binds no environment", {
  # `NoCtorVctrs` (constructor_tests.rs) has no trait impl, and
  # `VctrsMatchArgScale` (match_arg_impl_tests.rs) has a constructor but no
  # trait impl: nothing lands in a class environment, so neither has one.
  ns <- asNamespace("miniextendr")
  expect_false(exists("NoCtorVctrs", envir = ns, inherits = FALSE))
  expect_false(exists("VctrsMatchArgScale", envir = ns, inherits = FALSE))
})

test_that("each class environment is documented on its class's page", {
  for (class in c("trait_env_rec", "TraitEnvVctrs", "trait_env_vec")) {
    rd <- trait_env_rd(class)
    expect_false(is.null(rd), info = class)
    expect_true(class %in% trait_env_rd_tags(rd, "\\alias"), info = class)
  }
  # The S3 impl has no docs: its page exists for the environment, titled by
  # default.
  expect_identical(
    trait_env_rd_tags(trait_env_rd("trait_env_rec"), "\\title"),
    "trait_env_rec S3 Class"
  )
})
