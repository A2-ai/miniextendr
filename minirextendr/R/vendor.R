# Vendor management functions

# GitHub repo for miniextendr
MINIEXTENDR_REPO <- "CGMossa/miniextendr"

#' List available miniextendr versions
#'
#' Queries GitHub to find available releases/tags of miniextendr.
#' Uses `git ls-remote --tags` which respects credential helpers and
#' environment tokens (`GITHUB_TOKEN`).
#'
#' @return Character vector of available version tags
#' @export
miniextendr_available_versions <- function() {
  repo_url <- paste0("https://github.com/", MINIEXTENDR_REPO, ".git")
  output <- tryCatch(
    {
      system2("git", c("ls-remote", "--tags", "--refs", repo_url),
              stdout = TRUE, stderr = FALSE)
    },
    error = function(e) {
      cli::cli_warn(c(
        "Failed to fetch versions from GitHub",
        "i" = conditionMessage(e)
      ))
      return(NULL)
    }
  )

  if (is.null(output) || length(output) == 0) {
    cli::cli_alert_info("No releases found, using 'main' branch")
    return("main")
  }

  # Each line: "<hash>\trefs/tags/<tagname>"
  tags <- sub(".*refs/tags/", "", output)
  cli::cli_alert_info("Available versions: {paste(tags, collapse = ', ')}")
  tags
}

#' Download miniextendr archive from GitHub
#'
#' @param version Version tag to download
#' @param dest_path Path to save the archive
#' @return Path to downloaded archive
#' @noRd
download_miniextendr_archive <- function(version, dest_path) {
  # Try heads first (for branch names like "main")
  archive_url <- paste0(
    "https://github.com/", MINIEXTENDR_REPO,
    "/archive/refs/heads/", version, ".tar.gz"
  )

  download_result <- tryCatch(
    {
      utils::download.file(archive_url, dest_path, quiet = TRUE, mode = "wb")
      TRUE
    },
    error = function(e) {
      # Try as a tag instead
      tag_url <- paste0(
        "https://github.com/", MINIEXTENDR_REPO,
        "/archive/refs/tags/", version, ".tar.gz"
      )
      tryCatch(
        {
          utils::download.file(tag_url, dest_path, quiet = TRUE, mode = "wb")
          TRUE
        },
        error = function(e2) {
          FALSE
        }
      )
    }
  )

  if (!download_result) {
    cli::cli_abort(c(
      "Failed to download miniextendr",
      "i" = "Check that version '{version}' exists at github.com/{MINIEXTENDR_REPO}"
    ))
  }

  cli::cli_alert_success("Downloaded and cached miniextendr {version}")
  dest_path
}

