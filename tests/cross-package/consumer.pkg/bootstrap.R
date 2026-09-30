# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# pkgbuild runs it in the source directory before `R CMD build` copies the
# package: devtools::build/install/check, rcmdcheck, pak, and rv (with
# `directory`). Plain `R CMD build` and `R CMD INSTALL` never run it.
#
# A stub: this test package keeps the hook a scaffolded package has, but has
# nothing to do in it. configure runs on every install, and bootstrap.R never
# vendors. Its path dependencies (the workspace crates and shared-traits) are
# not staged: the package is only ever installed in place from this
# repository, where those paths exist.
