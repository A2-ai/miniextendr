# use_s7() must leave the package with an .onLoad() that calls
# S7::methods_register(). S7 records methods for other packages' generics
# (base operators such as `[`, base generics, external generics) at build
# time, and only that call registers them in a new session; without it they
# work right after install and fail in every later session.

# Minimal package directory: DESCRIPTION (plus `desc_extra` lines) and the
# given R/ files (name = file name, value = lines).
make_s7_pkg <- function(r_files = list(), desc_extra = character()) {
  pkg <- withr::local_tempdir("use-s7-", .local_envir = parent.frame())
  writeLines(
    c("Package: testpkg", "Title: Test", "Version: 0.1.0", desc_extra),
    file.path(pkg, "DESCRIPTION")
  )
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  writeLines(
    c("[package]", 'name = "testpkg"', "", "[dependencies]", 'miniextendr-api = "*"'),
    file.path(pkg, "src", "rust", "Cargo.toml")
  )
  dir.create(file.path(pkg, "R"))
  for (name in names(r_files)) {
    writeLines(r_files[[name]], file.path(pkg, "R", name))
  }
  pkg
}

flat_messages <- function(expr) {
  gsub("\\s+", " ", paste(testthat::capture_messages(expr), collapse = " "))
}

md5 <- function(pkg, files) unname(tools::md5sum(file.path(pkg, files)))

test_that("use_s7() adds an .onLoad that registers S7 methods, once", {
  pkg <- make_s7_pkg(list("testpkg-package.R" = '"_PACKAGE"'))
  suppressMessages(use_s7(pkg))

  zzz <- file.path(pkg, "R", "zzz.R")
  expect_identical(readLines(zzz), MX_S7_LOAD_HOOK)
  hooks <- s7_load_hooks(pkg)$hooks
  expect_length(hooks, 1L)
  expect_identical(hooks[[1L]], list(file = "R/zzz.R", registers = TRUE))
  env <- new.env()
  sys.source(zzz, envir = env)
  expect_identical(names(formals(env$.onLoad)), c("libname", "pkgname"))

  deps <- mx_desc_get_deps(file.path(pkg, "DESCRIPTION"))
  expect_true("S7" %in% deps$package[deps$type == "Imports"])

  # A second call changes nothing.
  before <- md5(pkg, c("DESCRIPTION", "R/zzz.R"))
  msgs <- flat_messages(use_s7(pkg))
  expect_identical(md5(pkg, c("DESCRIPTION", "R/zzz.R")), before)
  expect_match(msgs, "already registers S7 methods", fixed = TRUE)
})

test_that("use_s7() moves S7 from Suggests to Imports", {
  pkg <- make_s7_pkg(desc_extra = c("Suggests:", "    S7,", "    testthat (>= 3.0.0)"))
  msgs <- flat_messages(use_s7(pkg))
  deps <- mx_desc_get_deps(file.path(pkg, "DESCRIPTION"))
  expect_identical(deps$package[deps$type == "Suggests"], "testthat")
  expect_true("S7" %in% deps$package[deps$type == "Imports"])
  expect_match(msgs, "Moved S7 from Suggests to Imports", fixed = TRUE)

  # The field goes away when S7 was its only entry.
  pkg <- make_s7_pkg(desc_extra = "Suggests: S7")
  suppressMessages(use_s7(pkg))
  expect_true(is.na(mx_desc_get_field("Suggests", file.path(pkg, "DESCRIPTION"))))
})

test_that("use_s7() appends the hook to an R/zzz.R without an .onLoad", {
  attach_hook <- ".onAttach <- function(libname, pkgname) invisible()"
  pkg <- make_s7_pkg(list("zzz.R" = attach_hook))
  suppressMessages(use_s7(pkg))
  expect_identical(readLines(file.path(pkg, "R", "zzz.R")), c(attach_hook, "", MX_S7_LOAD_HOOK))
})

test_that("use_s7() leaves an existing .onLoad alone and names the line to add", {
  pkg <- make_s7_pkg(list(
    "hooks.R" = c(".onLoad <- function(libname, pkgname) {", "  options(testpkg.x = 1)", "}")
  ))
  before <- md5(pkg, "R/hooks.R")
  msgs <- flat_messages(use_s7(pkg))
  expect_identical(md5(pkg, "R/hooks.R"), before)
  expect_false(file.exists(file.path(pkg, "R", "zzz.R")))
  expect_match(msgs, "R/hooks.R", fixed = TRUE)
  expect_match(msgs, "suppressMessages(S7::methods_register())", fixed = TRUE)
})

