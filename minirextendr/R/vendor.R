# Vendor management.
#
# After PR #320 the vendoring story is simple: `cargo revendor` is the only
# tool that produces `vendor/` and `inst/vendor.tar.xz`, the artifacts the
# tarball-mode install path consumes. Everything previously here for
# pre-seeding `vendor/miniextendr-{api,macros,lint,engine}` from a github
# download or a local checkout is gone — Cargo.toml uses git-URL deps for
# those crates, so they get vendored alongside every other transitive dep.

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
      "--output", vendor_dir
    ),
    log_prefix = "cargo-revendor",
    wd = usethis::proj_get()
  )

  check_result(result, "cargo revendor")

  # CRAN-trim vendor/ before it is packaged into inst/vendor.tar.xz. The
  # rpkg/.Rbuildignore has matching patterns for .github/, ci/, dotfiles etc.
  # on the source tree, but .Rbuildignore does not filter inside the tarball —
  # this stripping is the only mechanism that cleans tarball contents.
  # Does NOT strip tests/ or benches/ — some crates (e.g. zerocopy) use
  # include_str!("../benches/...") in library source; deleting those dirs
  # breaks compilation. Cargo.toml surgery (dev-deps, [[bench]], [[test]])
  # is skipped for the same reason: cargo-revendor ties TOML surgery to
  # directory deletion (#330).
  strip_vendored_dir(vendor_dir)

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

#' Strip CRAN-unfriendly content from a vendored tree
#'
#' Walks every crate directory under `vendor_path` and removes build
#' artifacts, hidden dotfiles, and other content that would trigger CRAN
#' NOTEs (portable filenames, hidden files, long paths) on the produced
#' tarball.
#'
#' tests/ and benches/ are intentionally NOT stripped: some published
#' crates reference files inside those directories from regular library
#' source via `include_str!("../benches/X")` for documentation (zerocopy
#' is one). Stripping them breaks compilation post-vendor. They cost a
#' few MB across the dep graph; CRAN tolerates them.
#'
#' @noRd
strip_vendored_dir <- function(vendor_path) {
  if (!fs::dir_exists(vendor_path)) return(invisible())

  unwanted_dirs <- c("target", ".git", ".github",
                     "examples", "docs", "ci", ".circleci")

  crate_dirs <- fs::dir_ls(vendor_path, type = "directory")
  for (crate_dir in crate_dirs) {
    for (d in unwanted_dirs) {
      d_path <- fs::path(crate_dir, d)
      if (fs::dir_exists(d_path)) {
        fs::dir_delete(d_path)
      }
    }

    # Remove hidden dotfiles (except .cargo-checksum.json which cargo needs)
    all_files <- fs::dir_ls(crate_dir, all = TRUE, recurse = FALSE)
    dotfiles <- all_files[grepl("^\\.", basename(all_files))]
    dotfiles <- dotfiles[basename(dotfiles) != ".cargo-checksum.json"]
    for (f in dotfiles) {
      if (fs::is_dir(f)) {
        fs::dir_delete(f)
      } else {
        fs::file_delete(f)
      }
    }

    # Clear cargo-checksum.json (content was modified by stripping)
    checksum_file <- fs::path(crate_dir, ".cargo-checksum.json")
    if (fs::file_exists(checksum_file)) {
      writeLines('{"files":{}}', checksum_file)
    }
  }

  invisible()
}
