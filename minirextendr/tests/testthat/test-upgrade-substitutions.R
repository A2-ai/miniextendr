# upgrade_miniextendr_package() refuses to write a Makevars.in whose @VAR@s
# the retained configure.ac does not substitute (#1733).

# region: helpers -------------------------------------------------------------

# The template configure.ac of a layout, rendered for `package`.
render_configure_ac <- function(monorepo, package = "substpkg") {
  lines <- readLines(minirextendr:::configure_ac_template(monorepo), warn = FALSE)
  gsub("{{package}}", package, lines, fixed = TRUE)
}

# A package whose configure.ac is the current template without the lines
# that contain `drop`. The rest is the minimum is_miniextendr_package() accepts.
make_subst_pkg <- function(pkg, monorepo = FALSE, drop = NULL, package = "substpkg") {
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE, showWarnings = FALSE)
  writeLines(c(paste0("Package: ", package), "Version: 0.0.1"),
             file.path(pkg, "DESCRIPTION"))
  configure <- render_configure_ac(monorepo, package)
  if (!is.null(drop)) configure <- configure[!grepl(drop, configure, fixed = TRUE)]
  writeLines(configure, file.path(pkg, "configure.ac"))
  writeLines("# Previous scaffold", file.path(pkg, "src", "Makevars.in"))
  writeLines("[package]", file.path(pkg, "src", "rust", "Cargo.toml"))
  normalizePath(pkg)
}

# A standalone package at `root`, or a monorepo with the package in rpkg/.
make_subst_layout <- function(root, monorepo, drop = NULL) {
  if (monorepo) writeLines("[workspace]", file.path(root, "Cargo.toml"))
  make_subst_pkg(if (monorepo) file.path(root, "rpkg") else root, monorepo, drop)
  normalizePath(root)
}

# git with no user hooks, attributes or signing in play.
subst_git <- function(path, ...) {
  args <- c("-C", path,
            "-c", "user.name=Scaffold Test",
            "-c", "user.email=scaffold@example.invalid",
            "-c", "commit.gpgsign=false",
            "-c", paste0("core.hooksPath=", file.path(path, ".empty-hooks")),
            "-c", paste0("core.attributesFile=", file.path(path, ".empty-attributes")),
            ...)
  out <- system2("git", shQuote(args), stdout = TRUE, stderr = TRUE)
  expect_null(attr(out, "status"), info = paste(out, collapse = "\n"))
  invisible(out)
}

# md5 of every file and directory under `dir` outside .git/ (git status may
# refresh the index), so a changed, added or removed path changes the result.
tree_md5 <- function(dir) {
  paths <- list.files(dir, recursive = TRUE, all.files = TRUE, no.. = TRUE,
                      include.dirs = TRUE)
  paths <- sort(paths[paths != ".git" & !startsWith(paths, ".git/")])
  is_dir <- dir.exists(file.path(dir, paths))
  sums <- rep("<directory>", length(paths))
  sums[!is_dir] <- unname(tools::md5sum(file.path(dir, paths[!is_dir])))
  stats::setNames(sums, paths)
}

# The input files of a configure.ac's AC_CONFIG_FILES calls
# (`out:in`, or `out` for `out.in`).
ac_config_files_inputs <- function(configure_ac) {
  text <- paste(readLines(configure_ac, warn = FALSE), collapse = "\n")
  calls <- regmatches(text, gregexpr("AC_CONFIG_FILES\\(\\[[^]]*\\]", text))[[1L]]
  entries <- unlist(strsplit(sub("^AC_CONFIG_FILES\\(\\[", "", sub("\\]$", "", calls)),
                             "[[:space:]]+"))
  entries <- entries[nzchar(entries)]
  ifelse(grepl(":", entries, fixed = TRUE), sub("^[^:]*:", "", entries),
         paste0(entries, ".in"))
}

has_autoconf <- function() nzchar(Sys.which("autoconf"))

no_blas_subst <- "AC_SUBST([BLAS_LAPACK_PKG_LIBS])"

# endregion -------------------------------------------------------------------

# region: template pairs -------------------------------------------------------

test_that("the check reads exactly the AC_CONFIG_FILES inputs of each template configure.ac", {
  for (monorepo in c(FALSE, TRUE)) {
    inputs <- minirextendr:::configure_template_inputs(monorepo)
    expect_setequal(names(inputs),
                    ac_config_files_inputs(minirextendr:::configure_ac_template(monorepo)))
    expect_true(all(file.exists(inputs)), info = paste(inputs, collapse = "\n"))
  }
})

test_that("each template configure.ac substitutes every placeholder of its inputs (autoconf)", {
  skip_if_not(has_autoconf(), "autoconf not available")
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir()
    configure_ac <- file.path(tmp, "configure.ac")
    writeLines(render_configure_ac(monorepo), configure_ac)

    result <- minirextendr:::missing_configure_substitutions(
      configure_ac, minirextendr:::configure_template_inputs(monorepo)
    )

    expect_true(result$exact, info = paste("monorepo:", monorepo))
    expect_identical(result$missing, character(), info = paste("monorepo:", monorepo))
    # The scan saw the placeholders, including the one #1706 added.
    expect_true("BLAS_LAPACK_PKG_LIBS" %in% names(result$used_in))
    expect_identical(result$used_in[["PACKAGE_NAME"]], c("src/Makevars.in", "src/win.def.in"))
  }
})

