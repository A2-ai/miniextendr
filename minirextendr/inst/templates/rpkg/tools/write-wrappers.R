#!/usr/bin/env Rscript
# Generate R/<pkg>-wrappers.R and src/rust/wasm_registry.rs from the shared
# library R just linked. src/Makevars runs this from src/, after the link, as
#
#   Rscript ../tools/write-wrappers.R $(SHLIB)
#
# $(SHLIB) is the file name R's make gives the package's shared object
# (<pkg>.so, <pkg>.dll), so it also names the package. Loading it runs
# R_init_<pkg>; MINIEXTENDR_WRAPPER_GEN=1 keeps that to routine registration.
# Both writers are registered routines that walk the package's
# #[distributed_slice] tables, and each replaces its file only when the content
# changed, so an unchanged file keeps its mtime. Base R only.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) {
  stop("Usage: Rscript ../tools/write-wrappers.R <shlib>", call. = FALSE)
}
shlib <- normalizePath(args[[1L]], winslash = "/", mustWork = TRUE)
package <- tools::file_path_sans_ext(basename(shlib))
root <- dirname(dirname(shlib))

lib <- dyn.load(shlib)
.Call(getNativeSymbolInfo("miniextendr_write_wrappers", lib),
      file.path(root, "R", paste0(package, "-wrappers.R")))
.Call(getNativeSymbolInfo("miniextendr_write_wasm_registry", lib),
      file.path(root, "src", "rust", "wasm_registry.rs"))
dyn.unload(shlib)
