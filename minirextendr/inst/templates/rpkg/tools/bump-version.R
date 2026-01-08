#!/usr/bin/env Rscript
# Sync or bump version between DESCRIPTION and Cargo.toml files.
#
# Usage:
#   Rscript tools/bump-version.R                     # sync Cargo.toml to DESCRIPTION version
#   Rscript tools/bump-version.R --bump=patch        # bump patch version (0.1.0 -> 0.1.1)
#   Rscript tools/bump-version.R --bump=minor        # bump minor version (0.1.0 -> 0.2.0)
#   Rscript tools/bump-version.R --bump=major        # bump major version (0.1.0 -> 1.0.0)
#   Rscript tools/bump-version.R --bump=dev          # add dev suffix (0.1.0 -> 0.1.0.9000)
#   Rscript tools/bump-version.R --bump=dev+         # increment dev (0.1.0.9000 -> 0.1.0.9001)
#   Rscript tools/bump-version.R --set=1.2.3         # set specific version
#
# Modes (for Cargo.toml updates):
#   --mode=auto       (default) update [workspace.package] and [package] versions
#   --mode=workspace  only update [workspace.package] version
#   --mode=crate      only update explicit [package] version
#   --workspace-only  alias for --mode=workspace
#   --crate-only      alias for --mode=crate

desc_path <- "DESCRIPTION"
if (!file.exists(desc_path)) {
  stop("DESCRIPTION not found; run this script from the package root.")
}

desc <- read.dcf(desc_path)
version <- desc[1, "Version"]
if (is.na(version) || trimws(version) == "") {
  stop("DESCRIPTION Version is missing or empty.")
}

# Parse arguments
args <- commandArgs(trailingOnly = TRUE)
mode <- "auto"
op <- "sync"
op_value <- NULL

if (length(args) > 0) {
  for (arg in args) {
    if (identical(arg, "--")) {
      next
    } else if (identical(arg, "--workspace-only")) {
      mode <- "workspace"
    } else if (identical(arg, "--crate-only")) {
      mode <- "crate"
    } else if (grepl("^--mode=", arg)) {
      mode <- sub("^--mode=", "", arg)
    } else if (grepl("^--bump=", arg)) {
      op <- "bump"
      op_value <- sub("^--bump=", "", arg)
    } else if (grepl("^--set=", arg)) {
      op <- "set"
      op_value <- sub("^--set=", "", arg)
    } else if (identical(arg, "--sync")) {
      op <- "sync"
    } else if (identical(arg, "--help") || identical(arg, "-h")) {
      cat(paste0(
        "Usage: Rscript tools/bump-version.R [--mode=auto|workspace|crate] ",
        "[--workspace-only|--crate-only] ",
        "[--sync|--bump=major|minor|patch|dev|dev+|--set=x.y.z[.dev]]\n"
      ))
      quit(status = 0)
    } else {
      stop("Unknown argument: ", arg)
    }
  }
}

if (!mode %in% c("auto", "workspace", "crate")) {
  stop("Invalid mode: ", mode, ". Use auto, workspace, or crate.")
}

op <- tolower(trimws(op))
if (!op %in% c("sync", "bump", "set")) {
  stop("Invalid operation: ", op, ". Use sync, bump, or set.")
}

bump_version <- function(ver, which) {
  parts <- strsplit(ver, ".", fixed = TRUE)[[1]]
  nums <- suppressWarnings(as.integer(parts))
  if (any(is.na(nums)) || length(nums) < 3) {
    stop("Invalid DESCRIPTION Version: ", ver)
  }

  which <- tolower(trimws(which))
  if (!which %in% c("major", "minor", "patch", "dev", "dev+")) {
    stop("Invalid bump: ", which, ". Use major, minor, patch, dev, or dev+.")
  }

  if (which == "dev") {
    if (length(nums) > 3) {
      return(ver)
    }
    return(paste(c(nums[1:3], 9000), collapse = "."))
  }

  if (which == "dev+") {
    if (length(nums) > 3) {
      nums[4] <- nums[4] + 1
      return(paste(nums, collapse = "."))
    }
    return(paste(c(nums[1:3], 9000), collapse = "."))
  }

  nums <- nums[1:3]
  if (which == "major") {
    nums <- c(nums[1] + 1, 0, 0)
  } else if (which == "minor") {
    nums <- c(nums[1], nums[2] + 1, 0)
  } else if (which == "patch") {
    nums <- c(nums[1], nums[2], nums[3] + 1)
  }

  paste(nums, collapse = ".")
}

update_description_version <- function(path, new_version) {
  lines <- readLines(path, warn = FALSE)
  idx <- grep("^Version\\s*:", lines)
  if (length(idx) == 0) {
    stop("DESCRIPTION Version field not found.")
  }
  lines[idx[1]] <- sub(
    "^Version\\s*:\\s*.*$",
    paste0("Version: ", new_version),
    lines[idx[1]]
  )
  if (!identical(lines, readLines(path, warn = FALSE))) {
    writeLines(lines, path, sep = "\n")
  }
}

update_configure_ac_version <- function(path, new_version) {
  if (!file.exists(path)) {
    return(FALSE)
  }
  lines <- readLines(path, warn = FALSE)
  # Match AC_INIT([package], [version]) or AC_INIT([package], [version], ...)
  idx <- grep("^AC_INIT\\s*\\(\\s*\\[", lines)
  if (length(idx) == 0) {
    warning("No AC_INIT found in ", path)
    return(FALSE)
  }
  # Replace version in AC_INIT([pkg], [ver], ...) -> AC_INIT([pkg], [new_ver], ...)
  lines[idx[1]] <- sub(
    "^(AC_INIT\\s*\\(\\s*\\[[^]]*\\]\\s*,\\s*\\[)[^]]*",
    paste0("\\1", new_version),
    lines[idx[1]]
  )
  if (!identical(lines, readLines(path, warn = FALSE))) {
    writeLines(lines, path, sep = "\n")
    message("Updated ", path, " -> ", new_version)
  }
  TRUE
}

