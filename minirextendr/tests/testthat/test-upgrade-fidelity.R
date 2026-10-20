# Upgrade fidelity: what upgrade_miniextendr_package() keeps, writes and
# reports (#1711, #1712, #1669, #1670, #1713).

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
    expect_true("monorepo/rpkg/tools/write-wrappers.R" %in% seen$paths, info = from)
    # The workspace-root gitignore template is the one read outside
    # monorepo/rpkg/; it goes to the root .gitignore (#1670).
    expect_true("monorepo/gitignore" %in% seen$paths, info = from)
    expect_true(all(startsWith(setdiff(seen$paths, "monorepo/gitignore"), "monorepo/rpkg/")),
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
  expect_true("rpkg/tools/write-wrappers.R" %in% seen$paths)
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

# region: #1669 — leftovers of a pre-#1656 scaffold --------------------------

# git with no user hooks, attributes or signing in play.
fidelity_git <- function(path, ...) {
  args <- c("-C", path,
            "-c", "user.name=Scaffold Test",
            "-c", "user.email=scaffold@example.invalid",
            "-c", "commit.gpgsign=false",
            "-c", paste0("core.hooksPath=", file.path(path, ".empty-hooks")),
            "-c", paste0("core.attributesFile=", file.path(path, ".empty-attributes")),
            ...)
  out <- system2("git", shQuote(args), stdout = TRUE, stderr = TRUE)
  expect_null(attr(out, "status"), info = paste(out, collapse = "\n"))
  unname(as.character(out))
}

test_that("upgrade_gitignore() drops the retired wrapper-provenance entry", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE, quiet = TRUE)
  gitignore <- file.path(tmp, ".gitignore")
  writeLines(c(
    "my-notes/",
    "# Generated wrapper provenance, shipped with the package tarball",
    "/tools/wrapper-inputs.rds",
    "tools/wrapper-inputs.rds"
  ), gitignore)
  local_mocked_bindings(use_miniextendr_gitignore = function(...) invisible(),
                        .package = "minirextendr")

  suppressMessages(minirextendr:::upgrade_gitignore())

  # Only the exact retired lines go; a user's own variant stays.
  expect_identical(readLines(gitignore), c("my-notes/", "tools/wrapper-inputs.rds"))
})

test_that("the upgrade summary lists leftover files and leaves them in place", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  withr::defer(minirextendr:::set_template_type("rpkg"))
  pkg <- make_upgrade_pkg(withr::local_tempdir())
  fidelity_git(pkg, "init", "-q")
  dir.create(file.path(pkg, "tools"))
  leftovers <- file.path("tools", c("wrapper-freshness.R", "wrapper-inputs.rds"))
  writeLines("# old wrapper fast path", file.path(pkg, leftovers[[1L]]))
  saveRDS(list(), file.path(pkg, leftovers[[2L]]))
  win_def <- file.path("src", "fidpkg-win.def")
  writeLines("EXPORTS", file.path(pkg, win_def))
  # -f: once the upgrade's ignore rule exists a plain add would refuse it, and
  # a pre-#1656 package committed it before that rule.
  fidelity_git(pkg, "add", "-f", win_def)

  msgs <- capture_messages(upgrade_miniextendr_package(
    path = pkg, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
  ))
  text <- paste(msgs, collapse = "")

  expect_match(text, "No longer used by the package")
  expect_match(text, "tools/wrapper-freshness.R", fixed = TRUE)
  expect_match(text, "tools/wrapper-inputs.rds", fixed = TRUE)
  expect_match(text, "git rm --cached src/fidpkg-win.def", fixed = TRUE)
  expect_true(all(file.exists(file.path(pkg, leftovers))))
  expect_identical(fidelity_git(pkg, "ls-files", "--", win_def), win_def)
})