test_that("use_s7() accepts an .onLoad that already registers, by either S7 name", {
  for (call in c("S7::methods_register()", "S7::S7_on_load()")) {
    pkg <- make_s7_pkg(list("zzz.R" = sprintf(".onLoad <- function(...) %s", call)))
    before <- md5(pkg, "R/zzz.R")
    msgs <- flat_messages(use_s7(pkg))
    expect_identical(md5(pkg, "R/zzz.R"), before)
    expect_match(msgs, "already registers S7 methods", fixed = TRUE)
  }
})

test_that("use_s7() does not count a comment as the call", {
  pkg <- make_s7_pkg(list(
    "zzz.R" = c(".onLoad <- function(libname, pkgname) {", "  # S7::methods_register()", "}")
  ))
  expect_false(s7_load_hooks(pkg)$hooks[[1L]]$registers)
})

test_that("use_s7() reports duplicate .onLoad definitions instead of adding a third", {
  pkg <- make_s7_pkg(list(
    "a.R" = ".onLoad <- function(libname, pkgname) invisible()",
    "b.R" = ".onLoad = function(libname, pkgname) invisible()"
  ))
  msgs <- flat_messages(use_s7(pkg))
  expect_false(file.exists(file.path(pkg, "R", "zzz.R")))
  expect_match(msgs, "defined 2 times", fixed = TRUE)
  expect_match(msgs, "R/a.R", fixed = TRUE)
  expect_match(msgs, "R/b.R", fixed = TRUE)
})

test_that("use_s7() adds zzz.R to an existing Collate field, once", {
  pkg <- make_s7_pkg(
    list("a.R" = "a <- 1", "b.R" = "b <- 2"),
    desc_extra = c("Collate:", "    'b.R'", "    'a.R'")
  )
  suppressMessages(use_s7(pkg))
  collate <- mx_desc_get_field("Collate", file.path(pkg, "DESCRIPTION"))
  expect_identical(strsplit(collate, "\\s+")[[1]], c("'b.R'", "'a.R'", "'zzz.R'"))
  suppressMessages(use_s7(pkg))
  expect_identical(mx_desc_get_field("Collate", file.path(pkg, "DESCRIPTION")), collate)
})

test_that("the generated wrappers are not scanned for .onLoad", {
  pkg <- make_s7_pkg(list(
    "testpkg-wrappers.R" = ".onLoad <- function(libname, pkgname) S7::methods_register()"
  ))
  expect_length(s7_load_hooks(pkg)$hooks, 0L)
})

test_that("the package templates do not depend on S7", {
  for (template in c("templates/rpkg/package.R", "templates/monorepo/rpkg/package.R")) {
    lines <- readLines(system.file(template, package = "minirextendr", mustWork = TRUE))
    expect_false(any(grepl("S7", lines, fixed = TRUE)), info = template)
  }
})

test_that("doctor flags an S7 package whose .onLoad does not register methods", {
  pkg <- make_s7_pkg(desc_extra = "Imports: S7")
  msgs <- flat_messages(result <- miniextendr_doctor(pkg))
  expect_true("S7 methods not registered on load (no .onLoad)" %in% result$warn)
  expect_match(msgs, "minirextendr::use_s7()", fixed = TRUE)

  writeLines(".onLoad <- function(libname, pkgname) invisible()", file.path(pkg, "R", "zzz.R"))
  suppressMessages(result <- miniextendr_doctor(pkg))
  expect_true(
    "S7 methods not registered on load (.onLoad lacks methods_register)" %in% result$warn
  )

  unlink(file.path(pkg, "R", "zzz.R"))
  suppressMessages(use_s7(pkg))
  suppressMessages(result <- miniextendr_doctor(pkg))
  expect_true("S7 methods registered on load" %in% result$pass)
  expect_false(any(grepl("S7", result$warn, fixed = TRUE)))
})

test_that("doctor flags wrappers that use S7 without importing it", {
  pkg <- make_s7_pkg(list(
    "testpkg-wrappers.R" = 'Foo <- S7::new_class("Foo", package = "testpkg")'
  ))
  suppressMessages(result <- miniextendr_doctor(pkg))
  expect_true("S7 used by the wrappers but not in Imports" %in% result$warn)
  expect_true("S7 methods not registered on load (no .onLoad)" %in% result$warn)
})

