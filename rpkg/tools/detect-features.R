#!/usr/bin/env Rscript
# Configure-time Cargo feature detection for miniextendr.
#
# Contract
# --------
#   Consumed by : ./configure (the CARGO_FEATURES block).
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
#   (Exception: rule expressions in the ## BEGIN RULES block may use
#   requireNamespace() to probe for optional packages.)
#
# Design
# ------
#   1. Parse [features] from src/rust/Cargo.toml to discover what CAN be built.
#   2. Enable every feature by default, then subtract:
#        - a denylist of meta/aggregate/dev/option-default/risky features that
#          must never be auto-enabled (see DENY below), and
#        - conditional features whose rule says "no" on this machine
#          (e.g. vctrs needs the vctrs R package; connections needs R >= 4.3).
#   3. Print the survivors, comma-separated.
#
#   Override entirely by setting CARGO_FEATURES before ./configure.
#   Add/remove rules with: minirextendr::add_feature_rule() /
#     minirextendr::remove_feature_rule()

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
  #   coerce-default       : project-wide #[miniextendr] option defaults -- these
  #                          change codegen semantics, so they are opt-in.
  #   fast-default         : drops R-side preconditions crate-wide; runtime
  #                          coverage lives in the weekly feature-legs leg.
  #   r6-default,
  #   s7-default           : mutually exclusive class-system selectors; enabling
  #                          either changes the default class system. Opt-in.
  #   worker-default       : separate opt-in semantic from `worker-thread`.
  #   indicatif            : progress-bar integration, opt-in (not in the
  #                          default integration set).
  deny <- c(
    "default", "full", "nonapi",
    "macro-coverage", "growth-debug",
    "strict-default", "coerce-default", "fast-default",
    "r6-default", "s7-default", "worker-default",
    "indicatif"
  )

  features <- setdiff(available, deny)

  # Conditional rules injected by minirextendr::add_feature_rule().
  # A feature listed here is enabled only if its predicate returns TRUE.
  # Features NOT listed here are enabled unconditionally (default: enable).
  # Predicates must be conservative: when in doubt, return TRUE.
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
# Base-R line scanner -- no TOML library available at configure time.
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

# Predicate helper for rules: is the build toolchain's rustc at least `min`?
# Gates features whose crates have a high MSRV so an old toolchain cleanly
# skips them instead of failing deep in compilation. Per the contract above,
# an undeterminable version defers to enable (returns TRUE).
rustc_at_least <- function(min) {
  v <- tryCatch(system2("rustc", "--version", stdout = TRUE, stderr = FALSE),
                error = function(e) character())
  m <- regmatches(v, regexpr("[0-9]+\\.[0-9]+\\.[0-9]+", v))
  if (length(m) != 1L) return(TRUE)
  numeric_version(m) >= min
}

# Conditional rules: populated by minirextendr::add_feature_rule() between
# the markers below. main() reads this global. Features not listed here are
# enabled unconditionally (auto-discovery).
rules <- list()
## BEGIN RULES (do not edit this line)
rules[["vctrs"]] <- function() requireNamespace("vctrs", quietly = TRUE)
rules[["connections"]] <- function() getRversion() >= "4.3.0"
rules[["arrow"]] <- function() rustc_at_least("1.81.0")
rules[["datafusion"]] <- function() rustc_at_least("1.82.0")
## END RULES (do not edit this line)

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
