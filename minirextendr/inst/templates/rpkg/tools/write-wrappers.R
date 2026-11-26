#!/usr/bin/env Rscript
# Generate R/<pkg>-wrappers.R and src/rust/wasm_registry.rs from the shared
# library R just linked. src/Makevars runs this from src/, after the link, as
#
#   Rscript ../tools/write-wrappers.R $(SHLIB) $(IS_TARBALL_INSTALL) $(CARGO_FEATURES_FLAG)
#
# $(SHLIB) is the file name R's make gives the package's shared object
# (<pkg>.so, <pkg>.dll), so it also names the package. Loading it runs
# R_init_<pkg>; MINIEXTENDR_WRAPPER_GEN=1 keeps that to routine registration.
# Both writers are registered routines that walk the package's
# #[distributed_slice] tables, and each replaces its file only when the content
# changed, so an unchanged file keeps its mtime. The wrapper writer also gets
# the package's R/ directory, to check the names reexport!() re-exports against
# the hand-written R files there. Base R only.
#
# Outside a release tarball, the build's Cargo features are recorded in
# tools/cargo-features next to the wrappers they generated (rewritten only when
# they change). The file is not tracked but ships in the tarball, whose
# configure builds with the recorded features unless CARGO_FEATURES is set.
#
# A release tarball (one carrying inst/vendor.tar.xz) ships the wrappers its
# NAMESPACE and man/ were documented from. When this build generates different
# ones, it was compiled with other features or for another target than the
# tarball was built with, or the tarball was built from stale wrappers. The
# install stops here: R's load check fails on a missing export, but a missing
# S3 method only warns, and the package would install with broken dispatch.

args <- commandArgs(trailingOnly = TRUE)
if (!length(args) %in% 2:3) {
  stop("Usage: Rscript ../tools/write-wrappers.R <shlib> <is-tarball-install> ",
       "[<cargo-features-flag>]", call. = FALSE)
}
shlib <- normalizePath(args[[1L]], winslash = "/", mustWork = TRUE)
release_tarball <- identical(args[[2L]], "true")
features <- if (length(args) == 3L) sub("^--features=", "", args[[3L]]) else ""
package <- tools::file_path_sans_ext(basename(shlib))
root <- dirname(dirname(shlib))
wrappers <- file.path(root, "R", paste0(package, "-wrappers.R"))
record <- file.path(root, "tools", "cargo-features")

# Generate into a copy of the shipped file, so a mismatch leaves it untouched.
destination <- wrappers
if (release_tarball && file.exists(wrappers)) {
  destination <- file.path(tempfile("wrappers-"), basename(wrappers))
  dir.create(dirname(destination))
  invisible(file.copy(wrappers, destination))
}

lib <- dyn.load(shlib)
invisible(.Call(getNativeSymbolInfo("miniextendr_write_wrappers", lib),
                c(destination, file.path(root, "R"))))
if (!identical(destination, wrappers) &&
    !identical(readLines(wrappers), readLines(destination))) {
  recorded <- ""
  remedy <- "Install with the features the tarball was built with"
  if (file.exists(record)) {
    lines <- grep("^#", readLines(record), value = TRUE, invert = TRUE)
    recorded <- paste0(" The tarball was built with: ",
                       sub("^$", "none", trimws(paste(lines, collapse = ""))), ".")
    remedy <- paste(remedy, "(with CARGO_FEATURES unset, configure uses them)")
  }
  stop("This build generated different R wrappers than the ", package,
       " tarball ships, and the tarball's NAMESPACE and man/ were documented ",
       "from the shipped ones.\n",
       "Cargo features of this build: ", sub("^$", "none", features), ".",
       recorded, " A different feature set (CARGO_FEATURES, or ",
       "tools/detect-features.R deciding otherwise on this machine) or ",
       "another target compiles other #[miniextendr] items.\n",
       remedy, ", or rebuild the tarball with ",
       "minirextendr::miniextendr_build_tarball().",
       call. = FALSE)
}
if (!release_tarball) {
  # Binary mode, so the file has LF line endings on Windows too: configure
  # reads it with the shell.
  content <- charToRaw(paste0(
    "# Cargo features R/", basename(wrappers), " was generated with, written ",
    "by tools/write-wrappers.R.\n",
    "# A release tarball's configure builds with them unless CARGO_FEATURES ",
    "is set.\n",
    features, "\n"))
  if (!file.exists(record) ||
      !identical(readBin(record, "raw", file.size(record)), content)) {
    writeBin(content, record)
  }
}
invisible(.Call(getNativeSymbolInfo("miniextendr_write_wasm_registry", lib),
                file.path(root, "src", "rust", "wasm_registry.rs")))
dyn.unload(shlib)
