# #1516: preserve Cargo targets for opted-in source installs

R's install.R sets R_INSTALL_PKG on entry to do_install_source and unsets it
on exit. Both preclean and clean invoke run_clean inside that scope. In build.R,
cleanup_pkg sets R_PACKAGE_NAME/DIR/LIBRARY_DIR but does not set R_INSTALL_PKG.
Checked the repository background R sources and the installed R 4.6.1 version.

Cleanup now preserves its three Cargo/analyzer target directories only when
MINIEXTENDR_KEEP_TARGET=1 and R_INSTALL_PKG is nonempty. Other cleanup work is
unchanged, and a package build still cleans its staged copy. This is separate
from the existing tarball-mode Makevars cleanup, which remains unchanged.

The recipe audit found just rcmdinstall passes only caller-supplied arguments;
miniextendr_build uses devtools::install without requesting clean flags, and
scaffold DESCRIPTION files use Config/build/never-clean: true. Documentation,
the repository build skill, and the shipped package guide explain the cold-cache
cost of both flags and show the explicit opt-in when another tool requires them.

The first acceptance run stopped before compilation because the fixture passed a use_miniextendr-only argument to create_miniextendr_package. Removing that argument matches the public constructor signature. The 48 cleanup-context assertions already passed.

Acceptance passes 62 assertions (0 failures/warnings/skips): the context matrix, a real warm install with both cleanup flags, unchanged dependency-artifact mtime and no linkme compilation, a clean R CMD build artifact with the source cache retained, and an ordinary cold install with the target removed afterwards.

R CMD check caught an undeclared processx test dependency; the fixture now uses base-R system2 instead. The shipped-skill audit reported known generated/optional paths as warnings even though a clean scaffold should lack them. Those two existing allowlisted categories remain visible as INFO; missing ordinary paths, missing symbols, and invalid line references retain their original severity.

The dependency-inventory regression and R CMD check both identified the same processx reference; removing it fixes both findings. The corrected targeted run passes 71 assertions with 0 failures/warnings/skips. The skill audit reports 0 BLOCKING, 0 WARN, 7 INFO on a real scaffold; an empty-layout negative probe still exits 1 and reports the required src/rust/Cargo.toml as BLOCKING.