#' Download and vendor miniextendr crates
#'
#' Downloads miniextendr-api, miniextendr-macros, miniextendr-macros-core,
#' miniextendr-lint, and miniextendr-engine from GitHub and vendors them
#' into vendor/. Also
#' patches Cargo.toml files to remove workspace inheritance.
#'
#' Downloaded archives are cached in `tools::R_user_dir("minirextendr", "cache")`
#' to avoid repeated downloads of the same version.
#'
#' For local development (when GitHub repo is not available), set
#' `local_path` to the path of the miniextendr repository.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param version Version tag to download (default: "main" for latest).
#'   Ignored if `local_path` is provided.
#' @param dest Destination directory for vendored crates. Defaults to
#'   `vendor/` inside the project root.
#' @param refresh Force re-download even if cached (default: FALSE)
#' @param local_path Path to local miniextendr repository. If provided,
#'   copies crates from local path instead of downloading from GitHub.
#' @return Invisibly returns TRUE on success
#' @export
vendor_miniextendr <- function(path = ".",
                               version = "main",
                               dest = NULL,
                               refresh = FALSE,
                               local_path = NULL) {
  with_project(path)
  dest <- dest %||% usethis::proj_path("vendor")
  # If local_path is provided, use local vendoring

  if (!is.null(local_path)) {
    vendor_miniextendr_local(local_path, dest)
    # Add [patch] entries so dev mode (NOT_CRAN=true) resolves from vendor/
    # instead of fetching from git (which fails if miniextendr-macros isn't on crates.io)
    add_vendor_patches(dest)
    return(invisible(TRUE))
  }

  # Check cache first
  cache_dir <- tools::R_user_dir("minirextendr", "cache")
  fs::dir_create(cache_dir, recurse = TRUE)
  cache_file <- fs::path(cache_dir, paste0("miniextendr-", version, ".tar.gz"))

  if (fs::file_exists(cache_file) && !refresh) {
    cli::cli_alert_success("Using cached miniextendr {version}")
    archive_path <- cache_file
  } else {
    cli::cli_alert("Downloading miniextendr {version} from GitHub...")
    archive_path <- download_miniextendr_archive(version, cache_file)
  }

  # Create temp directory for extraction
  tmp_dir <- fs::path_temp("miniextendr")
  on.exit(unlink(tmp_dir, recursive = TRUE), add = TRUE)
  fs::dir_create(tmp_dir)

  # Extract archive
  cli::cli_alert("Extracting archive...")
  utils::untar(archive_path, exdir = tmp_dir)

  # Find extracted directory (GitHub archives always extract to exactly one top-level directory)
  extracted_dirs <- fs::dir_ls(tmp_dir, type = "directory")

  if (length(extracted_dirs) != 1) {
    cli::cli_abort(c(
      "Unexpected archive structure",
      "i" = "Expected exactly 1 top-level directory, found {length(extracted_dirs)}"
    ))
  }

  extracted_dir <- extracted_dirs[[1]]

  # Create vendor directory
  ensure_dir(dest)

  # Copy crates
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core", "miniextendr-lint", "miniextendr-engine")
  failed_crates <- character()

  for (crate in crates) {
    src_path <- fs::path(extracted_dir, crate)
    dest_path <- fs::path(dest, crate)

    if (!fs::dir_exists(src_path)) {
      cli::cli_warn("Crate {crate} not found in downloaded archive")
      failed_crates <- c(failed_crates, crate)
      next
    }

    # Remove existing if present
    if (fs::dir_exists(dest_path)) {
      fs::dir_delete(dest_path)
    }

    # Copy crate (excluding target, .git)
    fs::dir_copy(src_path, dest_path)

    # Strip build artifacts, tests, benchmarks, and hidden files
    strip_vendored_crate(dest_path)

    # Patch Cargo.toml to remove workspace inheritance
    cargo_toml <- fs::path(dest_path, "Cargo.toml")
    if (fs::file_exists(cargo_toml)) {
      patch_cargo_toml(cargo_toml, crate)
    }

    # Create .cargo-checksum.json (required when crate is in a vendor directory
    # that replaces crates-io via .cargo/config.toml)
    checksum_file <- fs::path(dest_path, ".cargo-checksum.json")
    writeLines('{"files": {}, "package": null}', checksum_file)

    cli::cli_alert_success("Vendored {crate}")
  }

  if (length(failed_crates) > 0) {
    cli::cli_abort(c(
      "Failed to vendor {length(failed_crates)} required crate(s)",
      "x" = "Missing: {paste(failed_crates, collapse = ', ')}",
      "i" = "The downloaded archive may be from an incompatible version"
    ))
  }

  cli::cli_alert_success("miniextendr crates vendored to {.path {dest}}")
  invisible(TRUE)
}

