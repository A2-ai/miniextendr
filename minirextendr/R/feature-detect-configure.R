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
#' add_feature_rule("rayon", detect = TRUE, optional_dep = TRUE)
#' add_feature_rule("vctrs", detect = 'requireNamespace("vctrs", quietly = TRUE)')
#' }
use_configure_feature_detection <- function(path = ".") {
  with_project(path)

  # Validate this is a miniextendr package
  configure_ac_path <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac_path)) {
    cli::cli_abort(c(
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
#' feature is enabled at configure time. Optionally also adds the crate as an
#' optional dependency via `cargo add --optional`.
#'
#' @param feature Cargo feature name (e.g., `"vctrs"`, `"rayon"`).
#' @param detect Detection expression. One of:
#'   - `TRUE` -- always enable the feature
#'   - A string containing an R expression that returns TRUE/FALSE
#'     (e.g., `'requireNamespace("vctrs", quietly = TRUE)'`)
#' @param cargo_spec Optional Cargo feature specification. If provided, also
#'   adds the feature to `[features]` in `Cargo.toml` via [add_cargo_feature()].
#'   For example, `"miniextendr-api/vctrs"`.
#' @param optional_dep If `TRUE`, also runs `cargo add <feature> --optional` to
#'   add the crate as an optional dependency (which auto-creates a Cargo feature
#'   with the same name). If a string, uses it as the dependency spec instead of
#'   the feature name (e.g., `"rayon@1.10"` for a pinned version).
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE
#' @export
#'
#' @examples
#' \dontrun{
#' # Always enable rayon (also adds it as optional dep)
#' add_feature_rule("rayon", detect = TRUE, optional_dep = TRUE)
#'
#' # Pin version for optional dep
#' add_feature_rule("rayon", detect = TRUE, optional_dep = "rayon@1.10")
#'
#' # Enable vctrs only if the R package is available
#' add_feature_rule("vctrs",
#'   detect = 'requireNamespace("vctrs", quietly = TRUE)',
#'   cargo_spec = "miniextendr-api/vctrs")
#' }
add_feature_rule <- function(feature, detect, cargo_spec = NULL,
                             optional_dep = FALSE, path = ".") {
  with_project(path)

  if (!is.character(feature) || length(feature) != 1 || !nzchar(feature)) {
    cli::cli_abort("{.arg feature} must be a single non-empty string")
  }

  # Validate detect
  if (!isTRUE(detect) && !(is.character(detect) && length(detect) == 1)) {
    cli::cli_abort("{.arg detect} must be TRUE or a single string containing an R expression")
  }

  # Validate optional_dep
  if (!isFALSE(optional_dep) && !isTRUE(optional_dep) &&
      !(is.character(optional_dep) && length(optional_dep) == 1 &&
        nzchar(optional_dep))) {
    cli::cli_abort("{.arg optional_dep} must be FALSE, TRUE, or a dependency spec string")
  }

  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (!fs::file_exists(script_path)) {
    cli::cli_abort(c(
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

  # Add optional dependency via cargo add
  if (!isFALSE(optional_dep)) {
    dep_spec <- if (isTRUE(optional_dep)) feature else optional_dep
    cargo_add(dep = dep_spec, optional = TRUE)
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
    cli::cli_abort(c(
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

#' Sync feature detection rules with Cargo.toml
#'
#' Reads `[features]` from `src/rust/Cargo.toml` via `cargo metadata` and adds
#' detection rules for any features that don't have one yet. New features are
#' added with `detect = TRUE` (always-enable) by default. Run this after adding
#' new features to keep `tools/detect-features.R` up to date.
#'
#' Skips the `"default"` pseudo-feature and any feature that already has a rule.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns a character vector of newly added feature names
#'   (empty if everything was already in sync).
#' @export
#'
#' @examples
#' \dontrun{
#' # After adding new Cargo features:
#' sync_feature_rules()
#' #> v Added feature detection rule for 'new_feature_1'
#' #> v Added feature detection rule for 'new_feature_2'
#' #> i 2 features added, 15 already had rules
#' }
sync_feature_rules <- function(path = ".") {
  with_project(path)

  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (!fs::file_exists(script_path)) {
    cli::cli_abort(c(
      "{.path tools/detect-features.R} not found",
      "i" = "Run {.code use_configure_feature_detection()} first"
    ))
  }

  # Get features from Cargo.toml
  cargo_info <- list_cargo_features(path = path)
  cargo_features <- setdiff(names(cargo_info$features), "default")

  # Get existing rules
  existing_rules <- parse_detect_features_script(script_path)
  existing_names <- names(existing_rules)

  # Find features without rules
  missing <- setdiff(cargo_features, existing_names)

  if (length(missing) == 0) {
    cli::cli_alert_info(
      "All {length(cargo_features)} features already have detection rules"
    )
    return(invisible(character()))
  }

  # Add a rule for each missing feature (default: always enable)
  for (feat in sort(missing)) {
    append_feature_rule(script_path, feat, detect = TRUE)
    cli::cli_alert_success("Added feature detection rule for {.val {feat}}")
  }

  cli::cli_alert_info(
    "{length(missing)} feature{?s} added, {length(existing_names)} already had rules"
  )

  invisible(missing)
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
    cli::cli_abort(c(
      "{.path tools/detect-features.R} not found",
      "i" = "Run {.code use_configure_feature_detection()} first"
    ))
  }

  parse_detect_features_script(script_path)
}

#' List Cargo features and optional dependencies
#'
#' Runs `cargo metadata` on `src/rust/Cargo.toml` and parses the output to
#' discover all defined features and optional dependencies. Useful when a crate
#' already has many features and you want to see which ones still need detection
#' rules.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return A list with:
#'   \describe{
#'     \item{features}{Named list: feature name -> character vector of specs
#'       (e.g., `list(rayon = "miniextendr-api/rayon")`) }
#'     \item{optional_deps}{Named list: dep name -> list with `version`
#'       and `features` (character vector of enabled features)}
#'     \item{without_rules}{Character vector of feature names that have no
#'       corresponding detection rule in `tools/detect-features.R` (NULL if
#'       detection is not set up)}
#'   }
#' @export
#'
#' @examples
#' \dontrun{
#' info <- list_cargo_features()
#' info$features        # all features defined in [features]
#' info$optional_deps   # optional dependencies (auto-create features)
#' info$without_rules   # features missing detection rules
#' }
list_cargo_features <- function(path = ".") {
  with_project(path)
  check_rust()
  manifest_path <- cargo_toml_path()

  # Run cargo metadata (--no-deps: only our package, no transitive deps)
  result <- run_command("cargo", c(
    "metadata", "--format-version=1", "--no-deps",
    "--manifest-path", manifest_path
  ))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
      "cargo metadata failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  json <- paste(result, collapse = "\n")
  parsed <- parse_cargo_metadata_json(json)

  # Check which features lack detection rules
  without_rules <- NULL
  script_path <- usethis::proj_path("tools", "detect-features.R")
  if (fs::file_exists(script_path)) {
    existing_rules <- parse_detect_features_script(script_path)
    feature_names <- setdiff(names(parsed$features), "default")
    without_rules <- setdiff(feature_names, names(existing_rules))
  }

  structure(
    list(
      features = parsed$features,
      optional_deps = parsed$optional_deps,
      without_rules = without_rules
    ),
    class = "miniextendr_cargo_features"
  )
}

#' @export
print.miniextendr_cargo_features <- function(x, ...) {
  # Features
  feature_names <- setdiff(names(x$features), "default")
  if (length(feature_names) > 0) {
    cli::cli_h3("Cargo features ({length(feature_names)})")
    for (name in sort(feature_names)) {
      specs <- x$features[[name]]
      if (length(specs) > 0) {
        cli::cli_li("{.val {name}} = [{paste(specs, collapse = ', ')}]")
      } else {
        cli::cli_li("{.val {name}} = []")
      }
    }
  } else {
    cli::cli_alert_info("No features defined")
  }

  # Optional deps
  if (length(x$optional_deps) > 0) {
    cli::cli_h3("Optional dependencies ({length(x$optional_deps)})")
    for (name in sort(names(x$optional_deps))) {
      dep <- x$optional_deps[[name]]
      feat_str <- if (length(dep$features) > 0) {
        paste0(" [", paste(dep$features, collapse = ", "), "]")
      } else ""
      cli::cli_li("{.val {name}} {dep$req}{feat_str}")
    }
  }

  # Without rules
  if (!is.null(x$without_rules)) {
    if (length(x$without_rules) > 0) {
      cli::cli_h3("Without detection rules ({length(x$without_rules)})")
      cli::cli_li("{.val {x$without_rules}}")
    } else {
      cli::cli_alert_success("All features have detection rules")
    }
  } else {
    cli::cli_alert_info("Feature detection not set up (run {.code use_configure_feature_detection()})")
  }

  invisible(x)
}

# =============================================================================
# Internal helpers
# =============================================================================

#' Parse cargo metadata JSON to extract features and optional deps
#'
#' Minimal JSON parser using regex — handles the specific structure of
#' `cargo metadata --no-deps` output without requiring jsonlite.
#'
#' @param json Raw JSON string from cargo metadata
#' @return List with `features` and `optional_deps`
#' @noRd
parse_cargo_metadata_json <- function(json) {
  # Extract the first package's features object:
  #   "features": { "name": ["spec1", "spec2"], ... }
  features <- list()
  features_match <- regmatches(json, regexpr('"features"\\s*:\\s*\\{[^}]*\\}', json))
  if (length(features_match) == 1) {
    # Extract individual feature entries: "name": ["spec1", "spec2"]
    inner <- sub('^"features"\\s*:\\s*\\{(.*)\\}$', "\\1", features_match)
    # Match each key-value pair
    entries <- gregexpr('"([^"]+)"\\s*:\\s*\\[([^]]*)\\]', inner, perl = TRUE)
    matches <- regmatches(inner, entries)[[1]]
    for (m in matches) {
      name <- sub('^"([^"]+)".*', "\\1", m)
      # Extract array contents
      arr_str <- sub('^"[^"]+"\\s*:\\s*\\[(.*)\\]$', "\\1", m)
      if (nzchar(trimws(arr_str))) {
        specs <- regmatches(arr_str, gregexpr('"([^"]+)"', arr_str))[[1]]
        specs <- gsub('^"|"$', "", specs)
      } else {
        specs <- character()
      }
      features[[name]] <- specs
    }
  }

  # Extract optional dependencies from the dependencies array.
  # Each dep is an object like: {"name":"serde","optional":true,"req":"^1",...}
  # Dep objects are flat (no nested objects), but contain arrays like "features":[]
  # so we can't use simple [^}]+ — instead extract the deps array by bracket counting.
  optional_deps <- list()
  deps_start <- regexpr('"dependencies"\\s*:\\s*\\[', json)
  if (deps_start > 0) {
    # Find matching ] by counting brackets from the opening [
    start_pos <- deps_start + attr(deps_start, "match.length") - 1L
    chars <- strsplit(substring(json, start_pos), "")[[1]]
    depth <- 0L
    end_offset <- 0L
    for (i in seq_along(chars)) {
      if (chars[i] == "[") depth <- depth + 1L
      if (chars[i] == "]") depth <- depth - 1L
      if (depth == 0L) { end_offset <- i; break }
    }
    deps_json <- substring(json, start_pos, start_pos + end_offset - 1L)

    # Extract each {...} dep object (flat objects, safe to split on })
    dep_objects <- regmatches(deps_json, gregexpr('\\{[^}]+\\}', deps_json))[[1]]
    for (obj in dep_objects) {
      is_optional <- grepl('"optional"\\s*:\\s*true', obj)
      if (!is_optional) next

      name <- sub('.*"name"\\s*:\\s*"([^"]+)".*', "\\1", obj)
      req <- if (grepl('"req"', obj)) {
        sub('.*"req"\\s*:\\s*"([^"]+)".*', "\\1", obj)
      } else "*"

      # Extract features array from this dep object
      dep_features <- character()
      if (grepl('"features"\\s*:\\s*\\[', obj)) {
        feat_str <- sub('.*"features"\\s*:\\s*\\[([^]]*)\\].*', "\\1", obj)
        if (nzchar(trimws(feat_str))) {
          dep_features <- regmatches(feat_str, gregexpr('"([^"]+)"', feat_str))[[1]]
          dep_features <- gsub('^"|"$', "", dep_features)
        }
      }

      optional_deps[[name]] <- list(
        version = req,
        features = dep_features
      )
    }
  }

  list(features = features, optional_deps = optional_deps)
}

#' Generate empty detect-features.R script
#'
#' @param package_name R package name
#' @param features_var Features environment variable name (e.g., "CARGO_FEATURES")
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
    cli::cli_abort("Could not find '## END RULES' marker in {.path {script_path}}")
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
