# webR namespace-import lint (#925, follow-up to #752)
#
# A miniextendr package that eagerly imports a compiled package at the
# namespace level (`importFrom(somePkg, ...)` / `import(somePkg)`) breaks the
# wasm `R CMD INSTALL`: the lazy-load step spawns a *host* (native) R whose
# `.libPaths()` points into the wasm library tree, `loadNamespace(somePkg)`
# makes that host R `dyn.load()` the wasm-built `somePkg.so`, and the install
# dies with `invalid ELF header`. See docs/WEBR.md ("Dependencies and webR")
# in the miniextendr repository.
#
# Detection is purely static -- no `loadNamespace()` (that is the failure we
# are guarding against), no network. An installed dependency is probed via
# its `libs/` directory and `DESCRIPTION` `NeedsCompilation` field; pure-R
# dependencies are walked recursively through their hard (`Depends`/`Imports`)
# dependency graph so pure-R umbrella packages (shiny -> httpuv,
# stringr -> stringi) are still caught. Dependencies that are not installed
# locally fall back to a curated list of well-known compiled packages.

# Base-priority packages are exempt from the lint: the host R resolves them
# from its own R.home("library"), never from the wasm library tree, so their
# (native) .so files load fine. The webR-shipped wasm copies of these are a
# separate problem, solved by the install-to-temp-lib pattern (#491/#744).
webr_base_priority_pkgs <- function() {
  c(
    "base", "compiler", "datasets", "grDevices", "graphics", "grid",
    "methods", "parallel", "splines", "stats", "stats4", "tcltk",
    "tools", "utils"
  )
}

# Curated fallback list of packages known to put compiled code into the
# namespace-load graph. Consulted only when a dependency is not installed
# locally, so the DESCRIPTION/libs probe cannot run. Two flavours:
#   - compiled themselves (`NeedsCompilation: yes` / ship a `libs/` dir),
#   - pure-R umbrellas whose hard Imports are compiled (shiny -> httpuv,
#     stringr -> stringi, httr/httr2 -> curl).
# Note: rlang, cli, glue, fs, and purrr are all `NeedsCompilation: yes`
# (verified 2026-06) despite being widely assumed pure-R.
webr_known_compiled_pkgs <- function() {
  c(
    # compiled themselves
    "arrow", "bit64", "cli", "curl", "data.table", "digest", "dplyr",
    "duckdb", "fs", "glue", "httpuv", "igraph", "jsonlite", "later",
    "lubridate", "magrittr", "odbc", "openssl", "processx", "promises",
    "ps", "purrr", "RMariaDB", "RPostgres", "RSQLite", "Rcpp", "readr",
    "readxl", "rlang", "sf", "stringi", "terra", "tibble", "tidyr",
    "vctrs", "vroom", "xml2", "yaml",
    # pure-R umbrellas over compiled hard Imports
    "httr", "httr2", "shiny", "stringr"
  )
}

# Probe one installed package: TRUE = compiled, FALSE = pure R, NA = cannot
# tell (not installed, or DESCRIPTION unreadable / NeedsCompilation absent).
webr_pkg_compiled_status <- function(pkg, lib_paths = .libPaths()) {
  pkg_dir <- find.package(pkg, lib.loc = lib_paths, quiet = TRUE)
  if (length(pkg_dir) == 0L) {
    return(NA)
  }
  pkg_dir <- pkg_dir[[1L]]
  if (dir.exists(file.path(pkg_dir, "libs"))) {
    return(TRUE)
  }
  desc_path <- file.path(pkg_dir, "DESCRIPTION")
  if (!file.exists(desc_path)) {
    return(NA)
  }
  needs_compilation <- tryCatch(
    unname(read.dcf(desc_path, fields = "NeedsCompilation")[1L, 1L]),
    error = function(e) NA_character_
  )
  if (is.na(needs_compilation)) {
    return(NA)
  }
  identical(tolower(trimws(needs_compilation)), "yes")
}

# Hard (namespace-loading) dependencies of an installed package: the
# `Depends` + `Imports` fields of its DESCRIPTION, version constraints and
# the `R` pseudo-dependency stripped. Returns character(0) when the package
# is not installed or its DESCRIPTION is unreadable.
webr_hard_deps <- function(pkg, lib_paths = .libPaths()) {
  pkg_dir <- find.package(pkg, lib.loc = lib_paths, quiet = TRUE)
  if (length(pkg_dir) == 0L) {
    return(character())
  }
  desc_path <- file.path(pkg_dir[[1L]], "DESCRIPTION")
  if (!file.exists(desc_path)) {
    return(character())
  }
  fields <- tryCatch(
    read.dcf(desc_path, fields = c("Depends", "Imports"))[1L, ],
    error = function(e) NULL
  )
  if (is.null(fields)) {
    return(character())
  }
  deps <- unlist(
    strsplit(fields[!is.na(fields)], ",", fixed = TRUE),
    use.names = FALSE
  )
  deps <- trimws(sub("\\(.*$", "", deps))
  deps <- deps[nzchar(deps) & deps != "R"]
  unique(deps)
}