#' Patch Cargo.toml to remove workspace inheritance
#'
#' @param path Path to Cargo.toml
#' @param crate_name Name of the crate
#' @noRd
patch_cargo_toml <- function(path, crate_name) {
  content <- readLines(path, warn = FALSE)

  # Replace workspace = true with actual values
  replacements <- list(
    'edition\\.workspace = true' = 'edition = "2024"',
    'version\\.workspace = true' = 'version = "0.1.0"',
    'license\\.workspace = true' = 'license = "MIT"',
    'repository\\.workspace = true' = 'repository = "https://github.com/CGMossa/miniextendr"',
    'homepage\\.workspace = true' = 'homepage = "https://github.com/CGMossa/miniextendr"',
    'keywords\\.workspace = true' = 'keywords = ["r", "ffi", "bindings"]',
    'categories\\.workspace = true' = 'categories = ["api-bindings", "external-ffi-bindings"]'
  )

  for (pattern in names(replacements)) {
    content <- gsub(pattern, replacements[[pattern]], content)
  }

  # Replace workspace dependencies with path/version
  dep_replacements <- list(
    'miniextendr-macros = \\{ workspace = true \\}' =
      'miniextendr-macros = { version = "0.1.0", path = "../miniextendr-macros" }',
    'miniextendr-macros-core = \\{ workspace = true \\}' =
      'miniextendr-macros-core = { version = "0.1.0", path = "../miniextendr-macros-core" }',
    'miniextendr-engine = \\{ workspace = true \\}' =
      'miniextendr-engine = { version = "0.1.0", path = "../miniextendr-engine" }',
    'proc-macro2 = \\{ workspace = true \\}' =
      'proc-macro2 = { version = "1.0", features = ["span-locations"] }',
    'quote = \\{ workspace = true \\}' =
      'quote = "1.0"',
    'syn = \\{ workspace = true \\}' =
      'syn = { version = "2.0", features = ["full", "extra-traits"] }',
    'linkme = \\{ workspace = true \\}' =
      'linkme = "0.3"'
  )

  for (pattern in names(dep_replacements)) {
    content <- gsub(pattern, dep_replacements[[pattern]], content)
  }

  # Remove dev-dependencies that create circular references or dangling paths when vendored
  # miniextendr-api in miniextendr-macros dev-deps is only for workspace testing
  content <- content[!grepl("^miniextendr-api = \\{ workspace = true \\}", content)]
  # miniextendr-engine in miniextendr-api dev-deps is not used by scaffolded packages
  content <- content[!grepl("^miniextendr-engine = ", content)]

  # Remove [[bench]], [[test]], and [dev-dependencies] sections entirely.
  # strip_vendored_crate() deletes benches/ and tests/ directories, so these

  # TOML sections become dangling references that cause cargo errors.
  content <- strip_toml_sections(content,
    c("[[bench]]", "[[test]]", "[dev-dependencies]"))

  # Validate: warn if any workspace = true entries remain unhandled
  remaining <- grep("workspace\\s*=\\s*true", content, value = TRUE)
  if (length(remaining) > 0) {
    # Escape curly braces so cli doesn't interpret TOML inline tables as glue
    escaped <- gsub("{", "{{", gsub("}", "}}", trimws(remaining), fixed = TRUE), fixed = TRUE)
    cli::cli_warn(c(
      "Unhandled workspace inheritance in {.path {path}}",
      "i" = "The following lines still reference workspace:",
      paste("  ", escaped)
    ))
  }

  writeLines(content, path)
}

#' Vendor miniextendr crates from local path
#'
#' Copies miniextendr crates from a local repository instead of downloading
#' from GitHub. Used for development when the GitHub repo is not available.
#'
#' @param local_path Path to local miniextendr repository
#' @param dest Destination directory for vendored crates
#' @return Invisibly returns TRUE on success
#' @noRd
vendor_miniextendr_local <- function(local_path, dest) {
  local_path <- normalizePath(local_path, mustWork = TRUE)

  cli::cli_alert("Vendoring miniextendr from local path: {.path {local_path}}")

  # Create vendor directory
  ensure_dir(dest)

  # Copy crates
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core", "miniextendr-lint", "miniextendr-engine")
  failed_crates <- character()

  for (crate in crates) {
    src_path <- fs::path(local_path, crate)
    dest_path <- fs::path(dest, crate)

    if (!fs::dir_exists(src_path)) {
      cli::cli_warn("Crate {crate} not found at {.path {src_path}}")
      failed_crates <- c(failed_crates, crate)
      next
    }

    # Remove existing if present
    if (fs::dir_exists(dest_path)) {
      fs::dir_delete(dest_path)
    }

    # Copy crate (excluding target, .git, etc.)
    fs::dir_copy(src_path, dest_path)

    # Strip build artifacts, tests, benchmarks, and hidden files
    strip_vendored_crate(dest_path)

    # Patch Cargo.toml to remove workspace inheritance
    cargo_toml <- fs::path(dest_path, "Cargo.toml")
    if (fs::file_exists(cargo_toml)) {
      patch_cargo_toml(cargo_toml, crate)
    }

    # Create .cargo-checksum.json (required when crate is in a vendor directory
    # that replaces crates-io via .cargo/config.toml)
    checksum_file <- fs::path(dest_path, ".cargo-checksum.json")
    writeLines('{"files": {}, "package": null}', checksum_file)

    cli::cli_alert_success("Vendored {crate}")
  }

  if (length(failed_crates) > 0) {
    cli::cli_abort(c(
      "Failed to vendor {length(failed_crates)} required crate(s)",
      "x" = "Missing: {paste(failed_crates, collapse = ', ')}",
      "i" = "Check that {.path {local_path}} is a valid miniextendr repository"
    ))
  }

  # Record the source path for future auto-sync
  writeLines(local_path, fs::path(dest, ".vendor-source"))

  cli::cli_alert_success("miniextendr crates vendored from local path to {.path {dest}}")

  invisible(TRUE)
}

