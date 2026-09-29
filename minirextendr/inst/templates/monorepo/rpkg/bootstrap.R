# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# pkgbuild runs it in the source directory before `R CMD build` copies the
# package: devtools::build/install/check, rcmdcheck, pak, and rv (with
# `directory`). Plain `R CMD build` and `R CMD INSTALL` never run it.
#
# Its one job: a path dependency outside the package (e.g. a core crate at
# `path = "../../../my-core"`) does not travel with the package directory, so
# tools/dev-bootstrap.R stages it under src/rust/vendor while the checkout is
# still here, and cleanup swaps in a manifest pointing there. Registry and git
# dependencies resolve over the network at install time. A package whose path
# dependencies all live inside it has nothing to stage.
#
# bootstrap.R never vendors. An offline (CRAN) tarball is a separate, explicit
# step that seals inst/vendor.tar.xz before the build
# (minirextendr::miniextendr_build_tarball()). That archive carries every
# dependency, so there is nothing left to stage.
source("tools/dev-bootstrap.R", local = TRUE)
if (file.exists("inst/vendor.tar.xz")) {
  clear_dev_bootstrap()
} else {
  prepare_dev_bootstrap()
}