test_that("the upgrade summary stays quiet without leftovers or a git repository", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  pkg <- make_upgrade_pkg(withr::local_tempdir())
  writeLines("EXPORTS", file.path(pkg, "src", "fidpkg-win.def"))

  msgs <- capture_messages(upgrade_miniextendr_package(
    path = pkg, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
  ))
  text <- paste(msgs, collapse = "")

  expect_no_match(text, "No longer used by the package")
  expect_no_match(text, "git rm --cached")
})

# endregion -------------------------------------------------------------------

# region: #1670 — monorepo root .gitignore -----------------------------------

test_that("a monorepo upgrade adds the root template's entries and keeps user lines", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  for (from in c("root", "pkg")) {
    tmp <- withr::local_tempdir()
    dirs <- make_upgrade_monorepo(tmp, "r bindings")
    root_ignore <- file.path(dirs$root, ".gitignore")
    # A pre-#1656 root file: no `<pkg>/src/*-win.def`, plus user lines.
    user <- c("# Team notes", "notes/", "/target/", "r bindings/src/Makevars")
    writeLines(user, root_ignore)

    run_upgrade(dirs[[from]])

    after <- readLines(root_ignore)
    expect_identical(after[seq_along(user)], user, info = from)
    expect_true("r bindings/src/*-win.def" %in% after, info = from)
    expect_true("r bindings/R/*-wrappers.R" %in% after, info = from)
    expect_false(any(grepl("{{rpkg_name}}", after, fixed = TRUE)), info = from)
    # Entries already present are not repeated; template comments are not copied.
    expect_identical(sum(after == "/target/"), 1L, info = from)
    expect_false("# Rust build artifacts" %in% after, info = from)

    # A rerun adds nothing.
    run_upgrade(dirs[[from]])
    expect_identical(readLines(root_ignore), after, info = from)
  }
})

# endregion -------------------------------------------------------------------

# region: #1713 — summary of the files an upgrade changed ---------------------

test_that("upgrade_changes() sorts files into changed, added and removed", {
  before <- c(a = "1", b = NA, c = "3", d = "4", e = NA)
  after <- c(a = "1x", b = "2", c = NA, d = "4", e = NA)

  expect_identical(minirextendr:::upgrade_changes(before, after),
                   list(changed = "a", added = "b", removed = "c"))

  text <- paste(capture_messages(minirextendr:::report_upgrade_changes(
    list(changed = "src/stub.c", added = character(), removed = "cleanup")
  )), collapse = "")
  expect_match(text, "Changed 1 file:.*src/stub\\.c.*Removed 1 file:.*cleanup")
  expect_match(text, "git diff", fixed = TRUE)
  # `git status` is only needed to see added files.
  expect_no_match(text, "git status", fixed = TRUE)
})

