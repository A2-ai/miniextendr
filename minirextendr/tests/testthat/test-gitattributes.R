# Exercise real scaffolds and Git's merge driver, including rerere (#1085).
attributes_git <- function(path, ..., status = 0L) {
  args <- c("-C", path,
            "-c", "user.name=Scaffold Test",
            "-c", "user.email=scaffold@example.invalid",
            "-c", "commit.gpgsign=false",
            "-c", "core.autocrlf=false",
            "-c", paste0("core.hooksPath=", file.path(path, ".empty-hooks")),
            "-c", paste0("core.attributesFile=", file.path(path, ".empty-attributes")),
            ...)
  run <- function() system2("git", shQuote(args), stdout = TRUE, stderr = TRUE)
  output <- if (status == 0L) run() else suppressWarnings(run())
  actual <- attr(output, "status")
  if (is.null(actual)) actual <- 0L
  expect_identical(actual, status, info = paste(output, collapse = "\n"))
  unname(as.character(output))
}

attributes_paths <- c("NAMESPACE", "R/attrpkg-wrappers.R", "configure", "man/add.Rd")
attributes_sources <- c("configure.ac", "R/handwritten.R", "src/rust/lib.rs", "README.md")

expect_generated_attributes <- function(path, prefix = "") {
  paths <- paste0(prefix, attributes_paths)
  expect_identical(attributes_git(path, "check-attr", "merge", "--", paths),
                   paste0(paths, ": merge: unset"))
  sources <- paste0(prefix, attributes_sources)
  expect_identical(attributes_git(path, "check-attr", "merge", "--", sources),
                   paste0(sources, ": merge: unspecified"))
}

test_that("standalone scaffolding adds merge rules without replacing user attributes", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  tmp <- withr::local_tempdir(pattern = "attributes standalone ")
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  attributes_git(tmp, "init", "-q")
  writeLines(c("Package: attrpkg", "Version: 0.0.1"), file.path(tmp, "DESCRIPTION"))
  custom <- c("# User attributes", "*.R text eol=lf", "*.csv diff")
  writeLines(custom, file.path(tmp, ".gitattributes"))

  suppressMessages(use_miniextendr(path = tmp, claude_skills = FALSE))
  expect_generated_attributes(tmp)
  after <- readLines(file.path(tmp, ".gitattributes"))
  expect_identical(after[seq_along(custom)], custom)
  expect_identical(attributes_git(tmp, "check-attr", "eol", "--", "R/attrpkg-wrappers.R"),
                   "R/attrpkg-wrappers.R: eol: lf")
  suppressMessages(use_miniextendr(path = tmp, claude_skills = FALSE))
  expect_identical(readLines(file.path(tmp, ".gitattributes")), after)
})

test_that("new and existing monorepos protect custom package subdirectories", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  for (new in c(TRUE, FALSE)) {
    tmp <- withr::local_tempdir(pattern = "attributes monorepo ")
    usethis::local_project(tmp, force = TRUE, setwd = FALSE)
    attributes_git(tmp, "init", "-q")
    writeLines("*.csv diff", file.path(tmp, ".gitattributes"))
    if (new) {
      suppressMessages(create_miniextendr_monorepo(
        tmp, package = "attrpkg", crate_name = "attr-core",
        rpkg_name = "r bindings", open = FALSE
      ))
    } else {
      # `use_miniextendr()` reads the workspace with `cargo metadata`, which
      # needs a library target to select, so give the crate a `src/lib.rs`.
      writeLines(c("[package]", 'name = "attr-core"', 'version = "0.1.0"',
                   'edition = "2024"'),
                 file.path(tmp, "Cargo.toml"))
      dir.create(file.path(tmp, "src"))
      writeLines("pub fn add(a: u64, b: u64) -> u64 { a + b }",
                 file.path(tmp, "src", "lib.rs"))
      suppressMessages(use_miniextendr(path = tmp, rpkg_name = "r bindings",
                                       claude_skills = FALSE))
    }
    expect_true(file.exists(file.path(tmp, "r bindings", ".gitattributes")))
    expect_generated_attributes(tmp, "r bindings/")
    # Probe a second package without its own attributes: the root template
    # must match nested paths, not just root-level R/ and man/ directories.
    expect_generated_attributes(tmp, "another/package/")
    expect_identical(readLines(file.path(tmp, ".gitattributes"))[1], "*.csv diff")
  }
})

