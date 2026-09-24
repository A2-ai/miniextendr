# configure's check for path dependencies outside the package (the #1580
# follow-up for installers that skip bootstrap.R). An installer that takes the
# package directory out of its repository strands such a dependency: a copy
# must stop in configure naming the path, and a tree of symlinks into the
# package (rv up to 0.12.0) is restaged from the package it mirrors.

# region: fixtures

configure_helper <- function() {
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  helper
}

write_files <- function(root, files) {
  for (file in names(files)) {
    dir.create(dirname(file.path(root, file)), recursive = TRUE, showWarnings = FALSE)
    writeLines(files[[file]], file.path(root, file))
  }
}

git_in <- function(repo, ...) {
  out <- system2("git", c("-C", shQuote(repo), "-c", "user.name=test", "-c", "user.email=test@example.com",
                          "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", ...),
                 stdout = TRUE, stderr = TRUE)
  stopifnot(is.null(attr(out, "status")))
  out
}

rust_crate <- function(dir, name, lib, ..., version = "0.1.0") {
  stats::setNames(list(c("[package]", sprintf('name = "%s"', name), sprintf('version = "%s"', version),
                         'edition = "2021"', ...), lib),
                  file.path(dir, c("Cargo.toml", "src/lib.rs")))
}

# A committed repository: package pkg/ whose crate reaches core/ (renamed, and
# itself reaching a versioned inner/) and a dev-only devsib/, all outside the
# package, plus an in-package satellite. Registry-free, so it builds offline.
local_outside_repo <- function(env = parent.frame()) {
  repo <- normalizePath(withr::local_tempdir(.local_envir = env), winslash = "/")
  write_files(repo, c(
    rust_crate("core", "core-lib", "pub fn f() -> i32 { 4 + inner_sib::h() }",
               "[dependencies]", 'inner-sib = { path = "../inner", version = "0.2" }'),
    rust_crate("inner", "inner-sib", "pub fn h() -> i32 { 3 }", version = "0.2.0"),
    rust_crate("devsib", "devsib", "pub fn d() -> i32 { 1 }"),
    list("pkg/DESCRIPTION" = c("Package: pkg", "Version: 0.1.0"),
         "pkg/src/rust/Cargo.toml" = c('[package]', 'name = "pkg-r"', 'version = "0.1.0"', 'edition = "2021"',
           "publish = false", "", "[workspace]", "", "[lib]", 'path = "lib.rs"', "", "[dependencies]",
           'core_library = { package = "core-lib", path = "../../../core" }',
           'satellite = { path = "satellite" }', "", "[dev-dependencies]",
           'devsib = { path = "../../../devsib" }'),
         "pkg/src/rust/lib.rs" = "pub fn g() -> i32 { core_library::f() + satellite::s() }",
         "pkg/src/rust/satellite/Cargo.toml" = c("[package]", 'name = "satellite"', 'version = "0.1.0"',
                                                 'edition = "2021"', "[lib]", 'path = "lib.rs"'),
         "pkg/src/rust/satellite/lib.rs" = "pub fn s() -> i32 { 1 }",
         ".gitignore" = c("target/", "**/src/rust/vendor/", "**/src/rust/.Cargo.toml.dev",
                          "**/src/rust/.dev-bootstrap.rds", "**/src/rust/.dev-vendor-backup-*/"))
  ))
  if (nzchar(Sys.which("cargo"))) {
    status <- system2("cargo", c("generate-lockfile", "--offline", "--quiet", "--manifest-path",
                                 shQuote(file.path(repo, "pkg/src/rust/Cargo.toml"))))
    stopifnot(status == 0L)
  }
  git_in(repo, "init", "-q")
  git_in(repo, "add", "-A")
  git_in(repo, "commit", "-q", "-m", "fixture")
  repo
}

