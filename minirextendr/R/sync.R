# Staleness detection and rebuild orchestration for render workflows

# =============================================================================
# State management
# =============================================================================

#' Path to render state file
#'
#' @param path Project root
#' @return Path to `.miniextendr/render-state.rds`
#' @noRd
render_state_path <- function(path = usethis::proj_get()) {
  file.path(path, "tools", ".miniextendr", "render-state.rds")
}

#' Compute hash of all build-relevant sources
#'
#' Hashes Rust sources, Cargo.toml, Cargo.lock, build.rs, document.rs.in,
#' DESCRIPTION, NAMESPACE, miniextendr.yml, and effective build knobs.
#'
#' @param path Project root
#' @return Single character string (MD5 digest)
#' @noRd
compute_source_hash <- function(path = usethis::proj_get()) {
  rust_dir <- file.path(path, "src", "rust")

  # Collect files to hash
  candidates <- character()

  # Rust sources
  if (dir.exists(rust_dir)) {
    rs_files <- list.files(rust_dir, pattern = "\\.rs$", recursive = TRUE,
                           full.names = TRUE)
    candidates <- c(candidates, rs_files)
  }

  # Build config files
  config_files <- c(
    file.path(rust_dir, "Cargo.toml"),
    file.path(rust_dir, "Cargo.lock"),
    file.path(rust_dir, "build.rs"),
    file.path(path, "src", "rust", "document.rs.in"),
    file.path(path, "DESCRIPTION"),
    file.path(path, "NAMESPACE"),
    file.path(path, "miniextendr.yml")
  )
  candidates <- c(candidates, config_files)

  # Filter to existing files only
  existing <- candidates[file.exists(candidates)]

  if (length(existing) == 0) {
    return("")
  }

  # Sort for deterministic ordering
  existing <- sort(existing)

  # Compute MD5 of each file, then hash the combined result
  sums <- tools::md5sum(existing)
  # Include build knobs from config
  config <- tryCatch(mx_config(path), error = function(e) mx_config_defaults())
  knobs <- paste(
    config$features, config$strict, config$coerce,
    config$rust_version, collapse = "|"
  )

  # Create a combined digest from all file hashes + knobs
  combined <- paste(c(sums, knobs), collapse = "\n")
  tmp <- tempfile()
  on.exit(unlink(tmp), add = TRUE)
  writeLines(combined, tmp)
  unname(tools::md5sum(tmp))
}

#' Read saved render state
#'
#' @param path Project root
#' @return List with `hash` and `stage_run`, or NULL if no state
#' @noRd
read_render_state <- function(path = usethis::proj_get()) {
  state_file <- render_state_path(path)
  if (!file.exists(state_file)) return(NULL)

  tryCatch(
    readRDS(state_file),
    error = function(e) {
      # Corrupted state file — remove and return NULL
      unlink(state_file)
      NULL
    }
  )
}

#' Save render state
#'
#' @param hash Current source hash
#' @param stage_run Stage that was executed
#' @param path Project root
#' @noRd
write_render_state <- function(hash, stage_run, path = usethis::proj_get()) {
  state_file <- render_state_path(path)
  state_dir <- dirname(state_file)
  if (!dir.exists(state_dir)) {
    dir.create(state_dir, recursive = TRUE, showWarnings = FALSE)
  }
  saveRDS(list(hash = hash, stage_run = stage_run, timestamp = Sys.time()),
          state_file)
}

# =============================================================================
# Main sync function
# =============================================================================

#' Sync miniextendr package for rendering
#'
#' Detects staleness of Rust sources and build artifacts, then rebuilds
#' as needed. Designed as a single entry point before rendering vignettes
#' (Rmd/qmd). Dispatches to [miniextendr_build()], [miniextendr_document()],
#' and related workflow functions.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param mode One of `"if_stale"` (default), `"always"`, or `"never"`.
#'   Controls whether to rebuild:
#'   - `"if_stale"`: only rebuild when source hash has changed
#'   - `"always"`: rebuild unconditionally
#'   - `"never"`: skip rebuild entirely (useful for pre-built packages)
#' @param stage How far to rebuild. One of:
#'   - `"install"` (default): full rebuild (configure + install + document + install)
#'   - `"wrappers"`: only regenerate R wrappers (document step)
#'   - `"build"`: configure + compile, but don't install
#' @param force Logical. If `TRUE`, equivalent to `mode = "always"`.
#' @param quiet Logical. If `TRUE`, suppress progress messages.
#' @return Invisibly returns a list with:
#'   - `fresh`: logical, whether sources were already up-to-date
#'   - `stage_run`: character, what stage was executed (or `"none"`)
#'   - `hash`: character, current source hash
#' @export
#'
#' @examples
#' \dontrun{
#' # Sync before rendering a vignette
#' miniextendr_sync()
#'
#' # Force full rebuild
#' miniextendr_sync(force = TRUE)
#'
#' # Only regenerate wrappers
#' miniextendr_sync(stage = "wrappers")
#' }
miniextendr_sync <- function(path = ".",
                              mode = c("if_stale", "always", "never"),
                              stage = c("install", "wrappers", "build"),
                              force = FALSE,
                              quiet = FALSE) {
  with_project(path)
  mode <- match.arg(mode)
  stage <- match.arg(stage)
  pkg_path <- usethis::proj_get()

  if (force) mode <- "always"

  current_hash <- compute_source_hash(pkg_path)

  # Check staleness
  if (mode == "never") {
    if (!quiet) cli::cli_alert_info("miniextendr_sync: mode = 'never', skipping rebuild")
    return(invisible(list(fresh = TRUE, stage_run = "none", hash = current_hash)))
  }

  if (mode == "if_stale") {
    saved <- read_render_state(pkg_path)
    if (!is.null(saved) && identical(saved$hash, current_hash)) {
      if (!quiet) cli::cli_alert_success("miniextendr: sources up-to-date, skipping rebuild")
      return(invisible(list(fresh = TRUE, stage_run = "none", hash = current_hash)))
    }
  }

  # Rebuild
  if (!quiet) cli::cli_alert("miniextendr_sync: rebuilding (stage = {.val {stage}})")

  stage_run <- switch(stage,
    wrappers = {
      miniextendr_document(path = pkg_path)
      "wrappers"
    },
    build = {
      miniextendr_build(path = pkg_path, install = FALSE)
      "build"
    },
    install = {
      miniextendr_build(path = pkg_path, install = TRUE)
      "install"
    }
  )

  # Update state
  write_render_state(current_hash, stage_run, pkg_path)

  if (!quiet) cli::cli_alert_success("miniextendr_sync: rebuild complete (stage = {.val {stage_run}})")

  invisible(list(fresh = FALSE, stage_run = stage_run, hash = current_hash))
}
