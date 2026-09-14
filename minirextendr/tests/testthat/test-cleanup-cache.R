# The cleanup opt-in must distinguish INSTALL from build/manual cleanup.

test_that("cleanup preserves targets only during opted-in installs", {
  skip_if_not(nzchar(Sys.which("sh")), "POSIX shell unavailable")
  for (template in c("rpkg", "monorepo/rpkg")) {
    cleanup <- system.file("templates", template, "cleanup", package = "minirextendr")
    for (install in c(FALSE, TRUE)) {
      for (keep in c("", "0", "1")) {
        pkg <- withr::local_tempdir()
        targets <- file.path(pkg, c("rust-target", "src/rust/target", "ra-target"))
        for (dir in c(targets, file.path(pkg, "src/rust/.cargo"))) {
          dir.create(dir, recursive = TRUE)
          writeLines("fixture", file.path(dir, "sentinel"))
        }
        writeLines("generated", file.path(pkg, "src/Makevars"))
        status <- withr::with_envvar(c(MINIEXTENDR_KEEP_TARGET = keep,
          R_INSTALL_PKG = if (install) "cacheprobe" else NA_character_),
          withr::with_dir(pkg, system2("sh", shQuote(cleanup))))
        expect_identical(status, 0L)
        expect_identical(dir.exists(targets), rep(install && keep == "1", 3L))
        expect_false(dir.exists(file.path(pkg, "src/rust/.cargo")))
        expect_false(file.exists(file.path(pkg, "src/Makevars")))
      }
    }
  }
})

test_that("real preclean and clean installs reuse the opted-in Cargo cache", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("cargo", "autoconf")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "unavailable"))
  }
  repo <- find_miniextendr_repo()
  withr::local_envvar(c(CARGO_TARGET_DIR = NA_character_, RUSTC_WRAPPER = "",
    CARGO_PROFILE = "dev", CARGO_TERM_COLOR = "never", R_INSTALL_PKG = NA_character_,
    MINIEXTENDR_KEEP_TARGET = NA_character_, MINIEXTENDR_FORCE_WRAPPER_GEN = NA_character_))
  root <- withr::local_tempdir()
  pkg <- file.path(root, "cacheprobe")
  suppressMessages(create_miniextendr_package(pkg, open = FALSE))
  suppressMessages(use_local_miniextendr(repo, path = pkg))
  lib <- file.path(root, "library")
  dir.create(lib)
  run_r <- function(args, keep = NA_character_) {
    result <- withr::with_envvar(c(MINIEXTENDR_KEEP_TARGET = keep),
      withr::with_dir(root, system2(file.path(R.home("bin"), "R"),
        shQuote(c("CMD", args)), stdout = TRUE, stderr = TRUE)))
    status <- attr(result, "status")
    if (is.null(status)) status <- 0L
    output <- paste(result, collapse = "\n")
    expect_identical(status, 0L, info = output)
    if (status != 0L) stop(output)
    output
  }
  install <- function(flags = character(), keep = NA_character_) {
    run_r(c("INSTALL", flags, "-l", lib, pkg), keep)
  }
  install()
  target <- file.path(pkg, "rust-target")
  artifact <- list.files(file.path(target, "debug/deps"),
                         pattern = "^liblinkme-.*\\.rlib$", full.names = TRUE)
  expect_length(artifact, 1L)
  before <- file.info(artifact)$mtime
  writeLines("cache survives", file.path(target, "sentinel"))
  warm <- install(c("--preclean", "--clean"), "1")
  expect_true(file.exists(file.path(target, "sentinel")))
  expect_identical(file.info(artifact)$mtime, before)
  expect_false(grepl("Compiling linkme", warm, fixed = TRUE), info = warm)

  # Even with the opt-in, build cleans its copy and leaves the checkout cache.
  build <- run_r(c("build", "--no-manual", "--no-build-vignettes", pkg), "1")
  expect_false(grepl("Warning", build, fixed = TRUE), info = build)
  tarball <- list.files(root, "^cacheprobe_.*\\.tar\\.gz$", full.names = TRUE)
  expect_length(tarball, 1L)
  members <- utils::untar(tarball, list = TRUE)
  expect_false(any(grepl("(^|/)(rust-target|ra-target|target)(/|$)", members)))
  expect_true(file.exists(file.path(target, "sentinel")))

  cold <- install(c("--preclean", "--clean"))
  expect_match(cold, "Compiling linkme", fixed = TRUE)
  expect_false(dir.exists(target))
})
