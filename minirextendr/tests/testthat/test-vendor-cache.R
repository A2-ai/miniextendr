vendor_cache_helper <- function() {
  env <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/vendor-cache.R",
                         package = "minirextendr"), envir = env)
  env
}

test_that("vendor cache entries follow archive contents and preserve source manifests", {
  helper <- vendor_cache_helper()
  root <- withr::local_tempdir()
  pkg <- file.path(root, "pkg")
  stage <- file.path(root, "stage")
  cache <- file.path(root, "cache")
  dir.create(file.path(pkg, "src/rust"), recursive = TRUE)
  dir.create(file.path(pkg, "inst"))
  dir.create(file.path(stage, "vendor/core/src"), recursive = TRUE)
  writeLines('[package]\nname="core"\nversion="0.1.0"',
             file.path(stage, "vendor/core/Cargo.toml"))
  writeLines("pub fn value() -> i32 { 7 }", file.path(stage, "vendor/core/src/lib.rs"))
  manifest <- file.path(pkg, "src/rust/Cargo.toml")
  writeLines(c('[dependencies]', 'renamed = { package = "core", version = "*" }',
               '[patch.crates-io]', 'core = { path = "../../vendor/core" }',
               '[profile.dev]', 'opt-level = 1'), manifest)
  writeLines("version = 4", file.path(pkg, "src/rust/Cargo.lock"))
  sources <- c(manifest, file.path(pkg, "src/rust/Cargo.lock"))
  before <- tools::md5sum(sources)
  archive <- file.path(pkg, "inst/vendor.tar.xz")
  pack <- function() withr::with_dir(stage,
    utils::tar(archive, files = "vendor", compression = "xz", tar = "internal"))
  pack()
  first <- helper$prepare_vendor_cache(pkg, cache)
  expect_true(dir.exists(first))
  expect_false(dir.exists(file.path(pkg, "vendor")))
  stamp <- file.info(file.path(first, "core/src/lib.rs"))$mtime
  expect_identical(helper$prepare_vendor_cache(pkg, cache), first)
  expect_equal(file.info(file.path(first, "core/src/lib.rs"))$mtime, stamp)
  config <- capture.output(helper$write_vendor_patches(pkg, first))
  expect_true(any(grepl(paste0(first, "/core"), config, fixed = TRUE)))
  expect_false(any(grepl("opt-level", config, fixed = TRUE)))
  expect_identical(tools::md5sum(sources), before)
  # Unrelated caller-owned files and older entries must survive a new archive.
  writeLines("keep me", file.path(cache, "caller-owned"))
  writeLines("pub fn value() -> i32 { 9 }", file.path(stage, "vendor/core/src/lib.rs"))
  pack()
  second <- helper$prepare_vendor_cache(pkg, cache)
  expect_false(identical(first, second))
  expect_match(readLines(file.path(second, "core/src/lib.rs")), "9", fixed = TRUE)
  expect_match(readLines(file.path(first, "core/src/lib.rs")), "7", fixed = TRUE)
  expect_identical(readLines(file.path(cache, "caller-owned")), "keep me")
  expect_identical(tools::md5sum(sources), before)
})

test_that("tarball cleanup preserves caller caches and removes default build directories", {
  skip_on_os("windows")
  skip_if_not(nzchar(Sys.which("make")), "make not available")
  template <- readLines(system.file("templates/rpkg/Makevars.in", package = "minirextendr"))
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  for (owned in c(FALSE, TRUE)) {
    pkg <- file.path(root, if (owned) "owned" else "default")
    target <- file.path(pkg, "rust-target")
    vendor <- file.path(pkg, "vendor", "hash", "vendor")
    dirs <- c(target, vendor, file.path(pkg, "src/rust/.cargo"))
    for (dir in dirs) dir.create(dir, recursive = TRUE)
    writeLines("target", file.path(target, "sentinel"))
    writeLines("vendor", file.path(vendor, "sentinel"))
    vars <- c(ABS_TOP_SRCDIR = pkg, ABS_RPKG_SRCDIR = file.path(pkg, "src"),
              CARGO_TARGET_DIR = target, VENDOR_OUT = vendor,
              CARGO_TARGET_DIR_USER_SET = tolower(as.character(owned)),
              VENDOR_OUT_USER_SET = tolower(as.character(owned)),
              IS_TARBALL_INSTALL = "true", PACKAGE_NAME = "probe")
    rendered <- template
    for (name in names(vars)) {
      rendered <- gsub(paste0("@", name, "@"), vars[[name]], rendered, fixed = TRUE)
    }
    makevars <- file.path(pkg, "Makevars")
    writeLines(rendered, makevars)
    log <- file.path(root, paste0("cleanup-", owned, ".log"))
    status <- withr::with_dir(pkg, system2("make", c(
      "-f", "Makevars", "-o", "probe.so", "-o", shQuote(file.path(pkg, "R/probe-wrappers.R")),
      "SHLIB=probe.so", "all"), stdout = log, stderr = log))
    expect_identical(status, 0L, info = paste(readLines(log), collapse = "\n"))
    expect_identical(file.exists(file.path(target, "sentinel")), owned)
    expect_identical(file.exists(file.path(vendor, "sentinel")), owned)
    expect_false(dir.exists(file.path(pkg, "src/rust/.cargo")))
  }
})

