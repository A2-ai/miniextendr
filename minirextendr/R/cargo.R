# Cargo command wrappers

#' Get the Cargo.toml path for current R package
#'
#' @return Path to src/rust/Cargo.toml
#' @noRd
cargo_toml_path <- function() {
  path <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(path)) {
    cli::cli_abort(c(
      "Cargo.toml not found at {.path {path}}",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first to generate Cargo.toml"
    ))
  }
  path
}

validate_non_empty_char <- function(x, arg) {
  if (!is.character(x) || length(x) == 0 || anyNA(x)) {
    cli::cli_abort("{arg} must be a non-empty character vector.")
  }
  if (any(!nzchar(trimws(x)))) {
    cli::cli_abort("{arg} must not contain empty strings.")
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
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_init <- function(path = ".", name = NULL, edition = "2024", quiet = FALSE) {
  with_project(path)
  check_rust()

  rust_dir <- usethis::proj_path("src", "rust")
  ensure_dir(rust_dir)

  manifest_path <- fs::path(rust_dir, "Cargo.toml")
  if (fs::file_exists(manifest_path)) {
    cli::cli_abort(c(
      "Cargo.toml already exists at {.path {manifest_path}}",
      "i" = "Remove it first if you want to re-initialize"
    ))
  }

  if (is.null(name)) {
    name <- to_rust_name(get_package_name())
  } else {
    validate_non_empty_char(name, "name")
    if (length(name) != 1) {
      cli::cli_abort("name must be a single string.")
    }
    name <- trimws(name)
  }

  validate_non_empty_char(edition, "edition")
  if (length(edition) != 1) {
    cli::cli_abort("edition must be a single string.")
  }
  edition <- trimws(edition)

  args <- c("init", "--lib", "--vcs", "none", "--edition", edition, "--name", name)
  if (quiet) {
    args <- c(args, "--quiet")
  }
  args <- c(args, rust_dir)

  cli::cli_alert("Running cargo init in {.path {rust_dir}}...")

  result <- run_command("cargo", args)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param dep Dependency specification. Can be:
#'   - A crate name: `"serde"`
#'   - Name with version: `"serde@1.0"` or `"serde@=1.0.38"`
#'   - Multiple crates: `c("serde", "tokio@1.0")`
#' @param features Character vector of features to activate.
#' @param no_default_features Logical. If TRUE, disable default features.
#' @param optional Logical. If TRUE, mark dependency as optional (exposed as a crate feature).
#' @param rename Character. Rename the dependency (useful for multiple versions).
#' @param crate_path Character. Path to local crate to add instead of from crates.io.
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
#' cargo_add("localcrate", crate_path = "../my-local-crate")
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
cargo_add <- function(path = ".",
                      dep,
                      features = NULL,
                      no_default_features = FALSE,
                      optional = FALSE,
                      rename = NULL,
                      crate_path = NULL,
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
  with_project(path)
  # Input validation
  validate_non_empty_char(dep, "dep")
  dep <- trimws(dep)
  validate_feature_names(features)
  if (!is.null(features)) {
    features <- trimws(features)
  }
  if (!is.null(git) && !is.null(crate_path)) {
    cli::cli_abort("Cannot specify both 'git' and 'crate_path' - choose one source")
  }
  if (dev && build) {
    cli::cli_abort("Cannot specify both 'dev' and 'build' - choose one section")
  }
  if ((!is.null(branch) || !is.null(tag) || !is.null(rev)) && is.null(git)) {
    cli::cli_abort("'branch', 'tag', and 'rev' require 'git' to be specified")
  }

  check_rust()
  manifest_path <- cargo_toml_path()

  # Build argument list
  args <- character()

  # Source options (mutually exclusive: crate_path, git, or crates.io)
  if (!is.null(crate_path)) {
    args <- c(args, "--path", crate_path)
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
  result <- run_command("cargo", c("add", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_rm <- function(path = ".",
                     dep,
                     dev = FALSE,
                     build = FALSE,
                     target = NULL,
                     dry_run = FALSE,
                     quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("remove", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_update <- function(path = ".",
                         dep = NULL,
                         precise = NULL,
                         dry_run = FALSE,
                         quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("update", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_build <- function(path = ".",
                        release = FALSE,
                        features = NULL,
                        no_default_features = FALSE,
                        all_features = FALSE,
                        target = NULL,
                        jobs = NULL,
                        offline = FALSE,
                        quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("build", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_check <- function(path = ".",
                        release = FALSE,
                        features = NULL,
                        no_default_features = FALSE,
                        all_features = FALSE,
                        target = NULL,
                        offline = FALSE,
                        quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("check", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_test <- function(path = ".",
                       release = FALSE,
                       features = NULL,
                       no_default_features = FALSE,
                       all_features = FALSE,
                       target = NULL,
                       no_run = FALSE,
                       offline = FALSE,
                       quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("test", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_clippy <- function(path = ".",
                         release = FALSE,
                         features = NULL,
                         no_default_features = FALSE,
                         all_features = FALSE,
                         target = NULL,
                         all_targets = FALSE,
                         offline = FALSE,
                         quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("clippy", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_fmt <- function(path = ".",
                      check = FALSE,
                      all = TRUE,
                      quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("fmt", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_doc <- function(path = ".",
                      open = FALSE,
                      no_deps = TRUE,
                      features = NULL,
                      no_default_features = FALSE,
                      all_features = FALSE,
                      target = NULL,
                      offline = FALSE,
                      quiet = FALSE) {
  with_project(path)
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

  result <- run_command("cargo", c("doc", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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

  result <- run_command("cargo", c("search", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
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
#' @param path Path to the R package root, or `"."` to use the current directory.
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
cargo_deps <- function(path = ".", depth = 1, duplicates = FALSE, invert = NULL) {
  with_project(path)
  check_rust()
  manifest_path <- cargo_toml_path()

  args <- c("--manifest-path", manifest_path, "--depth", as.character(depth))

  if (duplicates) {
    args <- c(args, "--duplicates")
  }

  if (!is.null(invert)) {
    args <- c(args, "--invert", invert)
  }

  result <- run_command("cargo", c("tree", args))

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
      "cargo tree failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  cli::cli_verbatim(result)
  invisible(result)
}

#' Create a new Rust crate in a workspace
#'
#' Wraps `cargo new` to create a new Rust crate. Unlike `cargo init`, this creates
#' a new directory for the crate. When in a Cargo workspace, runs from the workspace
#' root so the new crate is workspace-aware.
#'
#' Note: `cargo new` does not accept `--manifest-path`, so this function changes
#' to the appropriate directory before running the command.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param name Name of the crate to create. This will be the directory name.
#' @param lib Logical. If TRUE, create a library crate (default). If FALSE, create a binary.
#' @param edition Rust edition to use (default "2024").
#' @param vcs Version control system to initialize. One of "git", "hg", "pijul", "fossil", or "none".
#'   Default is "none" to avoid nested git repos.
#' @param add_to_workspace Logical. If TRUE and in a workspace, add the new crate
#'   to the workspace members list in Cargo.toml. Default is TRUE.
#' @param quiet Logical. If TRUE, suppress cargo output.
#'
#' @return Invisibly returns the path to the new crate directory
#' @export
#'
#' @examples
#' \dontrun{
#' # Create a new library crate
#' cargo_new("my-utils")
#'
#' # Create a binary crate
#' cargo_new("my-cli", lib = FALSE)
#'
#' # Create without adding to workspace
#' cargo_new("standalone-crate", add_to_workspace = FALSE)
#' }
cargo_new <- function(path = ".",
                      name,
                      lib = TRUE,
                      edition = "2024",
                      vcs = "none",
                      add_to_workspace = TRUE,
                      quiet = FALSE) {
  with_project(path)
  check_rust()

  # Validate inputs
  validate_non_empty_char(name, "name")
  if (length(name) != 1) {
    cli::cli_abort("name must be a single string.")
  }
  name <- trimws(name)

  # Validate name is a valid crate name
  if (!grepl("^[a-zA-Z][a-zA-Z0-9_-]*$", name)) {
    cli::cli_abort(c(
      "Invalid crate name: {.val {name}}",
      "i" = "Crate names must start with a letter and contain only letters, numbers, underscores, or hyphens."
    ))
  }

  validate_non_empty_char(edition, "edition")
  if (length(edition) != 1) {
    cli::cli_abort("edition must be a single string.")
  }
  edition <- trimws(edition)

  vcs <- match.arg(vcs, c("git", "hg", "pijul", "fossil", "none"))


  # Determine where to run cargo new from
  # If in a workspace, run from workspace root
  # Otherwise, run from current directory
  proj_path <- usethis::proj_get()
  workspace_root <- find_workspace_root(proj_path)

  if (!is.null(workspace_root)) {
    run_dir <- workspace_root
    cli::cli_alert_info("Detected Cargo workspace at {.path {workspace_root}}")
  } else {
    # Not in a workspace - run from project root
    run_dir <- proj_path
  }

  # Check if crate already exists
  new_crate_path <- file.path(run_dir, name)
  if (fs::dir_exists(new_crate_path)) {
    cli::cli_abort(c(
      "Directory already exists: {.path {new_crate_path}}",
      "i" = "Choose a different name or remove the existing directory."
    ))
  }

  # Build cargo new arguments
  args <- c("new", name)

  if (lib) {
    args <- c(args, "--lib")
  } else {
    args <- c(args, "--bin")
  }

  args <- c(args, "--edition", edition)
  args <- c(args, "--vcs", vcs)

  if (quiet) {
    args <- c(args, "--quiet")
  }

  # Run cargo new from the appropriate directory
  cli::cli_alert("Running {.code cargo new {name}} in {.path {run_dir}}...")

  result <- run_command("cargo", args, wd = run_dir)

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    cli::cli_abort(c(
      "cargo new failed",
      "i" = paste(result, collapse = "\n")
    ))
  }

  if (!quiet && length(result) > 0) {
    cli::cli_verbatim(result)
  }

  # Add to workspace if requested and in a workspace
  if (add_to_workspace && !is.null(workspace_root)) {
    workspace_toml <- file.path(workspace_root, "Cargo.toml")
    if (add_crate_to_workspace(workspace_toml, name)) {
      cli::cli_alert_success("Added {.val {name}} to workspace members")
    }
  }

  cli::cli_alert_success("Created new crate at {.path {new_crate_path}}")
  invisible(new_crate_path)
}

#' Find the root of a Cargo workspace
#'
#' Tries `git rev-parse --show-toplevel` first (fast, accurate when in a git repo),
#' then falls back to walking up the directory tree looking for a Cargo.toml with
#' `[workspace]`.
#'
#' @param path Path to start searching from
#' @return Path to workspace root, or NULL if not in a workspace
#' @noRd
find_workspace_root <- function(path) {
  path <- normalizePath(path, mustWork = FALSE)

  # Try git first — fast and handles deeply nested paths
  git_root <- tryCatch({
    out <- system2("git", c("rev-parse", "--show-toplevel"),
                   stdout = TRUE, stderr = TRUE)
    if (!is.null(attr(out, "status"))) NULL else trimws(out)
  }, error = function(e) NULL, warning = function(w) NULL)

  if (!is.null(git_root) && nzchar(git_root)) {
    cargo_toml <- file.path(git_root, "Cargo.toml")
    if (file.exists(cargo_toml)) {
      content <- readLines(cargo_toml, warn = FALSE)
      if (any(grepl("^\\[workspace\\]", content))) {
        return(normalizePath(git_root))
      }
    }
  }

  # Fallback: walk up the directory tree
  while (path != dirname(path)) {  # Stop at filesystem root
    cargo_toml <- file.path(path, "Cargo.toml")
    if (file.exists(cargo_toml)) {
      content <- readLines(cargo_toml, warn = FALSE)
      if (any(grepl("^\\[workspace\\]", content))) {
        return(path)
      }
    }
    path <- dirname(path)
  }

  NULL
}

#' Add a crate to workspace members
#'
#' Modifies the workspace Cargo.toml to add a new member.
#'
#' @param workspace_toml Path to workspace Cargo.toml
#' @param crate_name Name of crate to add
#' @return TRUE if successfully added, FALSE if already present
#' @noRd
add_crate_to_workspace <- function(workspace_toml, crate_name) {
  content <- readLines(workspace_toml, warn = FALSE)

  # Find the members = [ line
  members_line <- grep("^members\\s*=\\s*\\[", content)
  if (length(members_line) == 0) {
    cli::cli_warn("Could not find {.code members = []} in workspace Cargo.toml")
    return(FALSE)
  }

  # Check if crate is already in members
  members_pattern <- sprintf('"%s"', crate_name)
  if (any(grepl(members_pattern, content, fixed = TRUE))) {
    cli::cli_alert_info("{.val {crate_name}} is already in workspace members")
    return(FALSE)
  }

  # Handle one-line array: members = ["a", "b"]
  # If the opening [ and closing ] are on the same line, expand to multiline first
  line_text <- content[members_line]
  if (grepl("\\[.*\\]", line_text)) {
    # Extract the array content between [ and ]
    inner <- sub("^members\\s*=\\s*\\[(.*)\\]\\s*$", "\\1", line_text)
    items <- trimws(strsplit(inner, ",")[[1]])
    items <- items[nzchar(items)]

    # Rebuild as multiline
    new_lines <- "members = ["
    for (item in items) {
      # Ensure item has trailing comma
      item <- sub(",\\s*$", "", item)
      new_lines <- c(new_lines, sprintf('    %s,', item))
    }
    new_lines <- c(new_lines, sprintf('    "%s",', crate_name))
    new_lines <- c(new_lines, "]")

    content <- c(
      content[seq_len(members_line - 1)],
      new_lines,
      content[seq_len(length(content) - members_line) + members_line]
    )

    writeLines(content, workspace_toml)
    return(TRUE)
  }

  # Multiline array: find the closing ]
  bracket_depth <- 0
  insert_line <- NULL

  for (i in members_line:length(content)) {
    line <- content[i]
    bracket_depth <- bracket_depth + lengths(regmatches(line, gregexpr("\\[", line)))
    bracket_depth <- bracket_depth - lengths(regmatches(line, gregexpr("\\]", line)))

    if (bracket_depth == 0) {
      insert_line <- i
      break
    }
  }

  if (is.null(insert_line)) {
    cli::cli_warn("Could not find closing bracket for members array")
    return(FALSE)
  }

  # Try to match the indentation of existing members
  indent <- "    "  # Default 4 spaces
  for (j in (members_line + 1):(insert_line - 1)) {
    if (grepl('^\\s+".+"', content[j])) {
      indent <- sub('".*', "", content[j])
      break
    }
  }

  new_member_line <- sprintf('%s"%s",', indent, crate_name)

  # Insert before the closing bracket line
  content <- c(
    content[1:(insert_line - 1)],
    new_member_line,
    content[insert_line:length(content)]
  )

  writeLines(content, workspace_toml)
  TRUE
}