#' Sync vendored miniextendr crates from local source
#'
#' Auto-detects whether vendor/ was populated from a local miniextendr
#' repository and re-syncs crates if the source is available. This ensures
#' vendor/ stays up-to-date with workspace crate changes during development.
#'
#' Detection order:
#' 1. `.vendor-source` marker file in vendor/ (recorded by previous local vendor)
#' 2. Auto-scan parent directories for a miniextendr workspace
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if sync occurred, FALSE if no local source found.
#' @keywords internal
#' @export
vendor_sync <- function(path = ".") {
  with_project(path)
  vendor_dir <- usethis::proj_path("vendor")

  if (!fs::dir_exists(vendor_dir)) {
    cli::cli_alert_info("No vendor/ directory — nothing to sync")
    return(invisible(FALSE))
  }

  local_path <- detect_miniextendr_local(vendor_dir)

  if (is.null(local_path)) {
    cli::cli_alert_info("No local miniextendr source detected — vendor/ unchanged")
    return(invisible(FALSE))
  }

  cli::cli_alert("Syncing vendor/ from {.path {local_path}}")
  vendor_miniextendr_local(local_path, vendor_dir)
  invisible(TRUE)
}

#' Detect local miniextendr repository for vendor sync
#'
#' Checks multiple sources to find a local miniextendr monorepo:
#' 1. `.vendor-source` marker file in vendor/
#' 2. Walk up parent directories looking for miniextendr-api/Cargo.toml
#'
#' @param vendor_dir Path to vendor/ directory
#' @return Normalized path to miniextendr repo root, or NULL if not found
#' @noRd
detect_miniextendr_local <- function(vendor_dir) {
  # 1. Recorded source from previous local vendor
  source_file <- fs::path(vendor_dir, ".vendor-source")
  if (fs::file_exists(source_file)) {
    recorded <- trimws(readLines(source_file, n = 1, warn = FALSE))
    if (nzchar(recorded) && dir.exists(recorded)) {
      api_toml <- file.path(recorded, "miniextendr-api", "Cargo.toml")
      if (file.exists(api_toml)) {
        return(normalizePath(recorded, mustWork = TRUE))
      }
    }
  }

  # 3. Walk up parent directories looking for miniextendr workspace
  pkg_root <- dirname(vendor_dir)
  search_dir <- normalizePath(pkg_root, mustWork = TRUE)
  for (i in seq_len(10)) {
    search_dir <- dirname(search_dir)
    if (search_dir == dirname(search_dir)) break # hit filesystem root
    candidate <- file.path(search_dir, "miniextendr-api", "Cargo.toml")
    if (file.exists(candidate)) {
      return(normalizePath(search_dir, mustWork = TRUE))
    }
  }

  NULL
}

