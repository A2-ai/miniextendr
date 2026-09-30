# miniextendr_vendor() seals vendor/ into inst/vendor.tar.xz. A tarball
# install's configure maps every vendored source through the archive's
# vendor/.cargo-config.toml and stops when it is missing (#1555), so the
# dotfiles cargo-revendor writes must survive the staging copy and the tar.

test_that("miniextendr_vendor() archives vendor/.cargo-config.toml", {
  root <- withr::local_tempdir()
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.1.0\n", file.path(root, "DESCRIPTION"))
  dir.create(file.path(root, "src", "rust"), recursive = TRUE)
  writeLines('[package]\nname = "testpkg"\nversion = "0.1.0"',
             file.path(root, "src", "rust", "Cargo.toml"))
  # Stand-in for cargo-revendor: one vendored crate plus the source replacements.
  local_mocked_bindings(
    vendor_crates_io = function(path = ".") {
      vendor <- file.path(root, "vendor")
      dir.create(file.path(vendor, "core"), recursive = TRUE)
      writeLines('[package]\nname = "core"\nversion = "0.1.0"',
                 file.path(vendor, "core", "Cargo.toml"))
      writeLines('{"files":{},"package":null}', file.path(vendor, "core", ".cargo-checksum.json"))
      writeLines(c('[source.crates-io]', 'replace-with = "vendored-sources"', '',
                   '[source.vendored-sources]', 'directory = "/build/vendor"'),
                 file.path(vendor, ".cargo-config.toml"))
      invisible(TRUE)
    },
    .package = "minirextendr"
  )

  tarball <- suppressMessages(miniextendr_vendor(root))

  listing <- utils::untar(tarball, list = TRUE)
  expect_true("vendor/.cargo-config.toml" %in% listing)
  expect_true("vendor/core/.cargo-checksum.json" %in% listing)
})
