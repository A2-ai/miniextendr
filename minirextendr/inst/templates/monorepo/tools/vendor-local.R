#!/usr/bin/env Rscript
# vendor-local.R — Vendor workspace crates from a local monorepo
#
# Standalone script (no package dependencies). Called from configure.ac
# to sync workspace crates into vendor/ with workspace inheritance resolved.
#
# Usage:
#   Rscript tools/vendor-local.R <workspace-root> <dest-dir> [crate1 crate2 ...]
#
# If no crate names are given, defaults to:
#   miniextendr-api miniextendr-macros miniextendr-macros-core
#   miniextendr-lint miniextendr-engine

args <- commandArgs(trailingOnly = TRUE)

if (length(args) < 2) {
  stop("Usage: Rscript vendor-local.R <workspace-root> <dest-dir> [crate1 ...]")
}

workspace_root <- normalizePath(args[1], mustWork = TRUE)
dest_dir <- args[2]

default_crates <- c(
  "miniextendr-api", "miniextendr-macros", "miniextendr-macros-core",
  "miniextendr-lint", "miniextendr-engine"
)
crates <- if (length(args) > 2) args[3:length(args)] else default_crates

# --- Workspace TOML parsing ---

parse_workspace_toml <- function(ws_toml_path) {
  lines <- readLines(ws_toml_path, warn = FALSE)
  trimmed <- trimws(lines)
  section <- ""
  pkg <- list()
  deps <- list()

  for (line in trimmed) {
    if (grepl("^\\[", line)) {
      section <- line
      next
    }
    if (nchar(line) == 0 || grepl("^#", line)) next

    if (section == "[workspace.package]") {
      m <- regmatches(line, regexec("^([a-zA-Z_-]+)\\s*=\\s*(.+)$", line))[[1]]
      if (length(m) == 3) pkg[[m[2]]] <- m[3]
    } else if (section == "[workspace.dependencies]") {
      m <- regmatches(line, regexec("^([a-zA-Z0-9_-]+)\\s*=\\s*(.+)$", line))[[1]]
      if (length(m) == 3) deps[[m[2]]] <- m[3]
    }
  }

  list(package = pkg, dependencies = deps)
}

escape_regex <- function(x) {
  gsub("([.\\\\|^$*+?{}()\\[\\]])", "\\\\\\1", x)
}

strip_toml_sections <- function(lines, headers) {
  trimmed <- trimws(lines)
  is_target <- trimmed %in% headers
  is_any_header <- grepl("^\\[", trimmed)
  keep <- rep(TRUE, length(lines))
  in_section <- FALSE

  for (i in seq_along(lines)) {
    if (is_target[i]) {
      in_section <- TRUE
      keep[i] <- FALSE
    } else if (in_section) {
      if (is_any_header[i]) {
        in_section <- FALSE
      } else {
        keep[i] <- FALSE
      }
    }
  }
  lines[keep]
}

resolve_cargo_toml <- function(path, ws) {
  content <- readLines(path, warn = FALSE)

  # 1. Package-level workspace fields: key.workspace = true
  for (key in names(ws$package)) {
    pattern <- paste0(escape_regex(key), "\\.workspace\\s*=\\s*true")
    replacement <- paste0(key, " = ", ws$package[[key]])
    content <- gsub(pattern, replacement, content)
  }

  # 2. Dependency-level: name = { workspace = true }
  for (dep_name in names(ws$dependencies)) {
    dep_val <- ws$dependencies[[dep_name]]
    # Workspace members (have path key) → rewrite to relative path
    if (grepl("path\\s*=", dep_val)) {
      dep_val <- gsub('path\\s*=\\s*"([^"]+)"', 'path = "../\\1"', dep_val)
    }
    pattern <- paste0(
      escape_regex(dep_name),
      "\\s*=\\s*\\{\\s*workspace\\s*=\\s*true\\s*\\}"
    )
    replacement <- paste0(dep_name, " = ", dep_val)
    content <- gsub(pattern, replacement, content)
  }

  # 3. Strip [dev-dependencies], [[bench]], [[test]]
  content <- strip_toml_sections(content,
    c("[[bench]]", "[[test]]", "[dev-dependencies]"))

  # 4. Warn about unresolved entries
  remaining <- grep("workspace\\s*=\\s*true", content, value = TRUE)
  if (length(remaining) > 0) {
    message("Warning: unresolved workspace inheritance in ", path, ":")
    for (r in remaining) message("  ", trimws(r))
  }

  writeLines(content, path)
}

# --- Strip vendored crate ---

strip_vendored_crate <- function(crate_path) {
  unwanted <- c("target", ".git", ".github", "tests", "benches",
                 "examples", "docs", "ci", ".circleci")
  for (d in unwanted) {
    d_path <- file.path(crate_path, d)
    if (dir.exists(d_path)) unlink(d_path, recursive = TRUE)
  }

  # Remove dotfiles (except .cargo-checksum.json)
  all_files <- list.files(crate_path, all.files = TRUE, full.names = TRUE,
                          no.. = TRUE)
  dotfiles <- all_files[grepl("^\\.", basename(all_files))]
  dotfiles <- dotfiles[basename(dotfiles) != ".cargo-checksum.json"]
  for (f in dotfiles) {
    if (dir.exists(f)) unlink(f, recursive = TRUE) else unlink(f)
  }
}

# --- Main ---

ws_toml <- file.path(workspace_root, "Cargo.toml")
if (!file.exists(ws_toml)) {
  stop("Workspace Cargo.toml not found at: ", ws_toml)
}

ws <- parse_workspace_toml(ws_toml)

dir.create(dest_dir, showWarnings = FALSE, recursive = TRUE)

failed <- character()
for (crate in crates) {
  src <- file.path(workspace_root, crate)
  dest <- file.path(dest_dir, crate)

  if (!dir.exists(src)) {
    message("Warning: crate ", crate, " not found at ", src)
    failed <- c(failed, crate)
    next
  }

  # Remove existing
  if (dir.exists(dest)) unlink(dest, recursive = TRUE)

  # Copy
  dir.create(dest, recursive = TRUE)
  file.copy(list.files(src, full.names = TRUE, all.files = TRUE, no.. = TRUE),
            dest, recursive = TRUE)

  # Strip unwanted content
  strip_vendored_crate(dest)

  # Resolve workspace inheritance
  cargo_toml <- file.path(dest, "Cargo.toml")
  if (file.exists(cargo_toml)) {
    resolve_cargo_toml(cargo_toml, ws)
  }

  # Create checksum file
  writeLines('{"files": {}, "package": null}',
             file.path(dest, ".cargo-checksum.json"))

  message("Vendored: ", crate)
}

# Record source for future syncs
writeLines(workspace_root, file.path(dest_dir, ".vendor-source"))

if (length(failed) > 0) {
  stop("Failed to vendor: ", paste(failed, collapse = ", "))
}

message("Done: ", length(crates), " crates vendored to ", dest_dir)