# Walk the namespace-load graph rooted at `pkg` (breadth-first over hard
# dependencies of locally installed, pure-R nodes). Returns
# list(status =, via =) where status is one of:
#   "compiled"       -- a reachable node is verifiably compiled; via = which
#   "known-compiled" -- nothing verifiably compiled, but an unprobeable node
#                       is on the curated list; via = which
#   "unknown"        -- unprobeable node(s), none on the list; via = which
#   "pure-r"         -- every reachable node probed pure R; via = character()
webr_load_graph_status <- function(pkg, lib_paths = .libPaths()) {
  base_pkgs <- webr_base_priority_pkgs()
  queue <- pkg
  visited <- character()
  compiled_via <- character()
  unknown_via <- character()
  while (length(queue) > 0L) {
    current <- queue[[1L]]
    queue <- queue[-1L]
    if (current %in% visited || current %in% base_pkgs) next
    visited <- c(visited, current)
    status <- webr_pkg_compiled_status(current, lib_paths)
    if (isTRUE(status)) {
      compiled_via <- c(compiled_via, current)
    } else if (is.na(status)) {
      unknown_via <- c(unknown_via, current)
    } else {
      # pure R: its own hard deps still get namespace-loaded -- descend.
      queue <- c(queue, webr_hard_deps(current, lib_paths))
    }
  }
  if (length(compiled_via) > 0L) {
    return(list(status = "compiled", via = compiled_via))
  }
  denied <- intersect(unknown_via, webr_known_compiled_pkgs())
  if (length(denied) > 0L) {
    list(status = "known-compiled", via = denied)
  } else if (length(unknown_via) > 0L) {
    list(status = "unknown", via = unknown_via)
  } else {
    list(status = "pure-r", via = character())
  }
}

# Parse the package's NAMESPACE and classify every namespace-level import.
# Returns a data frame with columns package / directive / status / via
# (via is the comma-joined trail from webr_load_graph_status()).
webr_import_findings <- function(pkg_dir, lib_paths = .libPaths()) {
  empty <- data.frame(
    package = character(), directive = character(),
    status = character(), via = character(),
    stringsAsFactors = FALSE
  )
  if (!file.exists(file.path(pkg_dir, "NAMESPACE"))) {
    return(empty)
  }
  ns <- tryCatch(
    parseNamespaceFile(basename(pkg_dir), dirname(pkg_dir)),
    error = function(e) {
      cli::cli_abort(c(
        "Failed to parse {.path {file.path(pkg_dir, 'NAMESPACE')}}.",
        "x" = conditionMessage(e)
      ))
    }
  )

  pkgs <- character()
  directives <- character()
  add <- function(pkg, directive) {
    pkgs[[length(pkgs) + 1L]] <<- pkg
    directives[[length(directives) + 1L]] <<- directive
  }
  for (entry in ns$imports) {
    if (is.character(entry)) {
      # import(pkg) -- possibly several packages in one directive
      for (pkg in entry) add(pkg, "import")
    } else if ("except" %in% names(entry)) {
      # import(pkg, except = ...) -- list(pkg, except = chr)
      add(entry[[1L]], "import")
    } else {
      # importFrom(pkg, ...) -- list(pkg, chr)
      add(entry[[1L]], "importFrom")
    }
  }
  for (entry in ns$importClasses) add(entry[[1L]], "importClassesFrom")
  for (entry in ns$importMethods) add(entry[[1L]], "importMethodsFrom")

  if (length(pkgs) == 0L) {
    return(empty)
  }
  dup <- duplicated(paste0(pkgs, "\r", directives))
  pkgs <- pkgs[!dup]
  directives <- directives[!dup]
  keep <- !(pkgs %in% webr_base_priority_pkgs())
  pkgs <- pkgs[keep]
  directives <- directives[keep]
  if (length(pkgs) == 0L) {
    return(empty)
  }

  graph_cache <- list()
  status <- character(length(pkgs))
  via <- character(length(pkgs))
  for (i in seq_along(pkgs)) {
    pkg <- pkgs[[i]]
    if (is.null(graph_cache[[pkg]])) {
      graph_cache[[pkg]] <- webr_load_graph_status(pkg, lib_paths)
    }
    status[[i]] <- graph_cache[[pkg]]$status
    via[[i]] <- paste(graph_cache[[pkg]]$via, collapse = ", ")
  }
  data.frame(
    package = pkgs, directive = directives, status = status, via = via,
    stringsAsFactors = FALSE
  )
}

