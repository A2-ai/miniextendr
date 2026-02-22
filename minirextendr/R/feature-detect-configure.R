# Configure-time feature detection
#
# These functions manage tools/detect-features.R, a script that runs during
# ./configure to auto-detect which Cargo features should be enabled based on
# the build environment (e.g., "vctrs R package is installed -> enable vctrs
# Cargo feature").
#
# This is distinct from use_feature_detection() which generates runtime
# Rust->R code so R can query which features were *compiled in*.

# =============================================================================
# Public API
# =============================================================================

#' Set up configure-time feature detection
#'
#' Creates the infrastructure for automatically detecting which Cargo features
#' to enable at `./configure` time. This is a one-time setup that:
#'
#' 1. Creates `tools/detect-features.R` with empty rules
#' 2. Patches `configure.ac` to call the script when the features env var isn't set
#' 3. Runs `autoconf` to regenerate `configure`
#'
#' After setup, use [add_feature_rule()] to add detection rules.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' use_configure_feature_detection()
#' add_feature_rule("rayon", detect = TRUE)
#' add_feature_rule("vctrs", detect = 'requireNamespace("vctrs", quietly = TRUE)')
#' }
use_configure_feature_detection <- function(path = ".") {
  with_project(path)

  # Validate this is a miniextendr package
  configure_ac_path <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac_path)) {
    abort(c(
      "configure.ac not found",
      "i" = "Run {.code minirextendr::use_miniextendr()} first"
    ))
  }

  # Get package metadata
  data <- template_data()
  package_name <- data$package
  features_var <- data$features_var

  # 1. Create tools/detect-features.R
  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (fs::file_exists(script_path)) {
    cli::cli_alert_info("{.path tools/detect-features.R} already exists")
  } else {
    script_content <- generate_empty_detect_script(package_name, features_var)
    ensure_dir(dirname(script_path))
    writeLines(script_content, script_path)
    bullet_created("tools/detect-features.R")
  }

  # 2. Patch configure.ac
  patch_configure_ac_for_detection(configure_ac_path, features_var)

  # 3. Run autoconf
  if (nzchar(Sys.which("autoconf"))) {
    miniextendr_autoconf()
  } else {
    cli::cli_alert_info("Run {.code miniextendr_autoconf()} to regenerate configure")
  }

  invisible(TRUE)
}

#' Add a feature detection rule
#'
#' Adds a rule to `tools/detect-features.R` that controls whether a Cargo
#' feature is enabled at configure time.
#'
#' @param feature Cargo feature name (e.g., `"vctrs"`, `"rayon"`).
#' @param detect Detection expression. One of:
#'   - `TRUE` -- always enable the feature
#'   - A string containing an R expression that returns TRUE/FALSE
#'     (e.g., `'requireNamespace("vctrs", quietly = TRUE)'`)
#' @param cargo_spec Optional Cargo feature specification. If provided, also
#'   adds the feature to `[features]` in `Cargo.toml` via [add_cargo_feature()].
#'   For example, `"miniextendr-api/vctrs"`.
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' # Always enable rayon
#' add_feature_rule("rayon", detect = TRUE, cargo_spec = "miniextendr-api/rayon")
#'
#' # Enable vctrs only if the R package is available
#' add_feature_rule("vctrs",
#'   detect = 'requireNamespace("vctrs", quietly = TRUE)',
#'   cargo_spec = "miniextendr-api/vctrs")
#' }
add_feature_rule <- function(feature, detect, cargo_spec = NULL, path = ".") {
  with_project(path)

  if (!is.character(feature) || length(feature) != 1 || !nzchar(feature)) {
    abort("{.arg feature} must be a single non-empty string")
  }

  # Validate detect
  if (!isTRUE(detect) && !(is.character(detect) && length(detect) == 1)) {
    abort("{.arg detect} must be TRUE or a single string containing an R expression")
  }

  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (!fs::file_exists(script_path)) {
    abort(c(
      "{.path tools/detect-features.R} not found",
      "i" = "Run {.code use_configure_feature_detection()} first"
    ))
  }

  # Check for duplicate
  existing <- parse_detect_features_script(script_path)
  if (feature %in% names(existing)) {
    cli::cli_alert_info("Feature rule {.val {feature}} already exists")
    return(invisible(TRUE))
  }

  # Append rule
  append_feature_rule(script_path, feature, detect)
  cli::cli_alert_success("Added feature detection rule for {.val {feature}}")

  # Optionally add Cargo feature
  if (!is.null(cargo_spec)) {
    add_cargo_feature(feature, cargo_spec)
  }

  invisible(TRUE)
}