test_that("the upgrade names the owned files it changed, committed edits included", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  withr::defer(minirextendr:::set_template_type("rpkg"))

  for (from in c("standalone", "root", "pkg")) {
    tmp <- withr::local_tempdir()
    if (from == "standalone") {
      pkg <- make_upgrade_pkg(tmp)
      repo <- pkg
      prefix <- ""
    } else {
      dirs <- make_upgrade_monorepo(tmp, "mypkg-r")
      pkg <- dirs$pkg
      repo <- dirs$root
      # Paths are relative to the workspace root, wherever `path` points.
      prefix <- "mypkg-r/"
    }
    path <- if (from == "pkg") pkg else repo
    before_files <- list.files(repo, recursive = TRUE, all.files = TRUE)

    # First upgrade of a bare package: everything it writes is new, except
    # the files the fixture already had.
    first <- run_upgrade(path)
    expect_identical(first$removed, character(), info = from)
    expect_true(paste0(prefix, "inst/include/mx_abi.h") %in% first$added, info = from)
    expect_true(paste0(prefix, "src/Makevars.in") %in% first$changed, info = from)
    # Every file the upgrade created is reported, so upgrade_owned_files()
    # covers each file the upgrade writes.
    after_files <- list.files(repo, recursive = TRUE, all.files = TRUE)
    expect_identical(sort(setdiff(after_files, before_files)), sort(first$added),
                     info = from)

    fidelity_git(repo, "init", "-q")
    fidelity_git(repo, "add", "-A")
    fidelity_git(repo, "commit", "-q", "-m", "scaffold")

    # A committed local edit to a template-owned file.
    abi_rel <- paste0(prefix, "inst/include/mx_abi.h")
    abi <- file.path(repo, abi_rel)
    pristine <- readLines(abi)
    writeLines(c(pristine, "/* local edit */"), abi)
    fidelity_git(repo, "commit", "-q", "-am", "edit mx_abi.h")

    # allow_dirty = FALSE: a committed edit passes the uncommitted-changes check.
    msgs <- capture_messages(second <- upgrade_miniextendr_package(
      path = path, configure_ac = TRUE, autoconf = FALSE
    ))
    text <- paste(msgs, collapse = "")

    expect_identical(readLines(abi), pristine, info = from)
    expect_identical(second,
                     list(changed = abi_rel, added = character(), removed = character()),
                     info = from)
    # The summary agrees with what git sees.
    expect_identical(fidelity_git(repo, "diff", "--name-only"), abi_rel, info = from)
    expect_match(text, paste0("Changed 1 file:.*", abi_rel), info = from)
    expect_match(text, "git diff", fixed = TRUE, info = from)
    expect_match(text, "committed ones included", fixed = TRUE, info = from)

    # Upgrading the up-to-date package changes nothing and says so.
    msgs <- capture_messages(third <- upgrade_miniextendr_package(
      path = path, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
    ))
    text <- paste(msgs, collapse = "")

    expect_identical(third,
                     list(changed = character(), added = character(),
                          removed = character()),
                     info = from)
    expect_match(text, "No scaffold files changed", fixed = TRUE, info = from)
    expect_no_match(text, "git diff", fixed = TRUE, info = from)
  }
})

test_that("upgrading a fresh scaffold reports no changed files", {
  # The scaffolders write some files by their own route (the monorepo one
  # writes DESCRIPTION and the ignore files directly); an upgrade right after
  # must agree with them byte for byte, or every first upgrade reports noise.
  # Scaffolding only checks that cargo exists; nothing here compiles.
  local_mocked_bindings(check_rust = function() invisible(TRUE),
                        .package = "minirextendr")
  withr::defer(minirextendr:::set_template_type("rpkg"))
  # create_package() prints the new DESCRIPTION unless usethis is quiet.
  withr::local_options(usethis.quiet = TRUE)
  base <- withr::local_tempdir()
  # create_miniextendr_monorepo() leaves its root as the active project;
  # restore the caller's on exit.
  usethis::local_project(base, force = TRUE, setwd = FALSE, quiet = TRUE)

  pkg <- file.path(base, "freshpkg")
  suppressWarnings(suppressMessages({
    usethis::create_package(pkg, open = FALSE)
    use_miniextendr(path = pkg, claude_skills = FALSE)
  }))
  mono <- file.path(base, "freshmono")
  suppressWarnings(suppressMessages(create_miniextendr_monorepo(
    mono, package = "freshmono", crate_name = "freshmono-rs", open = FALSE
  )))

  for (path in c(pkg, mono)) {
    expect_identical(
      suppressMessages(upgrade_miniextendr_package(path, autoconf = FALSE,
                                                   allow_dirty = TRUE)),
      list(changed = character(), added = character(), removed = character()),
      info = basename(path)
    )
  }
})

# endregion -------------------------------------------------------------------

# region: #2208 — the upgrade replaces the skills and hook sections it ships --

