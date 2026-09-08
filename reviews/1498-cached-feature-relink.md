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
reused, switches to a second Cargo target directory and back, and checks that
a no-op install leaves the shared library untouched.
A Git repository keeps the fixture in source mode: without it, bootstrap
correctly selects tarball mode and intentionally skips subsequent wrapper
generation (#1022), which initially masked the intended test.

Validation: the source-mode test failed before the fix at the cached return
switch. After the fix, its 107 assertions pass; the full minirextendr suite
passes 976 assertions with no failures or warnings (11 explicit skips for
optional packages and separate CI-disabled end-to-end cases). All-manifest
format/check/test/Clippy, all three CI Clippy feature configurations, template
synchronization, agent instruction structure, and LLM documentation checks pass.

The first built-tarball check caught a fixture-only assumption: `proj_get()`
cannot discover a project inside the installed check test directory. Use
`usethis::local_project()` to establish and automatically restore the temporary
project, including the case where no previous project exists.

Final built-tarball validation: `just minirextendr-check` completes with
0 errors, 0 warnings, and 0 notes, including the real Cargo/R regression.
The main-package configure/install/force-document loop also passes with no
tracked documentation drift.
