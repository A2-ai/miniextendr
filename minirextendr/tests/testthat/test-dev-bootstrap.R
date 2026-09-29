test_that("development manifest activates only in an unchanged staged copy", {
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  original <- normalizePath(withr::local_tempdir(), winslash = "/")
  dir.create(file.path(original, "src/rust"), recursive = TRUE)
  manifest <- file.path(original, "src/rust/Cargo.toml")
  writeLines("source manifest", manifest)
  writeLines("portable manifest", file.path(original, "src/rust/.Cargo.toml.dev"))
  saveRDS(list(root = original, manifest = unname(tools::md5sum(manifest))),
          file.path(original, "src/rust/.dev-bootstrap.rds"))
  expect_false(helper$activate_dev_bootstrap(original))
  expect_identical(readLines(manifest), "source manifest")
  stage <- withr::local_tempdir()
  dir.create(file.path(stage, "src/rust"), recursive = TRUE)
  files <- list.files(file.path(original, "src/rust"), all.files = TRUE, full.names = TRUE, no.. = TRUE)
  expect_true(all(file.copy(files, file.path(stage, "src/rust"))))
  expect_true(helper$activate_dev_bootstrap(stage))
  expect_identical(readLines(file.path(stage, "src/rust/Cargo.toml")), "portable manifest")
  expect_false(file.exists(file.path(stage, "src/rust/.dev-bootstrap.rds")))
  expect_identical(readLines(manifest), "source manifest")
  expect_true(any(grepl(".dev-vendor-backup-", list.files(file.path(stage, "src/rust"), all.files = TRUE), fixed = TRUE)))
  stale <- withr::local_tempdir()
  dir.create(file.path(stale, "src/rust"), recursive = TRUE)
  expect_true(all(file.copy(files, file.path(stale, "src/rust"))))
  writeLines("changed source", file.path(stale, "src/rust/Cargo.toml"))
  expect_error(helper$activate_dev_bootstrap(stale), "changed after development bootstrap")
  expect_identical(readLines(file.path(stale, "src/rust/Cargo.toml")), "changed source")
})

# Path-dependency chain for the base-R stager: the R crate reaches a renamed,
# workspace-inherited sibling (which reaches a nested versioned sibling) and a
# version-less dev-only sibling. Registry-free, so the staged copy checks offline.
local_path_dep_monorepo <- function(env = parent.frame()) {
  mono <- normalizePath(withr::local_tempdir(.local_envir = env), winslash = "/")
  crate <- function(dir, manifest, lib) {
    dir.create(file.path(mono, dir), recursive = TRUE)
    writeLines(manifest, file.path(mono, dir, "Cargo.toml"))
    writeLines(lib, file.path(mono, dir, "lib.rs"))
  }
  writeLines(c('[workspace]', 'resolver = "3"', 'members = ["core", "inner", "devsib"]',
               'exclude = ["pkg"]', '[workspace.package]', 'version = "0.1.0"', 'edition = "2024"',
               '[workspace.dependencies]', 'inner-sib = { path = "inner", version = "0.2" }'),
             file.path(mono, "Cargo.toml"))
  crate("inner", c('[package]', 'name = "inner-sib"', 'version = "0.2.0"', 'edition.workspace = true',
                   '[lib]', 'path = "lib.rs"', '[dev-dependencies]',
                   'devsib = { path = "../devsib", version = "0.1" }'),
        "pub fn h() -> i32 { 3 }")
  crate("devsib", c('[package]', 'name = "devsib"', 'version.workspace = true', 'edition.workspace = true',
                    '[lib]', 'path = "lib.rs"'), "pub fn d() -> i32 { 1 }")
  crate("core", c('[package]', 'name = "core-lib"', 'version.workspace = true', 'edition.workspace = true',
                  '[lib]', 'path = "lib.rs"', '[dependencies]', 'inner-sib.workspace = true'),
        "pub fn f() -> i32 { 4 + inner_sib::h() }")
  crate("pkg/src/rust", c('[package]', 'name = "pkg-r"', 'version = "0.1.0"', 'edition = "2024"',
                          'publish = false', '', '[workspace]', '', '[lib]', 'path = "lib.rs"',
                          '[dependencies]', 'core_library = { package = "core-lib", path = "../../../core" }',
                          '[dev-dependencies]', 'devsib = { path = "../../../devsib" }'),
        "pub fn g() -> i32 { core_library::f() }")
  status <- system2("cargo", c("generate-lockfile", "--offline", "--quiet", "--manifest-path",
                               shQuote(file.path(mono, "pkg/src/rust/Cargo.toml"))))
  stopifnot(status == 0L)
  mono
}

