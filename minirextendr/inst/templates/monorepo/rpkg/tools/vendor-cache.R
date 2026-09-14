#!/usr/bin/env Rscript
# Optional, content-addressed vendor storage for repeated tarball installs.
# This helper uses only packages shipped with R and never rewrites Cargo sources.

prepare_vendor_cache <- function(root, cache) {
  archive <- file.path(root, "inst/vendor.tar.xz")
  digest <- unname(tools::md5sum(archive))
  if (is.na(digest)) stop("Cannot fingerprint ", archive, call. = FALSE)
  dir.create(cache, recursive = TRUE, showWarnings = FALSE)
  cache <- normalizePath(cache, winslash = "/", mustWork = TRUE)
  entry <- file.path(cache, digest)
  vendor <- file.path(entry, "vendor")
  if (!dir.exists(vendor)) {
    stage <- tempfile(".unpack-", tmpdir = cache)
    dir.create(stage)
    on.exit(unlink(stage, recursive = TRUE), add = TRUE)
    status <- utils::untar(archive, exdir = stage)
    if (status != 0L || !dir.exists(file.path(stage, "vendor"))) {
      stop("Failed to unpack ", archive, " into the vendor cache", call. = FALSE)
    }
    # Publish only complete extractions. Concurrent installs can independently
    # unpack; the loser uses the entry already published by the other process.
    if (!suppressWarnings(file.rename(stage, entry)) && !dir.exists(vendor)) {
      stop("Cannot publish vendor cache entry ", entry, call. = FALSE)
    }
  }
  vendor
}

# cargo-revendor --freeze writes a canonical inline [patch.crates-io] table.
# Redirect that generated table in Cargo configuration, leaving the shipped
# relative manifest portable and byte-for-byte untouched during configure.
write_vendor_patches <- function(root, vendor) {
  lines <- readLines(file.path(root, "src/rust/Cargo.toml"), warn = FALSE)
  start <- which(trimws(lines) == "[patch.crates-io]")
  if (!length(start)) return(invisible(NULL))
  rest <- lines[-seq_len(start)]
  next_table <- which(grepl("^[[:space:]]*\\[", rest))
  if (length(next_table)) rest <- rest[seq_len(next_table[[1L]] - 1L)]
  rest <- gsub('"../../vendor/', paste0('"', vendor, '/'), rest, fixed = TRUE)
  cat("[patch.crates-io]\n", paste(rest, collapse = "\n"), "\n", sep = "")
}

if (sys.nframe() == 0L) {
  args <- commandArgs(trailingOnly = TRUE)
  if (length(args) != 3L) stop("Usage: vendor-cache.R prepare|config <root> <cache>")
  if (args[[1L]] == "prepare") {
    cat(prepare_vendor_cache(args[[2L]], args[[3L]]))
  } else if (args[[1L]] == "config") {
    write_vendor_patches(args[[2L]], args[[3L]])
  } else stop("Unknown vendor-cache.R command: ", args[[1L]])
}
