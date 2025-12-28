# These smoke tests ensure exported helpers are present; heavier integration is handled elsewhere.

exported_funs <- c(
  "cargo_add", "cargo_build", "cargo_check", "cargo_clippy", "cargo_deps",
  "cargo_doc", "cargo_fmt", "cargo_init", "cargo_rm", "cargo_search",
  "cargo_test", "cargo_update", "create_miniextendr_package", "miniextendr_autoconf",
  "miniextendr_available_versions", "miniextendr_build", "miniextendr_check",
  "miniextendr_configure", "miniextendr_document", "miniextendr_update",
  "use_miniextendr", "use_miniextendr_bootstrap", "use_miniextendr_cargo_config",
  "use_miniextendr_cleanup", "use_miniextendr_config_scripts", "use_miniextendr_configure",
  "use_miniextendr_configure_win", "use_miniextendr_description", "use_miniextendr_document",
  "use_miniextendr_entrypoint", "use_miniextendr_gitignore", "use_miniextendr_makevars",
  "use_miniextendr_package_doc", "use_miniextendr_rbuildignore", "use_miniextendr_rust",
  "vendor_crates_io", "vendor_miniextendr"
)

test_that("exported helper functions exist", {
  missing <- exported_funs[!vapply(exported_funs, exists, logical(1), envir = asNamespace("minirextendr"))]
  expect_length(missing, 0, info = paste("missing:", paste(missing, collapse = ", ")))
})
