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

# region: AC_INIT version is not drift (#1789)

local_drift_package <- function(monorepo, env = parent.frame()) {
  tmp <- withr::local_tempdir(.local_envir = env)
  pkg <- tmp
  if (monorepo) {
    writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
    pkg <- file.path(tmp, "rpkg")
    dir.create(pkg)
  }
  writeLines("Package: driftcheck", file.path(pkg, "DESCRIPTION"))
  usethis::local_project(pkg, force = TRUE, setwd = FALSE, .local_envir = env)
  old_type <- minirextendr:::get_template_type()
  withr::defer(minirextendr:::set_template_type(old_type), envir = env)
  minirextendr:::set_template_type(if (monorepo) "monorepo" else "rpkg")
  suppressMessages(minirextendr:::use_miniextendr_configure(
    subdir = if (monorepo) "rpkg"
  ))
  file.path(pkg, "configure.ac")
}

test_that("drift check ignores the AC_INIT version, for both templates (#1789)", {
  for (monorepo in c(FALSE, TRUE)) {
    configure <- local_drift_package(monorepo)
    lines <- readLines(configure)
    lines[1] <- "AC_INIT([driftcheck], [0.2.0])"
    writeLines(lines, configure)
    expect_no_warning(minirextendr:::check_configure_ac_drift(monorepo))

    # Any other differing line still warns.
    writeLines(c(lines, "CARGO_FEATURES='serde'"), configure)
    expect_warning(
      minirextendr:::check_configure_ac_drift(monorepo),
      "configure.ac differs from the current template"
    )
  }
})

test_that("replacing configure.ac keeps the AC_INIT version (#1789)", {
  configure <- local_drift_package(FALSE)
  lines <- readLines(configure)
  lines[1] <- "AC_INIT([driftcheck], [0.2.0])"
  writeLines(c(lines, "CARGO_FEATURES='serde'"), configure)

  suppressMessages(minirextendr:::use_miniextendr_configure())

  new <- readLines(configure)
  expect_identical(new[1], "AC_INIT([driftcheck], [0.2.0])")
  expect_false("CARGO_FEATURES='serde'" %in% new)
  expect_no_warning(minirextendr:::check_configure_ac_drift(FALSE))
})

test_that("a fresh scaffold gets the template's AC_INIT version (#1789)", {
  configure <- local_drift_package(FALSE)
  expect_identical(readLines(configure)[1], "AC_INIT([driftcheck], [0.1.0])")
})

test_that("bump-version.R does not cause configure.ac drift (#1789)", {
  skip_if(!nzchar(Sys.which("Rscript")))
  configure <- local_drift_package(TRUE)
  pkg <- dirname(configure)
  ws <- dirname(pkg)
  dir.create(file.path(ws, "tools"))
  file.copy(
    system.file("templates", "monorepo", "tools", "bump-version.R",
                package = "minirextendr", mustWork = TRUE),
    file.path(ws, "tools", "bump-version.R")
  )
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  writeLines(c("[package]", "name = \"driftcheck\"", "version = \"0.1.0\""),
             file.path(pkg, "src", "rust", "Cargo.toml"))
  writeLines(c("Package: driftcheck", "Version: 0.0.0.9000"),
             file.path(pkg, "DESCRIPTION"))

  withr::with_dir(ws, system2("Rscript", c("tools/bump-version.R", basename(pkg)),
                              stdout = FALSE, stderr = FALSE))
  expect_identical(readLines(configure)[1],
                   "AC_INIT([driftcheck], [0.0.0.9000])")
  expect_no_warning(minirextendr:::check_configure_ac_drift(TRUE))
})

# endregion
