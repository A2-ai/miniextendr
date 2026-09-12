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