test_that("the base-R stager builds a relocatable copy without touching the checkout", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "cargo not available")
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  mono <- local_path_dep_monorepo()
  pkg <- file.path(mono, "pkg")
  rust <- file.path(pkg, "src/rust")
  sources <- file.path(mono, c("core/Cargo.toml", "inner/Cargo.toml", "devsib/Cargo.toml",
                               "pkg/src/rust/Cargo.toml", "pkg/src/rust/Cargo.lock"))
  before <- tools::md5sum(sources)
  listing <- function() list.files(mono, recursive = TRUE, all.files = TRUE)
  files <- listing()
  expect_true(suppressMessages(helper$prepare_dev_bootstrap(pkg)))
  expect_identical(tools::md5sum(sources), before)
  # Only the staging outputs appear; no lockfile or target dir lands in the monorepo.
  added <- setdiff(listing(), files)
  expect_true(all(startsWith(added, "pkg/src/rust/vendor/") |
                    added %in% c("pkg/src/rust/.Cargo.toml.dev", "pkg/src/rust/.dev-bootstrap.rds")),
              info = paste(added, collapse = ", "))
  expect_length(setdiff(files, listing()), 0L)
  expect_setequal(list.files(file.path(rust, "vendor")),
                  c("core-lib-0.1.0", "devsib-0.1.0", "inner-sib-0.2.0"))
  expect_length(list.files(file.path(rust, "vendor"), "^\\.cargo-checksum\\.json$",
                           recursive = TRUE, all.files = TRUE), 0L)
  portable <- readLines(file.path(rust, ".Cargo.toml.dev"))
  expect_true('core_library = { package = "core-lib", path = "vendor/core-lib-0.1.0" }' %in% portable)
  expect_true('devsib = { path = "vendor/devsib-0.1.0" }' %in% portable)
  expect_true('exclude = ["vendor"]' %in% portable)
  core <- readLines(file.path(rust, "vendor/core-lib-0.1.0/Cargo.toml"))
  expect_identical(core[which(core == "[dependencies.inner-sib]") + 1L], 'path = "../inner-sib-0.2.0"')

  # What R CMD build's cleanup sees: a copy under an unrelated ancestor workspace.
  outer <- normalizePath(withr::local_tempdir(), winslash = "/")
  writeLines(c("[workspace]", "members = []"), file.path(outer, "Cargo.toml"))
  expect_true(file.copy(pkg, outer, recursive = TRUE))
  copy <- file.path(outer, "pkg")
  expect_true(helper$activate_dev_bootstrap(copy))
  manifest <- file.path(copy, "src/rust/Cargo.toml")
  target <- withr::local_tempdir()
  log <- tempfile(fileext = ".log")
  status <- system2("cargo", c("check", "--locked", "--offline", "--quiet", "--manifest-path",
                               shQuote(manifest)), stdout = log, stderr = log,
                    env = paste0("CARGO_TARGET_DIR=", shQuote(target)))
  expect_identical(status, 0L, info = paste(readLines(log), collapse = "\n"))
  expect_identical(unname(tools::md5sum(file.path(copy, "src/rust/Cargo.lock"))),
                   unname(before[[5L]]))
  metadata <- paste(system2("cargo", c("metadata", "--no-deps", "--format-version", "1", "--offline",
                                       "--manifest-path", shQuote(manifest)), stdout = TRUE),
                    collapse = "")
  members <- regmatches(metadata, regexpr('"workspace_members":\\[[^]]*\\]', metadata))
  expect_length(gregexpr("path+file", members, fixed = TRUE)[[1L]], 1L)
})

test_that("the base-R stager reports every blocking dependency before staging", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "cargo not available")
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  mono <- local_path_dep_monorepo()
  writeLines(c('[package]', 'name = "core-lib"', 'version.workspace = true', 'edition.workspace = true',
               '[lib]', 'path = "lib.rs"', '[dependencies]', 'inner-sib = { path = "../inner" }',
               'leaf = { package = "devsib", path = "../devsib" }',
               'remote = { git = "https://example.invalid/remote.git" }'),
             file.path(mono, "core/Cargo.toml"))
  pkg <- file.path(mono, "pkg")
  err <- expect_error(helper$prepare_dev_bootstrap(pkg), "Cannot stage path dependencies:", fixed = TRUE)
  expect_match(conditionMessage(err), "`inner-sib` has no version requirement; add `version = \"0.2.0\"`", fixed = TRUE)
  expect_match(conditionMessage(err), "`leaf` has no version requirement", fixed = TRUE)
  expect_match(conditionMessage(err), "[workspace.dependencies]", fixed = TRUE)
  expect_match(conditionMessage(err), "`remote` is a git dependency", fixed = TRUE)
  expect_false(dir.exists(file.path(pkg, "src/rust/vendor")))
  expect_false(file.exists(file.path(pkg, "src/rust/.dev-bootstrap.rds")))
  expect_false(file.exists(file.path(pkg, "src/rust/.Cargo.toml.dev")))
})

