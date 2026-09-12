# A record binds the generated R bytes to content, not filesystem timestamps.
freshness_helper <- function() {
  env <- new.env(parent = baseenv())
  sys.source(system.file("templates/rpkg/tools/wrapper-freshness.R",
                         package = "minirextendr"), envir = env)
  env
}

test_that("wrapper provenance survives copies and detects changed inputs", {
  helper <- freshness_helper()
  root <- withr::local_tempdir()
  dir.create(file.path(root, "src/rust"), recursive = TRUE)
  dir.create(file.path(root, "tools"))
  dir.create(file.path(root, "R"))
  wrappers <- file.path(root, "R/probe-wrappers.R")
  writeLines("probe <- function() 1L", wrappers)
  writeLines("pub fn probe() -> i32 { 1 }", file.path(root, "src/rust/lib.rs"))
  writeLines('[package]\nname = "probe"', file.path(root, "src/rust/Cargo.toml"))
  writeLines("version = 4", file.path(root, "src/rust/Cargo.lock"))
  writeLines(c("CARGO_PROFILE = release", "CARGO_FEATURES_FLAG ="),
             file.path(root, "src/Makevars"))
  expect_false(helper$wrappers_current(root, wrappers))
  helper$write_wrapper_record(root, wrappers)
  expect_true(helper$wrappers_current(root, wrappers))
  before <- file.info(wrappers)$mtime
  Sys.setFileTime(file.path(root, "src/rust/lib.rs"), Sys.time() + 1000)
  expect_true(helper$wrappers_current(root, wrappers))
  expect_equal(file.info(wrappers)$mtime, before)

  copy <- file.path(withr::local_tempdir(), "copy")
  fs::dir_copy(root, copy)
  expect_true(helper$wrappers_current(copy, file.path(copy, "R/probe-wrappers.R")))
  # Generated output and build directories must not become recursive inputs.
  writeLines("// generated snapshot", file.path(root, "src/rust/wasm_registry.rs"))
  dir.create(file.path(root, "src/rust/target"))
  writeLines("// build output", file.path(root, "src/rust/target/generated.rs"))
  expect_true(helper$wrappers_current(root, wrappers))

  for (path in c("src/rust/lib.rs", "src/rust/Cargo.toml", "src/rust/Cargo.lock",
                 "R/probe-wrappers.R", "src/Makevars")) {
    full <- file.path(root, path)
    old <- readBin(full, "raw", file.info(full)$size)
    if (path == "src/Makevars") {
      writeLines("CARGO_PROFILE = dev", full)
    } else {
      cat("\n# changed\n", file = full, append = TRUE)
    }
    expect_false(helper$wrappers_current(root, wrappers), info = path)
    writeBin(old, full)
    expect_true(helper$wrappers_current(root, wrappers), info = path)
  }
  file.rename(file.path(root, "src/rust/lib.rs"), file.path(root, "src/rust/renamed.rs"))
  expect_false(helper$wrappers_current(root, wrappers))
  file.rename(file.path(root, "src/rust/renamed.rs"), file.path(root, "src/rust/lib.rs"))
  writeLines("not an RDS", helper$wrapper_record_path(root))
  expect_false(helper$wrappers_current(root, wrappers))
  saveRDS("wrong shape", helper$wrapper_record_path(root))
  expect_false(helper$wrappers_current(root, wrappers))
})

