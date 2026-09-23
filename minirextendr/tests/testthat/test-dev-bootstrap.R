test_that("development manifest activates only in an unchanged staged copy", {
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  original <- normalizePath(withr::local_tempdir(), winslash = "/")
  dir.create(file.path(original, "src/rust"), recursive = TRUE)
  manifest <- file.path(original, "src/rust/Cargo.toml")
  writeLines("source manifest", manifest)
  writeLines("portable manifest", file.path(original, "src/rust/.Cargo.toml.dev"))
  saveRDS(list(root = original, manifest = unname(tools::md5sum(manifest)), mode = "dev"),
          file.path(original, "src/rust/.dev-bootstrap.rds"))
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = "dev"))
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

test_that("activation follows the mode that staged the package", {
  helper <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/dev-bootstrap.R", package = "minirextendr"), helper)
  staged <- function(mode, env = parent.frame()) {
    original <- normalizePath(withr::local_tempdir(.local_envir = env), winslash = "/")
    stage <- withr::local_tempdir(.local_envir = env)
    dir.create(file.path(stage, "src/rust"), recursive = TRUE)
    manifest <- file.path(stage, "src/rust/Cargo.toml")
    writeLines("source manifest", manifest)
    writeLines("portable manifest", file.path(stage, "src/rust/.Cargo.toml.dev"))
    saveRDS(list(root = original, manifest = unname(tools::md5sum(manifest)), mode = mode),
            file.path(stage, "src/rust/.dev-bootstrap.rds"))
    stage
  }
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA))
  # A distribution build without cargo-revendor stages too, so dist activates.
  dist <- staged("dist")
  expect_true(helper$activate_dev_bootstrap(dist))
  expect_identical(readLines(file.path(dist, "src/rust/Cargo.toml")), "portable manifest")
  # A development staging left in the checkout is not reused by a dist build.
  dev <- staged("dev")
  expect_false(helper$activate_dev_bootstrap(dev))
  expect_identical(readLines(file.path(dev, "src/rust/Cargo.toml")), "source manifest")
  expect_false(file.exists(file.path(dev, "src/rust/.dev-bootstrap.rds")))
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
  expect_true(suppressMessages(helper$prepare_dev_bootstrap(pkg, mode = "dist")))
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
  expect_identical(readRDS(file.path(rust, ".dev-bootstrap.rds"))$mode, "dist")

  # What R CMD build's cleanup sees: a copy under an unrelated ancestor workspace.
  outer <- normalizePath(withr::local_tempdir(), winslash = "/")
  writeLines(c("[workspace]", "members = []"), file.path(outer, "Cargo.toml"))
  expect_true(file.copy(pkg, outer, recursive = TRUE))
  copy <- file.path(outer, "pkg")
  withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA))
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
  err <- expect_error(helper$prepare_dev_bootstrap(pkg, mode = "dist"), "without cargo-revendor")
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
  expect_false(helper$prepare_dev_bootstrap(pkg, mode = "dist"))
  expect_false(file.exists(file.path(pkg, "src/rust/.dev-bootstrap.rds")))
  expect_false(dir.exists(file.path(pkg, "src/rust/vendor")))
})

test_that("devtools installs path siblings without freezing the source or making xz", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "cargo-revendor")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "devprobe")
  core <- file.path(root, "core")
  lib <- file.path(root, "library")
  dir.create(lib)
  dir.create(file.path(core, "src"), recursive = TRUE)
  writeLines(c('[package]', 'name = "devvalue"', 'version = "0.1.0"', 'edition = "2024"'),
             file.path(core, "Cargo.toml"))
  writeLines('pub fn value() -> i32 { 7 }', file.path(core, "src/lib.rs"))
  withr::local_envvar(c(
    R_LIBS = paste(c(lib, .libPaths()), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = "", CARGO_BUILD_TARGET = NA,
    CARGO_TERM_COLOR = "never", CARGO_TARGET_DIR = NA, VENDOR_OUT = NA,
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA,
    MINIEXTENDR_BOOTSTRAP_MODE = "dev"
  ))
  suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
  writeLines(c('[package]', 'name = "devprobe"', 'version = "0.1.0"',
    'edition = "2024"', 'build = false', '[workspace]', '[lib]', 'path = "lib.rs"',
    'crate-type = ["staticlib"]', '[dependencies]',
    'miniextendr-api = { git = "https://github.com/A2-ai/miniextendr" }',
    sprintf('engine = { package = "devvalue", path = "%s" }', core)),
    file.path(pkg, "src/rust/Cargo.toml"))
  writeLines(c('use miniextendr_api::miniextendr;',
    'miniextendr_api::miniextendr_init!();', '#[miniextendr]',
    'pub fn value() -> i32 { engine::value() }'), file.path(pkg, "src/rust/lib.rs"))
  writeLines('useDynLib(devprobe, .registration = TRUE)', file.path(pkg, "NAMESPACE"))
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
  run("bash", "./configure", "configure")
  script <- file.path(root, "install.R")
  writeLines(sprintf('devtools::install(%s, build = TRUE, dependencies = FALSE, upgrade = FALSE, reload = FALSE, quiet = FALSE)',
                     deparse(pkg)), script)
  for (i in 1:2) {
    output <- run(file.path(R.home("bin"), "Rscript"), shQuote(script), paste0("dev-install-", i))
    expect_identical(tools::md5sum(manifest), before)
    expect_false(file.exists(file.path(pkg, "inst/vendor.tar.xz")))
    expect_false(file.exists(file.path(pkg, "src/rust/.Cargo.toml.prefreeze")))
    dirs <- list.dirs(file.path(pkg, "src/rust/vendor"), recursive = FALSE, full.names = FALSE)
    expect_identical(dirs, "devvalue-0.1.0")
    expect_match(output, "Prepared 1 path dependencies for development", fixed = TRUE)
    expect_false(grepl("warning:", output, fixed = TRUE), info = output)
  }
  probe <- file.path(root, "probe.R")
  writeLines('stopifnot(identical(loadNamespace("devprobe")$value(), 7L))', probe)
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "runtime")
  # Distribution remains the default when the dev opt-in is removed.
  build <- file.path(root, "build.R")
  writeLines(sprintf('devtools::build(%s, path = %s, binary = FALSE, vignettes = FALSE, manual = FALSE)',
                     deparse(pkg), deparse(root)), build)
  withr::with_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA),
    run(file.path(R.home("bin"), "Rscript"), shQuote(build), "distribution-build"))
  tarball <- list.files(root, "^devprobe_.*[.]tar[.]gz$", full.names = TRUE)
  expect_length(tarball, 1L)
  entries <- utils::untar(tarball, list = TRUE)
  expect_true("devprobe/inst/vendor.tar.xz" %in% entries)
  expect_false(any(grepl("dev-bootstrap.rds|Cargo.toml.dev|dev-vendor-backup|src/rust/vendor/", entries)))
  withr::with_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = NA),
    run(file.path(R.home("bin"), "R"), c("CMD", "INSTALL", "-l", shQuote(lib), shQuote(tarball)), "distribution-install"))
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "distribution-runtime")
})