test_that("path dependencies inside the package need no staging", {
  skip_on_cran()
  skip_if_not(nzchar(Sys.which("cargo")), "cargo not available")
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  pkg <- normalizePath(withr::local_tempdir(), winslash = "/")
  dir.create(file.path(pkg, "src/rust/satellite"), recursive = TRUE)
  writeLines(c('[package]', 'name = "pkg-r"', 'version = "0.1.0"', 'edition = "2024"', '[workspace]',
               '[lib]', 'path = "lib.rs"', '[dependencies]', 'satellite = { path = "satellite" }'),
             file.path(pkg, "src/rust/Cargo.toml"))
  writeLines(c('[package]', 'name = "satellite"', 'version = "0.1.0"', 'edition = "2024"',
               '[lib]', 'path = "lib.rs"'), file.path(pkg, "src/rust/satellite/Cargo.toml"))
  expect_false(helper$prepare_dev_bootstrap(pkg))
  expect_false(file.exists(file.path(pkg, "src/rust/.dev-bootstrap.rds")))
  expect_false(dir.exists(file.path(pkg, "src/rust/vendor")))
})

# A `cargo-revendor` on PATH that records any call and fails: bootstrap.R must
# never vendor, whatever tools the machine has.
local_recording_cargo_revendor <- function(env = parent.frame()) {
  bin <- withr::local_tempdir(.local_envir = env)
  marker <- file.path(bin, "called")
  writeLines(c("#!/bin/sh", sprintf("touch '%s'", marker), "exit 1"), file.path(bin, "cargo-revendor"))
  Sys.chmod(file.path(bin, "cargo-revendor"), "755")
  withr::local_envvar(PATH = paste(c(bin, Sys.getenv("PATH")), collapse = .Platform$path.sep),
                      .local_envir = env)
  marker
}

