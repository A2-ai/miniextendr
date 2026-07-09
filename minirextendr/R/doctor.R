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
#' @param webr Also lint namespace-level imports for webR compatibility (see
#'   [miniextendr_webr_import_lint()]). Off by default so packages that never
#'   target webR see no noise.
#' @return Invisibly returns a list with `pass`, `warn`, and `fail` entries.
#' @export
miniextendr_doctor <- function(path = ".", webr = FALSE) {
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

    # Check for relative path = ... entries in [dependencies] only.
    # [patch.crates-io] relative paths are written by use_vendor_lib() and are
    # intentional -- do NOT flag them.
    rel_path_deps <- parse_relative_path_deps(cargo_contents)
    if (length(rel_path_deps) > 0L) {
      for (dep_info in rel_path_deps) {
        cli::cli_alert_warning(
          paste0(
            "{.path src/rust/Cargo.toml} {.code [dependencies]} entry ",
            "{.val {dep_info$crate}} uses a relative {.code path = {dep_info$path}}. ",
            "This will break under {.code R CMD INSTALL} (path resolves against the ",
            "temp build dir, not your package root). Use an absolute path or manage ",
            "it via {.code use_vendor_lib()}."
          )
        )
        results$warn <- c(
          results$warn,
          paste0("relative path dep in [dependencies]: ", dep_info$crate)
        )
      }
    } else {
      results$pass <- c(results$pass, "no relative path deps in [dependencies]")
    }
  }

  # inst/vendor.tar.xz is gitignored and only belongs in the source tree
  # transiently (during `miniextendr_vendor()` + `R CMD build`). Its presence
  # is resolved once here (the #908 marker check below needs it); the actual
  # pass/warn/fail report happens in the single "Vendor tarball" check below
  # (#BUG6 -- this used to be reported twice, with contradictory severity).
  vendor_tarball_path <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )

  # -- Local miniextendr override marker (#908) --
  cli::cli_h2("Local override marker")
  # .miniextendr-local is a dev-only marker written by use_local_miniextendr().
  # It is gitignored and Rbuildignored; warn loudly if still present so the
  # developer doesn't accidentally ship a package that requires a local path.
  local_marker_path <- tryCatch(
    usethis::proj_path(".miniextendr-local"),
    error = function(e) NULL
  )
  if (!is.null(local_marker_path) && file.exists(local_marker_path)) {
    local_mx_path <- tryCatch(
      trimws(readLines(local_marker_path, n = 1L, warn = FALSE)),
      error = function(e) ""
    )
    tarball_present <- !is.null(vendor_tarball_path) && file.exists(vendor_tarball_path)
    if (tarball_present) {
      # Marker is inert (tarball mode wins), but still stale -- inform rather than warn.
      cli::cli_alert_info(
        "{.path .miniextendr-local} is present but inert: tarball mode wins over the \\
local override. Remove the marker before distributing: \\
{.code unuse_local_miniextendr()}."
      )
      results$warn <- c(results$warn, ".miniextendr-local present (inert: tarball mode wins)")
    } else {
      cli::cli_alert_warning(
        "Local miniextendr override is active ({.path .miniextendr-local} = \\
{.path {local_mx_path}}). Run {.code unuse_local_miniextendr()} before \\
vendoring or distributing this package."
      )
      results$warn <- c(results$warn, ".miniextendr-local override active \u2014 run unuse_local_miniextendr() before distributing")
    }
  } else if (!is.null(local_marker_path)) {
    cli::cli_alert_success("No {.path .miniextendr-local} override marker")
    results$pass <- c(results$pass, "no .miniextendr-local override")
  }

  # -- Hand-rolled [patch."https://github.com/A2-ai/miniextendr"] in Cargo.toml (#823 workaround) --
  # Warn if the user manually added this block to src/rust/Cargo.toml; the
  # supported path is use_local_miniextendr() which writes .miniextendr-local
  # and lets configure.ac manage the patch block in .cargo/config.toml.
  if (!is.null(cargo_toml_path) && file.exists(cargo_toml_path)) {
    cargo_lines_for_patch <- readLines(cargo_toml_path, warn = FALSE)
    if (any(grepl('patch.*github.com.*A2-ai.*miniextendr', cargo_lines_for_patch))) {
      cli::cli_alert_warning(
        "{.path src/rust/Cargo.toml} contains a hand-rolled \\
{.code [patch.\"https://github.com/A2-ai/miniextendr\"]} block. \\
This is the manual workaround from #823; use \\
{.code use_local_miniextendr()} instead so configure.ac manages the \\
patch block in {.path src/rust/.cargo/config.toml}."
      )
      results$warn <- c(results$warn, "hand-rolled [patch] block in src/rust/Cargo.toml \u2014 use use_local_miniextendr()")
    }
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
was present, a previous tarball-mode build may have deleted it via the \\
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
  # inst/vendor.tar.xz is gitignored and only belongs in the source tree
  # transiently (during `miniextendr_vendor()` + `R CMD build`). This is the
  # single check for it (#BUG6 -- previously duplicated as both a hard fail
  # and a separate warning, with contradictory guidance). Severity is gated
  # on whether we are in a developer source tree (a `.git` ancestor present):
  #   - dev source tree: this is the latch leak (CLAUDE.md "The latch leak
  #     (#441)") -- a previous build's trap-clean was bypassed, and if left
  #     behind it makes the post-build Makevars cleanup delete
  #     src/rust/.cargo/ from the source tree on the next
  #     `miniextendr_build()`. Fail loudly.
  #   - no `.git` ancestor: may be intentional CRAN-prep staging
  #     (bootstrap.R runs ./configure in a build-staging dir with no .git) or
  #     a legitimate offline install, but can also mean configure was run
  #     before `git init` and accidentally latched into tarball mode. Warn
  #     instead of failing.
  cli::cli_h2("Vendor tarball")

  is_dev_source_tree <- !is.null(find_root_with_file(".git", usethis::proj_get()))

  if (!is.null(vendor_tarball_path) && fs::file_exists(vendor_tarball_path)) {
    if (is_dev_source_tree) {
      cli::cli_alert_danger("{.path inst/vendor.tar.xz} is present in the source tree.")
      cli::cli_bullets(c(
        "i" = paste0(
          "This flips {.code ./configure} into offline tarball mode and hides ",
          "workspace edits. If left behind after an interrupted build, the ",
          "post-build Makevars cleanup also deletes {.path src/rust/.cargo/}, ",
          "breaking the monorepo [patch] override on the next ",
          "{.code miniextendr_build()}."
        ),
        "i" = "Run {.code miniextendr_clean_vendor_leak()} to remove it."
      ))
      results$fail <- c(results$fail, "stale inst/vendor.tar.xz in source tree")
    } else {
      cli::cli_alert_warning("{.path inst/vendor.tar.xz} is present.")
      cli::cli_bullets(c(
        "i" = paste0(
          "No {.code .git} ancestor found. If this is a dev tree, configure ",
          "may have run before {.code git init} and accidentally latched into ",
          "tarball mode."
        ),
        "i" = paste0(
          "Tarball mode skips wrapper regeneration, so new ",
          "{.code #[miniextendr]} functions never become callable."
        ),
        "i" = paste0(
          "If you are staging for CRAN, this is expected. Otherwise run ",
          "{.code git init}, then {.code miniextendr_clean_vendor_leak()}."
        )
      ))
      results$warn <- c(results$warn, "inst/vendor.tar.xz present (may flip tarball mode)")
    }
  } else {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak detected")
    results$pass <- c(results$pass, "No vendor tarball leak")
  }

  # -- Cargo.lock shape --
  cli::cli_h2("Cargo.lock shape")

  lock_path <- tryCatch(usethis::proj_path("src", "rust", "Cargo.lock"), error = function(e) NULL)
  if (!is.null(lock_path) && fs::file_exists(lock_path)) {
    lock_lines <- readLines(lock_path, warn = FALSE)
    # Only `path+` source entries count as drift. `checksum = "..."` lines are
    # canonical post-#408 (cargo-revendor writes valid .cargo-checksum.json), so
    # they are allowed in tarball-shape and must not be flagged here.
    n_path <- sum(grepl('^source = "path\\+', lock_lines))

    if (n_path > 0L) {
      cli::cli_alert_warning(
        "{.path src/rust/Cargo.lock} has drifted into source-shape ({n_path} {.code path+} source entr{?y/ies})."
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

  # -- webR namespace imports (opt-in) --
  if (isTRUE(webr)) {
    cli::cli_h2("webR namespace imports")
    webr_results <- webr_report_findings(
      webr_import_findings(usethis::proj_get())
    )
    results$pass <- c(results$pass, webr_results$pass)
    results$warn <- c(results$warn, webr_results$warn)
    results$fail <- c(results$fail, webr_results$fail)
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

# Parse src/rust/Cargo.toml lines and return a list of list(crate, path) for
# each [dependencies] entry that has a relative path = "..." value.
#
# Rules:
#   - Only entries inside a [dependencies] section are checked.
#   - [patch.crates-io] and all other sections are ignored.
#   - A path is relative when it does NOT start with "/" (or on Windows "X:/").
#   - Both bare `crate = { path = "..." }` inline tables and multi-line
#     dependency blocks preceded by `[dependencies.crate]` are handled.
#
# @param lines Character vector: the raw lines of Cargo.toml.
# @return A list of named lists with elements `crate` and `path`.
parse_relative_path_deps <- function(lines) {
  results <- list()

  # Track which TOML section we are in.
  # section: "deps" for [dependencies] / [dependencies.*], anything else ignored.
  section <- "other"
  current_crate <- NULL  # set when we enter [dependencies.crate_name]

  for (line in lines) {
    stripped <- trimws(line)

    # Skip blank lines and comments.
    if (nchar(stripped) == 0L || startsWith(stripped, "#")) {
      # A blank line between a multi-line dep block and the next entry doesn't
      # reset current_crate; only a new section header does.
      next
    }

    # Section header?
    if (startsWith(stripped, "[")) {
      # Match e.g. [dependencies], [dependencies.foo], [dev-dependencies], etc.
      if (grepl("^\\[dependencies\\]", stripped)) {
        section <- "deps"
        current_crate <- NULL
      } else if (grepl("^\\[dependencies\\.", stripped)) {
        section <- "deps"
        # Extract crate name from [dependencies.crate_name]
        current_crate <- sub("^\\[dependencies\\.([^]]+)\\].*", "\\1", stripped)
      } else {
        # Any other section (including [patch.crates-io]) -- stop watching.
        section <- "other"
        current_crate <- NULL
      }
      next
    }

    if (section != "deps") next

    # Inline table: `crate_name = { ..., path = "...", ... }`
    # e.g. my-crate = { path = "../my-crate" }
    #      my-crate = { version = "1.0", path = "../my-crate", features = [] }
    inline_match <- regmatches(
      stripped,
      regexpr(
        '^([A-Za-z0-9_-]+)\\s*=\\s*\\{[^}]*path\\s*=\\s*"([^"]*)"',
        stripped,
        perl = TRUE
      )
    )
    if (length(inline_match) > 0L && nchar(inline_match) > 0L) {
      m <- regmatches(
        stripped,
        regexec(
          '^([A-Za-z0-9_-]+)\\s*=\\s*\\{[^}]*path\\s*=\\s*"([^"]*)"',
          stripped,
          perl = TRUE
        )
      )[[1L]]
      crate <- m[[2L]]
      path  <- m[[3L]]
      if (!is_absolute_path(path)) {
        results <- c(results, list(list(crate = crate, path = path)))
      }
      next
    }

    # Multi-line block: we entered via [dependencies.crate_name] and now see
    # a line like `path = "..."`.
    if (!is.null(current_crate)) {
      path_match <- regmatches(
        stripped,
        regexec('^path\\s*=\\s*"([^"]*)"', stripped, perl = TRUE)
      )[[1L]]
      if (length(path_match) >= 2L) {
        path <- path_match[[2L]]
        if (!is_absolute_path(path)) {
          results <- c(results, list(list(crate = current_crate, path = path)))
        }
      }
    }
  }

  results
}

# Returns TRUE when path is absolute (starts with "/" or a Windows drive letter).
is_absolute_path <- function(path) {
  grepl("^(/|[A-Za-z]:[/\\\\])", path)
}
