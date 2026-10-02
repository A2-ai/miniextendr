# Upgrade fidelity: what upgrade_miniextendr_package() keeps, writes and
# reports (#1711, #1712, #1669, #1670).

# region: #1711 — config.guess / config.sub timestamps ----------------------

# A copy of the bundled script with its timestamp line replaced.
write_config_script <- function(dest, script, timestamp) {
  lines <- readLines(minirextendr:::script_path(script), warn = FALSE)
  stamp <- grep("^timestamp='", lines)[[1L]]
  if (is.null(timestamp)) {
    lines <- lines[-stamp]
  } else {
    lines[[stamp]] <- sprintf("timestamp='%s'", timestamp)
  }
  writeLines(lines, dest)
  dest
}

test_that("bundled config scripts carry a parseable timestamp", {
  for (script in c("config.guess", "config.sub")) {
    ts <- minirextendr:::config_script_timestamp(minirextendr:::script_path(script))
    expect_match(ts, "^[0-9]{4}-[0-9]{2}-[0-9]{2}$", info = script)
  }
})

test_that("copy_config_scripts() keeps a newer script and replaces an older one", {
  tmp <- withr::local_tempdir()
  newer <- write_config_script(file.path(tmp, "config.guess"), "config.guess", "2999-01-01")
  older <- write_config_script(file.path(tmp, "config.sub"), "config.sub", "2000-01-01")
  newer_bytes <- readBin(newer, "raw", file.size(newer))

  msgs <- capture_messages(minirextendr:::copy_config_scripts(tmp, display_prefix = "tools"))

  expect_identical(readBin(newer, "raw", file.size(newer)), newer_bytes)
  expect_match(paste(msgs, collapse = ""), "Kept.*tools/config\\.guess.*2999-01-01.*newer than the bundled")
  bundled_sub <- minirextendr:::script_path("config.sub")
  expect_identical(unname(tools::md5sum(older)), unname(tools::md5sum(bundled_sub)))
  expect_false(any(grepl("Kept.*config\\.sub", msgs)))
})

test_that("copy_config_scripts() replaces a script without a timestamp", {
  tmp <- withr::local_tempdir()
  for (script in c("config.guess", "config.sub")) {
    write_config_script(file.path(tmp, script), script, NULL)
  }
  suppressMessages(minirextendr:::copy_config_scripts(tmp, display_prefix = NULL))
  for (script in c("config.guess", "config.sub")) {
    expect_identical(unname(tools::md5sum(file.path(tmp, script))),
                     unname(tools::md5sum(minirextendr:::script_path(script))),
                     info = script)
  }
})

# endregion -------------------------------------------------------------------

# region: fixtures for real (unmocked) upgrades ------------------------------

# The minimum is_miniextendr_package() accepts. Upgrades in this file run with
# configure_ac = TRUE, so configure.ac is replaced from the template and the
# drift check does not run.
make_upgrade_pkg <- function(pkg, package = "fidpkg") {
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE, showWarnings = FALSE)
  writeLines(c(paste0("Package: ", package), "Version: 0.0.1"),
             file.path(pkg, "DESCRIPTION"))
  writeLines(c(sprintf("AC_INIT([%s], [0.0.1])", package), 'CARGO_FEATURES=""',
               "AC_OUTPUT"), file.path(pkg, "configure.ac"))
  writeLines("# Previous scaffold", file.path(pkg, "src", "Makevars.in"))
  writeLines("[package]", file.path(pkg, "src", "rust", "Cargo.toml"))
  normalizePath(pkg)
}

make_upgrade_monorepo <- function(root, subdir = "rpkg") {
  writeLines("[workspace]", file.path(root, "Cargo.toml"))
  pkg <- make_upgrade_pkg(file.path(root, subdir))
  list(root = normalizePath(root), pkg = pkg)
}

run_upgrade <- function(path, ...) {
  suppressMessages(upgrade_miniextendr_package(
    path = path, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE, ...
  ))
}

