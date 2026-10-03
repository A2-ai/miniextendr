# Does a retained configure.ac substitute every @VAR@ of the build templates
# an upgrade writes? (#1733)

#' Abort an upgrade whose configure.ac cannot serve the new build templates
#'
#' With `configure_ac = FALSE`, `upgrade_miniextendr_package()` writes the
#' current `src/Makevars.in` and `src/win.def.in` but keeps the package's
#' `configure.ac`. `configure` copies every `@VAR@` the retained file does not
#' substitute into `src/Makevars` unchanged, and the build then fails (the
#' linker reads `@BLAS_LAPACK_PKG_LIBS@` as a response file, for example).
#' The upgrade calls this before it writes anything, so an abort leaves the
#' package untouched.
#'
#' The error names every missing variable, the template `configure.ac` that
#' sets it (with the line of its `AC_SUBST` / `AC_ARG_VAR`), and the two
#' remedies: merge those blocks into `configure.ac`, or rerun with
#' `configure_ac = TRUE`.
#'
#' @param monorepo `TRUE` for the R package of a monorepo, which selects the
#'   templates under `templates/monorepo/rpkg/` over `templates/rpkg/`.
#' @param configure_ac Path to the package's `configure.ac`.
#' @param call The frame the error is reported from.
#' @return `NULL`, invisibly, when every variable is substituted (or there is
#'   no `configure.ac`); otherwise an error.
#' @keywords internal
check_configure_ac_substitutions <- function(monorepo,
                                             configure_ac = usethis::proj_path("configure.ac"),
                                             call = parent.frame()) {
  if (!file.exists(configure_ac)) return(invisible())

  template <- configure_ac_template(monorepo)
  result <- missing_configure_substitutions(configure_ac,
                                            configure_template_inputs(monorepo))
  missing <- result$missing
  if (length(missing) == 0L) return(invisible())

  n_missing <- length(missing)
  used_in <- unique(unlist(result$used_in[missing], use.names = FALSE))
  set_at <- template_subst_lines(template, missing)
  where <- ifelse(is.na(set_at), missing, sprintf("%s (line %d)", missing, set_at))
  # Name every variable: cli shortens vectors longer than 20 by default.
  missing <- cli::cli_vec(missing, list("vec-trunc" = Inf))
  where <- cli::cli_vec(where, list("vec-trunc" = Inf))
  approximate <- if (!result$exact) {
    c("!" = "autoconf was unavailable or failed on {.path configure.ac}, so the variables it substitutes come from a static scan of {.code AC_SUBST()} and {.code AC_ARG_VAR()}: this check is approximate.")
  }

  cli::cli_abort(c(
    "{.path configure.ac} does not substitute {.code {missing}}, which the current {.path {used_in}} template{?s} use{?s/}.",
    "x" = "{.code configure} would leave the {cli::qty(n_missing)}placeholder{?s} in place, and the package would fail to build.",
    "i" = "The template {.path {template}} sets {cli::qty(n_missing)}{?it/them}: {where}.",
    "i" = "Merge those blocks into {.path configure.ac}, or rerun with {.code configure_ac = TRUE} to replace {.path configure.ac} with the template.",
    "i" = "Nothing was written.",
    approximate
  ), call = call)
}

#' Template files that `configure` processes, by package path
#'
#' The `AC_CONFIG_FILES` inputs of the template `configure.ac`, mapped to the
#' bundled template the upgrade writes there (the same mapping as
#' `use_miniextendr_makevars()`).
#'
#' @param monorepo `TRUE` for the monorepo template set, `FALSE` for the
#'   standalone one.
#' @return Named character vector: names are paths in the package
#'   (`src/Makevars.in`, `src/win.def.in`), values the template files.
#' @keywords internal
configure_template_inputs <- function(monorepo) {
  dir <- dirname(configure_ac_template(monorepo))
  c("src/Makevars.in" = file.path(dir, "Makevars.in"),
    "src/win.def.in" = file.path(dir, "win.def.in"))
}

#' The `@VAR@` placeholders of files `configure` substitutes
#'
#' Uses config.status's own pattern, `@[a-zA-Z_][a-zA-Z_0-9]*@`.
#'
#' @param files Named character vector of files; the names label the result.
#' @return Named list with one character vector of variable names (without
#'   the `@`s) per file.
#' @keywords internal
configure_placeholders <- function(files) {
  lapply(files, function(file) {
    text <- readLines(file, warn = FALSE)
    tokens <- unlist(regmatches(text, gregexpr("@[a-zA-Z_][a-zA-Z_0-9]*@", text)))
    unique(gsub("@", "", tokens, fixed = TRUE))
  })
}

