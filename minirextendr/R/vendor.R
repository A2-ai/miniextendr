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
# Trim policy (post PR for #631): all CRAN-relevant trimming lives in
# `cargo-revendor --strip-toml-sections`. No additional R-side deletion
# of vendor/<crate>/ contents — any post-cargo-revendor deletion would
# invalidate the files map in `.cargo-checksum.json` (cargo recomputes
# files map *inside* its own strip pass, then we can't re-touch). The
# vendor.tar.xz is opaque to R CMD check, so the few MB of `examples/`
# / `docs/` saved by extra trimming aren't worth the verification bug.

#' Run cargo revendor and CRAN-trim the result
#'
#' Internal helper called by [miniextendr_vendor()]. Handles the cargo
#' side of vendoring (deferring to `cargo-revendor`) and CRAN-trim of
#' build artifacts / hidden files.
#'
#' Most users want [miniextendr_vendor()] which wraps this with lockfile
#' shape correction and tarball compression.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE on success.
#' @keywords internal
#' @export
vendor_crates_io <- function(path = ".") {
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

  result <- run_with_logging(
    "cargo",
    args = c(
      "revendor",
      "--manifest-path", cargo_toml,
      "--output", vendor_dir,
      # --strip-toml-sections strips [[test]] / [[bench]] / [[example]] /
      # [[bin]] / [dev-dependencies] from each vendored Cargo.toml and
      # prunes dangling [features] refs (see #330, #322), but leaves
      # tests/, benches/, examples/ on disk so crates that
      # include_str!() into those dirs (e.g. zerocopy) keep building.
      "--strip-toml-sections"
    ),
    log_prefix = "cargo-revendor",
    wd = usethis::proj_get()
  )

  check_result(result, "cargo revendor")

  # cargo-revendor's --strip-toml-sections (above) handles all the
  # CRAN-relevant trims: stripping `[[test]]` / `[[bench]]` / `[[example]]`
  # / `[[bin]]` / `[dev-dependencies]` from each vendored Cargo.toml,
  # pruning dangling `[features]` refs, removing always-safe base dirs
  # (`.github/`, `.circleci/`, `ci/`, `target/`), and — crucially —
  # recomputing each crate's `.cargo-checksum.json` so cargo's offline
  # source-replacement verification still succeeds.
  #
  # The vendor.tar.xz is opaque to R CMD check (it doesn't recurse into
  # nested tarballs), so any additional R-side stripping of `examples/`,
  # `docs/`, or hidden dotfiles after cargo-revendor would only shave a
  # couple of MB at the cost of invalidating the checksum files map —
  # see #631 for the failure mode this used to produce.

  cli::cli_alert_success("Vendored to {.path {vendor_dir}}")
  invisible(TRUE)
}

#' Verify `cargo revendor` is installed
#'
#' Errors with install instructions if the `cargo-revendor` subcommand is
#' missing. Called by [miniextendr_vendor()].
#'
#' @noRd
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

