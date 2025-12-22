# Cargo command wrappers

#' Get the Cargo.toml path for current R package
#'
#' @return Path to src/rust/Cargo.toml
#' @noRd
cargo_toml_path <- function() {
  path <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(path)) {
    abort(c(
      "Cargo.toml not found at {.path {path}}",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first to generate Cargo.toml"
    ))
  }
  path
}

#' Add a dependency to Cargo.toml
#'
#' Wraps `cargo add` to add Rust dependencies to your miniextendr package.
#' Automatically uses `src/rust/Cargo.toml` in the current R package.
#'
#' @param dep Dependency specification. Can be:
#'   - A crate name: `"serde"`
#'   - Name with version: `"serde@1.0"` or `"serde@=1.0.38"`
#'   - Multiple crates: `c("serde", "tokio@1.0")`
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param optional Logical. If TRUE, mark dependency as optional (exposed as a crate feature).
#' @param rename Character. Rename the dependency (useful for multiple versions).
#' @param path Character. Path to local crate to add instead of from crates.io.
#' @param git Character. Git repository URL to add dependency from.
#' @param branch Character. Git branch (used with `git`).
#' @param tag Character. Git tag (used with `git`).
#' @param rev Character. Git revision/commit hash (used with `git`).
#' @param registry Character. Package registry name for this dependency.
#' @param dev Logical. If TRUE, add as dev-dependency (for tests/examples/benchmarks).
#' @param build Logical. If TRUE, add as build-dependency (for build.rs).
#' @param target Character. Add as dependency for specific target platform (e.g., "x86_64-unknown-linux-gnu").
#' @param dry_run Logical. If TRUE, show what would happen without modifying Cargo.toml.
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' # Add serde with derive feature
#' cargo_add("serde", features = "derive")
#'
#' # Add specific version
#' cargo_add("tokio@1.0", features = c("full", "rt-multi-thread"))
#'
#' # Add as dev dependency (for tests)
#' cargo_add("criterion", dev = TRUE)
#'
#' # Add from git
#' cargo_add("mycrate", git = "https://github.com/user/mycrate")
#'
#' # Add from git with specific branch
#' cargo_add("mycrate", git = "https://github.com/user/mycrate", branch = "dev")
#'
#' # Add from local path
#' cargo_add("localcrate", path = "../my-local-crate")
#'
#' # Add multiple crates at once
#' cargo_add(c("serde", "serde_json"))
#'
#' # Add with no default features, only specific ones
#' cargo_add("tokio", no_default_features = TRUE, features = c("rt", "macros"))
#'
#' # Add as optional dependency (becomes a crate feature)
#' cargo_add("rayon", optional = TRUE)
#'
#' # Dry run to see what would happen
#' cargo_add("newcrate", dry_run = TRUE)
#' }
cargo_add <- function(dep,
                      features = NULL,
                      no_default_features = FALSE,
                      optional = FALSE,
                      rename = NULL,
                      path = NULL,
                      git = NULL,
                      branch = NULL,
                      tag = NULL,
                      rev = NULL,
                      registry = NULL,
                      dev = FALSE,
                      build = FALSE,
                      target = NULL,
                      dry_run = FALSE,
                      offline = FALSE,
                      quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  # Build argument list
  args <- character()

  # Source options (mutually exclusive: path, git, or crates.io)
  if (!is.null(path)) {
    args <- c(args, "--path", path)
  } else if (!is.null(git)) {
    args <- c(args, "--git", git)
    if (!is.null(branch)) args <- c(args, "--branch", branch)
    if (!is.null(tag)) args <- c(args, "--tag", tag)
    if (!is.null(rev)) args <- c(args, "--rev", rev)
  }

  if (!is.null(registry)) {
    args <- c(args, "--registry", registry)
  }

  # Feature options
  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  # Dependency options
  if (optional) {
    args <- c(args, "--optional")
  }

  if (!is.null(rename)) {
    args <- c(args, "--rename", rename)
  }

  # Section options
  if (dev) {
    args <- c(args, "--dev")
  } else if (build) {
    args <- c(args, "--build")
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  # Always inject manifest path
  args <- c(args, "--manifest-path", manifest_path)

  # Behavior flags
  if (dry_run) {
    args <- c(args, "--dry-run")
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  # Add dependency specs at the end
  args <- c(args, dep)

  # User feedback
  dep_str <- paste(dep, collapse = ", ")
  section <- if (dev) "dev-dependency" else if (build) "build-dependency" else "dependency"

  if (dry_run) {
    cli::cli_alert_info("Dry run: would add {section} {.val {dep_str}}")
  } else {
    cli::cli_alert("Adding {section}: {.val {dep_str}}")
  }

  # Run cargo add
  result <- system2("cargo", c("add", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo add failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  if (!dry_run) {
    cli::cli_alert_success("Added {.val {dep_str}}")
  }

  invisible(TRUE)
}

#' Remove a dependency from Cargo.toml
#'
#' Wraps `cargo remove` to remove Rust dependencies from your miniextendr package.
#'
#' @param dep Dependency name(s) to remove. Can be a character vector.
#' @param dev Logical. If TRUE, remove from dev-dependencies.
#' @param build Logical. If TRUE, remove from build-dependencies.
#' @param target Character. Remove from specific target platform dependencies.
#' @param dry_run Logical. If TRUE, don't actually modify Cargo.toml.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' # Remove a dependency
#' cargo_rm("serde")
#'
#' # Remove dev dependency
#' cargo_rm("criterion", dev = TRUE)
#'
#' # Remove multiple
#' cargo_rm(c("serde", "serde_json"))
#' }
cargo_rm <- function(dep,
                     dev = FALSE,
                     build = FALSE,
                     target = NULL,
                     dry_run = FALSE,
                     quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- character()

  if (dev) {
    args <- c(args, "--dev")
  } else if (build) {
    args <- c(args, "--build")
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  args <- c(args, "--manifest-path", manifest_path)

  if (dry_run) {
    args <- c(args, "--dry-run")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  args <- c(args, dep)

  dep_str <- paste(dep, collapse = ", ")
  cli::cli_alert("Removing: {.val {dep_str}}")

  result <- system2("cargo", c("remove", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo remove failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  if (!dry_run) {
    cli::cli_alert_success("Removed {.val {dep_str}}")
  }

  invisible(TRUE)
}

#' Update dependencies in Cargo.lock
#'
#' Wraps `cargo update` to update dependencies.
#'
#' @param dep Optional. Specific package(s) to update. If NULL, updates all.
#' @param precise Character. Update to exactly this version (use with single dep).
#' @param dry_run Logical. If TRUE, don't actually update.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' # Update all dependencies
#' cargo_update()
#'
#' # Update specific package
#' cargo_update("serde")
#'
#' # Update to specific version
#' cargo_update("serde", precise = "1.0.193")
#' }
cargo_update <- function(dep = NULL,
                         precise = NULL,
                         dry_run = FALSE,
                         quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (!is.null(dep)) {
    for (d in dep) {
      args <- c(args, "--package", d)
    }
  }

  if (!is.null(precise)) {
    args <- c(args, "--precise", precise)
  }

  if (dry_run) {
    args <- c(args, "--dry-run")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  if (is.null(dep)) {
    cli::cli_alert("Updating all dependencies...")
  } else {
    cli::cli_alert("Updating: {.val {paste(dep, collapse = ', ')}}")
  }

  result <- system2("cargo", c("update", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo update failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Dependencies updated")
  invisible(TRUE)
}

#' Search for crates on crates.io
#'
#' Wraps `cargo search` to find crates.
#'
#' @param query Search query string.
#' @param limit Maximum number of results (default 10).
#' @param registry Character. Alternative registry to search.
#'
#' @return Character vector of search results (invisibly)
#' @export
#'
#' @examples
#' \dontrun{
#' # Search for JSON crates
#' cargo_search("json")
#'
#' # Search with more results
#' cargo_search("async runtime", limit = 20)
#' }
cargo_search <- function(query, limit = 10, registry = NULL) {
  check_rust()

  args <- c(query, "--limit", as.character(limit))

  if (!is.null(registry)) {
    args <- c(args, "--registry", registry)
  }

  cli::cli_alert("Searching crates.io for: {.val {query}}")

  result <- system2("cargo", c("search", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo search failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (length(result) > 0) {
    cli::cli_verbatim(result)
  } else {
    cli::cli_alert_info("No results found")
  }

  invisible(result)
}

#' Show dependency tree
#'
#' Wraps `cargo tree` to display the dependency tree.
#'
#' @param depth Maximum depth to display (default 1 for direct deps only).
#' @param duplicates Logical. If TRUE, show only duplicate dependencies.
#' @param invert Character. Invert the tree, showing what depends on this package.
#'
#' @return Invisibly returns the tree output
#' @export
#'
#' @examples
#' \dontrun{
#' # Show direct dependencies
#' cargo_deps()
#'
#' # Show full tree
#' cargo_deps(depth = 99)
#'
#' # Find duplicates
#' cargo_deps(duplicates = TRUE)
#'
#' # What depends on syn?
#' cargo_deps(invert = "syn")
#' }
cargo_deps <- function(depth = 1, duplicates = FALSE, invert = NULL) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path, "--depth", as.character(depth))

  if (duplicates) {
    args <- c(args, "--duplicates")
  }

  if (!is.null(invert)) {
    args <- c(args, "--invert", invert)
  }

  result <- system2("cargo", c("tree", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo tree failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  cli::cli_verbatim(result)
  invisible(result)
}