#' Remove a feature detection rule
#'
#' Removes a rule from `tools/detect-features.R`.
#'
#' @param feature Cargo feature name to remove.
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if removed, FALSE if not found
#' @export
#'
#' @examples
#' \dontrun{
#' remove_feature_rule("rayon")
#' }
remove_feature_rule <- function(feature, path = ".") {
  with_project(path)

  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (!fs::file_exists(script_path)) {
    abort(c(
      "{.path tools/detect-features.R} not found",
      "i" = "Run {.code use_configure_feature_detection()} first"
    ))
  }

  removed <- remove_feature_rule_from_script(script_path, feature)
  if (removed) {
    cli::cli_alert_success("Removed feature detection rule for {.val {feature}}")
  } else {
    cli::cli_alert_info("No rule found for {.val {feature}}")
  }

  invisible(removed)
}

#' List feature detection rules
#'
#' Parses `tools/detect-features.R` and returns the current rules.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return A named list where names are feature names and values are detection
#'   expressions (either `"TRUE"` or an R expression string).
#' @export
#'
#' @examples
#' \dontrun{
#' list_feature_rules()
#' #> $rayon
#' #> [1] "TRUE"
#' #>
#' #> $vctrs
#' #> [1] "requireNamespace(\"vctrs\", quietly = TRUE)"
#' }
list_feature_rules <- function(path = ".") {
  with_project(path)

  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (!fs::file_exists(script_path)) {
    abort(c(
      "{.path tools/detect-features.R} not found",
      "i" = "Run {.code use_configure_feature_detection()} first"
    ))
  }

  parse_detect_features_script(script_path)
}

# =============================================================================
# Internal helpers
# =============================================================================

#' Generate empty detect-features.R script
#'
#' @param package_name R package name
#' @param features_var Features environment variable name (e.g., "MYPKG_FEATURES")
#' @return Character vector of script lines
#' @noRd
generate_empty_detect_script <- function(package_name, features_var) {
  c(
    sprintf("# Feature detection for %s", package_name),
    "# Called by ./configure to auto-detect available features.",
    "# Output: comma-separated list of Cargo features to enable.",
    "#",
    sprintf("# Users can override by setting the %s environment variable.", features_var),
    "# Add rules with: minirextendr::add_feature_rule()",
    "",
    "features <- character()",
    "",
    "## BEGIN RULES (do not edit this line)",
    "## END RULES (do not edit this line)",
    "",
    'cat(paste(features, collapse = ","))'
  )
}

#' Append a feature rule before the END marker
#'
#' @param script_path Path to detect-features.R
#' @param feature Feature name
#' @param detect Detection expression (TRUE or string)
#' @noRd
append_feature_rule <- function(script_path, feature, detect) {
  lines <- readLines(script_path, warn = FALSE)

  end_idx <- grep("^## END RULES", lines)
  if (length(end_idx) == 0) {
    abort("Could not find '## END RULES' marker in {.path {script_path}}")
  }

  # Format the detect expression
  if (isTRUE(detect)) {
    detect_str <- "TRUE"
    comment <- "always enable"
  } else {
    detect_str <- detect
    comment <- "enable if condition met"
  }

  rule_lines <- c(
    "",
    sprintf("# %s: %s", feature, comment),
    sprintf("if (%s) {", detect_str),
    sprintf('  features <- c(features, "%s")', feature),
    "}"
  )

  lines <- append(lines, rule_lines, after = end_idx[1] - 1)
  writeLines(lines, script_path)
}