test_that("upgrade adds missing merge rules and preserves existing rules on reruns", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir(pattern = "attributes upgrade ")
    usethis::local_project(tmp, force = TRUE, setwd = FALSE)
    attributes_git(tmp, "init", "-q")
    pkg <- if (monorepo) file.path(tmp, "custom-r") else tmp
    dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
    writeLines("[package]", file.path(pkg, "src", "rust", "Cargo.toml"))
    writeLines("# Previous scaffold", file.path(pkg, "src", "Makevars.in"))
    if (monorepo) writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
    writeLines(c("Package: attrpkg", "Version: 0.0.1"), file.path(pkg, "DESCRIPTION"))
    writeLines(c("CARGO_FEATURES", "CARGO_STATICLIB_NAME", "AC_CONFIG_AUX_DIR", "CARGO_TARGET_DIR"),
               file.path(pkg, "configure.ac"))
    custom <- c("# Keep custom rules", "*.R text eol=lf")
    if (monorepo) writeLines(custom, file.path(pkg, ".gitattributes"))

    suppressMessages(upgrade_miniextendr_package(path = tmp, autoconf = FALSE,
                                                allow_dirty = TRUE))
    expect_generated_attributes(tmp, if (monorepo) "custom-r/" else "")
    attr_file <- file.path(pkg, ".gitattributes")
    expect_true(file.exists(attr_file))
    if (!file.exists(attr_file)) next
    first <- readLines(attr_file)
    if (monorepo) expect_identical(first[seq_along(custom)], custom)
    suppressMessages(upgrade_miniextendr_package(path = tmp, autoconf = FALSE,
                                                allow_dirty = TRUE))
    expect_identical(readLines(attr_file), first)
  }
})

test_that("generated-file conflicts keep the current blob and give rerere no preimage", {
  skip_if_not(nzchar(Sys.which("git")), "git not available")
  tmp <- withr::local_tempdir(pattern = "attributes merge ")
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  attributes_git(tmp, "init", "-q")
  # Use the package writer, with no custom merge driver configured.
  minirextendr:::set_template_type("rpkg")
  withr::defer(minirextendr:::set_template_type("rpkg"))
  suppressMessages(minirextendr:::use_miniextendr_gitattributes(path = tmp))
  attributes_git(tmp, "config", "rerere.enabled", "true")
  write_generated <- function(value) {
    for (rel in attributes_paths) {
      dir.create(dirname(file.path(tmp, rel)), recursive = TRUE, showWarnings = FALSE)
      writeLines(c(value, "unchanged generated line"), file.path(tmp, rel))
    }
  }
  write_generated("base")
  attributes_git(tmp, "add", ".")
  attributes_git(tmp, "commit", "-qm", "Base generated artifacts")
  attributes_git(tmp, "checkout", "-qb", "incoming")
  write_generated("incoming")
  attributes_git(tmp, "commit", "-qam", "Regenerate incoming artifacts")
  attributes_git(tmp, "checkout", "-qb", "current", "HEAD~1")
  write_generated("current")
  attributes_git(tmp, "commit", "-qam", "Regenerate current artifacts")

  attributes_git(tmp, "merge", "--no-edit", "incoming", status = 1L)
  expect_setequal(attributes_git(tmp, "diff", "--name-only", "--diff-filter=U"),
                  attributes_paths)
  for (rel in attributes_paths) {
    expect_identical(readLines(file.path(tmp, rel)), c("current", "unchanged generated line"))
  }
  expect_length(attributes_git(tmp, "rerere", "status"), 0L)
  expect_length(list.files(file.path(tmp, ".git", "rr-cache"), recursive = TRUE), 0L)
})