# Record every template the scaffolding helpers resolve, as paths relative to
# inst/templates (e.g. "monorepo/rpkg/stub.c"). template_path() and
# use_template() each resolve templates, so both are wrapped.
local_template_recorder <- function(env = parent.frame()) {
  seen <- new.env(parent = emptyenv())
  seen$paths <- character()
  real_template_path <- minirextendr:::template_path
  real_use_template <- minirextendr:::use_template
  local_mocked_bindings(
    template_path = function(name, subdir = NULL) {
      path <- real_template_path(name, subdir = subdir)
      seen$paths <- c(seen$paths, sub("^.*/templates/", "", path))
      path
    },
    use_template = function(template, save_as = template, data = list(),
                            subdir = NULL, open = FALSE) {
      seen$paths <- c(seen$paths, paste(
        c(minirextendr:::get_template_type(), subdir, template), collapse = "/"
      ))
      real_use_template(template, save_as = save_as, data = data,
                        subdir = subdir, open = open)
    },
    .package = "minirextendr",
    .env = env
  )
  seen
}

# endregion -------------------------------------------------------------------

# region: #1712 — monorepo template set --------------------------------------

test_that("upgrade_layout() finds the workspace from the root or the package dir", {
  tmp <- withr::local_tempdir()
  dirs <- make_upgrade_monorepo(tmp, "mypkg-r")

  from_root <- minirextendr:::upgrade_layout(dirs$root)
  expect_identical(from_root$pkg, file.path(dirs$root, "mypkg-r"))
  expect_identical(from_root$root, dirs$root)
  expect_identical(from_root$subdir, "mypkg-r")

  from_pkg <- minirextendr:::upgrade_layout(dirs$pkg)
  expect_identical(from_pkg$pkg, dirs$pkg)
  expect_identical(from_pkg$root, dirs$root)
  expect_identical(from_pkg$subdir, "mypkg-r")
})

test_that("upgrade_layout() keeps a package nested deeper in a Rust repo standalone", {
  tmp <- withr::local_tempdir()
  writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
  pkg <- make_upgrade_pkg(file.path(tmp, "packages", "fidpkg"))

  layout <- minirextendr:::upgrade_layout(pkg)
  expect_identical(layout$pkg, pkg)
  expect_null(layout$root)
  expect_null(layout$subdir)
})

test_that("a monorepo upgrade renders the monorepo templates; a later standalone one does not", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  seen <- local_template_recorder()

  for (from in c("root", "pkg")) {
    tmp <- withr::local_tempdir()
    dirs <- make_upgrade_monorepo(tmp, "mypkg-r")
    seen$paths <- character()

    run_upgrade(dirs[[from]])

    expect_true("monorepo/rpkg/configure.ac" %in% seen$paths, info = from)
    expect_true("monorepo/rpkg/tools/lock-shape-check.R" %in% seen$paths, info = from)
    expect_true(all(startsWith(seen$paths, "monorepo/rpkg/")),
                info = paste(c(from, seen$paths), collapse = "\n"))
    expect_identical(minirextendr:::get_template_type(), "rpkg")

    # The package-level ignore files come from monorepo/rpkg/, not from the
    # workspace-root templates.
    pkg_ignore <- readLines(file.path(dirs$pkg, ".gitignore"))
    expect_false(any(grepl("{{rpkg_name}}", pkg_ignore, fixed = TRUE)), info = from)
    expect_true("src/*-win.def" %in% pkg_ignore, info = from)
    expect_true(file.exists(file.path(dirs$pkg, ".Rbuildignore")), info = from)
  }

  # A standalone upgrade later in the session gets the standalone set, even
  # when an earlier call left the "monorepo" type active.
  minirextendr:::set_template_type("monorepo")
  standalone <- make_upgrade_pkg(withr::local_tempdir())
  seen$paths <- character()

  run_upgrade(standalone)

  expect_true("rpkg/configure.ac" %in% seen$paths)
  expect_true(all(startsWith(seen$paths, "rpkg/")),
              info = paste(seen$paths, collapse = "\n"))
  expect_identical(minirextendr:::get_template_type(), "monorepo")
})

test_that("the monorepo and standalone configure.ac templates match", {
  # They differed only in dnl comments; #1712 brought them back in line.
  read_template <- function(...) {
    readLines(system.file("templates", ..., "configure.ac",
                          package = "minirextendr", mustWork = TRUE))
  }
  expect_identical(read_template("monorepo", "rpkg"), read_template("rpkg"))
})

# endregion -------------------------------------------------------------------
