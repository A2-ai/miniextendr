# #1498: cached Cargo archives skipped R relinking

Attempted: build a source package with default features, enable another feature,
then return to the already cached default build.

Failure: Cargo restored the original archive and its old modification time.
Make considered the newer shared library current, leaving the alternate native
code and generated R wrappers installed. The real Rust/R regression reproduced
both the stale runtime flag and the alternate wrapper argument on the return
switch.

Root cause: the link rule tracked archive timestamps without the selected build
configuration. Depending directly on Makevars would relink on every install:
autoconf rewrites its timestamp even when contents are identical.

Fix: compare Makevars with one package-local configuration record after Cargo
succeeds. Update the record only when contents change, and make it a shared
library prerequisite. Its fixed rust-target location survives feature/profile/
external-target switches, is already ignored, and follows existing cleanup.
Apply the same rule to rpkg, both minirextendr templates, and the three package
fixtures. Retain the archive dependency for normal code changes.

The regression uses real Cargo and R installs, checks installed functions and
wrapper formals in fresh processes, proves the default archive timestamp is
reused, and checks that a no-op install leaves the shared library untouched.
A Git repository keeps the fixture in source mode: without it, bootstrap
correctly selects tarball mode and intentionally skips subsequent wrapper
generation (#1022), which initially masked the intended test.
