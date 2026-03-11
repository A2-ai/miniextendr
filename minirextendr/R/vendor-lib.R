# Vendor-lib functions for monorepo library vendoring
#
# When an R package depends on a Rust crate from the same monorepo,
# use_vendor_lib() configures the dependency for both dev and CRAN builds:
# - Dev mode: resolved via [patch.crates-io] pointing to the monorepo crate
# - CRAN mode: `cargo package` bundles the crate as a tarball that configure
#   extracts at build time

#' Add a dependency to [dependencies] in Cargo.toml
#'
#' @param crate Crate name
#' @param version Version spec (e.g., "*", "0.1.0")
#' @return Invisibly returns TRUE if modified, FALSE if already present
#' @noRd
add_cargo_dependency <- function(crate, version) {
  cargo_path <- usethis::proj_path("src", "rust", "Cargo.toml")

  if (!fs::file_exists(cargo_path)) {
    cli::cli_abort(c(
      "Cargo.toml not found at {.path {cargo_path}}",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  lines <- readLines(cargo_path, warn = FALSE)
  dep_line <- sprintf('%s = "%s"', crate, version)

  # Check if already present
  if (any(grepl(sprintf("^%s\\s*=", crate), lines))) {
    cli::cli_alert_info("Dependency {.val {crate}} already in Cargo.toml")
    return(invisible(FALSE))
  }

  # Find [dependencies] section, insert before next section
  deps_idx <- grep("^\\[dependencies\\]", lines)
  if (length(deps_idx) == 0) {
    cli::cli_abort("No [dependencies] section found in Cargo.toml")
  }

  next_section <- grep("^\\[", lines)
  next_section <- next_section[next_section > deps_idx[1]]
  insert_at <- if (length(next_section) > 0) next_section[1] - 1 else length(lines)

  lines <- append(lines, dep_line, after = insert_at)
  writeLines(lines, cargo_path)
  cli::cli_alert_success("Added dependency {.val {crate}} to Cargo.toml")
  invisible(TRUE)
}

#' Add a crate to [patch.crates-io] in Cargo.toml
#'
#' Creates the [patch.crates-io] section if it doesn't exist.
#'
#' @param crate Crate name
#' @param dev_path Relative path from R package root to the crate
#' @return Invisibly returns TRUE if modified, FALSE if already present
#' @noRd
add_cargo_patch <- function(crate, dev_path) {
  cargo_path <- usethis::proj_path("src", "rust", "Cargo.toml")
  lines <- readLines(cargo_path, warn = FALSE)

  patch_line <- sprintf('%s = { path = "%s" }', crate, dev_path)

  # Find [patch.crates-io] section
  patch_idx <- grep("^\\[patch\\.crates-io\\]", lines)

  if (length(patch_idx) == 0) {
    # Create [patch.crates-io] section at end
    lines <- c(lines, "", "[patch.crates-io]", patch_line)
  } else {
    # Check if crate already patched
    next_section <- grep("^\\[", lines)
    next_section <- next_section[next_section > patch_idx[1]]
    end_idx <- if (length(next_section) > 0) next_section[1] - 1 else length(lines)

    section_lines <- lines[(patch_idx[1] + 1):end_idx]
    if (any(grepl(sprintf("^%s\\s*=", crate), section_lines))) {
      cli::cli_alert_info("Patch for {.val {crate}} already in [patch.crates-io]")
      return(invisible(FALSE))
    }

    lines <- append(lines, patch_line, after = end_idx)
  }

  writeLines(lines, cargo_path)
  cli::cli_alert_success("Added {.val {crate}} to [patch.crates-io]")
  invisible(TRUE)
}

#' Add vendor-lib blocks to configure.ac
#'
#' Modifies configure.ac to add:
#' 1. VENDOR_LIB variable (after VENDOR_OUT_CARGO)
#' 2. Updated dev-cargo-config block (vendor-lib-only check)
#' 3. vendor-lib AC_CONFIG_COMMANDS block (between cargo-vendor and post-vendor)
#'
#' If anchor points are not found, warns and prints manual instructions.
#'
#' @param crate Crate name
#' @param dev_path Relative path from R package root to the crate
#' @return Invisibly returns TRUE if modified
#' @noRd
add_vendor_lib_to_configure_ac <- function(crate, dev_path) {
  configure_ac <- usethis::proj_path("configure.ac")

  if (!fs::file_exists(configure_ac)) {
    cli::cli_abort("configure.ac not found")
  }

  lines <- readLines(configure_ac, warn = FALSE)

  # Check if vendor-lib already configured for this crate
  if (any(grepl(sprintf("vendor-lib-%s", crate), lines, fixed = TRUE))) {
    cli::cli_alert_info("Vendor-lib for {.val {crate}} already in configure.ac")
    return(invisible(FALSE))
  }

  # --- 1. Insert VENDOR_LIB variable after VENDOR_OUT_CARGO ---
  vendor_out_cargo_idx <- grep("AC_SUBST\\(\\[VENDOR_OUT_CARGO\\]\\)", lines)
  if (length(vendor_out_cargo_idx) == 0) {
    vendor_out_cargo_idx <- grep("AC_SUBST\\(\\[VENDOR_OUT\\]\\)", lines)
  }

  if (length(vendor_out_cargo_idx) > 0) {
    vendor_lib_var <- c(
      sprintf('VENDOR_LIB="$abs_top_srcdir/inst/%s-lib.tar.gz"', crate),
      "AC_SUBST([VENDOR_LIB])"
    )
    lines <- append(lines, vendor_lib_var, after = vendor_out_cargo_idx[1])
    cli::cli_alert_success("Added VENDOR_LIB variable to configure.ac")
  } else {
    cli::cli_warn(c(
      "Could not find AC_SUBST([VENDOR_OUT_CARGO]) anchor in configure.ac",
      "i" = 'Manually add: VENDOR_LIB="$abs_top_srcdir/inst/{crate}-lib.tar.gz"',
      "i" = "and: AC_SUBST([VENDOR_LIB])"
    ))
  }

  # --- 2. Update dev-cargo-config block (add vendor-lib-only check) ---
  # Work with content as a single string for multi-line replacement
  content <- paste(lines, collapse = "\n")

  old_dev_cfg_close <- paste(c(
    "  fi",
    "],",
    '[NOT_CRAN="$NOT_CRAN"])'
  ), collapse = "\n")

  new_dev_cfg_close <- paste(c(
    '  elif test ! -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then',
    "    # Vendor-lib-only: no full vendor, remove cargo config",
    '    if test -f "$RPKG_CFG"; then',
    '      rm "$RPKG_CFG"',
    '      echo "configure: removed cargo config (vendor-lib-only - no source replacement needed)"',
    "    fi",
    "  fi",
    "],",
    '[NOT_CRAN="$NOT_CRAN" abs_rpkg_dir="$abs_top_srcdir"])'
  ), collapse = "\n")

  if (grepl(old_dev_cfg_close, content, fixed = TRUE)) {
    content <- sub(old_dev_cfg_close, new_dev_cfg_close, content, fixed = TRUE)
    lines <- strsplit(content, "\n")[[1]]
    cli::cli_alert_success("Updated dev-cargo-config block in configure.ac")
  } else {
    cli::cli_warn(c(
      "Could not find dev-cargo-config closing pattern in configure.ac",
      "i" = "Manually add vendor-lib-only cargo config removal to dev-cargo-config block"
    ))
  }

  # --- 3. Insert vendor-lib AC_CONFIG_COMMANDS block before post-vendor ---
  post_vendor_idx <- grep("AC_CONFIG_COMMANDS\\(\\[post-vendor\\]", lines)
  if (length(post_vendor_idx) > 0) {
    vendor_lib_block <- c(
      sprintf("dnl vendor-lib: package and extract %s from monorepo", crate),
      sprintf("AC_CONFIG_COMMANDS([vendor-lib-%s],", crate),
      "[",
      sprintf('  _lib_crate="%s"', crate),
      sprintf('  _lib_dev_path="%s"', dev_path),
      '  _lib_tarball="$abs_rpkg_dir/inst/$_lib_crate-lib.tar.gz"',
      "",
      '  if test "$NOT_CRAN" = "true"; then',
      '    # Dev mode: build the crate tarball for future CRAN submission',
      '    _lib_manifest="$abs_rpkg_dir/$_lib_dev_path/Cargo.toml"',
      '    if test -f "$_lib_manifest"; then',
      '      _tmpdir="$(mktemp -d)"',
      '      echo "configure: packaging $_lib_crate for CRAN..."',
      '      (CARGO_TARGET_DIR="$_tmpdir" $CARGO_CMD package \\',
      '        --manifest-path "$_lib_manifest" --allow-dirty --no-verify 2>/dev/null)',
      '      _lib_crate_file="$(ls -1t "$_tmpdir/package"/$_lib_crate-*.crate 2>/dev/null | head -1)"',
      '      if test -n "$_lib_crate_file"; then',
      '        mkdir -p "$abs_rpkg_dir/inst"',
      '        cp "$_lib_crate_file" "$_lib_tarball"',
      '        echo "configure: created $(basename "$_lib_tarball")"',
      "      else",
      '        echo "configure: warning: cargo package produced no .crate for $_lib_crate" >&2',
      "      fi",
      '      rm -rf "$_tmpdir"',
      "    else",
      '      echo "configure: dev mode -- $_lib_crate resolved via [patch.crates-io]"',
      "    fi",
      "  else",
      '    # CRAN mode: extract vendored lib crate',
      '    if test -f "$_lib_tarball"; then',
      '      echo "configure: extracting $_lib_crate from $(basename "$_lib_tarball")"',
      '      mkdir -p "$VENDOR_OUT/$_lib_crate"',
      '      (cd "$VENDOR_OUT/$_lib_crate" && tar -xzf "$_lib_tarball" --strip-components=1)',
      "      if test $? -ne 0; then",
      '        echo "configure: error: failed to extract $_lib_tarball" >&2',
      "        exit 1",
      "      fi",
      '      # Rewrite [patch.crates-io] dev path to vendor path',
      "      \"$SED\" \"s|$_lib_dev_path|../../vendor/$_lib_crate|g\" src/rust/Cargo.toml > src/rust/Cargo.toml.tmp && \\",
      "        mv src/rust/Cargo.toml.tmp src/rust/Cargo.toml",
      '      echo "configure: patched [patch.crates-io] for $_lib_crate -> vendor/$_lib_crate"',
      "    else",
      '      echo "configure: warning: $_lib_tarball not found (run configure in dev mode first)" >&2',
      "    fi",
      "  fi",
      "],",
      '[NOT_CRAN="$NOT_CRAN" VENDOR_OUT="$VENDOR_OUT" SED="$SED" CARGO_CMD="$CARGO_CMD" abs_rpkg_dir="$abs_top_srcdir"])',
      ""
    )

    lines <- append(lines, vendor_lib_block, after = post_vendor_idx[1] - 1)
    cli::cli_alert_success("Added vendor-lib block to configure.ac")
  } else {
    cli::cli_warn(c(
      "Could not find AC_CONFIG_COMMANDS([post-vendor]) anchor in configure.ac",
      "i" = "Manually add the vendor-lib AC_CONFIG_COMMANDS block before post-vendor"
    ))
  }

  writeLines(lines, configure_ac)
  invisible(TRUE)
}

#' Add a monorepo library as a vendored dependency
#'
#' Configures your R package to depend on a Rust crate from the same monorepo.
#' In dev mode, the crate is resolved via `[patch.crates-io]` path.
#' For CRAN/offline builds, `cargo package` bundles the crate as a tarball
#' that configure extracts at build time.
#'
#' @param crate Crate name (e.g., "dvs")
#' @param version Version spec for Cargo.toml (e.g., "*" or "0.1.0")
#' @param dev_path Relative path from R package root to the monorepo crate
#'   (e.g., "../../../dvs")
#' @param path Path to the R package root
#' @return Invisibly returns TRUE
#' @export
use_vendor_lib <- function(crate, version = "*", dev_path, path = ".") {
  with_project(path)

  if (!is_miniextendr_package()) {
    cli::cli_abort("Not a miniextendr package")
  }

  # 1. Cargo.toml: add dependency + [patch.crates-io]
  add_cargo_dependency(crate, version)
  add_cargo_patch(crate, dev_path)

  # 2. configure.ac: add VENDOR_LIB + vendor-lib block + update dev-cargo-config
  add_vendor_lib_to_configure_ac(crate, dev_path)

  # 3. .gitignore: add tarball
  usethis::use_git_ignore(sprintf("inst/%s-lib.tar.gz", crate))

  cli::cli_alert_success("Added vendor-lib for {.val {crate}}")
  cli::cli_alert_info("Run {.code miniextendr_autoconf()} to regenerate configure")

  invisible(TRUE)
}