# A package in a fresh git repository, its build files already upgraded, so a
# later upgrade only touches what a test changes. Global and system git
# configuration are kept out: a developer's core.hooksPath cannot send the
# hooks elsewhere.
local_git_pkg <- function(env = parent.frame()) {
  empty_config <- withr::local_tempfile(.local_envir = env)
  file.create(empty_config)
  withr::local_envvar(GIT_CONFIG_GLOBAL = empty_config, GIT_CONFIG_NOSYSTEM = "1",
                      .local_envir = env)
  withr::defer(minirextendr:::set_template_type("rpkg"), envir = env)
  pkg <- make_upgrade_pkg(withr::local_tempdir(.local_envir = env))
  fidelity_git(pkg, "init", "-q")
  run_upgrade(pkg)
  pkg
}

no_changes <- list(changed = character(), added = character(), removed = character())

skill_file <- function(slug) file.path(".claude", "skills", slug, "SKILL.md")

bundled_skill <- function(slug) {
  readLines(file.path(minirextendr:::bundled_skills_dir(), slug, "SKILL.md"))
}

test_that("the upgrade replaces the miniextendr skills and keeps the user's own", {
  pkg <- local_git_pkg()
  suppressMessages(use_claude_skills(path = pkg))
  guide <- skill_file("miniextendr-guide")
  release <- skill_file("miniextendr-release")
  retired <- skill_file("miniextendr-old")
  own <- skill_file("my-own-skill")
  writeLines(c(bundled_skill("miniextendr-guide"), "A local note."), file.path(pkg, guide))
  file.remove(file.path(pkg, release))
  for (skill in c(retired, own)) {
    dir.create(dirname(file.path(pkg, skill)))
    writeLines("not bundled", file.path(pkg, skill))
  }

  changes <- run_upgrade(pkg)

  # The edit is overwritten, the missing file restored, and the skill the
  # bundle no longer ships deleted; the summary and return value say so.
  expect_identical(changes, list(changed = guide, added = release, removed = retired))
  expect_identical(readLines(file.path(pkg, guide)), bundled_skill("miniextendr-guide"))
  expect_identical(readLines(file.path(pkg, release)), bundled_skill("miniextendr-release"))
  expect_false(dir.exists(file.path(pkg, dirname(retired))))
  expect_identical(readLines(file.path(pkg, own)), "not bundled")
  installed <- basename(list.dirs(file.path(pkg, ".claude", "skills"), recursive = FALSE))
  bundled <- basename(list.dirs(minirextendr:::bundled_skills_dir(), recursive = FALSE))
  expect_setequal(installed, c(bundled, "my-own-skill"))

  # A rerun finds the skills current.
  expect_identical(run_upgrade(pkg), no_changes)
})

test_that("a package without miniextendr skills gets none", {
  pkg <- local_git_pkg()
  expect_false(dir.exists(file.path(pkg, ".claude")))
  own <- file.path(pkg, skill_file("my-own-skill"))
  dir.create(dirname(own), recursive = TRUE)
  writeLines("user-owned", own)

  expect_identical(run_upgrade(pkg), no_changes)
  expect_identical(list.files(file.path(pkg, ".claude", "skills")), "my-own-skill")
  expect_identical(readLines(own), "user-owned")
})

test_that("the upgrade rewrites the miniextendr hook sections and keeps the user's lines", {
  pkg <- local_git_pkg()
  hooks_dir <- file.path(pkg, ".git", "hooks")
  user_lines <- c("#!/bin/sh", "echo user check")
  writeLines(user_lines, file.path(hooks_dir, "pre-commit"))
  suppressMessages(use_miniextendr_git_hooks(path = pkg))
  # pre-commit: the user's lines, then the appended section. post-merge: a
  # standalone install.
  current <- lapply(miniextendr_hook_names, function(hook) readLines(file.path(hooks_dir, hook)))
  names(current) <- miniextendr_hook_names
  expect_identical(current[["pre-commit"]][1:2], user_lines)

  # Sections from an older minirextendr.
  for (hook in miniextendr_hook_names) {
    writeLines(c(current[[hook]], "# check from an older minirextendr"),
               file.path(hooks_dir, hook))
  }

  msgs <- capture_messages(changes <- upgrade_miniextendr_package(
    path = pkg, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
  ))

  hook_paths <- file.path(".git", "hooks", miniextendr_hook_names)
  expect_identical(changes, list(changed = hook_paths, added = character(),
                                 removed = character()))
  for (hook in miniextendr_hook_names) {
    expect_identical(readLines(file.path(hooks_dir, hook)), current[[hook]], info = hook)
  }
  text <- paste(msgs, collapse = "")
  expect_match(text, "Changed 2 files:.*\\.git/hooks/pre-commit.*\\.git/hooks/post-merge")

  expect_identical(run_upgrade(pkg), no_changes)
})