test_that("each template configure.ac passes the static fallback too", {
  local_mocked_bindings(autoconf_subst_vars = function(...) NULL, .package = "minirextendr")
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir()
    configure_ac <- file.path(tmp, "configure.ac")
    writeLines(render_configure_ac(monorepo), configure_ac)

    result <- minirextendr:::missing_configure_substitutions(
      configure_ac, minirextendr:::configure_template_inputs(monorepo)
    )

    expect_false(result$exact)
    expect_identical(result$missing, character(), info = paste("monorepo:", monorepo))
  }
})

test_that("rpkg's configure.ac substitutes every placeholder of its Makevars.in and win.def.in", {
  skip_if_no_local_repo()
  skip_if_not(has_autoconf(), "autoconf not available")
  rpkg <- file.path(find_miniextendr_repo(), "rpkg")
  configure_ac <- file.path(rpkg, "configure.ac")
  inputs <- c("src/Makevars.in" = file.path(rpkg, "src", "Makevars.in"),
              "src/win.def.in" = file.path(rpkg, "src", "win.def.in"))
  expect_setequal(names(inputs), ac_config_files_inputs(configure_ac))
  before <- list.files(rpkg, all.files = TRUE, no.. = TRUE)

  result <- minirextendr:::missing_configure_substitutions(configure_ac, inputs)

  expect_true(result$exact)
  expect_identical(result$missing, character())
  # autoconf ran elsewhere: no autom4te.cache or configure output in rpkg/.
  expect_identical(list.files(rpkg, all.files = TRUE, no.. = TRUE), before)
})

# endregion -------------------------------------------------------------------

# region: substituted-variable sources -----------------------------------------

test_that("parse_ac_subst_vars() reads the multi-line lists of a generated configure", {
  lines <- c("#! /bin/sh", "ac_subst_vars='LTLIBOBJS", "BLAS_LAPACK_PKG_LIBS",
             "PACKAGE_NAME", "SHELL'", "ac_subst_files=''", "ac_user_opts='")
  expect_identical(minirextendr:::parse_ac_subst_vars(lines),
                   c("LTLIBOBJS", "BLAS_LAPACK_PKG_LIBS", "PACKAGE_NAME", "SHELL"))
  expect_identical(
    minirextendr:::parse_ac_subst_vars(c("ac_subst_vars='A", "B'", "ac_subst_files='F'")),
    c("A", "B", "F")
  )
  expect_null(minirextendr:::parse_ac_subst_vars(c("#! /bin/sh", "exit 0")))
})

test_that("static_subst_vars() reads AC_SUBST and AC_ARG_VAR outside comments", {
  tmp <- withr::local_tempdir()
  configure_ac <- file.path(tmp, "configure.ac")
  writeLines(c(
    "AC_INIT([pkg], [0.1.0])",
    "AC_SUBST([BRACKETED])",
    "AC_SUBST(BARE, [value])",
    "AC_SUBST_FILE([FRAGMENT])",
    "AC_ARG_VAR([USER_VAR], [Documented variable])",
    "  AS_IF([true], [AC_SUBST([NESTED])])",
    "dnl AC_SUBST([IN_DNL])",
    "# AC_SUBST([IN_COMMENT])",
    "AC_OUTPUT"
  ), configure_ac)

  vars <- minirextendr:::static_subst_vars(configure_ac)

  expect_true(all(c("BRACKETED", "BARE", "FRAGMENT", "USER_VAR", "NESTED",
                    "PACKAGE_NAME", "PACKAGE_VERSION", "prefix") %in% vars))
  expect_false(any(c("IN_DNL", "IN_COMMENT") %in% vars))
})

test_that("autoconf_subst_vars() resolves m4_include from the package without writing there", {
  skip_if_not(has_autoconf(), "autoconf not available")
  pkg <- withr::local_tempdir(pattern = "subst include ")
  writeLines("AC_DEFUN([MX_EXTRA], [AC_SUBST([INCLUDED_VAR])])",
             file.path(pkg, "extra.m4"))
  writeLines(c("AC_INIT([pkg], [0.1.0])", "m4_include([extra.m4])", "MX_EXTRA",
               "AC_PROG_CC", "AC_OUTPUT"),
             file.path(pkg, "configure.ac"))
  before <- tree_md5(pkg)

  vars <- minirextendr:::autoconf_subst_vars(file.path(pkg, "configure.ac"))

  # INCLUDED_VAR comes from the included file and CC from AC_PROG_CC: the
  # static scan finds neither.
  expect_true(all(c("INCLUDED_VAR", "CC", "PACKAGE_NAME") %in% vars))
  expect_identical(tree_md5(pkg), before)
})

# endregion -------------------------------------------------------------------

