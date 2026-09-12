#!/usr/bin/env Rscript
# Content provenance for the #1022 pre-shipped-wrapper fast path (#1512).
# Keep this separate from wrappers.R: regenerating identical R code deliberately
# preserves that file's mtime and source-position comments. No package dependency
# is needed here; tools ships with R. Paths in the record are package-relative,
# so copies and tar extraction do not affect freshness.

wrapper_inputs <- function(root) {
  rust <- file.path(root, "src", "rust")
  paths <- list.files(rust, pattern = "\\.rs$", recursive = TRUE)
  paths <- paths[!grepl("(^|/)(target|ra_target|rust-target|ra-target)/", paths)]
  paths <- setdiff(paths, "wasm_registry.rs") # output of the same generation pass
  paths <- sort(unique(c(file.path("src", "rust", paths),
                         "src/rust/Cargo.toml", "src/rust/Cargo.lock")),
                method = "radix")
  present <- file.exists(file.path(root, paths))
  hashes <- structure(rep(NA_character_, length(paths)), names = paths)
  hashes[present] <- unname(tools::md5sum(file.path(root, paths[present])))
  makevars <- readLines(file.path(root, "src", "Makevars"), warn = FALSE)
  # Profile and features can change which wrappers are compiled. Absolute build
  # paths and the target triple are deliberately absent: tarballs move, and a
  # native wrapper snapshot is also used by the wasm build.
  knobs <- grep("^(CARGO_FEATURES_FLAG|CARGO_PROFILE)[[:space:]]*=", makevars,
                value = TRUE)
  list(version = 1L, sources = hashes, knobs = knobs)
}

wrapper_record_path <- function(root) file.path(root, "tools", "wrapper-inputs.rds")

wrappers_current <- function(root, wrappers) {
  if (!file.exists(wrappers) || file.info(wrappers)$size == 0) return(FALSE)
  record_path <- wrapper_record_path(root)
  if (!file.exists(record_path)) return(FALSE)
  record <- tryCatch(suppressWarnings(readRDS(record_path)), error = function(e) NULL)
  is.list(record) && identical(record$inputs, wrapper_inputs(root)) &&
    identical(record$wrappers, unname(tools::md5sum(wrappers)))
}

write_wrapper_record <- function(root, wrappers) {
  saveRDS(list(inputs = wrapper_inputs(root),
               wrappers = unname(tools::md5sum(wrappers))),
          wrapper_record_path(root), version = 2)
}

# Invoked only after Cargo has built and R has linked this exact shared library.
# A changed tarball wrapper may need new exports, S3 registrations, or docs.
# Stop before R's load check can accept missing S3 methods with only a warning.
generate_wrappers <- function(root, wrappers, shlib, wasm_registry, tarball) {
  old <- if (file.exists(wrappers) && file.info(wrappers)$size > 0) {
    unname(tools::md5sum(wrappers))
  } else NULL
  destination <- wrappers
  if (tarball && !is.null(old)) {
    # Compare in a temporary file. A failed attempt must not replace the stale
    # file and then let a second attempt mistake it for approved documentation.
    destination <- tempfile("miniextendr-wrappers-", fileext = ".R")
    on.exit(unlink(destination), add = TRUE)
    file.copy(wrappers, destination, overwrite = TRUE)
  }
  lib <- dyn.load(shlib)
  on.exit(dyn.unload(shlib), add = TRUE)
  .Call(getNativeSymbolInfo("miniextendr_write_wrappers", lib), destination)
  if (tarball && !is.null(old) &&
      !identical(old, unname(tools::md5sum(destination)))) {
    stop("Stale pre-shipped wrappers: ", wrappers, " changed during regeneration.\n",
         "Regenerate wrappers and documentation in the source package with ",
         "minirextendr::miniextendr_build(), then rebuild the tarball.\n",
         "Installation stopped before loading inconsistent S3 registrations or exports.",
         call. = FALSE)
  }
  .Call(getNativeSymbolInfo("miniextendr_write_wasm_registry", lib), wasm_registry)
  write_wrapper_record(root, wrappers)
  invisible(NULL)
}

if (sys.nframe() == 0L) {
  args <- commandArgs(trailingOnly = TRUE)
  if (length(args) == 3L && args[[1L]] == "check") {
    quit("no", status = if (wrappers_current(args[[2L]], args[[3L]])) 0L else 1L)
  } else if (length(args) == 6L && args[[1L]] == "generate") {
    generate_wrappers(args[[2L]], args[[3L]], args[[4L]], args[[5L]],
                      args[[6L]] == "true")
  } else {
    stop("Usage: wrapper-freshness.R check <root> <wrappers> OR ",
         "generate <root> <wrappers> <shlib> <wasm-registry> <tarball>", call. = FALSE)
  }
}
