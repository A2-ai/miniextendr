# Vendor management.
#
# After PR #320 the vendoring story is simple: `cargo revendor` is the only
# tool that produces `vendor/` and `inst/vendor.tar.xz`, the artifacts the
# tarball-mode install path consumes. Everything previously here for
# pre-seeding `vendor/miniextendr-{api,macros,lint,engine}` from a github
# download or a local checkout is gone — Cargo.toml uses git-URL deps for
# those crates, so they get vendored alongside every other transitive dep.
#
# Checksum policy (post PR #408): cargo-revendor computes real SHA-256
# checksums into .cargo-checksum.json after CRAN-trim, and retains
# `checksum = "..."` lines in Cargo.lock so cargo can verify vendored
# crates match the registry entries.  Do NOT overwrite .cargo-checksum.json
# to {"files":{}} and do NOT strip checksum lines from Cargo.lock — both
# defeat cargo's verification and diverge from `just vendor` output.
#
# Trim policy (#1714): all CRAN-relevant trimming lives in
# `cargo-revendor --strip-all`. No additional R-side deletion of
# vendor/<crate>/ contents: any post-cargo-revendor deletion would
# invalidate the files map in `.cargo-checksum.json`, which cargo-revendor
# recomputes after its own strip pass (#631).

#' Run cargo revendor and CRAN-trim the result
#'
#' Internal helper called by [miniextendr_vendor()]. Handles the cargo
#' side of vendoring (deferring to `cargo-revendor`) and CRAN-trim of
#' build artifacts / hidden files.
#'
#' Most users want [miniextendr_vendor()], which also compresses the result
#' into `inst/vendor.tar.xz`.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param tarball Path of a `.tar.xz` archive for `cargo-revendor` to write
#'   from the vendored tree, or `NULL` to vendor only. With an archive,
#'   `cargo-revendor` blanks the vendored `.md` files that no Rust source
#'   includes and recomputes their `.cargo-checksum.json` entries before
#'   compressing, and it re-vendors even when its cache says `vendor/` is
#'   current, because a cache hit skips compression.
#' @param revendor_args Character vector of extra arguments appended to the
#'   `cargo revendor` call, for example `c("--compression-level", "9")`.
#'   The default trim is `--strip-all`; any `--strip-*` flag here replaces it,
#'   so `"--strip-toml-sections"` keeps the vendored `tests/`, `benches/` and
#'   `examples/` directories on disk.
#' @return Invisibly returns TRUE on success.
#' @keywords internal
#' @export
vendor_crates_io <- function(path = ".", tarball = NULL, revendor_args = character()) {
  if (!is.character(revendor_args)) {
    cli::cli_abort("{.arg revendor_args} must be a character vector.")
  }
  with_project(path)
  check_rust()
  check_cargo_revendor()

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    cli::cli_abort(c(
      "Cargo.toml not found",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  vendor_dir <- usethis::proj_path("vendor")

  cli::cli_alert("Running cargo revendor...")

  # --strip-all strips [[test]] / [[bench]] / [[example]] / [[bin]] /
  # [dev-dependencies] from each vendored Cargo.toml, prunes the [features]
  # entries that pointed at a removed dev-dependency (#322), and deletes
  # tests/, benches/ and examples/, except a directory the crate's own
  # source reaches through include_str!() / include_bytes!() / include!()
  # (zerocopy, #330) or #[path]. A caller's own --strip-* flag replaces it:
  # cargo-revendor rejects --strip-toml-sections next to the other strip
  # flags.
  strip <- if (any(startsWith(revendor_args, "--strip-"))) character() else "--strip-all"
  args <- c(
    "revendor",
    "--manifest-path", cargo_toml,
    "--output", vendor_dir,
    strip,
    # --freeze rewrites any local path-dependency sibling (a core crate at
    # `path = "../../../my-core"`) to point at vendor/, so the sealed tarball
    # is self-contained — a path dep is NOT source-replaceable, so without
    # this the shipped Cargo.toml would reference a sibling that does not
    # travel in the tarball. It mutates src/rust/Cargo.{toml,lock} in place;
    # the manifest stays frozen so the subsequent `R CMD build` seals it (see
    # miniextendr_vendor()'s closing guidance for restoring source shape).
    # cargo-revendor keeps the pre-freeze bytes in
    # src/rust/.Cargo.toml.prefreeze (#1509), which is what
    # miniextendr_clean_vendor_leak() restores from.
    # Inert for a git-only package with no path sibling to rewrite.
    "--freeze"
  )
  if (!is.null(tarball)) {
    args <- c(args, "--compress", tarball, "--blank-md", "--source-marker", "--force")
  }
  args <- c(args, revendor_args)
  result <- run_with_logging("cargo", args = args, log_prefix = "cargo-revendor",
                             wd = usethis::proj_get())
  check_result(result, "cargo revendor")

  # Besides the trim above, cargo-revendor removes the always-safe base dirs
  # (`.github/`, `.circleci/`, `ci/`, `target/`) and hidden files, then
  # recomputes each crate's `.cargo-checksum.json`, so cargo's offline
  # source-replacement verification still succeeds.

  cli::cli_alert_success("Vendored to {.path {vendor_dir}}")
  invisible(TRUE)
}

#' Verify `cargo revendor` is installed
#'
#' Errors with install instructions if the `cargo-revendor` subcommand is
#' missing. Called by [miniextendr_vendor()].
#'
#' @return `TRUE` invisibly when `cargo revendor --help` succeeds; otherwise
#'   aborts.
#' @keywords internal
check_cargo_revendor <- function() {
  probe <- suppressWarnings(tryCatch(
    system2("cargo", c("revendor", "--help"), stdout = FALSE, stderr = FALSE),
    error = function(e) 127L
  ))
  if (!identical(probe, 0L)) {
    cli::cli_abort(c(
      "{.code cargo revendor} is not installed",
      "i" = "Install from the miniextendr repository:",
      "*" = "{.code cargo install --path cargo-revendor}",
      "i" = "Source: {.url https://github.com/A2-ai/miniextendr/tree/main/cargo-revendor}"
    ))
  }
  invisible(TRUE)
}

