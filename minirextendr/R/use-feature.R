# Feature configuration functions
#
# These functions configure miniextendr features that require R package dependencies.
# They handle both the Cargo feature (if applicable) and the R package import.

# =============================================================================
# Helper functions
# =============================================================================

#' Add a feature to Cargo.toml.in
#'
#' Adds a feature line to the [features] section of src/rust/Cargo.toml.in.
#'
#' @param feature_name Name of the feature (e.g., "vctrs")
#' @param feature_spec Feature specification (e.g., "miniextendr-api/vctrs")
#' @return Invisibly returns TRUE if modified, FALSE if already present
#' @noRd
add_cargo_feature <- function(feature_name, feature_spec)
{
  cargo_in <- usethis::proj_path("src", "rust", "Cargo.toml.in")

  if (!fs::file_exists(cargo_in)) {
    abort(c(
      "Cargo.toml.in not found at {.path {cargo_in}}",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  lines <- readLines(cargo_in, warn = FALSE)
  feature_line <- sprintf('%s = ["%s"]', feature_name, feature_spec)

  # Check if feature already exists
  if (any(grepl(sprintf("^%s\\s*=", feature_name), lines))) {
    cli::cli_alert_info("Feature {.val {feature_name}} already in Cargo.toml.in")
    return(invisible(FALSE))
  }


  # Find [features] section and add after it
  features_idx <- grep("^\\[features\\]", lines)
  if (length(features_idx) == 0) {
    abort("No [features] section found in Cargo.toml.in")
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

  cli::cli_alert_success("Added feature {.val {feature_name}} to Cargo.toml.in")
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
    abort("DESCRIPTION file not found")
  }

  d <- desc::desc(desc_path)

  # Check if already in Imports
  imports <- d$get_deps()
  imports <- imports[imports$type == "Imports", ]

  if (pkg %in% imports$package) {
    cli::cli_alert_info("{.pkg {pkg}} already in Imports")
    return(invisible(FALSE))
  }

  # Add to Imports
  if (!is.null(min_version)) {
    d$set_dep(pkg, type = "Imports", version = min_version)
  } else {
    d$set_dep(pkg, type = "Imports")
  }

  d$write()
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
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_vctrs()
#' }
use_vctrs <- function() {
  # Add Cargo feature
  add_cargo_feature("vctrs", "miniextendr-api/vctrs")

  # Add R package dependency
  add_import("vctrs", ">= 0.6.0")

  cli::cli_alert_info("Run {.code ./configure} to regenerate Cargo.toml")
  cli::cli_alert_info("See {.url https://vctrs.r-lib.org/} for vctrs documentation")

  invisible(TRUE)
}

#' Enable R6 class system support
#'
#' Adds R6 as a dependency for using the R6 class system with `#[miniextendr(r6)]`.
#'
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_r6()
#' }
use_r6 <- function() {
  add_import("R6")

  cli::cli_alert_info("Use {.code #[miniextendr(r6)]} on impl blocks for R6 classes")
  cli::cli_alert_info("See {.url https://r6.r-lib.org/} for R6 documentation")

  invisible(TRUE)
}

#' Enable S7 class system support
#'
#' Adds S7 as a dependency for using the S7 class system with `#[miniextendr(s7)]`.
#'
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_s7()
#' }
use_s7 <- function() {
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
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_s4()
#' }
use_s4 <- function() {
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
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_rayon()
#' }
use_rayon <- function() {
  add_cargo_feature("rayon", "miniextendr-api/rayon")

  cli::cli_alert_info("Run {.code ./configure} to regenerate Cargo.toml")
  cli::cli_alert_info("Use {.code rayon::prelude::*} for parallel iterators")

  invisible(TRUE)
}

#' Enable serde serialization support
#'
#' Configures your package to use serde for serialization.
#' Useful for JSON, TOML, or other format support.
#'
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_serde()
#' }
use_serde <- function() {
  add_cargo_feature("serde", "miniextendr-api/serde")

  cli::cli_alert_info("Run {.code ./configure} to regenerate Cargo.toml")
  cli::cli_alert_info("Derive with {.code #[derive(Serialize, Deserialize)]}")

  invisible(TRUE)
}