# rv up to 0.12.0 builds from real directories whose files are symlinks into
# the package (`symlink_package`); later releases copy the package directory.
# Either tree sits where the package's outside siblings are absent.
local_build_tree <- function(from, mode = c("link", "copy"), env = parent.frame()) {
  mode <- match.arg(mode)
  to <- file.path(normalizePath(withr::local_tempdir(.local_envir = env), winslash = "/"), "build")
  dirs <- list.dirs(from, full.names = FALSE)
  for (dir in c("", dirs[nzchar(dirs)])) dir.create(file.path(to, dir), recursive = TRUE, showWarnings = FALSE)
  files <- list.files(from, recursive = TRUE, all.files = TRUE, no.. = TRUE)
  made <- if (mode == "link") file.symlink(file.path(from, files), file.path(to, files)) else
    file.copy(file.path(from, files), file.path(to, files), copy.mode = TRUE)
  stopifnot(all(made))
  to
}

is_symlink <- function(path) nzchar(Sys.readlink(path))

# endregion

test_that("missing outside paths are found in every table cargo loads", {
  helper <- configure_helper()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  build <- file.path(root, "build")
  write_files(root, c(
    rust_crate("present", "present", "pub fn p() {}"),
    list("build/src/rust/satellite/Cargo.toml" = "[package]",
         "build/src/rust/Cargo.toml" = c(
           "[package]", 'name = "pkg-r"', 'version = "0.1.0"', "[workspace]", "[lib]", 'path = "../../../lib.rs"',
           "[dependencies]", 'a = { path = "../../../a" }', 'present = { path = "../../../present" }',
           'satellite = { path = "satellite" }', 'inside = { path = "not-here" }',
           '# gone = { path = "../../../commented" }',
           "[dependencies.b]", 'version = "0.1"', "path = '../../../b'",
           "[dev-dependencies]", 'c.path = "./../../../c/"',
           "[build-dependencies]", 'd = { path = "../../../d" }',
           "[target.'cfg(windows)'.dependencies]", 'e = { path = "../../../e" }',
           "[workspace.dependencies]", 'f = { path = "../../../x/../f" }',
           "[patch.crates-io]", sprintf('g = { path = "%s/g" }', root),
           "[replace]", '"h:0.1.0" = { path = "../../../h" }'))))
  paths <- helper$dev_bootstrap_paths(build)
  missing <- helper$missing_path_dependencies(paths)
  expect_identical(missing$dir, file.path(root, c("a", "b", "c", "d", "e", "f", "g")))
  expect_identical(missing$section, c("[dependencies]", "[dependencies.b]", "[dev-dependencies]",
    "[build-dependencies]", "[target.'cfg(windows)'.dependencies]", "[workspace.dependencies]",
    "[patch.crates-io]"))
  # A missing path keeps its `..` through normalizePath(); cargo collapses it.
  expect_identical(helper$collapse_dots("/r/pkg/src/rust/../../../core/./x/"), "/r/core/x")
  expect_identical(helper$collapse_dots("C:\\r\\pkg\\..\\core"), "C:/r/core")
})

test_that("a copy stops before cargo, naming the path and the installs that work", {
  helper <- configure_helper()
  repo <- local_outside_repo()
  copy <- local_build_tree(file.path(repo, "pkg"), "copy")
  expect_null(helper$linked_origin(helper$dev_bootstrap_paths(copy)))
  err <- expect_error(helper$configure_path_dependencies(copy), "outside the package directory are missing")
  msg <- conditionMessage(err)
  core <- file.path(dirname(copy), "core")
  expect_match(msg, sprintf('`path = "../../../core"` under [dependencies] in src/rust/Cargo.toml: %s has no Cargo.toml',
                            core), fixed = TRUE)
  expect_match(msg, "`path = \"../../../devsib\"` under [dev-dependencies]", fixed = TRUE)
  expect_false(grepl("satellite", msg, fixed = TRUE))
  expect_match(msg, "bootstrap.R did not run for this build.", fixed = TRUE)
  expect_match(msg, "rv >= 0.23.0 with a git source plus `directory`", fixed = TRUE)
  expect_match(msg, 'pak::pak("<owner>/<repo>/<subdirectory>")', fixed = TRUE)
  expect_match(msg, "build the tarball in the checkout with devtools::build() and install that tarball",
               fixed = TRUE)
  expect_false(dir.exists(file.path(copy, "src/rust/vendor")))
})

