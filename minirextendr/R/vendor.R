# Vendor management functions

# GitHub repo for miniextendr
MINIEXTENDR_REPO <- "CGMossa/miniextendr"

#' List available miniextendr versions
#'
#' Queries GitHub to find available releases/tags of miniextendr.
#'
#' @return Character vector of available version tags
#' @export
miniextendr_available_versions <- function() {
  url <- paste0("https://api.github.com/repos/", MINIEXTENDR_REPO, "/tags")

  response <- tryCatch(
    {
      jsonlite::fromJSON(url)
    },
    error = function(e) {
      warn(c(
        "Failed to fetch versions from GitHub",
        "i" = conditionMessage(e)
      ))
      return(NULL)
    }
  )

  if (is.null(response) || length(response) == 0) {
    cli::cli_alert_info("No releases found, using 'main' branch")
    return("main")
  }

  tags <- response$name
  cli::cli_alert_info("Available versions: {paste(tags, collapse = ', ')}")
  tags
}

#' Download and vendor miniextendr crates
#'
#' Downloads miniextendr-api, miniextendr-macros, and miniextendr-lint
#' from GitHub and vendors them into src/vendor/. Also patches
#' Cargo.toml files to remove workspace inheritance.
#'
#' @param version Version tag to download (default: "main" for latest)
#' @param dest Destination directory for vendored crates
#' @return Invisibly returns TRUE on success
#' @export
vendor_miniextendr <- function(version = "main",
                               dest = usethis::proj_path("src", "vendor")) {
  cli::cli_alert("Downloading miniextendr {version} from GitHub...")

  # Create temp directory for download
  tmp_dir <- fs::path_temp("miniextendr")
  on.exit(unlink(tmp_dir, recursive = TRUE), add = TRUE)
  fs::dir_create(tmp_dir)

  # Download archive
  archive_url <- paste0(
    "https://github.com/", MINIEXTENDR_REPO,
    "/archive/refs/heads/", version, ".tar.gz"
  )

  # Try tags if heads fails
  archive_path <- fs::path(tmp_dir, "miniextendr.tar.gz")

  download_result <- tryCatch(
    {
      curl::curl_download(archive_url, archive_path, quiet = TRUE)
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
          curl::curl_download(tag_url, archive_path, quiet = TRUE)
          TRUE
        },
        error = function(e2) {
          FALSE
        }
      )
    }
  )

  if (!download_result) {
    abort(c(
      "Failed to download miniextendr",
      "i" = "Check that version '{version}' exists at github.com/{MINIEXTENDR_REPO}"
    ))
  }

  # Extract archive
  cli::cli_alert("Extracting archive...")
  utils::untar(archive_path, exdir = tmp_dir)

  # Find extracted directory (github archives as repo-branch/)
  extracted_dirs <- fs::dir_ls(tmp_dir, type = "directory")
  extracted_dir <- extracted_dirs[grepl("miniextendr", extracted_dirs)][1]

  if (is.na(extracted_dir) || !fs::dir_exists(extracted_dir)) {
    abort("Failed to find extracted miniextendr directory")
  }

  # Create vendor directory
  ensure_dir(dest)

  # Copy crates
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-lint")

  for (crate in crates) {
    src_path <- fs::path(extracted_dir, crate)
    dest_path <- fs::path(dest, crate)

    if (!fs::dir_exists(src_path)) {
      warn("Crate {crate} not found in downloaded archive")
      next
    }

    # Remove existing if present
    if (fs::dir_exists(dest_path)) {
      fs::dir_delete(dest_path)
    }

    # Copy crate (excluding target, .git)
    fs::dir_copy(src_path, dest_path)

    # Remove unwanted files
    unwanted <- c("target", ".git", ".gitignore", ".DS_Store")
    for (u in unwanted) {
      u_path <- fs::path(dest_path, u)
      if (fs::file_exists(u_path) || fs::dir_exists(u_path)) {
        fs::file_delete(u_path)
      }
    }

    # Patch Cargo.toml to remove workspace inheritance
    cargo_toml <- fs::path(dest_path, "Cargo.toml")
    if (fs::file_exists(cargo_toml)) {
      patch_cargo_toml(cargo_toml, crate)
    }

    cli::cli_alert_success("Vendored {crate}")
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
    'miniextendr-engine = \\{ workspace = true \\}' =
      'miniextendr-engine = { version = "0.1.0", path = "../miniextendr-engine" }',
    'proc-macro2 = \\{ workspace = true \\}' =
      'proc-macro2 = { version = "1.0", features = ["span-locations"] }',
    'quote = \\{ workspace = true \\}' =
      'quote = "1.0"',
    'syn = \\{ workspace = true \\}' =
      'syn = { version = "2.0", features = ["full", "extra-traits"] }'
  )

  for (pattern in names(dep_replacements)) {
    content <- gsub(pattern, dep_replacements[[pattern]], content)
  }

  writeLines(content, path)
}

#' Update vendored miniextendr crates
#'
#' Downloads and replaces the vendored miniextendr crates with a new version.
#'
#' @param version Version to update to (default: "main" for latest)
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_update <- function(version = "main") {
  cli::cli_alert_info("Updating miniextendr to version: {version}")
  vendor_miniextendr(version = version)
}

#' Vendor external crates.io dependencies
#'
#' Runs `cargo vendor` to download all external crates.io dependencies
#' (like proc-macro2, syn, quote) for offline/CRAN builds. This is separate
#' from `vendor_miniextendr()` which downloads the miniextendr crates.
#'
#' @return Invisibly returns TRUE on success
#' @export
vendor_crates_io <- function() {
  check_rust()

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    abort(c(
      "Cargo.toml not found",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  vendor_dir <- usethis::proj_path("src", "vendor")

  cli::cli_alert("Running cargo vendor...")

  result <- withr::with_dir(usethis::proj_get(), {
    system2(
      "cargo",
      c("vendor", "--manifest-path", cargo_toml, vendor_dir),
      stdout = TRUE,
      stderr = TRUE
    )
  })

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "cargo vendor failed",
      "i" = paste(utils::tail(result, 10), collapse = "\n")
    ))
  }

  cli::cli_alert_success("External dependencies vendored")
  invisible(TRUE)
}
