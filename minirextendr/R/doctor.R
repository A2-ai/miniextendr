# Diagnostic functions for miniextendr project health

#' Run comprehensive miniextendr project diagnostics
#'
#' The primary diagnostic function for miniextendr projects. Checks the
#' health of a miniextendr project, including toolchain availability,
#' vendored crate status, generated file freshness, and common
#' configuration mistakes.
#'
#' For more targeted checks, see [miniextendr_status()] (file presence)
#' and [miniextendr_validate()] (configuration correctness).
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns a list with `pass`, `warn`, and `fail` entries.
#' @export
miniextendr_doctor <- function(path = ".") {
  with_project(path)
  cli::cli_h1("miniextendr doctor")

  results <- list(pass = character(), warn = character(), fail = character())

  # ── Toolchain checks ──
  cli::cli_h2("Toolchain")

  # Rust
  rustc <- Sys.which("rustc")
  if (nzchar(rustc)) {
    version <- tryCatch(
      run_command("rustc", c("--version"))[1],
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

  # ── Cargo.toml check ──
  cli::cli_h2("Cargo.toml")
  cargo_toml_path <- tryCatch(usethis::proj_path("src", "rust", "Cargo.toml"), error = function(e) NULL)
  if (is.null(cargo_toml_path) || !file.exists(cargo_toml_path)) {
    cli::cli_alert_danger("{.path src/rust/Cargo.toml} not found")
    results$fail <- c(results$fail, "src/rust/Cargo.toml not found")
  } else {
    cargo_contents <- readLines(cargo_toml_path, warn = FALSE)
    if (any(grepl("miniextendr-api", cargo_contents, fixed = TRUE))) {
      cli::cli_alert_success("{.path src/rust/Cargo.toml} declares {.code miniextendr-api}")
      results$pass <- c(results$pass, "Cargo.toml declares miniextendr-api")
    } else {
      cli::cli_alert_danger("{.path src/rust/Cargo.toml} is missing {.code miniextendr-api} dependency")
      results$fail <- c(results$fail, "Cargo.toml missing miniextendr-api")
    }
  }

  # ── Generated file freshness ──
  cli::cli_h2("Generated files")

  template_pairs <- list(
    list(template = "src/Makevars.in", generated = "src/Makevars")
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

  # ── Vendor tarball leak ──
  cli::cli_h2("Vendor tarball")

  tarball_path <- tryCatch(usethis::proj_path("inst", "vendor.tar.xz"), error = function(e) NULL)
  if (!is.null(tarball_path) && fs::file_exists(tarball_path)) {
    cli::cli_alert_warning("{.path inst/vendor.tar.xz} is present.")
    cli::cli_bullets(c(
      "i" = paste0(
        "If you are in dev iteration this flips {.code ./configure} into offline ",
        "tarball mode and hides workspace edits."
      ),
      "i" = "Run {.code miniextendr_clean_vendor_leak()} (or {.code just clean-vendor-leak} in the monorepo) to remove it.",
      "i" = "If you are mid CRAN-prep and have not run {.code R CMD build} yet, this is intentional — ignore."
    ))
    results$warn <- c(results$warn, "inst/vendor.tar.xz present (may flip tarball mode)")
  } else {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak detected")
    results$pass <- c(results$pass, "No vendor tarball leak")
  }

  # ── Git Hooks ──
  cli::cli_h2("Git Hooks")

  hook_status <- has_miniextendr_git_hooks(usethis::proj_get())
  if (all(hook_status)) {
    cli::cli_alert_success("miniextendr git hooks installed (pre-commit, post-merge)")
    results$pass <- c(results$pass, "git hooks installed")
  } else {
    missing <- names(hook_status)[!hook_status]
    cli::cli_alert_warning("Missing miniextendr git hooks: {paste(missing, collapse = ', ')}")
    cli::cli_alert_info("Run {.code minirextendr::use_miniextendr_git_hooks()} to install")
    results$warn <- c(results$warn, "git hooks missing")
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