#' Remove a feature rule from the script
#'
#' @param script_path Path to detect-features.R
#' @param feature Feature name to remove
#' @return TRUE if removed, FALSE if not found
#' @noRd
remove_feature_rule_from_script <- function(script_path, feature) {
  lines <- readLines(script_path, warn = FALSE)

  begin_idx <- grep("^## BEGIN RULES", lines)
  end_idx <- grep("^## END RULES", lines)
  if (length(begin_idx) == 0 || length(end_idx) == 0) {
    return(FALSE)
  }

  # Find the rule block: comment line + if block
  # Pattern: a comment with the feature name, followed by if/features/}
  comment_pattern <- sprintf("^# %s:", feature)
  comment_idx <- grep(comment_pattern, lines)
  # Only consider comments within the rules section
  comment_idx <- comment_idx[comment_idx > begin_idx[1] & comment_idx < end_idx[1]]

  if (length(comment_idx) == 0) {
    return(FALSE)
  }

  # Find the closing brace of the if block after the comment
  block_start <- comment_idx[1]
  closing_brace <- grep("^}", lines)
  closing_brace <- closing_brace[closing_brace > block_start]
  if (length(closing_brace) == 0) {
    return(FALSE)
  }
  block_end <- closing_brace[1]

  # Also remove any blank line before the comment
  if (block_start > 1 && lines[block_start - 1] == "") {
    block_start <- block_start - 1
  }

  lines <- lines[-(block_start:block_end)]
  writeLines(lines, script_path)
  TRUE
}

#' Parse detect-features.R to extract rules
#'
#' @param script_path Path to detect-features.R
#' @return Named list of feature -> detect expression pairs
#' @noRd
parse_detect_features_script <- function(script_path) {
  lines <- readLines(script_path, warn = FALSE)

  begin_idx <- grep("^## BEGIN RULES", lines)
  end_idx <- grep("^## END RULES", lines)
  if (length(begin_idx) == 0 || length(end_idx) == 0) {
    return(list())
  }

  rules_section <- lines[(begin_idx[1] + 1):(end_idx[1] - 1)]

  # Find all if(...) { ... features <- c(features, "name") ... } blocks
  # Extract: the if-condition and the feature name from the c() call
  result <- list()

  if_indices <- grep("^if \\(", rules_section)
  for (i in if_indices) {
    # Extract condition from: if (CONDITION) {
    condition <- sub("^if \\((.+)\\) \\{$", "\\1", rules_section[i])

    # Look for the feature name in the lines following the if
    for (j in (i + 1):length(rules_section)) {
      if (grepl("^}", rules_section[j])) break
      feature_match <- regmatches(
        rules_section[j],
        regexpr('features <- c\\(features, "([^"]+)"\\)', rules_section[j])
      )
      if (length(feature_match) == 1) {
        feature_name <- sub('.*"([^"]+)".*', "\\1", feature_match)
        result[[feature_name]] <- condition
      }
    }
  }

  result
}

#' Patch configure.ac to call detect-features.R
#'
#' Replaces the "default to empty" block with one that calls the detection
#' script when the features env var isn't set.
#'
#' @param configure_ac_path Path to configure.ac
#' @param features_var Features environment variable name
#' @noRd
patch_configure_ac_for_detection <- function(configure_ac_path, features_var) {
  lines <- readLines(configure_ac_path, warn = FALSE)
  text <- paste(lines, collapse = "\n")

  # Check if already patched
  if (grepl("detect-features\\.R", text, fixed = FALSE)) {
    cli::cli_alert_info("configure.ac already patched for feature detection")
    return(invisible(FALSE))
  }

  # Build the old pattern (the 4-line block to replace)
  # Using the actual features_var in the pattern
  # Note: $ and { must be escaped for regex (\\$ for literal $, \\{ for literal {)
  old_block <- paste0(
    'if test -z "\\$\\{', features_var, '\\+x}"; then\n',
    '  dnl ', features_var, ' not set - use empty \\(no extra features\\)\n',
    '  ', features_var, '=""\n',
    'fi'
  )

  # Build the replacement
  new_block <- paste0(
    'if test -z "${', features_var, '+x}"; then\n',
    '  dnl ', features_var, ' not set - auto-detect via R script if available\n',
    '  if test -f "${srcdir}/tools/detect-features.R"; then\n',
    '    ', features_var, '=$("${R_HOME}/bin/Rscript" "${srcdir}/tools/detect-features.R" 2>/dev/null || echo "")\n',
    '    if test -n "${', features_var, '}"; then\n',
    '      AC_MSG_NOTICE([Auto-detected features: ${', features_var, '}])\n',
    '    fi\n',
    '  else\n',
    '    ', features_var, '=""\n',
    '  fi\n',
    'fi'
  )

  new_text <- sub(old_block, new_block, text)

  if (identical(text, new_text)) {
    cli::cli_alert_warning("Could not find feature default block in configure.ac")
    return(invisible(FALSE))
  }

  writeLines(new_text, configure_ac_path)
  cli::cli_alert_success("Patched {.path configure.ac} for feature detection")
  invisible(TRUE)
}
