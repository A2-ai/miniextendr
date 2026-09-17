# Real Cargo cache switches must reach both the installed library and wrappers.
# The probe has two tiny feature variants and reuses cached Cargo outputs.
test_that("cached feature switches relink libraries and regenerate wrappers", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_no_local_repo()
  for (command in c("autoconf", "bash", "cargo", "make", "git")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  repo <- find_miniextendr_repo()
  # Canonical paths also exercise nested-project selection on macOS, where
  # /var and /private/var aliases otherwise hide the active-parent case.
  root <- normalizePath(withr::local_tempdir(), winslash = "/")
  target <- file.path(root, "cargo-target")
  library <- file.path(root, "library")
  dir.create(library)
  usethis::local_project(root, force = TRUE, setwd = FALSE, quiet = TRUE)
  withr::local_envvar(c(
    R_LIBS = paste(.libPaths(), collapse = .Platform$path.sep),
    CARGO_TARGET_DIR = target, CARGO_PROFILE = "dev", CARGO_FEATURES = "",
    CARGO_BUILD_TARGET = NA, CARGO_BUILD_STD_FLAG = NA, RUST_TOOLCHAIN = NA,
    MINIEXTENDR_BOOTSTRAP = NA, MINIEXTENDR_FORCE_WRAPPER_GEN = NA,
    ROXYGEN_PKG = NA
  ))
  sources <- c(
    standalone = system.file("templates/rpkg/Makevars.in", package = "minirextendr"),
    monorepo = system.file("templates/monorepo/rpkg/Makevars.in", package = "minirextendr"),
    rpkg = file.path(repo, "rpkg/src/Makevars.in"),
    model = file.path(repo, "tests/model_project/src/Makevars.in"),
    producer = file.path(repo, "tests/cross-package/producer.pkg/src/Makevars.in"),
    consumer = file.path(repo, "tests/cross-package/consumer.pkg/src/Makevars.in")
  )

  run <- function(command, args, wd, label) {
    log <- file.path(root, paste0(label, ".log"))
    status <- withr::with_dir(wd, system2(command, args, stdout = log, stderr = log))
    output <- paste(readLines(log, warn = FALSE), collapse = "\n")
    expect_identical(status, 0L, info = output)
    if (status != 0L) stop(output, call. = FALSE)
    output
  }

  for (layout in names(sources)) {
    pkg <- file.path(root, paste0("cache", layout))
    package <- basename(pkg)
    suppressMessages(create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE))
    # A real source checkout prevents bootstrap from creating a vendor tarball,
    # whose intentional wrapper-generation skip would mask this regression.
    run("git", c("init", "--quiet"), pkg, paste0(layout, "-git-init"))
    file.copy(sources[[layout]], file.path(pkg, "src/Makevars.in"), overwrite = TRUE)
    # Seed the registration directive that roxygen normally supplies before an
    # end-user build; exported R names are unnecessary for namespace probes.
    writeLines(sprintf("useDynLib(%s, .registration = TRUE)", package),
               file.path(pkg, "NAMESPACE"))
    # Local path dependency avoids fetching the framework or its git metadata.
    # Disable the scaffold's lint build script; this is a minimal runtime probe.
    writeLines(c(
      "[package]", sprintf('name = "%s"', package), 'version = "0.1.0"',
      'edition = "2024"', "build = false", "[workspace]", "[lib]",
      'path = "lib.rs"', 'crate-type = ["staticlib"]',
      "[features]", "default = []", "alternate = []", "[dependencies]",
      sprintf('miniextendr-api = { path = "%s" }',
              normalizePath(file.path(repo, "miniextendr-api"), winslash = "/")),
      "[profile.dev]", "codegen-units = 1"
    ), file.path(pkg, "src/rust/Cargo.toml"))
    writeLines(c(
      "use miniextendr_api::miniextendr;",
      "miniextendr_api::miniextendr_init!();",
      "#[miniextendr]",
      'pub fn cache_enabled() -> bool { cfg!(feature = "alternate") }',
      '#[cfg(not(feature = "alternate"))]', "#[miniextendr]",
      "pub fn value(logical_value: bool) -> bool { logical_value }",
      '#[cfg(feature = "alternate")]', "#[miniextendr]",
      "pub fn value(integer_value: i32) -> i32 { integer_value }"
    ), file.path(pkg, "src/rust/lib.rs"))
    run("autoconf", character(), pkg, paste0(layout, "-autoconf"))

    install <- function(alternate, label) {
      withr::with_envvar(c(CARGO_FEATURES = if (alternate) "alternate" else ""), {
        output <- run(file.path(R.home("bin"), "R"),
                      c("CMD", "INSTALL", "--no-multiarch", "--no-byte-compile",
                        "-l", shQuote(library), shQuote(pkg)), root, label)
        expect_match(output, "source install (cargo network)", fixed = TRUE)
        output
      })
    }
    verify <- function(alternate, label) {
      probe <- file.path(root, "verify.R")
      # Each fresh R process loads the actual installed DLL and wrapper file.
      writeLines(c(
        sprintf("ns <- loadNamespace(%s, lib.loc = %s)",
                deparse(package), deparse(library)),
        "print(list(feature = ns$cache_enabled(), formals = names(formals(ns$value))))",
        sprintf("stopifnot(identical(ns$cache_enabled(), %s))", alternate),
        sprintf("stopifnot(identical(names(formals(ns$value)), %s))",
                deparse(if (alternate) "integer_value" else "logical_value")),
        if (alternate) "stopifnot(identical(ns$value(7L), 7L))"
        else "stopifnot(identical(ns$value(TRUE), TRUE))"
      ), probe)
      run(file.path(R.home("bin"), "Rscript"), shQuote(probe), root, label)
    }
    install(FALSE, paste0(layout, "-default"))
    verify(FALSE, paste0(layout, "-verify-default"))
    archive <- file.path(target, "debug", paste0("lib", package, ".a"))
    original_archive_time <- file.info(archive)$mtime
    changed_output <- install(TRUE, paste0(layout, "-alternate"))
    expect_match(changed_output, paste0("NOTE: ", package, "-wrappers.R changed"),
                 fixed = TRUE)
    verify(TRUE, paste0(layout, "-verify-alternate"))
    install(FALSE, paste0(layout, "-cached-default"))
    # Establish that this is an old cached archive, not a new compilation.
    expect_equal(file.info(archive)$mtime, original_archive_time)
    verify(FALSE, paste0(layout, "-verify-cached-default"))

    dll <- file.path(pkg, "src", paste0(package, .Platform$dynlib.ext))
    linked_time <- file.info(dll)$mtime
    install(FALSE, paste0(layout, "-noop"))
    expect_equal(file.info(dll)$mtime, linked_time,
                 info = paste(layout, "no-op install relinked the shared library"))
    verify(FALSE, paste0(layout, "-verify-noop"))

    # A body-only edit relinks the DLL without changing generated R code (#1530).
    # The writer deliberately keeps identical content; make must still settle.
    wrappers <- file.path(pkg, "R", paste0(package, "-wrappers.R"))
    original_wrappers <- readLines(wrappers, warn = FALSE)
    rust_source <- file.path(pkg, "src/rust/lib.rs")
    code <- readLines(rust_source, warn = FALSE)
    code <- sub('pub fn cache_enabled() -> bool { cfg!(feature = "alternate") }',
                'pub fn cache_enabled() -> bool { let enabled = cfg!(feature = "alternate"); enabled }',
                code, fixed = TRUE)
    Sys.sleep(1) # Distinguish timestamps even on second-resolution filesystems.
    writeLines(code, rust_source)
    body_output <- install(FALSE, paste0(layout, "-body-edit"))
    expect_gt(as.numeric(file.info(dll)$mtime), as.numeric(linked_time))
    expect_match(body_output, "Checking R wrappers", fixed = TRUE)
    expect_false(grepl(paste0("NOTE: ", package, "-wrappers.R changed"),
                       body_output, fixed = TRUE))
    expect_identical(readLines(wrappers, warn = FALSE), original_wrappers)
    expect_gte(as.numeric(file.info(wrappers)$mtime), as.numeric(file.info(dll)$mtime))
    settled_time <- file.info(wrappers)$mtime
    linked_time <- file.info(dll)$mtime
    for (attempt in 1:3) {
      output <- install(FALSE, paste0(layout, "-settled-", attempt))
      expect_false(grepl("Checking R wrappers", output, fixed = TRUE), info = output)
      expect_equal(file.info(wrappers)$mtime, settled_time)
      expect_equal(file.info(dll)$mtime, linked_time)
    }
    verify(FALSE, paste0(layout, "-verify-body-edit"))

    if (layout == "standalone") {
      original_archive_time <- file.info(archive)$mtime
      # A record kept inside the selected Cargo target would miss the return
      # switch: that directory still contains its old default record/archive.
      withr::with_envvar(c(CARGO_TARGET_DIR = file.path(root, "other-target")), {
        install(TRUE, "other-target-alternate")
        verify(TRUE, "verify-other-target")
      })
      install(FALSE, "original-target-default")
      expect_equal(file.info(archive)$mtime, original_archive_time)
      verify(FALSE, "verify-original-target")
      linked_time <- file.info(dll)$mtime
      install(FALSE, "original-target-noop")
      expect_equal(file.info(dll)$mtime, linked_time)
      verify(FALSE, "verify-original-target-noop")
    }
  }
})
