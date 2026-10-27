# src/Makevars.win carries the Windows system libraries Rust needs. It is
# written wherever src/Makevars.in is, so a scaffold has it and an upgrade
# replaces an old copy.

for (layout in c("rpkg", "monorepo")) {
  test_that(paste("Windows Makevars is scaffolded from the template for", layout), {
    tmp <- withr::local_tempdir()
    usethis::local_project(tmp, force = TRUE, setwd = FALSE)
    old_type <- minirextendr:::get_template_type()
    withr::defer(minirextendr:::set_template_type(old_type))
    minirextendr:::set_template_type(layout)
    subdir <- if (layout == "monorepo") "rpkg" else NULL

    suppressMessages(minirextendr:::use_miniextendr_makevars(subdir = subdir))
    dest <- file.path(tmp, "src", "Makevars.win")
    expect_identical(readLines(dest), readLines(
      minirextendr:::template_path("Makevars.win", subdir = subdir)
    ))
  })
}

test_that("upgrade replaces a stale Makevars.win and reports it", {
  for (monorepo in c(FALSE, TRUE)) {
    tmp <- withr::local_tempdir()
    pkg <- if (monorepo) file.path(tmp, "rpkg") else tmp
    if (monorepo) writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
    dir.create(file.path(pkg, "src", "rust"), recursive = TRUE, showWarnings = FALSE)
    writeLines(c("Package: winpkg", "Version: 0.0.1"), file.path(pkg, "DESCRIPTION"))
    writeLines(c("AC_INIT([winpkg], [0.0.1])", 'CARGO_FEATURES=""', "AC_OUTPUT"),
               file.path(pkg, "configure.ac"))
    writeLines("# Previous scaffold", file.path(pkg, "src", "Makevars.in"))
    writeLines("[package]", file.path(pkg, "src", "rust", "Cargo.toml"))
    # An older scaffold's Makevars.win, without $(BLAS_LAPACK_PKG_LIBS).
    writeLines(c("include Makevars",
                 "PKG_LIBS = $(CARGO_LIBDIR)/lib$(CARGO_STATICLIB_NAME).a -lws2_32"),
               file.path(pkg, "src", "Makevars.win"))

    changes <- suppressMessages(upgrade_miniextendr_package(
      path = tmp, configure_ac = TRUE, autoconf = FALSE, allow_dirty = TRUE
    ))
    layout <- if (monorepo) file.path("monorepo", "rpkg") else "rpkg"
    template <- system.file("templates", layout, "Makevars.win",
                            package = "minirextendr", mustWork = TRUE)
    expect_identical(readLines(file.path(pkg, "src", "Makevars.win")), readLines(template),
                     info = layout)
    shown <- if (monorepo) "rpkg/src/Makevars.win" else "src/Makevars.win"
    expect_true(shown %in% changes$changed, info = layout)
  }
})

test_that("the upgrade's dirty check covers Makevars.win", {
  expect_true("src/Makevars.win" %in% minirextendr:::upgrade_scaffolding_files)
})
