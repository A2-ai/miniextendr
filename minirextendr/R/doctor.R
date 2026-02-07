# Diagnostic functions for miniextendr project health

#' Run comprehensive miniextendr project diagnostics
#'
#' Checks the health of a miniextendr project, including toolchain
#' availability, vendored crate status, generated file freshness,
#' and common configuration mistakes.
#'
#' @return Invisibly returns a list with `pass`, `warn`, and `fail` entries.
#' @export
miniextendr_doctor <- function() {
  cli::cli_h1("miniextendr doctor")

  results <- list(pass = character(), warn = character(), fail = character())

  # ── Toolchain checks ──
  cli::cli_h2("Toolchain")

  # Rust
  rustc <- Sys.which("rustc")
  if (nzchar(rustc)) {
    version <- tryCatch(
      system2("rustc", "--version", stdout = TRUE, stderr = TRUE)[1],
      error = function(e) "unknown"
    )
    cli::cli_alert_success("Rust: {version}")
    results$pass <- c(results$pass, "Rust installed")
  } else {
    cli::cli_alert_danger("Rust not found")
    results$fail <- c(results$fail, "Rust not found - install from https://rustup.rs")
  }

  # Cargo
  cargo <- Sys.which("cargo")
  if (nzchar(cargo)) {
    cli::cli_alert_success("cargo available")
    results$pass <- c(results$pass, "cargo available")
  } else {
    cli::cli_alert_danger("cargo not found")
    results$fail <- c(results$fail, "cargo not found")
  }

  # autoconf
  autoconf <- Sys.which("autoconf")
  if (nzchar(autoconf)) {
    cli::cli_alert_success("autoconf available")
    results$pass <- c(results$pass, "autoconf available")
  } else {
    cli::cli_alert_warning("autoconf not found (needed for configure.ac changes)")
    results$warn <- c(results$warn, "autoconf not found")
  }

  # R development headers
  r_home <- R.home()
  r_include <- file.path(r_home, "include", "R.h")
  if (file.exists(r_include)) {
    cli::cli_alert_success("R development headers present")
    results$pass <- c(results$pass, "R headers present")
  } else {
    cli::cli_alert_danger("R development headers not found at {.path {r_include}}")
    results$fail <- c(results$fail, "R development headers missing")
  }

  # ── Vendored crates ──
  cli::cli_h2("Vendored crates")

  vendor_dir <- tryCatch(usethis::proj_path("src", "vendor"), error = function(e) NULL)
  if (is.null(vendor_dir)) {
    cli::cli_alert_info("Not in a project context, skipping vendor checks")
  } else {
    required_crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core", "miniextendr-lint", "miniextendr-engine")
    for (crate in required_crates) {
      crate_path <- file.path(vendor_dir, crate)
      if (dir.exists(crate_path)) {
        cli::cli_alert_success("{crate} vendored")
        results$pass <- c(results$pass, paste(crate, "vendored"))
      } else {
        cli::cli_alert_danger("{crate} not vendored")
        results$fail <- c(results$fail, paste(crate, "not vendored - run vendor_miniextendr()"))
      }
    }
  }

  # ── Generated file freshness ──
  cli::cli_h2("Generated files")

  template_pairs <- list(
    list(template = "src/Makevars.in", generated = "src/Makevars"),
    list(template = "src/entrypoint.c.in", generated = "src/entrypoint.c"),
    list(template = "src/rust/document.rs.in", generated = "src/rust/document.rs")
  )

  for (pair in template_pairs) {
    template_path <- tryCatch(usethis::proj_path(pair$template), error = function(e) NULL)
    generated_path <- tryCatch(usethis::proj_path(pair$generated), error = function(e) NULL)

    if (is.null(template_path)) next

    if (!file.exists(template_path)) next

    if (!file.exists(generated_path)) {
      cli::cli_alert_warning("{.path {pair$generated}} missing (run ./configure)")
      results$warn <- c(results$warn, paste(pair$generated, "missing"))
    } else {
      tmpl_mtime <- file.mtime(template_path)
      gen_mtime <- file.mtime(generated_path)
      if (tmpl_mtime > gen_mtime) {
        cli::cli_alert_warning("{.path {pair$generated}} is stale (template is newer)")
        results$warn <- c(results$warn, paste(pair$generated, "stale"))
      } else {
        cli::cli_alert_success("{.path {pair$generated}} up to date")
        results$pass <- c(results$pass, paste(pair$generated, "fresh"))
      }
    }
  }

  # ── NAMESPACE check ──
  cli::cli_h2("NAMESPACE")

  namespace_path <- tryCatch(usethis::proj_path("NAMESPACE"), error = function(e) NULL)
  if (!is.null(namespace_path) && file.exists(namespace_path)) {
    ns_content <- readLines(namespace_path, warn = FALSE)
    if (any(grepl("useDynLib", ns_content))) {
      cli::cli_alert_success("NAMESPACE contains useDynLib")
      results$pass <- c(results$pass, "useDynLib present")
    } else {
      cli::cli_alert_danger("NAMESPACE missing useDynLib directive")
      results$fail <- c(results$fail, "NAMESPACE missing useDynLib")
    }
  }

  # ── Summary ──
  cli::cli_h2("Summary")
  cli::cli_alert_success("{length(results$pass)} passed")
  if (length(results$warn) > 0) {
    cli::cli_alert_warning("{length(results$warn)} warning(s)")
  }
  if (length(results$fail) > 0) {
    cli::cli_alert_danger("{length(results$fail)} failure(s)")
    for (f in results$fail) {
      cli::cli_bullets(c("x" = f))
    }
  }

  invisible(results)
}
