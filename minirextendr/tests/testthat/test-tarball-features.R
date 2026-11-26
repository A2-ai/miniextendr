# A release tarball records the Cargo features its wrappers were generated
# with (tools/cargo-features, written by tools/write-wrappers.R), and its
# configure builds with them when CARGO_FEATURES is unset (#1664). Built where
# the `extra` rule holds and installed where it does not, the tarball still
# compiles `extra_fn()`, which its NAMESPACE exports. An explicit
# CARGO_FEATURES overrides the record, and write-wrappers.R stops that install
# when it changes the wrappers (#1665).
test_that("a release tarball installs with the Cargo features it recorded", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "cargo-revendor", "make")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  repo <- find_miniextendr_repo()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "featrec")
  library <- file.path(root, "library")
  dir.create(library)
  withr::local_envvar(c(
    R_LIBS = paste(.libPaths(), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = NA, CARGO_BUILD_TARGET = NA,
    CARGO_TERM_COLOR = "never", CARGO_TARGET_DIR = file.path(root, "cargo-target"),
    # The tarball installs share one unpacked vendor tree, so the second one
    # recompiles only the package.
    VENDOR_OUT = file.path(root, "vendor-cache"),
    MX_FEATREC_EXTRA = "true"
  ))
  suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
  writeLines(c('[package]', 'name = "featrec"', 'version = "0.1.0"',
    'edition = "2024"', 'build = false', '[workspace]', '[lib]', 'path = "lib.rs"',
    'crate-type = ["staticlib"]', '[features]', 'default = []', 'extra = []',
    '[dependencies]',
    sprintf('miniextendr-api = { path = "%s/miniextendr-api" }', repo),
    '[profile.dev]', 'codegen-units = 1'), file.path(pkg, "src/rust/Cargo.toml"))
  writeLines(c('use miniextendr_api::miniextendr;',
    'miniextendr_api::miniextendr_init!();',
    '#[miniextendr]', 'pub fn base_fn() -> i32 { 1 }',
    '#[cfg(feature = "extra")]', '#[miniextendr]', 'pub fn extra_fn() -> i32 { 2 }'),
    file.path(pkg, "src/rust/lib.rs"))
  writeLines(c('useDynLib(featrec, .registration = TRUE)', 'export(base_fn)',
               'export(extra_fn)'), file.path(pkg, "NAMESPACE"))
  suppressMessages({
    use_configure_feature_detection(path = pkg)
    add_feature_rule(
      "extra", detect = 'identical(Sys.getenv("MX_FEATREC_EXTRA"), "true")', path = pkg
    )
  })

  run <- function(command, args, label, wd = root) {
    log <- file.path(root, paste0(label, ".log"))
    status <- withr::with_dir(wd, system2(command, args, stdout = log, stderr = log))
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"))
  }
  install <- function(path, label) run(file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "--no-byte-compile", "-l",
      shQuote(library), shQuote(path)), label)
  extra_fn <- function(label) {
    probe <- file.path(root, paste0(label, ".R"))
    writeLines(sprintf('cat(loadNamespace("featrec", lib.loc = %s)$extra_fn())',
                       deparse(library)), probe)
    run(file.path(R.home("bin"), "Rscript"), shQuote(probe), label)
  }

  expect_identical(run("autoconf", character(), "autoconf", pkg)$status, 0L)
  configured <- run("bash", "./configure", "configure", pkg)
  expect_match(configured$output, "Auto-detected features: extra", fixed = TRUE)
  source_install <- install(pkg, "source-install")
  expect_identical(source_install$status, 0L, info = source_install$output)
  record <- file.path(pkg, "tools", "cargo-features")
  expect_identical(utils::tail(readLines(record), 1L), "extra")

  suppressMessages(miniextendr_vendor(pkg))
  tarball <- devtools::build(pkg, path = root, binary = FALSE, vignettes = FALSE,
                             manual = FALSE, quiet = TRUE)
  expect_true("featrec/tools/cargo-features" %in% utils::untar(tarball, list = TRUE))

  # The rule no longer holds, as on another machine.
  withr::local_envvar(c(MX_FEATREC_EXTRA = "false"))
  recorded <- install(tarball, "recorded-install")
  expect_identical(recorded$status, 0L, info = recorded$output)
  expect_match(recorded$output, "Cargo features recorded in the tarball: extra", fixed = TRUE)
  expect_false(grepl("Auto-detected features", recorded$output, fixed = TRUE))
  called <- extra_fn("call-extra")
  expect_identical(called$status, 0L, info = called$output)
  expect_identical(called$output, "2")

  overridden <- withr::with_envvar(c(CARGO_FEATURES = ""),
                                   install(tarball, "overridden-install"))
  expect_false(identical(overridden$status, 0L))
  expect_match(overridden$output, "different R wrappers than the featrec tarball ships",
               fixed = TRUE)
  expect_match(overridden$output,
               "Cargo features of this build: none. The tarball was built with: extra.",
               fixed = TRUE)
})
