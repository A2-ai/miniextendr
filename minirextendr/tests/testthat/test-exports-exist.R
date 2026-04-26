# These smoke tests ensure exported helpers are present; heavier integration is handled elsewhere.

exported_funs <- c(
  "cargo_add", "cargo_build", "cargo_check", "cargo_clippy", "cargo_deps",
  "cargo_doc", "cargo_fmt", "cargo_init", "cargo_new", "cargo_rm", "cargo_search",
  "cargo_test", "cargo_update", "create_miniextendr_package", "miniextendr_autoconf",
  "miniextendr_available_versions", "miniextendr_build", "miniextendr_cache_info",
  "miniextendr_check", "miniextendr_check_rust", "miniextendr_clear_cache",
  "miniextendr_configure", "miniextendr_dev_link",
  "miniextendr_doctor", "miniextendr_validate",
  "miniextendr_vendor",
  "miniextendr_config", "miniextendr_config_defaults",
  "upgrade_miniextendr_package",
  "use_miniextendr", "use_miniextendr_bootstrap", "use_miniextendr_build_rs",
  "use_miniextendr_cleanup", "use_miniextendr_config_scripts", "use_miniextendr_configure",
  "use_miniextendr_configure_win", "use_miniextendr_description",
  "use_miniextendr_gitignore", "use_miniextendr_makevars",
  "use_miniextendr_mx_abi", "use_miniextendr_package_doc", "use_miniextendr_rbuildignore",
  "use_miniextendr_stub",
  "use_miniextendr_rust",
  "use_configure_feature_detection", "add_feature_rule",
  "remove_feature_rule", "list_feature_rules", "list_cargo_features",
  "use_vendor_lib",
  "rust_source", "rust_function", "rust_source_clean",
  "vendor_crates_io", "vendor_miniextendr"
)

test_that("exported helper functions exist", {
  missing <- exported_funs[!vapply(exported_funs, exists, logical(1), envir = asNamespace("minirextendr"))]
  expect_equal(
    length(missing), 0,
    label = paste("missing exports:", paste(missing, collapse = ", "))
  )
})
