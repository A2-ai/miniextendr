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

validate_non_empty_char <- function(x, arg) {
  if (!is.character(x) || length(x) == 0 || anyNA(x)) {
    abort("{arg} must be a non-empty character vector.")
  }
  if (any(!nzchar(trimws(x)))) {
    abort("{arg} must not contain empty strings.")
  }
  invisible(TRUE)
}

validate_feature_names <- function(features) {
  if (is.null(features)) {
    return(invisible(TRUE))
  }

  validate_non_empty_char(features, "features")

  features <- trimws(features)
  invalid <- features[!grepl("^[A-Za-z0-9_][A-Za-z0-9._:/-]*$", features)]
  if (length(invalid) > 0) {
    abort(c(
      "Invalid feature name(s).",
      "i" = "Invalid: {paste(invalid, collapse = ', ')}"
    ))
  }

  invisible(TRUE)
}

#' Initialize a Rust crate
#'
#' Wraps `cargo init` to create a new Rust crate in src/rust.
#'
#' @param name Optional crate name. Defaults to the package name (Rust-safe).
#' @param edition Rust edition to use (default "2024").
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_init()
#' }
cargo_init <- function(name = NULL, edition = "2024", quiet = FALSE) {
  check_rust()

  rust_dir <- usethis::proj_path("src", "rust")
  ensure_dir(rust_dir)

  manifest_path <- fs::path(rust_dir, "Cargo.toml")
  if (fs::file_exists(manifest_path)) {
    abort(c(
      "Cargo.toml already exists at {.path {manifest_path}}",
      "i" = "Remove it first if you want to re-initialize"
    ))
  }

  if (is.null(name)) {
    name <- to_rust_name(get_package_name())
  } else {
    validate_non_empty_char(name, "name")
    if (length(name) != 1) {
      abort("name must be a single string.")
    }
    name <- trimws(name)
  }

  validate_non_empty_char(edition, "edition")
  if (length(edition) != 1) {
    abort("edition must be a single string.")
  }
  edition <- trimws(edition)

  args <- c("init", "--lib", "--vcs", "none", "--edition", edition, "--name", name)
  if (quiet) {
    args <- c(args, "--quiet")
  }
  args <- c(args, rust_dir)

  cli::cli_alert("Running cargo init in {.path {rust_dir}}...")

  result <- system2("cargo", args, stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo init failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Initialized Rust crate")
  invisible(TRUE)
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
  # Input validation
  validate_non_empty_char(dep, "dep")
  dep <- trimws(dep)
  validate_feature_names(features)
  if (!is.null(features)) {
    features <- trimws(features)
  }
  if (!is.null(git) && !is.null(path)) {
    abort("Cannot specify both 'git' and 'path' - choose one source")
  }
  if (dev && build) {
    abort("Cannot specify both 'dev' and 'build' - choose one section")
  }
  if ((!is.null(branch) || !is.null(tag) || !is.null(rev)) && is.null(git)) {
    abort("'branch', 'tag', and 'rev' require 'git' to be specified")
  }

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

#' Build Rust crate
#'
#' Wraps `cargo build` to compile the Rust crate for this package.
#'
#' @param release Logical. If TRUE, build with --release.
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param all_features Logical. If TRUE, enable all features.
#' @param target Character. Build for specific target platform.
#' @param jobs Integer. Number of parallel jobs to run.
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' # Build debug
#' cargo_build()
#'
#' # Build release with features
#' cargo_build(release = TRUE, features = c("serde", "rayon"))
#' }
cargo_build <- function(release = FALSE,
                        features = NULL,
                        no_default_features = FALSE,
                        all_features = FALSE,
                        target = NULL,
                        jobs = NULL,
                        offline = FALSE,
                        quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (release) {
    args <- c(args, "--release")
  }

  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (all_features) {
    args <- c(args, "--all-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  if (!is.null(jobs)) {
    args <- c(args, "--jobs", as.character(jobs))
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  cli::cli_alert("Running cargo build...")

  result <- system2("cargo", c("build", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo build failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Build complete")
  invisible(TRUE)
}

#' Check Rust crate
#'
#' Wraps `cargo check` to type-check the Rust crate for this package.
#'
#' @param release Logical. If TRUE, check with --release.
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param all_features Logical. If TRUE, enable all features.
#' @param target Character. Check for specific target platform.
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_check()
#' }
cargo_check <- function(release = FALSE,
                        features = NULL,
                        no_default_features = FALSE,
                        all_features = FALSE,
                        target = NULL,
                        offline = FALSE,
                        quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (release) {
    args <- c(args, "--release")
  }

  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (all_features) {
    args <- c(args, "--all-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  cli::cli_alert("Running cargo check...")

  result <- system2("cargo", c("check", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo check failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Check complete")
  invisible(TRUE)
}

#' Run Rust tests
#'
#' Wraps `cargo test` to run Rust tests for this package.
#'
#' @param release Logical. If TRUE, run tests with --release.
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param all_features Logical. If TRUE, enable all features.
#' @param target Character. Test for specific target platform.
#' @param no_run Logical. If TRUE, compile tests but don't run them.
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_test()
#' }
cargo_test <- function(release = FALSE,
                       features = NULL,
                       no_default_features = FALSE,
                       all_features = FALSE,
                       target = NULL,
                       no_run = FALSE,
                       offline = FALSE,
                       quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (release) {
    args <- c(args, "--release")
  }

  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (all_features) {
    args <- c(args, "--all-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  if (no_run) {
    args <- c(args, "--no-run")
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  cli::cli_alert("Running cargo test...")

  result <- system2("cargo", c("test", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo test failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Tests complete")
  invisible(TRUE)
}

#' Run clippy lints
#'
#' Wraps `cargo clippy` to run Rust lints for this package.
#'
#' @param release Logical. If TRUE, run clippy with --release.
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param all_features Logical. If TRUE, enable all features.
#' @param target Character. Lint for specific target platform.
#' @param all_targets Logical. If TRUE, lint all targets (tests/examples/benches).
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_clippy()
#' }
cargo_clippy <- function(release = FALSE,
                         features = NULL,
                         no_default_features = FALSE,
                         all_features = FALSE,
                         target = NULL,
                         all_targets = FALSE,
                         offline = FALSE,
                         quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (release) {
    args <- c(args, "--release")
  }

  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (all_features) {
    args <- c(args, "--all-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  if (all_targets) {
    args <- c(args, "--all-targets")
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  cli::cli_alert("Running cargo clippy...")

  result <- system2("cargo", c("clippy", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo clippy failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Clippy complete")
  invisible(TRUE)
}

#' Format Rust sources
#'
#' Wraps `cargo fmt` to format Rust sources for this package.
#'
#' @param check Logical. If TRUE, check formatting without modifying files.
#' @param all Logical. If TRUE, format all packages in the workspace.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_fmt()
#' }
cargo_fmt <- function(check = FALSE,
                      all = TRUE,
                      quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (all) {
    args <- c(args, "--all")
  }

  if (check) {
    args <- c(args, "--check")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  if (check) {
    cli::cli_alert("Checking Rust formatting...")
  } else {
    cli::cli_alert("Formatting Rust sources...")
  }

  result <- system2("cargo", c("fmt", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo fmt failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Formatting complete")
  invisible(TRUE)
}

#' Build Rust documentation
#'
#' Wraps `cargo doc` to build documentation for this package.
#'
#' @param open Logical. If TRUE, open docs after building.
#' @param no_deps Logical. If TRUE, do not build docs for dependencies.
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param all_features Logical. If TRUE, enable all features.
#' @param target Character. Build docs for specific target platform.
#' @param offline Logical. If TRUE, run without network access.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns TRUE on success
#' @export
#'
#' @examples
#' \dontrun{
#' cargo_doc(no_deps = TRUE)
#' }
cargo_doc <- function(open = FALSE,
                      no_deps = TRUE,
                      features = NULL,
                      no_default_features = FALSE,
                      all_features = FALSE,
                      target = NULL,
                      offline = FALSE,
                      quiet = FALSE) {
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path)

  if (open) {
    args <- c(args, "--open")
  }

  if (no_deps) {
    args <- c(args, "--no-deps")
  }

  if (no_default_features) {
    args <- c(args, "--no-default-features")
  }

  if (all_features) {
    args <- c(args, "--all-features")
  }

  if (!is.null(features) && length(features) > 0) {
    args <- c(args, "--features", paste(features, collapse = ","))
  }

  if (!is.null(target)) {
    args <- c(args, "--target", target)
  }

  if (offline) {
    args <- c(args, "--offline")
  }

  if (quiet) {
    args <- c(args, "--quiet")
  }

  cli::cli_alert("Building cargo docs...")

  result <- system2("cargo", c("doc", args), stdout = TRUE, stderr = TRUE)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo doc failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  cli::cli_alert_success("Docs complete")
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
