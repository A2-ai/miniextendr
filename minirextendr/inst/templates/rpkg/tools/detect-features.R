#!/usr/bin/env Rscript
# Configure-time Cargo feature detection for {{package}}.
#
# Contract
# --------
#   Consumed by : ./configure (see configure.ac, the CARGO_FEATURES block).
#                 configure runs `Rscript tools/detect-features.R` and captures
#                 stdout. The `2>/dev/null || echo ""` wrapper means any failure
#                 here (nonzero exit OR empty stdout) makes configure fall back
#                 to its hardcoded feature list. So a wrong-but-nonempty answer
#                 is worse than no answer: when uncertain, we ENABLE (matching
#                 the hardcoded fallback's "all optional features on" default).
#   Output      : a single comma-separated feature string on stdout, nothing
#                 else. Every diagnostic goes to stderr so it never pollutes the
#                 captured value.
#   Failure mode: on any internal error the script prints nothing and exits
#                 nonzero, so configure uses its fallback list.
#
# Dependencies
# ------------
#   Base R + utils ONLY. This runs at ./configure time on end-user machines
#   where no package library (not even this package) is available. Do NOT
#   library()/requireNamespace() anything beyond base/utils, and do NOT call
#   minirextendr::* (configure.ac must not depend on minirextendr).
#
# Design
# ------
#   1. Parse [features] from src/rust/Cargo.toml to discover what CAN be built.
#   2. Enable every feature by default, then subtract:
#        - a denylist of meta/aggregate/dev/option-default/risky features that
#          must never be auto-enabled (see DENY below), and
#        - conditional features whose detection rule says "no" on this machine
#          (e.g. vctrs needs the vctrs R package; connections needs R >= 4.3).
#   3. Print the survivors, comma-separated.
#
#   Override entirely by setting CARGO_FEATURES before ./configure.

# Robustness wrapper: any uncaught error -> empty stdout + nonzero exit so
# configure falls back to its hardcoded list.
main <- function() {
  args <- commandArgs(trailingOnly = FALSE)
  # Locate src/rust/Cargo.toml relative to this script (configure invokes us
  # from the package root, but resolve from the script path to be safe).
  cargo_toml <- find_cargo_toml(args)
  if (is.null(cargo_toml) || !file.exists(cargo_toml)) {
    stop("could not locate src/rust/Cargo.toml")
  }

  available <- parse_cargo_features(cargo_toml)
  if (length(available) == 0L) {
    stop("no [features] parsed from Cargo.toml")
  }

  # Features that must never be auto-enabled.
  #   default, full        : meta / aggregate selectors.
  #   nonapi               : triggers R CMD check non-API WARNINGs.
  #   macro-coverage,
  #   growth-debug         : development / diagnostic only (macro-coverage does
  #                          not even compile without worker-thread).
  #   strict-default,
  #   coerce-default       : project-wide #[miniextendr] option defaults — these
  #                          change codegen semantics, so they are opt-in.
  #   r6-default,
  #   s7-default           : mutually exclusive class-system selectors; enabling
  #                          either changes the default class system. Opt-in.
  #   worker-default       : separate opt-in semantic from `worker-thread`.
  #   indicatif            : progress-bar integration, opt-in (not in the
  #                          default integration set).
  deny <- c(
    "default", "full", "nonapi",
    "macro-coverage", "growth-debug",
    "strict-default", "coerce-default",
    "r6-default", "s7-default", "worker-default",
    "indicatif"
  )

  features <- setdiff(available, deny)

  # Conditional rules. A feature listed here is enabled only if its predicate
  # is TRUE. Features NOT listed here are enabled unconditionally (default:
  # enable). Predicates must be conservative: when in doubt, return TRUE.
  rules <- list(
    # vctrs: the Rust feature wraps the vctrs R package's C ABI, so it only
    # makes sense when vctrs is installed and loadable.
    vctrs = function() requireNamespace("vctrs", quietly = TRUE),
    # connections: the custom-connections C API (R_new_custom_connection)
    # requires R >= 4.3.0. getRversion() is base R.
    connections = function() getRversion() >= "4.3.0"
  )

  for (feat in names(rules)) {
    if (feat %in% features) {
      keep <- isTRUE(tryCatch(rules[[feat]](), error = function(e) FALSE))
      if (!keep) {
        features <- setdiff(features, feat)
        message(sprintf("detect-features: disabling '%s' (rule not satisfied)", feat))
      }
    }
  }

  # Deterministic order for stable configure output / diffs.
  features <- sort(unique(features))

  if (length(features) == 0L) {
    stop("rule set eliminated every feature")
  }

  cat(paste(features, collapse = ","))
}

# Parse the [features] table of a Cargo.toml, returning feature names.
# Base-R line scanner — no TOML library available at configure time.
parse_cargo_features <- function(path) {
  lines <- readLines(path, warn = FALSE)
  in_features <- FALSE
  names_out <- character()
  for (ln in lines) {
    trimmed <- trimws(ln)
    # Section header?
    if (grepl("^\\[", trimmed)) {
      in_features <- identical(trimmed, "[features]")
      next
    }
    if (!in_features) next
    if (!nzchar(trimmed) || startsWith(trimmed, "#")) next
    # A feature entry looks like `name = [ ... ]`. Names are bare identifiers
    # (letters, digits, _, -). Strip an inline trailing comment first.
    code <- sub("#.*$", "", trimmed)
    m <- regmatches(code, regexpr("^[A-Za-z0-9_.+-]+(?=\\s*=)", code, perl = TRUE))
    if (length(m) == 1L && nzchar(m)) {
      names_out <- c(names_out, m)
    }
  }
  unique(names_out)
}

# Resolve src/rust/Cargo.toml. Prefer a path relative to this script (so the
# script works regardless of the working directory configure runs it from),
# then fall back to the conventional path from the package root.
find_cargo_toml <- function(args) {
  file_arg <- grep("^--file=", args, value = TRUE)
  candidates <- character()
  if (length(file_arg) == 1L) {
    script_path <- sub("^--file=", "", file_arg)
    script_dir <- dirname(normalizePath(script_path, mustWork = FALSE))
    # tools/ -> ../src/rust/Cargo.toml
    candidates <- c(candidates, file.path(script_dir, "..", "src", "rust", "Cargo.toml"))
  }
  candidates <- c(candidates, file.path("src", "rust", "Cargo.toml"))
  for (cand in candidates) {
    if (file.exists(cand)) return(normalizePath(cand))
  }
  NULL
}

ok <- tryCatch({
  main()
  TRUE
}, error = function(e) {
  message(sprintf("detect-features: %s", conditionMessage(e)))
  FALSE
})

if (!isTRUE(ok)) {
  quit(status = 1L, save = "no")
}