# Print one cli line per finding plus a remediation block when anything is
# flagged. Returns (invisibly) a list(pass =, warn =, fail =) shaped like
# miniextendr_doctor()'s results, so doctor can merge it directly.
webr_report_findings <- function(findings) {
  results <- list(pass = character(), warn = character(), fail = character())
  if (nrow(findings) == 0L) {
    cli::cli_alert_success("No namespace-level imports of non-base packages")
    results$pass <- "no namespace-level imports of non-base packages"
    return(invisible(results))
  }
  for (i in seq_len(nrow(findings))) {
    pkg <- findings$package[[i]]
    directive <- findings$directive[[i]]
    via <- findings$via[[i]]
    switch(findings$status[[i]],
      "compiled" = {
        if (identical(via, pkg)) {
          cli::cli_alert_danger(
            "{.code {directive}({pkg})}: {.pkg {pkg}} is compiled -- the wasm \\
install's lazy-load step will fail with {.code invalid ELF header}"
          )
        } else {
          cli::cli_alert_danger(
            "{.code {directive}({pkg})}: {.pkg {pkg}} pulls compiled \\
{.pkg {via}} into the namespace-load graph -- the wasm install's lazy-load \\
step will fail with {.code invalid ELF header}"
          )
        }
        results$fail <- c(
          results$fail,
          paste0(directive, "(", pkg, ") loads compiled code under webR")
        )
      },
      "known-compiled" = {
        cli::cli_alert_warning(
          "{.code {directive}({pkg})}: {.pkg {via}} is on the known-compiled \\
list (not installed locally, so the {.field NeedsCompilation}/{.path libs/} \\
probe could not run)"
        )
        results$warn <- c(
          results$warn,
          paste0(directive, "(", pkg, ") matches the known-compiled list")
        )
      },
      "unknown" = {
        cli::cli_alert_info(
          "{.code {directive}({pkg})}: could not verify {.pkg {via}} (not \\
installed locally; not on the known-compiled list)"
        )
      },
      "pure-r" = {
        cli::cli_alert_success(
          "{.code {directive}({pkg})}: pure-R namespace-load graph"
        )
        results$pass <- c(
          results$pass,
          paste0(directive, "(", pkg, ") is pure R")
        )
      }
    )
  }
  if (any(findings$status %in% c("compiled", "known-compiled"))) {
    cli::cli_bullets(c(
      "i" = "Move compiled dependencies from {.field Imports} to \\
{.field Suggests} and call them with {.code pkg::fn()} behind a \\
{.code requireNamespace()} guard -- that keeps them out of the \\
namespace-load graph.",
      "i" = "See {.path docs/WEBR.md} (\"Dependencies and webR\") in the \\
miniextendr repository."
    ))
  }
  invisible(results)
}

#' Lint namespace imports for webR compatibility
#'
#' Flags namespace-level `importFrom()` / `import()` (and
#' `importClassesFrom()` / `importMethodsFrom()`) directives whose target
#' package -- or anything in its hard (`Depends`/`Imports`) dependency
#' graph -- ships compiled code. Under webR, the wasm `R CMD INSTALL`
#' lazy-load step spawns a *host* (native) R whose `.libPaths()` points into
#' the wasm library tree; namespace-loading a compiled dependency there makes
#' the host R `dyn.load()` a wasm-built `.so` and the install fails with
#' `invalid ELF header`. The fix is to demote compiled dependencies to
#' `Suggests` and call them via `pkg::fn()` behind `requireNamespace()`.
#'
#' Detection is a local static probe -- no `loadNamespace()` (that is the
#' very failure being guarded against) and no network. An installed
#' dependency counts as compiled if it has a `libs/` directory or
#' `NeedsCompilation: yes` in its `DESCRIPTION`; pure-R dependencies are
#' walked recursively so pure-R umbrella packages over compiled hard imports
#' (e.g. shiny over httpuv) are still caught. Dependencies that are not
#' installed locally fall back to a curated list of well-known compiled
#' packages; anything not on that list is reported as unverifiable, not
#' flagged.
#'
#' Base-priority packages (`stats`, `utils`, `methods`, ...) are exempt: the
#' host R resolves them from its own `R.home("library")`, never from the wasm
#' library tree.
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory.
#' @param lib_paths Library paths to probe for installed dependencies.
#'   Defaults to `.libPaths()`.
#' @return Invisibly, a data frame with columns `package`, `directive`,
#'   `status` (one of `"compiled"`, `"known-compiled"`, `"unknown"`,
#'   `"pure-r"`), and `via` (the compiled or unverifiable packages reached
#'   through the namespace-load graph, comma-joined).
#' @seealso [miniextendr_doctor()] with `webr = TRUE` runs this lint as part
#'   of the full diagnostic.
#' @export
miniextendr_webr_import_lint <- function(path = ".", lib_paths = .libPaths()) {
  with_project(path)
  findings <- webr_import_findings(usethis::proj_get(), lib_paths)
  webr_report_findings(findings)
  invisible(findings)
}