# PATH keeping every tool except cargo-revendor (normally a sibling of cargo in
# ~/.cargo/bin), so bootstrap.R takes its base-R fallback.
local_path_without_cargo_revendor <- function(env = parent.frame()) {
  tool <- unname(Sys.which("cargo-revendor"))
  if (!nzchar(tool)) return(invisible(NULL))
  shim <- withr::local_tempdir(.local_envir = env)
  keep <- setdiff(list.files(dirname(tool), full.names = TRUE), tool)
  file.symlink(keep, file.path(shim, basename(keep)))
  dirs <- strsplit(Sys.getenv("PATH"), .Platform$path.sep, fixed = TRUE)[[1L]]
  dirs <- dirs[normalizePath(dirs, mustWork = FALSE) != normalizePath(dirname(tool))]
  withr::local_envvar(PATH = paste(c(shim, dirs), collapse = .Platform$path.sep), .local_envir = env)
  stopifnot(!nzchar(Sys.which("cargo-revendor")))
}

test_that("without cargo-revendor, dist and dev builds stage nested path siblings", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  local_path_without_cargo_revendor()
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
    CARGO_TERM_COLOR = "never", CARGO_TARGET_DIR = NA, VENDOR_OUT = NA,
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA,
    MINIEXTENDR_BOOTSTRAP_MODE = NA
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
  run("bash", "./configure", "configure")
  probe <- file.path(root, "probe.R")
  writeLines('stopifnot(identical(loadNamespace("fallbackprobe")$value(), 7L))', probe)

  # Distribution: pkgbuild runs bootstrap.R, R CMD build's cleanup activates.
  # pak's git client checks files out without mode bits, and R CMD build skips
  # a cleanup that is not executable; bootstrap restores the bit.
  Sys.chmod(file.path(pkg, "cleanup"), "644")
  build <- file.path(root, "build.R")
  writeLines(sprintf('devtools::build(%s, path = %s, binary = FALSE, vignettes = FALSE, manual = FALSE)',
                     deparse(pkg), deparse(root)), build)
  output <- run(file.path(R.home("bin"), "Rscript"), shQuote(build), "fallback-build")
  expect_true(file_test("-x", file.path(pkg, "cleanup")))
  expect_match(output, "staged path dependencies under src/rust/vendor", fixed = TRUE)
  expect_identical(tools::md5sum(manifest), before)
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
  run(file.path(R.home("bin"), "R"), c("CMD", "INSTALL", "-l", shQuote(lib), shQuote(tarball)), "fallback-install")
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "fallback-runtime")

  # Development mode no longer requires the tool either.
  install <- file.path(root, "install.R")
  writeLines(sprintf('devtools::install(%s, build = TRUE, dependencies = FALSE, upgrade = FALSE, reload = FALSE, quiet = FALSE)',
                     deparse(pkg)), install)
  output <- withr::with_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = "dev"),
    run(file.path(R.home("bin"), "Rscript"), shQuote(install), "fallback-dev-install"))
  expect_match(output, "Staged 2 path dependencies under src/rust/vendor without cargo-revendor", fixed = TRUE)
  expect_identical(tools::md5sum(manifest), before)
  expect_false(file.exists(file.path(pkg, "inst/vendor.tar.xz")))
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "fallback-dev-runtime")

  # Plain R CMD build skips bootstrap.R; running it first replaces the dev staging.
  run(file.path(R.home("bin"), "Rscript"), "bootstrap.R", "fallback-bootstrap")
  run(file.path(R.home("bin"), "R"), c("CMD", "build", "--no-build-vignettes", "--no-manual", "."),
      "fallback-r-cmd-build")
  plain <- list.files(pkg, "^fallbackprobe_.*[.]tar[.]gz$", full.names = TRUE)
  expect_length(plain, 1L)
  entries <- utils::untar(plain, list = TRUE)
  expect_true("fallbackprobe/src/rust/vendor/innervalue-0.1.0/Cargo.toml" %in% entries)
  expect_false(any(grepl("dev-bootstrap.rds|Cargo.toml.dev|dev-vendor-backup", entries)))
})
