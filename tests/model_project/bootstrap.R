# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# pkgbuild runs it in the source directory before `R CMD build` copies the
# package: devtools::build/install/check, rcmdcheck, pak, and rv (with
# `directory`). Plain `R CMD build` and `R CMD INSTALL` never run it.
#
# A stub: this test package keeps the hook a scaffolded package has, but has
# nothing to do in it. configure runs on every install, bootstrap.R never
# vendors, and no path dependency leaves the package, so nothing needs staging.