test_that("the upgrade leaves a hook without a miniextendr section alone", {
  pkg <- local_git_pkg()
  hooks_dir <- file.path(pkg, ".git", "hooks")
  user_hook <- c("#!/bin/sh", "echo user check")
  writeLines(user_hook, file.path(hooks_dir, "pre-commit"))

  expect_identical(run_upgrade(pkg), no_changes)
  expect_identical(readLines(file.path(hooks_dir, "pre-commit")), user_hook)
  expect_false(file.exists(file.path(hooks_dir, "post-merge")))
})

test_that("outside a git repository the upgrade replaces the skills and skips the hooks", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  pkg <- make_upgrade_pkg(withr::local_tempdir())
  run_upgrade(pkg)
  expect_false(dir.exists(file.path(pkg, ".claude")))
  expect_null(minirextendr:::resolve_git_hooks_dir(pkg))

  suppressMessages(use_claude_skills(path = pkg))
  guide <- skill_file("miniextendr-guide")
  writeLines("edited", file.path(pkg, guide))

  expect_identical(run_upgrade(pkg), list(changed = guide, added = character(),
                                          removed = character()))
  expect_identical(readLines(file.path(pkg, guide)), bundled_skill("miniextendr-guide"))
})

test_that("a monorepo's skills are replaced at the workspace root", {
  withr::defer(minirextendr:::set_template_type("rpkg"))
  dirs <- make_upgrade_monorepo(withr::local_tempdir(), "mypkg-r")
  run_upgrade(dirs$root)
  suppressMessages(use_claude_skills(path = dirs$root))
  guide <- skill_file("miniextendr-guide")

  for (from in c("root", "pkg")) {
    writeLines("edited", file.path(dirs$root, guide))

    changes <- run_upgrade(dirs[[from]])

    expect_identical(changes, list(changed = guide, added = character(),
                                   removed = character()), info = from)
    expect_identical(readLines(file.path(dirs$root, guide)),
                     bundled_skill("miniextendr-guide"), info = from)
    expect_false(dir.exists(file.path(dirs$pkg, ".claude")), info = from)
  }
})

test_that("an uncommitted skill edit stops the upgrade; a committed one is replaced", {
  pkg <- local_git_pkg()
  suppressMessages(use_claude_skills(path = pkg))
  fidelity_git(pkg, "add", "-A")
  fidelity_git(pkg, "commit", "-q", "-m", "scaffold")
  guide <- skill_file("miniextendr-guide")
  edited <- c(bundled_skill("miniextendr-guide"), "A local note.")
  writeLines(edited, file.path(pkg, guide))

  upgrade <- function() {
    suppressMessages(upgrade_miniextendr_package(
      path = pkg, configure_ac = TRUE, autoconf = FALSE
    ))
  }
  expect_error(upgrade(), "miniextendr-guide/SKILL.md", fixed = TRUE)
  expect_identical(readLines(file.path(pkg, guide)), edited)

  fidelity_git(pkg, "commit", "-q", "-am", "edit a skill")
  expect_identical(upgrade(), list(changed = guide, added = character(),
                                   removed = character()))
  # git holds the edit for the author to merge back.
  expect_identical(fidelity_git(pkg, "diff", "--name-only"), guide)
})

# endregion -------------------------------------------------------------------