test_that("a vendored tarball, in-package paths and a bootstrap staging pass untouched", {
  helper <- configure_helper()
  repo <- local_outside_repo()
  pkg <- file.path(repo, "pkg")
  # The checkout itself: the siblings are where the manifest says.
  expect_false(helper$configure_path_dependencies(pkg))
  # Tarball mode vendors everything, so a copy has nothing to check.
  copy <- local_build_tree(pkg, "copy")
  dir.create(file.path(copy, "inst"))
  file.create(file.path(copy, "inst/vendor.tar.xz"))
  expect_false(helper$configure_path_dependencies(copy))
  # Only in-package path dependencies.
  inside <- normalizePath(withr::local_tempdir(), winslash = "/")
  write_files(inside, list("src/rust/Cargo.toml" = c("[package]", 'name = "p"', "[dependencies]",
                                                     'satellite = { path = "satellite" }')))
  expect_false(helper$configure_path_dependencies(inside))
  expect_identical(git_in(repo, "status", "--porcelain", "--ignored"), character())
})

test_that("a tree of symlinks is restaged from the package it mirrors, never writing through", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "cargo not available")
  helper <- configure_helper()
  repo <- local_outside_repo()
  pkg <- file.path(repo, "pkg")
  sources <- list.files(repo, recursive = TRUE, all.files = TRUE, full.names = TRUE)
  sources <- sources[!grepl("/.git/", sources, fixed = TRUE)]
  before <- tools::md5sum(sources)
  tree <- local_build_tree(pkg, "link")
  manifest <- file.path(tree, "src/rust/Cargo.toml")
  lock <- file.path(tree, "src/rust/Cargo.lock")
  expect_true(is_symlink(manifest) && is_symlink(lock))
  expect_identical(helper$linked_origin(helper$dev_bootstrap_paths(tree))$root, pkg)
  messages <- capture_messages(expect_true(helper$configure_path_dependencies(tree)))
  expect_match(messages[[1L]], "bootstrap.R did not run for this build.", fixed = TRUE)
  expect_match(messages[[1L]], sprintf("This build directory links to %s; staging its path dependencies from there.",
                                       pkg), fixed = TRUE)
  expect_match(messages[[2L]], "Staged 4 path dependencies under src/rust/vendor", fixed = TRUE)
  # The manifest and lockfile are the tree's own files now; the crates are
  # staged beside them, the in-package one too, as bootstrap.R stages them.
  expect_false(is_symlink(manifest) || is_symlink(lock))
  expect_setequal(list.files(file.path(tree, "src/rust/vendor")),
                  c("core-lib-0.1.0", "devsib-0.1.0", "inner-sib-0.2.0", "satellite-0.1.0"))
  staged <- readLines(manifest)
  expect_true('core_library = { package = "core-lib", path = "vendor/core-lib-0.1.0" }' %in% staged)
  expect_true('satellite = { path = "vendor/satellite-0.1.0" }' %in% staged)
  expect_true('exclude = ["vendor"]' %in% staged)
  expect_false(any(grepl("^\\.(dev-vendor-backup|Cargo\\.toml\\.linked|Cargo\\.lock\\.linked)-",
                         list.files(file.path(tree, "src/rust"), all.files = TRUE))))
  target <- withr::local_tempdir()
  log <- tempfile(fileext = ".log")
  status <- system2("cargo", c("check", "--locked", "--offline", "--quiet", "--manifest-path", shQuote(manifest)),
                    stdout = log, stderr = log, env = paste0("CARGO_TARGET_DIR=", shQuote(target)))
  expect_identical(status, 0L, info = paste(readLines(log), collapse = "\n"))
  # A second configure finds the staging and leaves it alone.
  expect_false(helper$configure_path_dependencies(tree))
  # Nothing reached the mirrored checkout.
  expect_identical(tools::md5sum(sources), before)
  expect_identical(git_in(repo, "status", "--porcelain", "--ignored"), character())

  # A tree that links to a copy cannot reach the siblings either.
  orphan <- local_build_tree(local_build_tree(pkg, "copy"), "link")
  err <- expect_error(helper$configure_path_dependencies(orphan), "outside the package directory are missing")
  expect_match(conditionMessage(err), "where they are missing too", fixed = TRUE)
})