if (identical(op, "bump")) {
  if (is.null(op_value) || trimws(op_value) == "") {
    stop("Missing bump type. Use --bump=major|minor|patch|dev|dev+")
  }
  new_version <- bump_version(version, op_value)
  if (identical(new_version, version)) {
    message("DESCRIPTION already ", op_value, " version: ", version)
  } else {
    version <- new_version
    update_description_version(desc_path, version)
    message("Updated DESCRIPTION Version -> ", version)
    update_configure_ac_version("configure.ac", version)
  }
} else if (identical(op, "set")) {
  if (is.null(op_value) || trimws(op_value) == "") {
    stop("Missing version. Use --set=x.y.z[.dev]")
  }
  op_value <- trimws(op_value)
  version <- op_value
  update_description_version(desc_path, version)
  message("Updated DESCRIPTION Version -> ", version)
  update_configure_ac_version("configure.ac", version)
}

# Cargo versions cannot include a 4th dot segment; replace the 3rd dot with a dash.
# e.g., 0.1.0.9000 -> 0.1.0-9000
version_cargo <- sub("^((?:[^.]*\\.){2}[^.]*)\\.", "\\1-", version)

cargo_paths <- list.files("src/rust", pattern = "Cargo.toml", recursive = TRUE, full.names = TRUE)
if (length(cargo_paths) == 0) {
  message("No Cargo.toml files found under src/rust.")
  quit(status = 0)
}

update_cargo_version <- function(path, new_version, mode) {
  lines <- readLines(path, warn = FALSE)
  out <- lines
  section <- NULL
  found_package_version <- FALSE
  found_package_version_workspace <- FALSE
  found_workspace_package_version <- FALSE
  seen_package <- FALSE
  seen_workspace_package <- FALSE
  update_package <- mode %in% c("auto", "crate")
  update_workspace <- mode %in% c("auto", "workspace")

  for (i in seq_along(out)) {
    line <- out[i]

    if (grepl("^\\s*\\[package\\]\\s*$", line)) {
      section <- "package"
      seen_package <- TRUE
      next
    }

    if (grepl("^\\s*\\[workspace\\.package\\]\\s*$", line)) {
      section <- "workspace.package"
      seen_workspace_package <- TRUE
      next
    }

    if (grepl("^\\s*\\[.*\\]\\s*$", line) &&
        !grepl("^\\s*\\[package\\]\\s*$", line) &&
        !grepl("^\\s*\\[workspace\\.package\\]\\s*$", line)) {
      section <- NULL
    }

    if (identical(section, "package")) {
      if (grepl("^\\s*version\\.workspace\\s*=\\s*true\\s*(#.*)?$", line)) {
        found_package_version_workspace <- TRUE
      } else if (grepl("^\\s*version\\s*=\\s*['\"].*['\"]\\s*(#.*)?$", line)) {
        if (update_package) {
          out[i] <- sub(
            "^(\\s*version\\s*=\\s*)(['\"])([^'\"]*)(\\2)(\\s*)(#.*)?$",
            paste0("\\1\\2", new_version, "\\2\\5\\6"),
            line,
            perl = TRUE
          )
        }
        found_package_version <- TRUE
      }
    }

    if (identical(section, "workspace.package") &&
        grepl("^\\s*version\\s*=\\s*['\"].*['\"]\\s*(#.*)?$", line)) {
      if (update_workspace) {
        out[i] <- sub(
          "^(\\s*version\\s*=\\s*)(['\"])([^'\"]*)(\\2)(\\s*)(#.*)?$",
          paste0("\\1\\2", new_version, "\\2\\5\\6"),
          line,
          perl = TRUE
        )
      }
      found_workspace_package_version <- TRUE
    }
  }

  if (identical(mode, "workspace") && !seen_workspace_package) {
    message("Skipped (no [workspace.package]): ", path)
    return(TRUE)
  }

  if (identical(mode, "crate") && !seen_package) {
    message("Skipped (no [package]): ", path)
    return(TRUE)
  }

  if (identical(mode, "workspace") && !found_workspace_package_version) {
    warning("No [workspace.package] version found in ", path)
    return(FALSE)
  }

  if (identical(mode, "crate") && !found_package_version && !found_package_version_workspace) {
    warning("No [package] version or version.workspace found in ", path)
    return(FALSE)
  }

  if (identical(mode, "auto") &&
      !found_package_version &&
      !found_package_version_workspace &&
      !found_workspace_package_version) {
    warning("No [package] or [workspace.package] version found in ", path)
    return(FALSE)
  }

  if (!identical(lines, out)) {
    writeLines(out, path, sep = "\n")
    message("Updated version in ", path, " -> ", new_version)
    return(TRUE)
  }

  if (identical(mode, "crate") &&
      found_package_version_workspace &&
      !found_package_version) {
    message("Skipped (version.workspace): ", path)
    return(TRUE)
  }

  if (identical(mode, "auto") &&
      found_package_version_workspace &&
      !found_package_version &&
      !found_workspace_package_version) {
    message("Skipped (version.workspace): ", path)
    return(TRUE)
  }

  message("Already up to date: ", path)
  TRUE
}

for (path in cargo_paths) {
  update_cargo_version(path, version_cargo, mode)
}

# Also sync configure.ac (uses R version format, not Cargo format)
update_configure_ac_version("configure.ac", version)
