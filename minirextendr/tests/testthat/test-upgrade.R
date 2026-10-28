# Tests for upgrade functionality

test_that("upgrade_gitignore removes obsolete entries", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)

  # Create a .gitignore with some current and obsolete entries
  gitignore <- file.path(tmp, ".gitignore")
  writeLines(c(
    "src/rust/target/",
    "src/entrypoint.c",
    "src/mx_abi.c",
    "src/rust/document.rs",
    ".cargo/config.toml",
    "vendor/"
  ), gitignore)

  # Mock the gitignore template lookup to avoid package dependency
  local_mocked_bindings(
    use_miniextendr_gitignore = function(...) invisible(),
    .package = "minirextendr"
  )

  minirextendr:::upgrade_gitignore()

  result <- readLines(gitignore)
  expect_true("src/rust/target/" %in% result)
  expect_true("vendor/" %in% result)
  expect_false("src/entrypoint.c" %in% result)
  expect_false("src/mx_abi.c" %in% result)
  expect_false("src/rust/document.rs" %in% result)
  # Superseded mis-anchored cargo-config pattern is dropped (#1226)
  expect_false(".cargo/config.toml" %in% result)
})

test_that("check_configure_ac_drift detects drift despite all legacy markers", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))

  # All three markers used by the old heuristic are present, but the local
  # override and other current build-system logic are absent.
  outdated <- c(
    "AC_INIT([driftcheck], [0.1.0])",
    "AC_CONFIG_AUX_DIR([tools])",
    "CARGO_STATICLIB_NAME=driftcheck",
    "CARGO_TARGET_DIR=target",
    "AC_OUTPUT"
  )
  configure <- file.path(tmp, "configure.ac")
  writeLines(outdated, configure)

  expect_warning(
    minirextendr:::check_configure_ac_drift(),
    "configure.ac differs from the current template"
  )
  expect_identical(readLines(configure), outdated)
})

test_that("check_configure_ac_drift accepts the freshly rendered template", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  suppressMessages(minirextendr:::use_miniextendr_configure())

  expect_no_warning(minirextendr:::check_configure_ac_drift())
})

test_that("check_configure_ac_drift reports arbitrary edits without replacing them", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  suppressMessages(minirextendr:::use_miniextendr_configure())

  configure <- file.path(tmp, "configure.ac")
  customized <- c(readLines(configure), "CARGO_FEATURES='serde'")
  writeLines(customized, configure)

  expect_warning(
    minirextendr:::check_configure_ac_drift(),
    "configure.ac differs from the current template"
  )
  expect_identical(readLines(configure), customized)
})

test_that("configure drift uses the package layout, not prior scaffold state", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  old_type <- minirextendr:::get_template_type()
  withr::defer(minirextendr:::set_template_type(old_type))
  minirextendr:::set_template_type("rpkg")
  suppressMessages(minirextendr:::use_miniextendr_configure())

  minirextendr:::set_template_type("monorepo")
  expect_no_warning(minirextendr:::check_configure_ac_drift())
  expect_identical(minirextendr:::get_template_type(), "monorepo")
})

test_that("configure drift accepts the current monorepo package template", {
  tmp <- withr::local_tempdir()
  writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
  pkg <- file.path(tmp, "rpkg")
  dir.create(pkg)
  writeLines("Package: driftcheck", file.path(pkg, "DESCRIPTION"))
  usethis::local_project(pkg, force = TRUE, setwd = FALSE)
  old_type <- minirextendr:::get_template_type()
  withr::defer(minirextendr:::set_template_type(old_type))
  minirextendr:::set_template_type("monorepo")
  suppressMessages(minirextendr:::use_miniextendr_configure(subdir = "rpkg"))

  minirextendr:::set_template_type("rpkg")
  expect_no_warning(minirextendr:::check_configure_ac_drift())
  expect_identical(minirextendr:::get_template_type(), "rpkg")
})