#' Placeholders of `configure` inputs that a configure.ac does not substitute
#'
#' @param configure_ac Path to a `configure.ac`.
#' @param inputs Named character vector of the files its `configure` processes,
#'   as [configure_template_inputs()] returns.
#' @return A list with `missing` (sorted variable names), `used_in` (named
#'   list: for each placeholder, the names of the `inputs` that use it) and
#'   `exact` (`FALSE` when the substituted set came from the static scan).
#' @keywords internal
missing_configure_substitutions <- function(configure_ac, inputs) {
  placeholders <- configure_placeholders(inputs)
  needed <- unique(unlist(placeholders, use.names = FALSE))
  used_in <- lapply(stats::setNames(needed, needed), function(var) {
    names(placeholders)[vapply(placeholders, function(p) var %in% p, logical(1))]
  })
  substituted <- configure_ac_subst_vars(configure_ac)
  list(missing = sort(setdiff(needed, substituted$vars)),
       used_in = used_in,
       exact = substituted$exact)
}

#' Variables a configure.ac substitutes
#'
#' Exact when autoconf is available: the `ac_subst_vars` and `ac_subst_files`
#' lists of the generated script hold every `AC_SUBST` (including those from
#' `AC_ARG_VAR` and macros such as `AC_PROG_CC`) and the `AC_INIT` outputs.
#' Otherwise [static_subst_vars()] approximates them. Both add the names
#' config.status substitutes in every `AC_CONFIG_FILES` output
#' ([config_status_builtin_vars()]).
#'
#' @param configure_ac Path to a `configure.ac`.
#' @return A list with `vars` (character) and `exact` (logical).
#' @keywords internal
configure_ac_subst_vars <- function(configure_ac) {
  vars <- autoconf_subst_vars(configure_ac)
  exact <- !is.null(vars)
  if (!exact) vars <- static_subst_vars(configure_ac)
  list(vars = unique(c(vars, config_status_builtin_vars())), exact = exact)
}

#' Variables a configure.ac substitutes, as autoconf reports them
#'
#' Runs `autoconf` in a temporary directory, with the package directory on the
#' include path so `m4_include()` and `aclocal.m4` resolve as they do there.
#' Nothing is written to the package directory: autom4te's cache lands in the
#' temporary directory, which is removed afterwards.
#'
#' @param configure_ac Path to a `configure.ac`.
#' @return Character vector of variable names, or `NULL` when autoconf is not
#'   installed, fails, or its output has no `ac_subst_vars` list.
#' @keywords internal
autoconf_subst_vars <- function(configure_ac) {
  if (!nzchar(Sys.which("autoconf"))) return(NULL)
  configure_ac <- normalizePath(configure_ac, mustWork = TRUE)
  work <- tempfile("mx-autoconf-")
  dir.create(work)
  on.exit(unlink(work, recursive = TRUE), add = TRUE)
  script <- file.path(work, "configure")

  # shQuote: system2() goes through a shell.
  out <- run_command("autoconf", shQuote(c("-I", dirname(configure_ac),
                                           "-o", script, configure_ac)),
                     wd = work)
  if (!is.null(attr(out, "status")) || !file.exists(script)) return(NULL)
  parse_ac_subst_vars(readLines(script, warn = FALSE))
}

#' Read the substituted-variable lists of a generated configure script
#'
#' autoconf writes `ac_subst_vars='A\nB\n...'` (and `ac_subst_files='...'`)
#' near the top of `configure`, one name per line.
#'
#' @param lines Lines of a generated `configure` script.
#' @return Character vector of variable names, or `NULL` when the script has
#'   no `ac_subst_vars` list.
#' @keywords internal
parse_ac_subst_vars <- function(lines) {
  read_list <- function(name) {
    prefix <- paste0(name, "='")
    start <- which(startsWith(lines, prefix))
    if (length(start) == 0L) return(NULL)
    start <- start[[1L]]
    first <- lines[[start]]
    # The opening line can close the list itself (`ac_subst_files=''`).
    end <- if (first != prefix && endsWith(first, "'")) {
      start
    } else {
      after <- which(endsWith(lines, "'") & seq_along(lines) > start)
      if (length(after) == 0L) return(NULL)
      after[[1L]]
    }
    body <- paste(lines[start:end], collapse = "\n")
    body <- sub("'$", "", substring(body, nchar(prefix) + 1L))
    vars <- strsplit(trimws(body), "[[:space:]]+")[[1L]]
    vars[nzchar(vars)]
  }
  vars <- read_list("ac_subst_vars")
  if (is.null(vars)) return(NULL)
  c(vars, read_list("ac_subst_files"))
}