test_that("a staging bootstrap left behind, or activated in a build copy, passes", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "cargo not available")
  helper <- configure_helper()
  repo <- local_outside_repo()
  pkg <- file.path(repo, "pkg")
  expect_true(suppressMessages(helper$prepare_dev_bootstrap(pkg, mode = "dist")))
  staged <- tools::md5sum(list.files(file.path(pkg, "src/rust"), recursive = TRUE, all.files = TRUE,
                                     full.names = TRUE))
  expect_false(helper$configure_path_dependencies(pkg))
  expect_identical(tools::md5sum(names(staged)), staged)
  # What R CMD build's cleanup leaves in its copy: the manifest points at vendor/.
  copy <- local_build_tree(pkg, "copy")
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA))
  expect_true(helper$activate_dev_bootstrap(copy))
  expect_false(helper$configure_path_dependencies(copy))
})

test_that("R CMD INSTALL of a monorepo package fails in configure from a copy and builds from a symlink tree", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "git")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  mono <- file.path(root, "mono")
  lib <- file.path(root, "library")
  dir.create(lib)
  withr::local_envvar(c(
    R_LIBS = paste(c(lib, .libPaths()), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = "", CARGO_BUILD_TARGET = NA,
    CARGO_TERM_COLOR = "never", CARGO_TARGET_DIR = NA, VENDOR_OUT = NA,
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA, MINIEXTENDR_BOOTSTRAP_MODE = NA
  ))
  suppressMessages(create_miniextendr_monorepo(mono, package = "outsideprobe", crate_name = "core",
                                               rpkg_name = "rpkg", open = FALSE))
  expect_true('core_library = { package = "core", path = "../../../core" }' %in%
              readLines(file.path(mono, "rpkg/src/rust/Cargo.toml")))
  status <- system2("cargo", c("generate-lockfile", "--quiet", "--manifest-path",
                               shQuote(file.path(mono, "rpkg/src/rust/Cargo.toml"))))
  skip_if(status != 0L, "cannot resolve the scaffold's dependencies")
  git_in(mono, "add", "-A")
  git_in(mono, "commit", "-q", "-m", "scaffold")
  # rv installs from its own clean checkout, which has no configure output.
  checkout <- file.path(root, "checkout")
  status <- system2("git", c("clone", "-q", shQuote(mono), shQuote(checkout)))
  expect_identical(status, 0L)
  install <- function(pkg, name) {
    log <- file.path(root, paste0(name, ".log"))
    status <- withr::with_dir(root, system2(file.path(R.home("bin"), "R"),
      c("CMD", "INSTALL", "--no-multiarch", "-l", shQuote(lib), shQuote(pkg)), stdout = log, stderr = log))
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"))
  }

  copy <- install(local_build_tree(file.path(checkout, "rpkg"), "copy"), "copy-install")
  expect_false(identical(copy$status, 0L))
  expect_match(copy$output, "bootstrap.R did not run for this build.", fixed = TRUE)
  expect_match(copy$output, "configure: error: a path dependency outside the package is missing", fixed = TRUE)
  expect_false(grepl("failed to load manifest", copy$output, fixed = TRUE), info = copy$output)

  linked <- install(local_build_tree(file.path(checkout, "rpkg"), "link"), "link-install")
  expect_identical(linked$status, 0L, info = linked$output)
  expect_match(linked$output, "Staged 1 path dependencies under src/rust/vendor", fixed = TRUE)
  probe <- file.path(root, "probe.R")
  writeLines('stopifnot(identical(outsideprobe:::core_greeting(), "Hello from core!"))', probe)
  status <- withr::with_dir(root, system2(file.path(R.home("bin"), "Rscript"), shQuote(probe)))
  expect_identical(status, 0L)
  expect_identical(git_in(checkout, "status", "--porcelain", "--ignored"), character())
})
