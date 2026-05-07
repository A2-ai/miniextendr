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
# Tarball-shape (post-#408):
#   - no `source = "path+..."` for framework crates (must be `git+url#<sha>`)
#   - no `[[patch.unused]]` blocks
#   - `checksum = "..."` lines ARE allowed (cargo-revendor recomputes valid
#     .cargo-checksum.json that matches them)
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

# Check 2: no `[[patch.unused]]` blocks.
# These appear when `[patch.crates-io]` in `.cargo/config.toml` references a
# crate the lock doesn't depend on. Not a tarball-shape violation per se, but
# spurious commit-time diff and a sign that the patch override is wider than
# the manifest needs.
unused_re <- "^\\[\\[patch.unused\\]\\]"
unused_count <- length(grep(unused_re, content))
if (unused_count > 0) {
  message(sprintf("configure: ERROR — Cargo.lock has %d [[patch.unused]] block(s).", unused_count))
  message("These come from a [patch.crates-io] entry that the manifest doesn't actually use.")
  message("")
  message("Recovery: narrow the [patch.crates-io] block in .cargo/config.toml,")
  message("then run `just vendor` (monorepo) or rebuild the package tarball.")
  quit("no", status = 1)
}

# checksum = "..." lines are ALLOWED (cargo-revendor recomputes valid
# .cargo-checksum.json post-trim, see PR #408).

quit("no", status = 0)