#' Variables a configure.ac substitutes, by a static scan
#'
#' The fallback when autoconf is unavailable: the first argument of every
#' `AC_SUBST()`, `AC_SUBST_FILE()` and `AC_ARG_VAR()` outside `dnl` and `#`
#' comment lines, plus what `AC_INIT` and `AC_OUTPUT` always substitute
#' ([autoconf_default_subst_vars()]). It misses variables substituted by other
#' macros (`AC_PROG_CC` and the like) and counts calls inside macros that are
#' never expanded.
#'
#' @param configure_ac Path to a `configure.ac`.
#' @return Character vector of variable names.
#' @keywords internal
static_subst_vars <- function(configure_ac) {
  lines <- readLines(configure_ac, warn = FALSE)
  lines <- lines[!grepl("^[[:space:]]*(dnl([[:space:]]|$)|#)", lines)]
  calls <- unlist(regmatches(lines, gregexpr(
    "AC_(SUBST(_FILE)?|ARG_VAR)\\([[:space:]]*\\[?[[:space:]]*[A-Za-z_][A-Za-z0-9_]*",
    lines
  )))
  unique(c(sub("^.*[[(][[:space:]]*", "", calls), autoconf_default_subst_vars()))
}

#' Variables every autoconf-generated configure substitutes
#'
#' The `AC_INIT` and `AC_OUTPUT` outputs (`PACKAGE_NAME`, `prefix`, `LIBS`,
#' ...), as autoconf 2.7x lists them in `ac_subst_vars` for a `configure.ac`
#' with no `AC_SUBST` of its own. Used by [static_subst_vars()].
#'
#' @return Character vector of variable names.
#' @keywords internal
autoconf_default_subst_vars <- function() {
  c("PACKAGE_NAME", "PACKAGE_TARNAME", "PACKAGE_VERSION", "PACKAGE_STRING",
    "PACKAGE_BUGREPORT", "PACKAGE_URL", "SHELL", "PATH_SEPARATOR",
    "exec_prefix", "prefix", "program_transform_name", "bindir", "sbindir",
    "libexecdir", "datarootdir", "datadir", "sysconfdir", "sharedstatedir",
    "localstatedir", "runstatedir", "includedir", "oldincludedir", "docdir",
    "infodir", "htmldir", "dvidir", "pdfdir", "psdir", "libdir", "localedir",
    "mandir", "DEFS", "ECHO_C", "ECHO_N", "ECHO_T", "LIBS", "build_alias",
    "host_alias", "target_alias", "LIBOBJS", "LTLIBOBJS")
}

#' Variables config.status substitutes in every output file
#'
#' config.status replaces these in each `AC_CONFIG_FILES` output itself; they
#' are not in `ac_subst_vars`.
#'
#' @return Character vector of variable names.
#' @keywords internal
config_status_builtin_vars <- function() {
  c("configure_input", "srcdir", "abs_srcdir", "top_srcdir", "abs_top_srcdir",
    "builddir", "abs_builddir", "top_builddir", "abs_top_builddir",
    "top_build_prefix")
}

#' Where a template configure.ac sets each variable
#'
#' @param template Path to a template `configure.ac`.
#' @param vars Variable names.
#' @return Integer vector parallel to `vars`: the line of the first
#'   `AC_SUBST()` / `AC_ARG_VAR()` naming the variable, `NA` when there is none.
#' @keywords internal
template_subst_lines <- function(template, vars) {
  lines <- readLines(template, warn = FALSE)
  vapply(vars, function(var) {
    pattern <- paste0("AC_(SUBST(_FILE)?|ARG_VAR)\\([[:space:]]*\\[?[[:space:]]*",
                      var, "[^A-Za-z0-9_]")
    hit <- grep(pattern, lines)
    if (length(hit) == 0L) NA_integer_ else hit[[1L]]
  }, integer(1), USE.NAMES = FALSE)
}
