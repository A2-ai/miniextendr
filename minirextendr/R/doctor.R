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

  # -- Toolchain checks --
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

  # -- Cargo.toml check --
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

  # -- Stale vendor tarball check --
  # inst/vendor.tar.xz is gitignored and only belongs in the source tree
  # transiently (during `just r-cmd-build` / `just r-cmd-check`). If it is
  # left behind after an interrupted build, configure switches to tarball mode
  # and the post-build cleanup in Makevars.in deletes src/rust/.cargo/ from the
  # source tree -- silently breaking the monorepo [patch] override on the next
  # `just rcmdinstall`.
  cli::cli_h2("Install-mode signal")

  vendor_tarball_path <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )
  if (!is.null(vendor_tarball_path) && file.exists(vendor_tarball_path)) {
    cli::cli_alert_danger(
      "{.path inst/vendor.tar.xz} is present in the source tree"
    )
    cli::cli_alert_info(
      "This file is gitignored and should only exist transiently during \\
{.code just r-cmd-build} / {.code just r-cmd-check}. \\
Its presence flips configure to tarball mode, which makes the post-build \\
Makevars cleanup delete {.path src/rust/.cargo/} from the source tree on \\
the next {.code just rcmdinstall}."
    )
    cli::cli_alert_info(
      "Fix: {.code rm inst/vendor.tar.xz && bash ./configure}"
    )
    results$fail <- c(results$fail, "stale inst/vendor.tar.xz in source tree")
  } else {
    cli::cli_alert_success("No stale {.path inst/vendor.tar.xz} in source tree")
    results$pass <- c(results$pass, "no stale vendor.tar.xz")
  }

  cargo_config_path <- tryCatch(
    usethis::proj_path("src", "rust", ".cargo", "config.toml"),
    error = function(e) NULL
  )
  if (!is.null(cargo_config_path) && !file.exists(cargo_config_path)) {
    cli::cli_alert_danger(
      "{.path src/rust/.cargo/config.toml} is missing"
    )
    cli::cli_alert_info(
      "This is generated by {.code bash ./configure}. If {.path inst/vendor.tar.xz} \\
was present, a previous {.code just rcmdinstall} may have deleted it via the \\
tarball-mode cleanup in Makevars. Fix: \\
{.code rm -f inst/vendor.tar.xz && bash ./configure}"
    )
    results$fail <- c(results$fail, "src/rust/.cargo/config.toml missing")
  } else if (!is.null(cargo_config_path)) {
    cli::cli_alert_success("{.path src/rust/.cargo/config.toml} present")
    results$pass <- c(results$pass, "cargo config.toml present")
  }

  # -- Generated file freshness --
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

  # -- NAMESPACE check --
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

  # -- Vendor tarball leak --
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
      "i" = "If you are mid CRAN-prep and have not run {.code R CMD build} yet, this is intentional -- ignore."
    ))
    results$warn <- c(results$warn, "inst/vendor.tar.xz present (may flip tarball mode)")
  } else {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak detected")
    results$pass <- c(results$pass, "No vendor tarball leak")
  }

  # -- Cargo.lock shape --
  cli::cli_h2("Cargo.lock shape")

  lock_path <- tryCatch(usethis::proj_path("src", "rust", "Cargo.lock"), error = function(e) NULL)
  if (!is.null(lock_path) && fs::file_exists(lock_path)) {
    lock_lines <- readLines(lock_path, warn = FALSE)
    n_checksums <- sum(grepl("^checksum = ", lock_lines))
    n_path <- sum(grepl('^source = "path\\+', lock_lines))

    if (n_checksums > 0L || n_path > 0L) {
      cli::cli_alert_warning(
        "{.path src/rust/Cargo.lock} has drifted into source-shape ({n_path} {.code path+} entr{?y/ies}, {n_checksums} {.code checksum =} line{?s})."
      )
      cli::cli_alert_info(
        "Run {.code miniextendr_repair_lock()} to restore tarball-shape."
      )
      results$warn <- c(results$warn, "Cargo.lock drifted into source-shape")
    } else {
      cli::cli_alert_success("{.path src/rust/Cargo.lock} is in tarball-shape")
      results$pass <- c(results$pass, "Cargo.lock in tarball-shape")
    }
  }

  # -- Git Hooks --
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

  # -- Summary --
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