test_that("doctor says nothing about S7 for a package that does not use it", {
  pkg <- make_s7_pkg(list("testpkg-wrappers.R" = "f <- function() 1"))
  msgs <- flat_messages(result <- miniextendr_doctor(pkg))
  expect_false(any(grepl("S7", c(result$pass, result$warn, result$fail), fixed = TRUE)))
  expect_false(grepl("S7 method registration", msgs, fixed = TRUE))
})

# End to end: a scaffolded package with an S7 method on base `[` keeps it in a
# fresh session. The negative control disables S7::methods_register() in that
# session, which must break the call, so the test shows the hook is what makes
# it work.
test_that("a scaffolded S7 package's operator method works in a fresh session", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("cargo", "autoconf")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  repo <- find_miniextendr_repo()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "s7probe")
  lib <- file.path(root, "library")
  dir.create(lib)
  withr::local_envvar(c(
    R_LIBS = paste(c(lib, .libPaths()), collapse = .Platform$path.sep),
    CARGO_TARGET_DIR = NA, CARGO_PROFILE = "dev", CARGO_FEATURES = "",
    CARGO_BUILD_TARGET = NA, CARGO_TERM_COLOR = "never", VENDOR_OUT = NA,
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA, R_INSTALL_PKG = NA
  ))
  suppressMessages({
    create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE)
    use_local_miniextendr(repo, path = pkg)
    use_s7(path = pkg)
  })
  writeLines(c(
    "use miniextendr_api::miniextendr;",
    "miniextendr_api::miniextendr_init!();",
    "",
    "/// A bag of numbers.",
    "#[derive(miniextendr_api::ExternalPtr)]",
    "pub struct Bag {",
    "    values: Vec<f64>,",
    "}",
    "",
    "/// S7 bag with a `[` method.",
    "#[miniextendr(s7)]",
    "impl Bag {",
    "    /// Create a bag.",
    "    /// @param values Numeric values.",
    "    pub fn new(values: Vec<f64>) -> Self {",
    "        Self { values }",
    "    }",
    "",
    "    /// Values at 1-based positions, as `bag[i]`.",
    "    /// @param i Positions.",
    "    #[miniextendr(s7(generic = \"[\"))]",
    "    pub fn subset(&self, i: Vec<i32>) -> Vec<f64> {",
    "        i.iter()",
    "            .filter_map(|&k| usize::try_from(k).ok()?.checked_sub(1))",
    "            .filter_map(|k| self.values.get(k).copied())",
    "            .collect()",
    "    }",
    "}"
  ), file.path(pkg, "src", "rust", "lib.rs"))
  # miniextendr_build() would write this through roxygen; R CMD INSTALL alone
  # needs it to load the shared library.
  writeLines("useDynLib(s7probe, .registration = TRUE)", file.path(pkg, "NAMESPACE"))

  run <- function(args, name) {
    log <- file.path(root, paste0(name, ".log"))
    # Run outside the repository so its .Rprofile (rv) does not replace
    # .libPaths().
    status <- withr::with_dir(root, system2(args[[1L]], args[-1L], stdout = log, stderr = log))
    output <- paste(readLines(log, warn = FALSE), collapse = "\n")
    list(status = status, output = output)
  }
  install <- run(c(file.path(R.home("bin"), "R"), "CMD", "INSTALL", "-l", shQuote(lib), shQuote(pkg)), "install")
  expect_identical(install$status, 0L, info = install$output)
  skip_if(install$status != 0L, "install failed")

  probe <- function(disable_register) {
    script <- file.path(root, paste0("probe-", disable_register, ".R"))
    writeLines(c(
      if (disable_register) {
        'assignInNamespace("methods_register", function() invisible(), "S7")'
      },
      'ns <- loadNamespace("s7probe")',
      "bag <- ns$Bag(c(10, 20, 30))",
      "res <- tryCatch(bag[c(3L, 1L)], error = function(e) conditionMessage(e))",
      "cat(\"RESULT:\", deparse(res), \"\\n\")"
    ), script)
    run(c(file.path(R.home("bin"), "Rscript"), shQuote(script)), paste0("probe-", disable_register))
  }
  fresh <- probe(FALSE)
  expect_identical(fresh$status, 0L, info = fresh$output)
  expect_match(fresh$output, "RESULT: c(30, 10)", fixed = TRUE)

  control <- probe(TRUE)
  expect_identical(control$status, 0L, info = control$output)
  expect_false(grepl("RESULT: c(30, 10)", control$output, fixed = TRUE))
})
