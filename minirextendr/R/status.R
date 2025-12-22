# Status and introspection functions

#' Check if current project has miniextendr setup
#'
#' @return TRUE if the project appears to be a miniextendr package
#' @export
has_miniextendr <- function() {
  is_miniextendr_package()
}

#' Show miniextendr setup status
#'
#' Displays which miniextendr files are present and which are missing
#' in the current project.
#'
#' @return Invisibly returns a list with present and missing files
#' @export
miniextendr_status <- function() {
  cli::cli_h1("miniextendr status")

  # Define expected files
  expected <- list(
    "Build System" = c(
      "configure.ac",
      "configure",
      "bootstrap.R",
      "cleanup",
      "cleanup.win",
      "cleanup.ucrt",
      "configure.win",
      "configure.ucrt",
      "config.guess",
      "config.sub"
    ),
    "Rust Project" = c(
      "src/rust/Cargo.toml.in",
      "src/rust/lib.rs",
      "src/rust/build.rs",
      "src/rust/cargo-config.toml.in",
      "src/rust/document.rs.in"
    ),
    "Source Templates" = c(
      "src/Makevars.in",
      "src/entrypoint.c.in"
    ),
    "Vendored Crates" = c(
      "src/vendor/miniextendr-api",
      "src/vendor/miniextendr-macros",
      "src/vendor/miniextendr-lint"
    ),
    "Generated Files" = c(
      "src/Makevars",
      "src/rust/Cargo.toml",
      "src/entrypoint.c",
      "R/miniextendr_wrappers.R"
    )
  )

  present <- list()
  missing <- list()

  for (category in names(expected)) {
    cli::cli_h2(category)
    cat_present <- character()
    cat_missing <- character()

    for (file in expected[[category]]) {
      path <- usethis::proj_path(file)
      exists <- fs::file_exists(path) || fs::dir_exists(path)

      if (exists) {
        cli::cli_alert_success("{.path {file}}")
        cat_present <- c(cat_present, file)
      } else {
        cli::cli_alert_warning("{.path {file}} - missing")
        cat_missing <- c(cat_missing, file)
      }
    }

    present[[category]] <- cat_present
    missing[[category]] <- cat_missing
  }

  # Summary
  total_present <- sum(lengths(present))
  total_missing <- sum(lengths(missing))
  total <- total_present + total_missing

  cli::cli_h2("Summary")
  cli::cli_alert_info("{total_present}/{total} files present")

  if (total_missing > 0) {
    cli::cli_alert_warning("{total_missing} files missing")
  }

  invisible(list(present = present, missing = missing))
}

#' Validate miniextendr configuration
#'
#' Checks that the miniextendr setup is valid and ready to build.
#'
#' @return Invisibly returns TRUE if valid, otherwise shows warnings/errors
#' @export
miniextendr_check <- function() {
  cli::cli_h1("miniextendr check")

  issues <- character()
  warnings <- character()

  # Check DESCRIPTION
  cli::cli_h2("DESCRIPTION")
  desc_path <- usethis::proj_path("DESCRIPTION")
  if (!fs::file_exists(desc_path)) {
    issues <- c(issues, "DESCRIPTION not found")
    cli::cli_alert_danger("DESCRIPTION not found")
  } else {
    d <- desc::desc(desc_path)

    # Check Config fields
    bootstrap <- d$get_field("Config/build/bootstrap", default = "")
    if (bootstrap != "TRUE") {
      warnings <- c(warnings, "Config/build/bootstrap should be TRUE")
      cli::cli_alert_warning("Config/build/bootstrap not set to TRUE")
    } else {
      cli::cli_alert_success("Config/build/bootstrap = TRUE")
    }

    # Check SystemRequirements
    sys_req <- d$get_field("SystemRequirements", default = "")
    if (!grepl("Rust", sys_req, ignore.case = TRUE)) {
      warnings <- c(warnings, "SystemRequirements should mention Rust")
      cli::cli_alert_warning("SystemRequirements doesn't mention Rust")
    } else {
      cli::cli_alert_success("SystemRequirements mentions Rust")
    }
  }

  # Check configure.ac
  cli::cli_h2("configure.ac")
  configure_ac <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac)) {
    issues <- c(issues, "configure.ac not found")
    cli::cli_alert_danger("configure.ac not found")
  } else {
    contents <- readLines(configure_ac, warn = FALSE)
    pkg_name <- get_package_name()

    # Check AC_INIT
    if (!any(grepl(paste0("AC_INIT.*", pkg_name), contents))) {
      warnings <- c(warnings, "AC_INIT doesn't match package name")
      cli::cli_alert_warning("AC_INIT package name may not match DESCRIPTION")
    } else {
      cli::cli_alert_success("AC_INIT package name matches")
    }
  }

  # Check Rust toolchain
  cli::cli_h2("Rust toolchain")
  tryCatch(
    {
      check_rust()
      rustc_version <- system2("rustc", "--version", stdout = TRUE)
      cli::cli_alert_success("Rust installed: {rustc_version}")
    },
    error = function(e) {
      issues <- c(issues, "Rust not found")
      cli::cli_alert_danger("Rust not found")
    }
  )

  # Check vendored crates
  cli::cli_h2("Vendored crates")
  vendor_path <- usethis::proj_path("src", "vendor", "miniextendr-api")
  if (!fs::dir_exists(vendor_path)) {
    warnings <- c(warnings, "miniextendr crates not vendored")
    cli::cli_alert_warning("miniextendr crates not vendored - run use_miniextendr_vendor()")
  } else {
    cli::cli_alert_success("miniextendr crates vendored")
  }

  # Summary
  cli::cli_h2("Result")
  if (length(issues) > 0) {
    cli::cli_alert_danger("{length(issues)} issue(s) found")
    for (issue in issues) {
      cli::cli_bullets(c("x" = issue))
    }
    invisible(FALSE)
  } else if (length(warnings) > 0) {
    cli::cli_alert_warning("{length(warnings)} warning(s)")
    cli::cli_alert_success("Package should build, but check warnings above")
    invisible(TRUE)
  } else {
    cli::cli_alert_success("All checks passed!")
    invisible(TRUE)
  }
}
