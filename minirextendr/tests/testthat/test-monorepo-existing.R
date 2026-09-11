# Existing Cargo projects must scaffold without guessing a sibling directory or
# assuming that their public API matches our new-library hello() example.

write_existing_library <- function(path, name = "core") {
  dir.create(file.path(path, "src"), recursive = TRUE, showWarnings = FALSE)
  writeLines(c("[package]", paste0('name = "', name, '"'),
               'version = "0.1.0"', 'edition = "2024"'),
             file.path(path, "Cargo.toml"))
  writeLines("pub fn add(a: u64, b: u64) -> u64 { a + b }",
             file.path(path, "src", "lib.rs"))
}

test_that("existing root and virtual workspaces produce distinct binding packages", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not_installed("jsonlite")
  local_mocked_bindings(miniextendr_autoconf = function(...) invisible(),
                        .package = "minirextendr")

  for (virtual in c(FALSE, TRUE)) {
    root <- withr::local_tempdir()
    core <- if (virtual) file.path(root, "crates", "logic") else root
    write_existing_library(core)
    if (virtual) {
      writeLines(c("[workspace]", 'members = ["crates/*"]', 'resolver = "3"'),
                 file.path(root, "Cargo.toml"))
    }
    manifest_before <- readLines(file.path(core, "Cargo.toml"))
    messages <- capture_messages(use_miniextendr(
      path = root, template_type = "monorepo", claude_skills = FALSE
    ))
    messages <- gsub("[[:space:]]+", " ", paste(messages, collapse = " "))
    expect_match(messages, "from this workspace root", fixed = TRUE)
    expect_true(any(grepl('path = "rpkg"', messages, fixed = TRUE)))
    rust_dir <- file.path(root, "rpkg", "src", "rust")
    metadata <- jsonlite::fromJSON(paste(system2("cargo", c(
      "metadata", "--no-deps", "--format-version", "1", "--manifest-path",
      shQuote(file.path(rust_dir, "Cargo.toml"))
    ), stdout = TRUE), collapse = "\n"), simplifyVector = FALSE)
    binding <- metadata$packages[[1L]]
    expect_identical(binding$name, "core-r")
    expect_identical(binding$targets[[1L]]$name, "core")
    dep <- Filter(function(dep) identical(dep$name, "core"), binding$dependencies)[[1L]]
    expect_identical(dep$rename, "core_library")
    expect_identical(normalizePath(dep$path), normalizePath(core))
    expect_identical(readLines(file.path(core, "Cargo.toml")), manifest_before)
    source <- paste(readLines(file.path(rust_dir, "lib.rs")), collapse = "\n")
    active_source <- readLines(file.path(rust_dir, "lib.rs"))
    active_source <- active_source[!grepl("^//", active_source)]
    expect_false(any(grepl("::hello()", active_source, fixed = TRUE)))
    expect_match(source, "core_library", fixed = TRUE)
    expect_false(grepl("{{", source, fixed = TRUE))
  }
})

test_that("a virtual workspace with multiple libraries requires an explicit member", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not_installed("jsonlite")
  root <- withr::local_tempdir()
  write_existing_library(file.path(root, "first"), "first-lib")
  write_existing_library(file.path(root, "second"), "second-lib")
  writeLines(c("[workspace]", 'members = ["first", "second"]', 'resolver = "3"'),
             file.path(root, "Cargo.toml"))
  expect_error(suppressMessages(use_miniextendr(
    path = root, template_type = "monorepo", claude_skills = FALSE
  )), 'crate_name = "<package-name>"', fixed = TRUE)
  expect_false(dir.exists(file.path(root, "rpkg")))
  selected <- minirextendr:::get_monorepo_crate(
    file.path(root, "Cargo.toml"), crate_name = "second-lib"
  )
  expect_identical(selected$name, "second-lib")
  expect_identical(normalizePath(dirname(selected$manifest_path)),
                   normalizePath(file.path(root, "second")))
  expect_error(minirextendr:::get_monorepo_crate(
    file.path(root, "Cargo.toml"), crate_name = "missing"
  ), "No workspace library package")
})

test_that("a root library is preferred over other workspace members", {
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not_installed("jsonlite")
  root <- withr::local_tempdir()
  write_existing_library(root, "root-lib")
  write_existing_library(file.path(root, "child"), "child-lib")
  cat('\n[workspace]\nmembers = ["child"]\n',
      file = file.path(root, "Cargo.toml"), append = TRUE)
  expect_identical(minirextendr:::get_monorepo_crate(
    file.path(root, "Cargo.toml")
  )$name, "root-lib")
})


test_that("existing monorepos build twice and restore their path dependencies (#1429)", {
  run_e2e <- tolower(Sys.getenv("MINIEXTENDR_RUN_E2E", "")) %in% c("1", "true", "yes")
  if (!run_e2e) skip_on_ci()
  for (command in c("cargo", "cargo-revendor", "autoconf", "R")) {
    skip_if_not(nzchar(Sys.which(command)), paste(command, "not available"))
  }
  skip_if_no_local_repo()
  repo <- find_miniextendr_repo()
  skip_if_not_installed("callr")
  skip_if_not_installed("pkgload")

  for (virtual in c(FALSE, TRUE)) {
    root <- withr::local_tempdir()
    core <- if (virtual) file.path(root, "crates", "logic") else root
    write_existing_library(core)
    if (virtual) {
      writeLines(c("[workspace]", 'members = ["crates/*"]', 'resolver = "3"'),
                 file.path(root, "Cargo.toml"))
    }
    # A source checkout uses the workspace dependency; bootstrap.R still
    # freezes it when building the tarball during miniextendr_build().
    expect_identical(system2("git", c("-C", shQuote(root), "init", "--quiet")), 0L)
    suppressMessages(use_miniextendr(root, template_type = "monorepo", claude_skills = FALSE))
    rpkg <- file.path(root, "rpkg")
    suppressMessages(use_local_miniextendr(repo, path = rpkg))
    lib <- file.path(root, "library")
    dir.create(lib)
    rust_source <- file.path(rpkg, "src", "rust", "lib.rs")
    cat('\n/// Add through the existing library.\n/// @param a First number.\n/// @param b Second number.\n/// @return Their sum.\n#[miniextendr]\npub fn core_sum(a: i32, b: i32) -> i32 {\n    core_library::add(a, b)\n}\n',
        file = rust_source, append = TRUE)
    # Use i32 in this fixture's API so the regression concerns the dependency,
    # rather than adding checked integer conversion requirements to the test.
    writeLines("pub fn add(a: i32, b: i32) -> i32 { a + b }",
               file.path(core, "src", "lib.rs"))
    manifest <- file.path(rpkg, "src", "rust", "Cargo.toml")
    before <- readLines(manifest)
    for (iteration in seq_len(2L)) {
      callr::r(function(repo, rpkg, lib) {
        pkgload::load_all(file.path(repo, "minirextendr"), quiet = TRUE)
        .libPaths(c(lib, .libPaths()))
        minirextendr::miniextendr_build(path = rpkg)
      }, args = list(repo = repo, rpkg = rpkg, lib = lib), show = TRUE)
      expect_identical(readLines(manifest), before,
                       info = paste("manifest restored after build", iteration))
      expect_false(file.exists(file.path(rpkg, "inst", "vendor.tar.xz")))
      value <- callr::r(function(lib) {
        .libPaths(c(lib, .libPaths()))
        getExportedValue("core", "core_sum")(2L, 3L)
      }, args = list(lib = lib))
      expect_identical(value, 5L)
    }
  }
})
