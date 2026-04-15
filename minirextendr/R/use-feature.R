# Feature configuration functions
#
# These functions configure miniextendr features that require R package dependencies.
# They handle both the Cargo feature (if applicable) and the R package import.

# =============================================================================
# Helper functions
# =============================================================================

#' Add a feature to Cargo.toml
#'
#' Adds a feature line to the `[features]` section of src/rust/Cargo.toml.
#'
#' @param feature_name Name of the feature (e.g., "vctrs")
#' @param feature_spec Feature specification (e.g., "miniextendr-api/vctrs")
#' @return Invisibly returns TRUE if modified, FALSE if already present
#' @noRd
add_cargo_feature <- function(feature_name, feature_spec)
{
  cargo_in <- usethis::proj_path("src", "rust", "Cargo.toml")

  if (!fs::file_exists(cargo_in)) {
    cli::cli_abort(c(
      "Cargo.toml not found at {.path {cargo_in}}",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  lines <- readLines(cargo_in, warn = FALSE)
  feature_line <- sprintf('%s = ["%s"]', feature_name, feature_spec)

  # Check if feature already exists
  if (any(grepl(sprintf("^%s\\s*=", feature_name), lines))) {
    cli::cli_alert_info("Feature {.val {feature_name}} already in Cargo.toml")
    return(invisible(FALSE))
  }


  # Find [features] section and add after it
  features_idx <- grep("^\\[features\\]", lines)
  if (length(features_idx) == 0) {
    cli::cli_abort("No [features] section found in Cargo.toml")
  }

  # Find end of features section (next section or EOF)
  next_section <- grep("^\\[", lines)
  next_section <- next_section[next_section > features_idx[1]]
  if (length(next_section) > 0) {
    insert_at <- next_section[1] - 1
  } else {
    insert_at <- length(lines)
  }

  # Insert the feature line
  lines <- append(lines, feature_line, after = insert_at)
  writeLines(lines, cargo_in)

  cli::cli_alert_success("Added feature {.val {feature_name}} to Cargo.toml")
  invisible(TRUE)
}

#' Add an R package to Imports in DESCRIPTION
#'
#' @param pkg Package name to add to Imports
#' @param min_version Optional minimum version (e.g., ">= 0.6.0")
#' @return Invisibly returns TRUE if modified, FALSE if already present
#' @noRd
add_import <- function(pkg, min_version = NULL) {
  desc_path <- usethis::proj_path("DESCRIPTION")

  if (!fs::file_exists(desc_path)) {
    cli::cli_abort("DESCRIPTION file not found")
  }

  # Check if already in Imports
  deps <- mx_desc_get_deps(desc_path)
  imports <- deps[deps$type == "Imports", ]

  if (pkg %in% imports$package) {
    cli::cli_alert_info("{.pkg {pkg}} already in Imports")
    return(invisible(FALSE))
  }

  # Add to Imports
  mx_desc_set_dep(desc_path, pkg, type = "Imports", version = min_version)
  cli::cli_alert_success("Added {.pkg {pkg}} to Imports in DESCRIPTION")
  invisible(TRUE)
}

# =============================================================================
# Feature configuration functions
# =============================================================================

#' Enable vctrs support
#'
#' Configures your package to use vctrs for custom vector types.
#' This enables:
#' - The `vctrs` Cargo feature in miniextendr-api
#' - `vctrs::new_vctr()`, `vctrs::new_rcrd()`, `vctrs::new_list_of()` constructors
#' - S3 method generation for vctrs generics (`vec_proxy`, `vec_restore`, etc.)
#' - Automatic `@importFrom vctrs` in generated R wrappers
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_vctrs()
#' }
use_vctrs <- function(path = ".") {
  with_project(path)
  # Add Cargo feature
  add_cargo_feature("vctrs", "miniextendr-api/vctrs")

  # Add R package dependency
  add_import("vctrs", ">= 0.6.0")

  cli::cli_alert_info("See {.url https://vctrs.r-lib.org/} for vctrs documentation")

  invisible(TRUE)
}

#' Enable R6 class system support
#'
#' Adds R6 as a dependency for using the R6 class system with `#[miniextendr(r6)]`.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_r6()
#' }
use_r6 <- function(path = ".") {
  with_project(path)
  add_import("R6")

  cli::cli_alert_info("Use {.code #[miniextendr(r6)]} on impl blocks for R6 classes")
  cli::cli_alert_info("See {.url https://r6.r-lib.org/} for R6 documentation")

  invisible(TRUE)
}

#' Enable S7 class system support
#'
#' Adds S7 as a dependency for using the S7 class system with `#[miniextendr(s7)]`.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_s7()
#' }
use_s7 <- function(path = ".") {
  with_project(path)
  add_import("S7", ">= 0.2.0")

  cli::cli_alert_info("Use {.code #[miniextendr(s7)]} on impl blocks for S7 classes")
  cli::cli_alert_info("See {.url https://rconsortium.github.io/S7/} for S7 documentation")

  invisible(TRUE)
}

#' Enable S4 class system support
#'
#' Adds methods as a dependency for using the S4 class system with `#[miniextendr(s4)]`.
#' Note: The methods package is usually already available as a base R package.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_s4()
#' }
use_s4 <- function(path = ".") {
  with_project(path)
  add_import("methods")

  cli::cli_alert_info("Use {.code #[miniextendr(s4)]} on impl blocks for S4 classes")

  invisible(TRUE)
}

#' Enable rayon parallel processing support
#'
#' Configures your package to use rayon for parallel iterators.
#' This is an optional feature that adds compile-time cost but enables
#' easy parallelization of Rust code.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_rayon()
#' }
use_rayon <- function(path = ".") {
  with_project(path)
  add_cargo_feature("rayon", "miniextendr-api/rayon")

  cli::cli_alert_info("Use {.code rayon::prelude::*} for parallel iterators")

  invisible(TRUE)
}

#' Enable serde serialization support
#'
#' Configures your package to use serde for serialization.
#' Useful for JSON, TOML, or other format support.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_serde()
#' }
use_serde <- function(path = ".") {
  with_project(path)
  add_cargo_feature("serde", "miniextendr-api/serde")

  cli::cli_alert_info("Derive with {.code #[derive(Serialize, Deserialize)]}")

  invisible(TRUE)
}

# =============================================================================
# Feature Detection Generator
# =============================================================================

#' Generate feature detection code
#'
#' Generates Rust code that exposes enabled Cargo features to R, and R helper
#' functions to query them. This allows tests to skip when features are missing.
#'
#' @details
#' This function scans your `src/rust/Cargo.toml` for features and generates:
#'
#' 1. **Rust code** (`<package>_enabled_features()`) - Returns a vector of enabled feature names
#' 2. **R helpers** (`has_feature()`, `skip_if_missing_feature()`) - For runtime feature checks
#'
#' The generated Rust function uses `cfg!(feature = "...")` at compile time to build

#' a list of enabled features. This is useful for:
#'
#' - Skipping tests when optional features are not compiled in
#' - Conditional code paths based on available features
#' - Documentation of what's included in a build
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param package_name Package name (default: derived from DESCRIPTION)
#' @param features Character vector of feature names to include. Default NULL
#'   scans Cargo.toml automatically.
#' @param rust_file Path for generated Rust code (default: "src/rust/feature_detection.rs")
#' @param r_file Path for generated R helpers (default: "R/feature_helpers.R")
#' @param overwrite Logical, whether to overwrite existing files
#'
#' @return Invisibly returns list of generated file paths
#' @export
#'
#' @examples
#' \dontrun{
#' # Auto-detect features from Cargo.toml
#' use_feature_detection()
#'
#' # Manually specify features
#' use_feature_detection(features = c("rayon", "serde", "uuid"))
#' }
use_feature_detection <- function(path = ".",
                                   package_name = NULL,
                                   features = NULL,
                                   rust_file = "src/rust/feature_detection.rs",
                                   r_file = "R/feature_helpers.R",
                                   overwrite = FALSE) {
  with_project(path)

  # Get package name from DESCRIPTION if not provided
  if (is.null(package_name)) {
    desc_path <- usethis::proj_path("DESCRIPTION")
    if (fs::file_exists(desc_path)) {
      package_name <- mx_desc_get_field("Package", file = desc_path)
    } else {
      cli::cli_abort("Could not determine package name. Provide it explicitly or run from package root.")
    }
  }

  # Detect features from Cargo.toml if not provided
  if (is.null(features)) {
    features <- detect_cargo_features()
    if (length(features) == 0) {
      cli::cli_alert_warning("No features found in Cargo.toml")
      cli::cli_alert_info("Add features to [features] section or specify manually")
    }
  }

  cli::cli_alert_info("Generating feature detection for {length(features)} features")

  # Generate Rust code
  rust_path <- usethis::proj_path(rust_file)
  if (fs::file_exists(rust_path) && !overwrite) {
    cli::cli_alert_warning("{.path {rust_file}} already exists. Use {.code overwrite = TRUE} to replace.")
  } else {
    rust_code <- generate_feature_detection_rust(package_name, features)
    ensure_dir(dirname(rust_path))
    writeLines(rust_code, rust_path)
    cli::cli_alert_success("Created {.path {rust_file}}")
  }

  # Generate R helpers
  r_path <- usethis::proj_path(r_file)
  if (fs::file_exists(r_path) && !overwrite) {
    cli::cli_alert_warning("{.path {r_file}} already exists. Use {.code overwrite = TRUE} to replace.")
  } else {
    r_code <- generate_feature_detection_r(package_name)
    ensure_dir(dirname(r_path))
    writeLines(r_code, r_path)
    cli::cli_alert_success("Created {.path {r_file}}")
  }

  cli::cli_alert_info("Remember to:")
  cli::cli_bullets(c(
    " " = "Add {.code mod feature_detection;} to lib.rs",
    " " = "Run {.code devtools::document()} to update NAMESPACE"
  ))

  invisible(list(rust = rust_path, r = r_path))
}

#' Update feature detection to match Cargo.toml
#'
#' Re-scans Cargo.toml and regenerates feature detection code.
#' Equivalent to calling `use_feature_detection(overwrite = TRUE)`.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param overwrite Logical, whether to overwrite existing files (default TRUE)
#' @return Invisibly returns list of generated file paths
#' @keywords internal
#' @export
#'
#' @examples
#' \dontrun{
#' # After adding new features via use_rayon(), use_serde(), etc.
#' update_feature_detection()
#' }
update_feature_detection <- function(path = ".", overwrite = TRUE) {
  use_feature_detection(path = path, overwrite = overwrite)
}

#' Detect features from Cargo.toml
#'
#' Parses the `[features]` section of src/rust/Cargo.toml to extract feature names.
#'
#' @return Character vector of feature names
#' @noRd
detect_cargo_features <- function() {
  cargo_in <- usethis::proj_path("src", "rust", "Cargo.toml")

  if (!fs::file_exists(cargo_in)) {
    cli::cli_warn("Cargo.toml not found at {.path {cargo_in}}")
    return(character())
  }

  lines <- readLines(cargo_in, warn = FALSE)

  # Find [features] section
  features_idx <- grep("^\\[features\\]", lines)
  if (length(features_idx) == 0) {
    return(character())
  }

  # Find next section
  next_section <- grep("^\\[", lines)
  next_section <- next_section[next_section > features_idx[1]]
  if (length(next_section) > 0) {
    end_idx <- next_section[1] - 1
  } else {
    end_idx <- length(lines)
  }

  # Extract feature names (lines like: feature_name = [...])
  feature_lines <- lines[(features_idx[1] + 1):end_idx]
  feature_lines <- feature_lines[grepl("^[a-zA-Z0-9_-]+\\s*=", feature_lines)]

  # Extract just the feature names
  features <- sub("\\s*=.*", "", feature_lines)
  features <- trimws(features)

  # Filter out "default" feature
  features <- features[features != "default"]

  features
}

#' Generate Rust feature detection code
#'
#' @param package_name Package name
#' @param features Vector of feature names
#' @return Character string of Rust code
#' @noRd
generate_feature_detection_rust <- function(package_name, features) {
  package_rs <- gsub("[.-]", "_", package_name)
  fn_name <- paste0(package_rs, "_enabled_features")

  # Build cfg! checks for each feature
  checks <- vapply(features, function(f) {
    sprintf('    if cfg!(feature = "%s") {\n        features.push("%s");\n    }', f, f)
  }, character(1))

  code <- sprintf(
'//! Feature detection - generated by minirextendr::use_feature_detection()
//!
//! This module provides runtime access to compile-time feature flags.
//! Regenerate with: minirextendr::update_feature_detection()

use miniextendr_api::miniextendr;

/// Returns a vector of enabled Cargo features
///
/// This function is auto-generated from Cargo.toml features.
/// Use `%s_has_feature()` in R to check for specific features.
#[miniextendr]
pub fn %s() -> Vec<&\'static str> {
    let mut features = Vec::new();

%s

    features
}
',
    package_rs,
    fn_name,
    paste(checks, collapse = "\n")
  )

  code
}

#' Generate R feature helper code
#'
#' @param package_name Package name
#' @return Character string of R code
#' @noRd
generate_feature_detection_r <- function(package_name) {
  package_rs <- gsub("[.-]", "_", package_name)
  fn_name <- paste0(package_rs, "_enabled_features")

  code <- sprintf(
'# Feature detection helpers - generated by minirextendr::use_feature_detection()
#
# These functions provide access to compile-time feature flags at runtime.
# Regenerate with: minirextendr::update_feature_detection()

#\' Check if a feature is enabled
#\'
#\' Check if a specific optional feature was compiled into the package.
#\'
#\' @param name Character string naming the feature to check.
#\' @return Logical `TRUE` if the feature is enabled, `FALSE` otherwise.
#\' @examples
#\' %s_has_feature("rayon")
#\' %s_has_feature("serde")
#\' @export
%s_has_feature <- function(name) {
  name %%in%% %s()
}

#\' Skip test if feature is missing
#\'
#\' For use in testthat tests to skip tests when an optional feature is not enabled.
#\'
#\' @param name Character string naming the required feature.
#\' @return Invisibly returns `NULL`. Called for its side effect of skipping tests.
#\' @examples
#\' \\dontrun{
#\' test_that("rayon feature works", {
#\'   skip_if_missing_feature("%s")
#\'   # ... test code ...
#\' })
#\' }
#\' @export
skip_if_missing_feature <- function(name) {
  if (!%s_has_feature(name)) {
    testthat::skip(paste("feature not enabled:", name))
  }
}
',
    package_rs, package_rs,
    package_rs, fn_name,
    package_name,
    package_rs
  )

  code
}

# =============================================================================
# Render workflow scaffolding helpers
# =============================================================================

#' Add miniextendr knitr setup to a vignette
#'
#' Prepends a setup chunk that calls [miniextendr_knitr_setup()] to
#' the specified Rmd/qmd file, ensuring the Rust package is built
#' before code chunks run.
#'
#' @param path Path to the Rmd or qmd file.
#' @return Invisibly returns `TRUE` if the file was modified.
#' @export
#'
#' @examples
#' \dontrun{
#' use_miniextendr_knitr("vignettes/my-vignette.Rmd")
#' }
use_miniextendr_knitr <- function(path) {
  if (missing(path) || !is.character(path) || length(path) != 1) {
    cli::cli_abort("Provide a single path to an Rmd or qmd file")
  }
  if (!file.exists(path)) {
    cli::cli_abort("File not found: {.path {path}}")
  }

  lines <- readLines(path, warn = FALSE)

  # Check if setup chunk already exists
  if (any(grepl("miniextendr_knitr_setup", lines, fixed = TRUE))) {
    cli::cli_alert_info("miniextendr_knitr_setup() already present in {.path {path}}")
    return(invisible(FALSE))
  }

  setup_chunk <- c(
    "```{r setup, include=FALSE}",
    "minirextendr::miniextendr_knitr_setup()",
    "```",
    ""
  )

  # Insert after YAML front matter
  yaml_delims <- grep("^---\\s*$", lines)
  if (length(yaml_delims) >= 2) {
    insert_after <- yaml_delims[2]
  } else {
    insert_after <- 0
  }

  lines <- append(lines, setup_chunk, after = insert_after)
  writeLines(lines, path)
  cli::cli_alert_success("Added miniextendr_knitr_setup() chunk to {.path {path}}")
  invisible(TRUE)
}

#' Set miniextendr rmarkdown output format
#'
#' Replaces the output format in an Rmd file's YAML header with
#' a miniextendr-aware wrapper format (e.g., `miniextendr_html_document`).
#'
#' @param path Path to the Rmd file.
#' @param format Output format name. One of `"html_document"` (default),
#'   `"pdf_document"`, or `"word_document"`.
#' @return Invisibly returns `TRUE` if the file was modified.
#' @export
#'
#' @examples
#' \dontrun{
#' use_miniextendr_rmarkdown("vignettes/my-vignette.Rmd")
#' }
use_miniextendr_rmarkdown <- function(path,
                                       format = c("html_document",
                                                   "pdf_document",
                                                   "word_document")) {
  if (missing(path) || !is.character(path) || length(path) != 1) {
    cli::cli_abort("Provide a single path to an Rmd file")
  }
  if (!file.exists(path)) {
    cli::cli_abort("File not found: {.path {path}}")
  }
  format <- match.arg(format)

  mx_format <- paste0("minirextendr::miniextendr_", format)
  lines <- readLines(path, warn = FALSE)

  # Check if already using miniextendr format
  if (any(grepl(mx_format, lines, fixed = TRUE))) {
    cli::cli_alert_info("{.val {mx_format}} already in {.path {path}}")
    return(invisible(FALSE))
  }

  # Find and replace output format in YAML header
  yaml_delims <- grep("^---\\s*$", lines)
  if (length(yaml_delims) < 2) {
    cli::cli_abort("No YAML front matter found in {.path {path}}")
  }

  yaml_start <- yaml_delims[1]
  yaml_end <- yaml_delims[2]

  # Look for output: line in YAML
  output_idx <- grep("^output:", lines[yaml_start:yaml_end]) + yaml_start - 1
  if (length(output_idx) == 0) {
    # No output field — add one before end of YAML
    lines <- append(lines, paste0("output: ", mx_format), after = yaml_end - 1)
  } else {
    # Replace existing output line
    lines[output_idx[1]] <- paste0("output: ", mx_format)
  }

  writeLines(lines, path)
  cli::cli_alert_success("Set output format to {.val {mx_format}} in {.path {path}}")
  invisible(TRUE)
}

#' Add miniextendr pre-render hook to Quarto project
#'
#' Adds a `project.pre-render` entry to `_quarto.yml` that calls
#' [miniextendr_quarto_pre_render()] before rendering.
#'
#' @param path Path to the directory containing `_quarto.yml`,
#'   or the `_quarto.yml` file itself.
#' @return Invisibly returns `TRUE` if the file was modified.
#' @export
#'
#' @examples
#' \dontrun{
#' use_miniextendr_quarto("vignettes/")
#' }
use_miniextendr_quarto <- function(path = ".") {
  if (dir.exists(path)) {
    qmd_path <- file.path(path, "_quarto.yml")
  } else {
    qmd_path <- path
  }

  pre_render_cmd <- "Rscript -e 'minirextendr::miniextendr_quarto_pre_render()'"

  if (!file.exists(qmd_path)) {
    # Create minimal _quarto.yml
    writeLines(c(
      "project:",
      paste0("  pre-render: ", pre_render_cmd)
    ), qmd_path)
    cli::cli_alert_success("Created {.path {qmd_path}} with pre-render hook")
    return(invisible(TRUE))
  }

  lines <- readLines(qmd_path, warn = FALSE)

  # Check if already present
  if (any(grepl("miniextendr_quarto_pre_render", lines, fixed = TRUE))) {
    cli::cli_alert_info("miniextendr pre-render hook already in {.path {qmd_path}}")
    return(invisible(FALSE))
  }

  # Look for project: section
  project_idx <- grep("^project:", lines)
  if (length(project_idx) == 0) {
    # Append project section
    lines <- c(lines, "", "project:",
               paste0("  pre-render: ", pre_render_cmd))
  } else {
    # Insert pre-render after project: line
    lines <- append(lines, paste0("  pre-render: ", pre_render_cmd),
                    after = project_idx[1])
  }

  writeLines(lines, qmd_path)
  cli::cli_alert_success("Added miniextendr pre-render hook to {.path {qmd_path}}")
  invisible(TRUE)
}
