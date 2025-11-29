# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# This bundles miniextendr dependencies to inst/vendor for self-contained builds

message("Running bootstrap.R...")

pkg_root <- getwd()
rust_dir <- file.path(pkg_root, "src", "rust")
inst_vendor <- file.path(pkg_root, "inst", "vendor")
configure_script <- file.path(pkg_root, "configure")

# Source locations in monorepo
miniextendr_api_src <- file.path(pkg_root, "..", "miniextendr-api")
miniextendr_macros_src <- file.path(pkg_root, "..", "miniextendr-macros")

# Destination in inst/vendor
miniextendr_api_dst <- file.path(inst_vendor, "miniextendr-api")
miniextendr_macros_dst <- file.path(inst_vendor, "miniextendr-macros")

# Workspace Cargo.toml (to extract version/edition)
workspace_cargo_toml <- file.path(pkg_root, "..", "Cargo.toml")

# Ensure inst/vendor directory exists
dir.create(inst_vendor, recursive = TRUE, showWarnings = FALSE)

# Extract workspace package values from root Cargo.toml
get_workspace_values <- function() {
  version <- "0.1.0" # fallback

  edition <- "2024" # fallback

  if (file.exists(workspace_cargo_toml)) {
    lines <- readLines(workspace_cargo_toml)
    for (line in lines) {
      if (grepl('^\\s*version\\s*=\\s*"([^"]+)"', line)) {
        version <- sub('.*version\\s*=\\s*"([^"]+)".*', '\\1', line)
      }
      if (grepl('^\\s*edition\\s*=\\s*"([^"]+)"', line)) {
        edition <- sub('.*edition\\s*=\\s*"([^"]+)".*', '\\1', line)
      }
    }
  }

  list(version = version, edition = edition)
}

workspace_values <- get_workspace_values()

# Helper to copy directory recursively (excluding target dirs)
copy_rust_pkg <- function(src, dst) {
  if (!dir.exists(src)) {
    stop("Source directory does not exist: ", src)
  }

  # Remove destination if exists
  if (dir.exists(dst)) {
    message("  Removing existing: ", basename(dst))
    unlink(dst, recursive = TRUE)
  }

  # Create destination
  dir.create(dst, recursive = TRUE, showWarnings = FALSE)

  # Copy files (excluding target/, .git, etc.)
  files <- list.files(src, all.files = TRUE, no.. = TRUE)
  exclude <- c("target", ".git", ".gitignore")
  files <- files[!files %in% exclude]

  for (f in files) {
    src_path <- file.path(src, f)
    dst_path <- file.path(dst, f)
    if (dir.exists(src_path)) {
      dir.create(dst_path, recursive = TRUE, showWarnings = FALSE)
      file.copy(src_path, dst, recursive = TRUE, overwrite = TRUE)
    } else {
      file.copy(src_path, dst_path, overwrite = TRUE)
    }
  }
}

# Patch Cargo.toml to remove workspace inheritance
patch_cargo_toml <- function(path) {
  if (!file.exists(path)) {
    stop("Cargo.toml not found: ", path)
  }

  lines <- readLines(path)

  # Replace workspace inheritance with actual values from workspace Cargo.toml
  version_replacement <- sprintf('version = "%s"', workspace_values$version)
  edition_replacement <- sprintf('edition = "%s"', workspace_values$edition)
  lines <- gsub('version\\.workspace\\s*=\\s*true', version_replacement, lines)
  lines <- gsub('edition\\.workspace\\s*=\\s*true', edition_replacement, lines)

  writeLines(lines, path)
}

# Step 1: Copy miniextendr-macros to inst/vendor
message("Copying miniextendr-macros to inst/vendor...")
if (dir.exists(miniextendr_macros_src)) {
  copy_rust_pkg(miniextendr_macros_src, miniextendr_macros_dst)
  patch_cargo_toml(file.path(miniextendr_macros_dst, "Cargo.toml"))
  message("  Done")
} else {
  message("  Source not found, skipping (may already be bundled)")
}

# Step 2: Copy miniextendr-api to inst/vendor
message("Copying miniextendr-api to inst/vendor...")
if (dir.exists(miniextendr_api_src)) {
  copy_rust_pkg(miniextendr_api_src, miniextendr_api_dst)
  patch_cargo_toml(file.path(miniextendr_api_dst, "Cargo.toml"))
  # Update path reference to sibling miniextendr-macros
  api_cargo <- file.path(miniextendr_api_dst, "Cargo.toml")
  lines <- readLines(api_cargo)
  lines <- gsub(
    'path\\s*=\\s*"[^"]*miniextendr-macros"',
    'path = "../miniextendr-macros"',
    lines
  )
  writeLines(lines, api_cargo)
  message("  Done")
} else {
  message("  Source not found, skipping (may already be bundled)")
}

# Step 3: Run configure to generate Cargo.toml, config.toml, Makevars
if (file.exists(configure_script)) {
  message("Running configure...")
  status <- system("./configure")
  if (status != 0) {
    stop("configure failed with status ", status)
  }
}

message("bootstrap.R completed successfully")