test_that("configure drift picks the template the upgrade's layout picks (#1720)", {
  # upgrade_layout() counts a package as a monorepo's only when its immediate
  # parent holds the workspace Cargo.toml, where detect_project_type() accepts
  # one in any ancestor. A package two levels below the Cargo.toml is
  # therefore checked against the standalone template.
  ws <- withr::local_tempdir()
  writeLines("[workspace]", file.path(ws, "Cargo.toml"))
  seen <- logical()
  local_mocked_bindings(
    configure_ac_template = function(monorepo) {
      seen <<- c(seen, monorepo)
      system.file("templates", "rpkg", "configure.ac",
                  package = "minirextendr", mustWork = TRUE)
    },
    .package = "minirextendr"
  )
  check_in <- function(pkg) {
    dir.create(pkg, recursive = TRUE)
    writeLines("Package: driftcheck", file.path(pkg, "DESCRIPTION"))
    writeLines("AC_INIT([driftcheck], [0.1.0])", file.path(pkg, "configure.ac"))
    usethis::with_project(pkg, force = TRUE, setwd = FALSE,
      suppressWarnings(minirextendr:::check_configure_ac_drift()))
  }

  check_in(file.path(ws, "rpkg"))
  check_in(file.path(ws, "crates", "rpkg"))
  expect_identical(seen, c(TRUE, FALSE))
})

# A package at `version` whose configure.ac is `configure` (the template
# rendered for the package at `ac_version` by default); `monorepo` puts it in
# `rpkg/` under a workspace Cargo.toml. Returns the package directory.
make_versioned_pkg <- function(root, version, monorepo = FALSE, ac_version = "0.1.0",
                               configure = NULL) {
  pkg <- if (monorepo) file.path(root, "rpkg") else root
  if (monorepo) writeLines("[workspace]", file.path(root, "Cargo.toml"))
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE, showWarnings = FALSE)
  writeLines(c("Package: verpkg", paste("Version:", version)), file.path(pkg, "DESCRIPTION"))
  writeLines("[package]", file.path(pkg, "src", "rust", "Cargo.toml"))
  writeLines("# Previous scaffold", file.path(pkg, "src", "Makevars.in"))
  if (is.null(configure)) {
    configure <- readLines(minirextendr:::configure_ac_template(monorepo), warn = FALSE)
    configure <- gsub("{{package}}", "verpkg", configure, fixed = TRUE)
    configure <- gsub("{{version}}", ac_version, configure, fixed = TRUE)
  }
  writeLines(configure, file.path(pkg, "configure.ac"))
  pkg
}

test_that("upgrade with configure_ac = TRUE writes DESCRIPTION's version into AC_INIT", {
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir()
    pkg <- make_versioned_pkg(tmp, "0.5.1", monorepo,
                              configure = c("AC_INIT([verpkg], [0.1.0])", 'CARGO_FEATURES=""', "AC_OUTPUT"))
    suppressMessages(upgrade_miniextendr_package(path = tmp, configure_ac = TRUE, autoconf = FALSE,
                                                allow_dirty = TRUE))
    expect_identical(readLines(file.path(pkg, "configure.ac"))[[1L]], "AC_INIT([verpkg], [0.5.1])",
                     info = paste("monorepo:", monorepo))
  }
})

test_that("upgrade with configure_ac = FALSE ignores an AC_INIT version that differs from DESCRIPTION's", {
  # Every package scaffolded before AC_INIT took DESCRIPTION's version has
  # AC_INIT([pkg], [0.1.0]); nothing reads that version.
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir()
    pkg <- make_versioned_pkg(tmp, "0.5.1", monorepo)
    before <- readLines(file.path(pkg, "configure.ac"))
    expect_no_warning(suppressMessages(upgrade_miniextendr_package(
      path = tmp, configure_ac = FALSE, autoconf = FALSE, allow_dirty = TRUE
    )))
    expect_identical(readLines(file.path(pkg, "configure.ac")), before)
  }
})

test_that("configure drift still warns on a real edit next to a different AC_INIT version", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  pkg <- make_versioned_pkg(tmp, "0.5.1")
  configure <- file.path(pkg, "configure.ac")
  customized <- c(readLines(configure), "CARGO_FEATURES='serde'")
  writeLines(customized, configure)

  expect_warning(
    minirextendr:::check_configure_ac_drift(monorepo = FALSE),
    "configure.ac differs from the current template"
  )
  expect_identical(readLines(configure), customized)
})