# region: the upgrade ------------------------------------------------------------

test_that("an upgrade with the defaults refuses a configure.ac missing a substitution and changes nothing", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  for (monorepo in c(FALSE, TRUE)) {
    root <- make_subst_layout(withr::local_tempdir(pattern = "subst upgrade "),
                              monorepo, drop = no_blas_subst)
    subst_git(root, "init", "-q")
    subst_git(root, "add", "-A")
    subst_git(root, "commit", "-q", "-m", "scaffold")
    before <- tree_md5(root)

    err <- expect_error(suppressMessages(upgrade_miniextendr_package(root)),
                        class = "rlang_error", info = paste("monorepo:", monorepo))
    msg <- cli::ansi_strip(conditionMessage(err))

    expect_match(msg, "BLAS_LAPACK_PKG_LIBS", fixed = TRUE)
    expect_match(msg, "src/Makevars.in", fixed = TRUE)
    # Points at the template configure.ac of this layout, at the AC_SUBST line.
    template <- minirextendr:::configure_ac_template(monorepo)
    expect_match(gsub("\\s+", "", msg), gsub("\\s+", "", template), fixed = TRUE)
    subst_line <- grep(no_blas_subst, readLines(template), fixed = TRUE)
    expect_match(msg, sprintf("BLAS_LAPACK_PKG_LIBS (line %d)", subst_line), fixed = TRUE)
    # Both remedies, and no claim of an approximate check when autoconf ran.
    expect_match(msg, "Merge those blocks into", fixed = TRUE)
    expect_match(msg, "configure_ac[^A-Za-z]+TRUE")
    expect_match(msg, "Nothing was written", fixed = TRUE)
    if (has_autoconf()) expect_no_match(msg, "approximate")
    # Only the one variable is reported.
    expect_no_match(msg, "CARGO_STATICLIB_NAME")

    expect_identical(tree_md5(root), before, info = paste("monorepo:", monorepo))
  }
})

test_that("without autoconf the refusal says the check is approximate", {
  local_mocked_bindings(autoconf_subst_vars = function(...) NULL, .package = "minirextendr")
  root <- make_subst_layout(withr::local_tempdir(), FALSE, drop = no_blas_subst)
  before <- tree_md5(root)

  err <- expect_error(suppressMessages(
    upgrade_miniextendr_package(root, autoconf = FALSE, allow_dirty = TRUE)
  ), "BLAS_LAPACK_PKG_LIBS")

  expect_match(cli::ansi_strip(conditionMessage(err)), "approximate", fixed = TRUE)
  expect_identical(tree_md5(root), before)
})

test_that("a refusal names every missing variable, however many", {
  # A pre-template configure.ac substitutes none of the Makevars.in variables;
  # cli would shorten a list this long to 20 entries.
  tmp <- withr::local_tempdir()
  configure_ac <- file.path(tmp, "configure.ac")
  writeLines(c("AC_INIT([substpkg], [0.0.1])", 'CARGO_FEATURES=""', "AC_OUTPUT"),
             configure_ac)
  inputs <- minirextendr:::configure_template_inputs(FALSE)
  expected <- minirextendr:::missing_configure_substitutions(configure_ac, inputs)$missing
  expect_gt(length(expected), 20L)

  err <- expect_error(
    minirextendr:::check_configure_ac_substitutions(FALSE, configure_ac = configure_ac),
    class = "rlang_error"
  )
  msg <- cli::ansi_strip(conditionMessage(err))

  for (var in expected) expect_match(msg, var, fixed = TRUE)
  expect_no_match(msg, "…", fixed = TRUE)
  expect_no_match(msg, "PACKAGE_NAME", fixed = TRUE)
})

test_that("configure_ac = TRUE upgrades the package the check refuses", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  for (monorepo in c(FALSE, TRUE)) {
    root <- make_subst_layout(withr::local_tempdir(), monorepo, drop = no_blas_subst)
    pkg <- if (monorepo) file.path(root, "rpkg") else root

    expect_no_error(suppressMessages(upgrade_miniextendr_package(
      root, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
    )))

    expect_true(any(grepl("@BLAS_LAPACK_PKG_LIBS@", readLines(file.path(pkg, "src", "Makevars.in")),
                          fixed = TRUE)))
    expect_true(no_blas_subst %in% trimws(readLines(file.path(pkg, "configure.ac"))))
  }
})

test_that("an upgrade with the defaults proceeds when configure.ac substitutes everything", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  for (monorepo in c(FALSE, TRUE)) {
    root <- make_subst_layout(withr::local_tempdir(), monorepo)
    pkg <- if (monorepo) file.path(root, "rpkg") else root

    expect_no_warning(expect_no_error(suppressMessages(upgrade_miniextendr_package(
      root, autoconf = FALSE, allow_dirty = TRUE
    ))))

    expect_true(any(grepl("@BLAS_LAPACK_PKG_LIBS@", readLines(file.path(pkg, "src", "Makevars.in")),
                          fixed = TRUE)))
  }
})

# endregion -------------------------------------------------------------------