test_that("two tarball installs share vendor and Cargo caches and rebuild only the package", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "cargo-revendor", "make")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  repo <- find_miniextendr_repo()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "cacheprobe")
  core <- file.path(root, "core")
  library <- file.path(root, "library")
  cache <- file.path(root, "vendor-cache")
  target <- file.path(root, "cargo-target")
  dir.create(library)
  dir.create(file.path(core, "src"), recursive = TRUE)
  writeLines(c('[package]', 'name = "cachevalue"', 'version = "0.1.0"',
               'edition = "2024"'), file.path(core, "Cargo.toml"))
  writeLines('pub fn value() -> i32 { 7 }', file.path(core, "src/lib.rs"))
  writeLines('fn main() { println!("cargo:rerun-if-changed=build.rs"); }',
             file.path(core, "build.rs"))
  withr::local_envvar(c(
    R_LIBS = paste(.libPaths(), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = "", CARGO_BUILD_TARGET = NA,
    CARGO_TARGET_DIR = target, VENDOR_OUT = NA,
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA
  ))
  suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
  writeLines(c('[package]', 'name = "cacheprobe"', 'version = "0.1.0"',
    'edition = "2024"', 'build = false', '[workspace]', '[lib]', 'path = "lib.rs"',
    'crate-type = ["staticlib"]', '[dependencies]',
    sprintf('miniextendr-api = { path = "%s/miniextendr-api" }', repo),
    sprintf('cachevalue = { path = "%s" }', core),
    '[profile.dev]', 'codegen-units = 1'), file.path(pkg, "src/rust/Cargo.toml"))
  writeLines(c('use miniextendr_api::miniextendr;',
    'miniextendr_api::miniextendr_init!();', '#[miniextendr]',
    'pub fn value() -> i32 { cachevalue::value() }'), file.path(pkg, "src/rust/lib.rs"))
  writeLines('useDynLib(cacheprobe, .registration = TRUE)', file.path(pkg, "NAMESPACE"))
  run <- function(command, args, label, wd = root) {
    log <- file.path(root, paste0(label, ".log"))
    elapsed <- system.time(status <- withr::with_dir(wd,
      system2(command, args, stdout = log, stderr = log)))[["elapsed"]]
    output <- paste(readLines(log, warn = FALSE), collapse = "\n")
    expect_identical(status, 0L, info = output)
    if (status != 0L) stop(output, call. = FALSE)
    list(output = output, elapsed = elapsed)
  }
  run("autoconf", character(), "autoconf", pkg)
  run("bash", "./configure", "configure", pkg)
  config <- readLines(file.path(pkg, "src/rust/.cargo/config.toml"))
  expect_true(any(grepl(paste0('target-dir = "', target, '"'), config, fixed = TRUE)))
  install <- function(path, label) run(file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "--no-byte-compile", "-l",
      shQuote(library), shQuote(path)), label)
  install(pkg, "source-install")
  tarball <- devtools::build(pkg, path = root, binary = FALSE, vignettes = FALSE, manual = FALSE)
  expect_true("cacheprobe/tools/vendor-cache.R" %in% utils::untar(tarball, list = TRUE))
  dir.create(cache)
  writeLines("keep me", file.path(cache, "caller-owned"))
  extracted <- file.path(root, c("first", "second"))
  for (directory in extracted) {
    dir.create(directory)
    utils::untar(tarball, exdir = directory)
  }
  packages <- file.path(extracted, "cacheprobe")
  sources <- unlist(lapply(packages, function(package) file.path(package, "src/rust",
    c("Cargo.toml", "Cargo.lock", "lib.rs"))), use.names = FALSE)
  before <- tools::md5sum(sources)
  withr::local_envvar(c(VENDOR_OUT = cache))
  first <- install(packages[[1L]], "first-shared-install")
  expect_true(dir.exists(target))
  second <- install(packages[[2L]], "second-shared-install")
  expect_identical(tools::md5sum(sources), before)
  lines <- strsplit(second$output, "\n", fixed = TRUE)[[1L]]
  compiled <- grep("^[[:space:]]*Compiling ", lines, value = TRUE)
  expect_identical(length(compiled), 1L, info = second$output)
  expect_match(compiled, "Compiling cacheprobe ", fixed = TRUE)
  expect_false(grepl("warning:", second$output, fixed = TRUE), info = second$output)
  expect_match(second$output, "using pre-shipped", fixed = TRUE)
  expect_true(dir.exists(target))
  expect_identical(readLines(file.path(cache, "caller-owned")), "keep me")
  probe <- file.path(root, "probe.R")
  writeLines(sprintf('stopifnot(identical(loadNamespace("cacheprobe", lib.loc = %s)$value(), 7L))',
                     deparse(library)), probe)
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "verify-runtime")
  message(sprintf("Shared vendor install times: first %.2fs; second %.2fs (%d crate rebuilt)",
                  first$elapsed, second$elapsed, length(compiled)))
  # The same newly frozen tarball still installs without either cache opt-in.
  ordinary <- withr::with_envvar(c(VENDOR_OUT = NA, CARGO_TARGET_DIR = NA),
    install(tarball, "ordinary-tarball-install"))
  expect_match(ordinary$output, "using pre-shipped", fixed = TRUE)
  expect_false(grepl("warning:", ordinary$output, fixed = TRUE), info = ordinary$output)
  run(file.path(R.home("bin"), "Rscript"), shQuote(probe), "verify-ordinary-runtime")
  expect_true(dir.exists(target))
  expect_identical(readLines(file.path(cache, "caller-owned")), "keep me")
})
