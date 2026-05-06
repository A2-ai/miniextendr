#!/usr/bin/env Rscript
# tools/lock-shape-check.R
#
# Asserts rpkg/src/rust/Cargo.lock is in tarball-shape when running in tarball
# install mode. Invoked from configure.ac AC_CONFIG_COMMANDS([lock-shape-check]).
#
# In source mode, the lock is allowed to drift — cargo silently rewrites it
# during `cargo build`. The pre-commit hook + lock-shape-check just-recipe
# protect commits; this script only fires in tarball mode where drift is fatal.
#
# Usage: Rscript tools/lock-shape-check.R <mode> <lockfile>
#   mode     : "tarball" | "source"
#   lockfile : path to Cargo.lock

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 2) {
  message("usage: Rscript tools/lock-shape-check.R <mode> <lockfile>")
  quit("no", status = 1)
}
mode     <- args[[1]]   # "tarball" | "source"
lockfile <- args[[2]]   # path to Cargo.lock

if (mode != "tarball") quit("no", status = 0)
if (!file.exists(lockfile)) quit("no", status = 0)

content <- readLines(lockfile, warn = FALSE)

# Check 1: no path+... source entries for framework crates.
# In tarball mode, framework crates must use git+https://github.com/A2-ai/miniextendr#<sha>
# so that cargo's source-replacement can match them against the vendored layout.
path_re <- "^source = \"path\\+"
path_violations <- grep(path_re, content, value = TRUE)
if (length(path_violations) > 0) {
  message("configure: ERROR — Cargo.lock has source = \"path+...\" entries:")
  for (v in path_violations) message("  ", v)
  message("")
  message("This lock is in source-shape, not tarball-shape. Tarball install requires")
  message("  source = \"git+https://github.com/A2-ai/miniextendr#<sha>\"")
  message("for miniextendr-{api,lint,macros} so cargo's source replacement matches")
  message("the vendored layout.")
  message("")
  message("Recovery: run `just vendor` (monorepo) or rebuild the package tarball.")
  quit("no", status = 1)
}

# Check 2: no checksum = lines.
# Vendored crates ship with empty .cargo-checksum.json; cargo 1.95+ refuses to
# verify registry checksums against vendored sources.
# NOTE: this rule will be re-evaluated when item 2 of
# plans/lockfile-mode-unification.md lands (cargo-revendor checksum recompute).
sum_re <- "^checksum = "
sum_count <- length(grep(sum_re, content))
if (sum_count > 0) {
  message(sprintf("configure: ERROR — Cargo.lock has %d checksum = line(s).", sum_count))
  message("Vendored crates ship with empty .cargo-checksum.json; cargo offline install")
  message("refuses to verify registry checksums against them.")
  message("")
  message("Recovery: run `just vendor` (monorepo) or rebuild the package tarball.")
  quit("no", status = 1)
}

quit("no", status = 0)