#' Vendor external crates.io dependencies
#'
#' Runs `cargo vendor` to download all external crates.io dependencies
#' (like proc-macro2, syn, quote) for offline/CRAN builds. This is separate
#' from [vendor_miniextendr()] which vendors the miniextendr workspace crates.
#'
#' Most users should use [miniextendr_vendor()] instead, which calls this
#' function as part of the full CRAN vendor workflow.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE on success
#' @keywords internal
#' @export
vendor_crates_io <- function(path = ".") {
  with_project(path)
  check_rust()

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    cli::cli_abort(c(
      "Cargo.toml not found",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  vendor_dir <- usethis::proj_path("vendor")

  cli::cli_alert("Running cargo vendor...")

  result <- run_with_logging(
    "cargo",
    args = c("vendor", "--manifest-path", cargo_toml, vendor_dir),
    log_prefix = "cargo-vendor",
    wd = usethis::proj_get()
  )

  check_result(result, "cargo vendor")

  # Strip CRAN-unfriendly content from vendored crates
  strip_vendored_dir(vendor_dir)

  cli::cli_alert_success("External dependencies vendored")
  invisible(TRUE)
}

#' Strip CRAN-unfriendly content from a single vendored crate
#'
#' Removes build artifacts, tests, benchmarks, examples, hidden files,
#' and other content that causes CRAN NOTEs (portable filenames,
#' hidden files, long paths).
#'
#' @param crate_path Path to the vendored crate directory
#' @noRd
strip_vendored_crate <- function(crate_path) {
  # Directories to remove entirely
  unwanted_dirs <- c("target", ".git", ".github", "tests", "benches",
                     "examples", "docs", "ci", ".circleci")
  for (d in unwanted_dirs) {
    d_path <- fs::path(crate_path, d)
    if (fs::dir_exists(d_path)) {
      fs::dir_delete(d_path)
    }
  }

  # Remove hidden dotfiles (except .cargo-checksum.json which cargo needs)
  all_files <- fs::dir_ls(crate_path, all = TRUE, recurse = FALSE)
  dotfiles <- all_files[grepl("^\\.", basename(all_files))]
  dotfiles <- dotfiles[basename(dotfiles) != ".cargo-checksum.json"]
  for (f in dotfiles) {
    if (fs::is_dir(f)) {
      fs::dir_delete(f)
    } else {
      fs::file_delete(f)
    }
  }
}

#' Remove TOML sections from a character vector of lines
#'
#' Removes complete sections (header line through end of section) for the
#' given TOML headers. A section ends at the next header (`[...]`) or EOF.
#'
#' @param lines Character vector of TOML file lines
#' @param headers Character vector of section headers to remove, e.g.
#'   `c("[[bench]]", "[dev-dependencies]")`
#' @return Filtered character vector with those sections removed
#' @noRd
strip_toml_sections <- function(lines, headers) {
  trimmed <- trimws(lines)

  # Check if a line matches any target header (exact match after trim)
  is_target <- trimmed %in% headers
  # Also match table subsections: [dev-dependencies.X] for header [dev-dependencies]
  for (h in headers) {
    if (!startsWith(h, "[[") && endsWith(h, "]")) {
      prefix <- paste0(substr(h, 1, nchar(h) - 1), ".")
      is_target <- is_target | startsWith(trimmed, prefix)
    }
  }

  # Check if a line starts any TOML section (single or double bracket)
  is_any_header <- grepl("^\\[", trimmed)

  keep <- rep(TRUE, length(lines))
  in_section <- FALSE

  for (i in seq_along(lines)) {
    if (is_target[i]) {
      in_section <- TRUE
      keep[i] <- FALSE
    } else if (in_section) {
      if (is_any_header[i]) {
        # Hit a new section — stop stripping
        in_section <- FALSE
      } else {
        keep[i] <- FALSE
      }
    }
  }

  lines[keep]
}

#' Strip CRAN-unfriendly content from an entire vendor directory
#'
#' Walks all crates in a cargo vendor output directory and strips
#' tests, benchmarks, examples, hidden files, and other content that
#' causes CRAN NOTEs.
#'
#' @param vendor_path Path to the vendor directory
#' @noRd
strip_vendored_dir <- function(vendor_path) {
  if (!fs::dir_exists(vendor_path)) return(invisible())

  crate_dirs <- fs::dir_ls(vendor_path, type = "directory")
  for (crate_dir in crate_dirs) {
    strip_vendored_crate(crate_dir)

    # Clear checksums (content was modified by stripping)
    checksum_file <- fs::path(crate_dir, ".cargo-checksum.json")
    writeLines('{"files":{}}', checksum_file)
  }
  invisible()
}

#' Clear miniextendr download cache
#'
#' Removes cached miniextendr archives from the user cache directory.
#'
#' @param version Optional version to clear. If NULL, clears all cached versions.
#' @return Invisibly returns TRUE
#' @export
miniextendr_clear_cache <- function(version = NULL) {
  cache_dir <- tools::R_user_dir("minirextendr", "cache")

  if (!fs::dir_exists(cache_dir)) {
    cli::cli_alert_info("No cache directory found")
    return(invisible(TRUE))
  }

  if (is.null(version)) {
    # Clear all
    files <- fs::dir_ls(cache_dir, glob = "*.tar.gz")
    if (length(files) == 0) {
      cli::cli_alert_info("Cache is empty")
    } else {
      fs::file_delete(files)
      cli::cli_alert_success("Cleared {length(files)} cached archive(s)")
    }
  } else {
    # Clear specific version
    cache_file <- fs::path(cache_dir, paste0("miniextendr-", version, ".tar.gz"))
    if (fs::file_exists(cache_file)) {
      fs::file_delete(cache_file)
      cli::cli_alert_success("Cleared cached miniextendr {version}")
    } else {
      cli::cli_alert_info("No cached archive for version {version}")
    }
  }

  invisible(TRUE)
}

#' Show miniextendr cache info
#'
#' Displays information about cached miniextendr archives.
#'
#' @return Invisibly returns a data frame with cache info
#' @export
miniextendr_cache_info <- function() {
  cache_dir <- tools::R_user_dir("minirextendr", "cache")

  cli::cli_h2("miniextendr cache")
  cli::cli_alert_info("Cache directory: {.path {cache_dir}}")

  if (!fs::dir_exists(cache_dir)) {
    cli::cli_alert_info("Cache directory does not exist")
    return(invisible(data.frame()))
  }

  files <- fs::dir_ls(cache_dir, glob = "*.tar.gz")

  if (length(files) == 0) {
    cli::cli_alert_info("No cached archives")
    return(invisible(data.frame()))
  }

  info <- fs::file_info(files)
  info$version <- gsub("^miniextendr-|\\.tar\\.gz$", "", basename(files))

  cli::cli_alert_success("{length(files)} cached archive(s):")
  for (i in seq_along(files)) {
    size_mb <- round(info$size[i] / 1024 / 1024, 2)
    cli::cli_bullets(c(" " = "{info$version[i]} ({size_mb} MB)"))
  }

  invisible(info[, c("version", "size", "modification_time")])
}

#' Check for path dependencies in Cargo.toml
#'
#' Scans `[dependencies]` and `[build-dependencies]` sections for path-based
#' dependencies. `[patch.*]` sections are excluded since those are normal
#' dev-mode behavior handled by configure.ac.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return A data frame with columns `crate` and `path`, or zero-row data frame
#'   if no path deps found.
#' @noRd
check_path_deps <- function(path = ".") {
  cargo_toml <- file.path(path, "src", "rust", "Cargo.toml")
  if (!file.exists(cargo_toml)) {
    return(data.frame(crate = character(), path = character(), stringsAsFactors = FALSE))
  }

  lines <- readLines(cargo_toml, warn = FALSE)
  trimmed <- trimws(lines)

  # Track which TOML section we're in
  # Only flag path deps in [dependencies] and [build-dependencies]
  crates <- character()
  paths <- character()
  in_relevant_section <- FALSE

  for (line in trimmed) {
    # Detect section headers
    if (grepl("^\\[", line)) {
      in_relevant_section <- line %in% c("[dependencies]", "[build-dependencies]")
      next
    }

    if (!in_relevant_section) next

    # Match lines like: crate-name = { path = "..." ... }
    m <- regmatches(line, regexec('^([a-zA-Z0-9_-]+)\\s*=.*path\\s*=\\s*"([^"]+)"', line))[[1]]
    if (length(m) == 3) {
      crates <- c(crates, m[2])
      paths <- c(paths, m[3])
    }
  }

  data.frame(crate = crates, path = paths, stringsAsFactors = FALSE)
}

#' Add `[patch]` entries to Cargo.toml for vendored crates
#'
#' After vendoring miniextendr crates to vendor/, adds a
#' `[patch."https://github.com/CGMossa/miniextendr"]` section to
#' src/rust/Cargo.toml so that dev mode (NOT_CRAN=true) resolves
#' dependencies from vendor/ instead of fetching from git.
#'
#' @param vendor_dir Path to the vendor directory (vendor/)
#' @noRd
add_vendor_patches <- function(vendor_dir) {
  # Derive Cargo.toml path: vendor is at package root, Cargo.toml is src/rust/Cargo.toml
  pkg_root <- dirname(vendor_dir)
  cargo_toml <- file.path(pkg_root, "src", "rust", "Cargo.toml")

  if (!file.exists(cargo_toml)) return(invisible())

  content <- readLines(cargo_toml, warn = FALSE)

  # Don't add if already has [patch] section
  if (any(grepl("^\\[patch\\.", content))) return(invisible())

  # Don't add if no git URLs to patch (path deps don't need patching)
  if (!any(grepl("github\\.com/CGMossa", content))) return(invisible())

  # Only patch crates that are actual dependencies (not miniextendr-engine,

  # which is only a dev-dependency of miniextendr-api)
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core",
              "miniextendr-lint")

  patch_lines <- c(
    "",
    '[patch."https://github.com/CGMossa/miniextendr"]'
  )
  for (crate in crates) {
    if (dir.exists(file.path(vendor_dir, crate))) {
      patch_lines <- c(patch_lines,
        sprintf('%s = { path = "../../vendor/%s" }', crate, crate))
    }
  }

  writeLines(c(content, patch_lines), cargo_toml)
  cli::cli_alert_success("Added [patch] entries to {.path Cargo.toml} for vendored crates")
}
