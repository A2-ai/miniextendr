# Exercise the generated configure script, including the legitimate tarball
# paths that must remain usable beneath a repository's .git ancestor.
test_that("monorepo configure rejects leaked tarballs without blocking builds", {
  skip_on_os("windows")
  for (command in c("autoconf", "bash", "cargo", "rustc")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  template <- system.file("templates", "monorepo", "rpkg",
                          package = "minirextendr", mustWork = TRUE)
  scripts <- system.file("scripts", package = "minirextendr", mustWork = TRUE)

  check_case <- function(name, git = "directory", tarball = TRUE,
                         stamp = character(), bootstrap = NA_character_,
                         reject = FALSE) {
    root <- withr::local_tempdir()
    workspace <- file.path(root, "workspace with spaces")
    pkg <- file.path(workspace, "packages", "guardpkg")
    dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
    dir.create(file.path(pkg, "tools"))
    dir.create(file.path(pkg, "inst"))
    if (git == "directory") dir.create(file.path(workspace, ".git"))
    if (git == "file") {
      writeLines("gitdir: /worktree-metadata", file.path(workspace, ".git"))
    }
    writeLines(c("Package: guardpkg", "Version: 0.1.0", stamp),
               file.path(pkg, "DESCRIPTION"))
    writeLines(character(), file.path(pkg, "NAMESPACE"))
    writeLines(gsub("{{package}}", "guardpkg",
                    readLines(file.path(template, "configure.ac")), fixed = TRUE),
               file.path(pkg, "configure.ac"))
    for (file in c("Makevars.in", "win.def.in")) {
      file.copy(file.path(template, file), file.path(pkg, "src", file))
    }
    file.copy(file.path(template, "tools", "lock-shape-check.R"),
              file.path(pkg, "tools", "lock-shape-check.R"))
    for (file in c("config.guess", "config.sub")) {
      file.copy(file.path(scripts, file), file.path(pkg, "tools", file))
    }
    # A dependency-free crate with an existing lockfile keeps configure fully
    # offline without substituting any of its tool discovery or build commands.
    writeLines(c("[package]", 'name = "guardpkg"', 'version = "0.1.0"',
                 'edition = "2024"', "[workspace]", "[lib]", 'path = "lib.rs"'),
               file.path(pkg, "src", "rust", "Cargo.toml"))
    writeLines(c("version = 4", "[[package]]", 'name = "guardpkg"',
                 'version = "0.1.0"'), file.path(pkg, "src", "rust", "Cargo.lock"))
    writeLines("pub fn value() -> i32 { 1 }", file.path(pkg, "src", "rust", "lib.rs"))
    sources <- file.path(pkg, "src", "rust", c("Cargo.toml", "Cargo.lock", "lib.rs"))
    before <- tools::md5sum(sources)
    withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP = bootstrap, R_HOME = R.home(),
                         CARGO_FEATURES = "", CARGO_PROFILE = "release",
                         CARGO_TARGET_DIR = NA, CARGO_BUILD_TARGET = NA,
                         RUST_TOOLCHAIN = NA, CC = NA, COPYFILE_DISABLE = "1"))
    archive <- file.path(pkg, "inst", "vendor.tar.xz")
    if (tarball) {
      payload <- file.path(root, "payload")
      dir.create(file.path(payload, "vendor"), recursive = TRUE)
      writeLines("vendor payload", file.path(payload, "vendor", "README"))
      withr::with_dir(payload, {
        utils::tar(archive, files = "vendor", compression = "xz", tar = "internal")
      })
      archive_before <- tools::md5sum(archive)
    }
    log <- file.path(root, "autoconf.log")
    generated <- withr::with_dir(pkg, {
      system2("autoconf", stdout = log, stderr = log)
    })
    expect_identical(generated, 0L, info = paste(readLines(log), collapse = "\n"))
    log <- file.path(root, "configure.log")
    configured <- withr::with_dir(pkg, {
      system2("bash", "./configure", stdout = log, stderr = log)
    })
    output <- paste(readLines(log), collapse = "\n")
    expect_identical(configured, if (reject) 1L else 0L,
                     info = paste(name, output))
    if (reject) {
      expect_match(output, "leaked vendor tarball detected", fixed = TRUE)
      expect_match(output, "minirextendr::miniextendr_doctor", fixed = TRUE)
      expect_false(file.exists(file.path(pkg, "src", "Makevars")))
      expect_false(file.exists(file.path(pkg, "src", "rust", ".cargo", "config.toml")))
      expect_false(dir.exists(file.path(pkg, "vendor")))
    } else {
      mode <- if (tarball) "tarball install (offline, vendored)" else "source install (cargo network)"
      expect_match(output, mode, fixed = TRUE)
      expect_true(file.exists(file.path(pkg, "src", "Makevars")))
      if (tarball) {
        expect_identical(readLines(file.path(pkg, "vendor", "README")), "vendor payload")
      }
    }
    expect_identical(tools::md5sum(sources), before)
    if (tarball) expect_identical(tools::md5sum(archive), archive_before)
  }

  check_case("source", tarball = FALSE)
  check_case("leaked tarball", reject = TRUE)
  check_case("worktree leaked tarball", git = "file", reject = TRUE)
  check_case("built tarball under repository", stamp = "Packaged: 2026-09-08")
  check_case("installed package under repository", stamp = "Built: R 4.6.1")
  check_case("bootstrap", bootstrap = "1")
  check_case("set but empty bootstrap", bootstrap = "")
  check_case("tarball outside repository", git = "none")
})