test_that("every pkgbuild frontend stages nested path siblings and never vendors", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  revendor_called <- local_recording_cargo_revendor()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "fallbackprobe")
  lib <- file.path(root, "library")
  dir.create(lib)
  for (crate in c("core/src", "inner/src")) dir.create(file.path(root, crate), recursive = TRUE)
  writeLines(c('[package]', 'name = "innervalue"', 'version = "0.1.0"', 'edition = "2024"'),
             file.path(root, "inner/Cargo.toml"))
  writeLines('pub fn value() -> i32 { 3 }', file.path(root, "inner/src/lib.rs"))
  # Only the sibling-to-sibling edge needs a version: `cargo package` rewrites it.
  writeLines(c('[package]', 'name = "devvalue"', 'version = "0.1.0"', 'edition = "2024"',
               '[dependencies]', 'innervalue = { path = "../inner", version = "0.1" }'),
             file.path(root, "core/Cargo.toml"))
  writeLines('pub fn value() -> i32 { 4 + innervalue::value() }', file.path(root, "core/src/lib.rs"))
  withr::local_envvar(c(
    R_LIBS = paste(c(lib, .libPaths()), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = "", CARGO_BUILD_TARGET = NA,
    CARGO_TERM_COLOR = "never", CARGO_TARGET_DIR = NA, VENDOR_OUT = NA
  ))
  suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
  writeLines(c('[package]', 'name = "fallbackprobe"', 'version = "0.1.0"',
    'edition = "2024"', 'build = false', '[workspace]', '[lib]', 'path = "lib.rs"',
    'crate-type = ["staticlib"]', '[dependencies]',
    'miniextendr-api = { git = "https://github.com/A2-ai/miniextendr" }',
    'engine = { package = "devvalue", path = "../../../core" }'),
    file.path(pkg, "src/rust/Cargo.toml"))
  writeLines(c('use miniextendr_api::miniextendr;',
    'miniextendr_api::miniextendr_init!();', '#[miniextendr]',
    'pub fn value() -> i32 { engine::value() }'), file.path(pkg, "src/rust/lib.rs"))
  writeLines('useDynLib(fallbackprobe, .registration = TRUE)', file.path(pkg, "NAMESPACE"))
  manifest <- file.path(pkg, "src/rust/Cargo.toml")
  before <- tools::md5sum(manifest)
  run <- function(command, args, name) {
    log <- file.path(root, paste0(name, ".log"))
    status <- withr::with_dir(pkg, system2(command, args, stdout = log, stderr = log))
    output <- paste(readLines(log, warn = FALSE), collapse = "\n")
    expect_identical(status, 0L, info = output)
    if (status != 0L) stop(output, call. = FALSE)
    output
  }
  run("autoconf", character(), "autoconf")
  probe <- file.path(root, "probe.R")
  writeLines('stopifnot(identical(loadNamespace("fallbackprobe")$value(), 7L))', probe)

  # devtools::build(): pkgbuild runs bootstrap.R, R CMD build's cleanup activates.
  # pak's git client checks files out without mode bits, and R CMD build skips
  # a cleanup that is not executable; bootstrap restores the bit.
  Sys.chmod(file.path(pkg, "cleanup"), "644")
  build <- file.path(root, "build.R")
  writeLines(sprintf('devtools::build(%s, path = %s, binary = FALSE, vignettes = FALSE, manual = FALSE)',
                     deparse(pkg), deparse(root)), build)
  output <- run(file.path(R.home("bin"), "Rscript"), shQuote(build), "build")
  expect_true(file_test("-x", file.path(pkg, "cleanup")))
  expect_match(output, "Staged 2 path dependencies under src/rust/vendor", fixed = TRUE)
  expect_false(grepl("Warning", output, fixed = TRUE), info = output)
  expect_identical(tools::md5sum(manifest), before)
  expect_false(file.exists(file.path(pkg, "inst/vendor.tar.xz")))
  tarball <- list.files(root, "^fallbackprobe_.*[.]tar[.]gz$", full.names = TRUE)
  expect_length(tarball, 1L)
  entries <- utils::untar(tarball, list = TRUE)
  expect_false("fallbackprobe/inst/vendor.tar.xz" %in% entries)
  expect_true(all(c("fallbackprobe/src/rust/vendor/devvalue-0.1.0/Cargo.toml",
                    "fallbackprobe/src/rust/vendor/innervalue-0.1.0/Cargo.toml") %in% entries))
  expect_false(any(grepl("dev-bootstrap.rds|Cargo.toml.dev|dev-vendor-backup|cargo-checksum", entries)))
  sealed <- file.path(root, "sealed")
  utils::untar(tarball, files = "fallbackprobe/src/rust/Cargo.toml", exdir = sealed)
  expect_true('engine = { package = "devvalue", path = "vendor/devvalue-0.1.0" }' %in%
              readLines(file.path(sealed, "fallbackprobe/src/rust/Cargo.toml")))
  run(file.path(R.home("bin"), "R"), c("CMD", "INSTALL", "-l", shQuote(lib), shQuote(tarball)), "install-tarball")
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "runtime-tarball")

  # devtools::install(): the same bootstrap, then an install from R CMD build's copy.
  install <- file.path(root, "install.R")
  writeLines(sprintf('devtools::install(%s, build = TRUE, dependencies = FALSE, upgrade = FALSE, reload = FALSE, quiet = FALSE)',
                     deparse(pkg)), install)
  for (i in 1:2) {
    output <- run(file.path(R.home("bin"), "Rscript"), shQuote(install), paste0("devtools-install-", i))
    expect_match(output, "Staged 2 path dependencies under src/rust/vendor", fixed = TRUE)
    expect_identical(tools::md5sum(manifest), before)
    expect_false(file.exists(file.path(pkg, "inst/vendor.tar.xz")))
    expect_false(file.exists(file.path(pkg, "src/rust/.Cargo.toml.prefreeze")))
    run(file.path(R.home("bin"), "Rscript"), shQuote(probe), paste0("runtime-devtools-", i))
  }

  # Plain R CMD build skips bootstrap.R; running it first replaces the staging.
  run(file.path(R.home("bin"), "Rscript"), "bootstrap.R", "bootstrap")
  run(file.path(R.home("bin"), "R"), c("CMD", "build", "--no-build-vignettes", "--no-manual", "."),
      "r-cmd-build")
  plain <- list.files(pkg, "^fallbackprobe_.*[.]tar[.]gz$", full.names = TRUE)
  expect_length(plain, 1L)
  entries <- utils::untar(plain, list = TRUE)
  expect_true("fallbackprobe/src/rust/vendor/innervalue-0.1.0/Cargo.toml" %in% entries)
  expect_false("fallbackprobe/inst/vendor.tar.xz" %in% entries)
  expect_false(any(grepl("dev-bootstrap.rds|Cargo.toml.dev|dev-vendor-backup", entries)))
  expect_false(file.exists(revendor_called))
})