test_that("a stale S3 tarball fails before R accepts a dangling registration", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "cargo-revendor", "make")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  repo <- find_miniextendr_repo()
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  pkg <- file.path(root, "freshprobe")
  library <- file.path(root, "library")
  dir.create(library)
  withr::local_envvar(c(
    R_LIBS = paste(.libPaths(), collapse = .Platform$path.sep),
    CARGO_PROFILE = "dev", CARGO_FEATURES = "", CARGO_BUILD_TARGET = NA,
    CARGO_TARGET_DIR = file.path(root, "target"),
    MINIEXTENDR_FORCE_WRAPPER_GEN = NA, ROXYGEN_PKG = NA
  ))
  suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
  writeLines(c("[package]", 'name = "freshprobe"', 'version = "0.1.0"',
    'edition = "2024"', 'build = false', '[workspace]', '[lib]', 'path = "lib.rs"',
    'crate-type = ["staticlib"]', '[dependencies]',
    sprintf('miniextendr-api = { path = "%s/miniextendr-api" }', repo),
    '[profile.dev]', 'codegen-units = 1'), file.path(pkg, "src/rust/Cargo.toml"))
  rust <- file.path(pkg, "src/rust/lib.rs")
  writeLines(c('use miniextendr_api::miniextendr;',
    'miniextendr_api::miniextendr_init!();', '#[miniextendr]',
    'pub fn probe() -> i32 { 1 }'), rust)
  writeLines('useDynLib(freshprobe, .registration = TRUE)', file.path(pkg, "NAMESPACE"))
  run <- function(command, args, label, wd = root) {
    log <- file.path(root, paste0(label, ".log"))
    status <- withr::with_dir(wd, system2(command, args, stdout = log, stderr = log))
    list(status = status, output = paste(readLines(log, warn = FALSE), collapse = "\n"))
  }
  result <- run("autoconf", character(), "autoconf", pkg)
  expect_identical(result$status, 0L, info = result$output)
  result <- run("bash", "./configure", "configure", pkg)
  expect_identical(result$status, 0L, info = result$output)
  install <- function(path, label) run(file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "--no-byte-compile", "-l",
      shQuote(library), shQuote(path)), label)
  result <- install(pkg, "source-install")
  expect_identical(result$status, 0L, info = result$output)
  helper <- freshness_helper()
  wrappers <- file.path(pkg, "R/freshprobe-wrappers.R")
  expect_true(helper$wrappers_current(pkg, wrappers))
  # Run the actual Makevars target against the source build, pretending only
  # that the DLL is newer. A matching record must skip dyn.load; forcing it
  # must still enter the generation branch. Neither runs the all: cleanup.
  make_args <- c("-f", "Makevars", "-o", "freshprobe.so", "-W", "freshprobe.so",
                 shQuote(wrappers), "SHLIB=freshprobe.so", "IS_TARBALL_INSTALL=true",
                 paste0("R_HOME=", shQuote(R.home())))
  result <- run("make", make_args, "verified-fast-path", file.path(pkg, "src"))
  expect_identical(result$status, 0L, info = result$output)
  expect_match(result$output, "using pre-shipped", fixed = TRUE)
  result <- withr::with_envvar(c(MINIEXTENDR_FORCE_WRAPPER_GEN = "1"),
    run("make", make_args, "forced-generation", file.path(pkg, "src")))
  expect_identical(result$status, 0L, info = result$output)
  expect_match(result$output, "MINIEXTENDR_FORCE_WRAPPER_GEN set", fixed = TRUE)
  writeLines(character(), wrappers)
  result <- run("make", make_args, "empty-wrapper-fallback", file.path(pkg, "src"))
  expect_identical(result$status, 0L, info = result$output)
  expect_true(helper$wrappers_current(pkg, wrappers))

  cat('\n#[miniextendr(s3(generic = "summary", class = "freshprobe"))]\npub fn summary_probe(object: i32) -> i32 { object + 10 }\n',
      file = rust, append = TRUE)
  writeLines(c('useDynLib(freshprobe, .registration = TRUE)', 'S3method(summary,freshprobe)'),
             file.path(pkg, "NAMESPACE"))
  expect_false(helper$wrappers_current(pkg, wrappers))
  old <- unname(tools::md5sum(wrappers))
  # Exercise the actual pkgbuild/bootstrap pipeline and its copy semantics.
  tarball <- devtools::build(pkg, path = root, binary = FALSE, vignettes = FALSE, manual = FALSE)
  expect_identical(unname(tools::md5sum(wrappers)), old)
  result <- install(tarball, "stale-tarball-install")
  expect_false(identical(result$status, 0L), info = result$output)
  expect_match(result$output, "Stale pre-shipped wrappers:", fixed = TRUE)
  expect_match(result$output, "freshprobe-wrappers.R", fixed = TRUE)
  expect_match(result$output, "minirextendr::miniextendr_build()", fixed = TRUE)
  expect_false(grepl("was declared in NAMESPACE but not found", result$output, fixed = TRUE))
})
