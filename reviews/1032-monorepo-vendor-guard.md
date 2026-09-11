# Monorepo configure accepted leaked vendor tarballs (#1032)

The generated monorepo configure script was run against eight small package
fixtures. Both a repository directory and a worktree-style `.git` file above
the package let a leftover `inst/vendor.tar.xz` select tarball mode. Configure
unpacked it and generated build files instead of reporting the leak.

The monorepo template lacked the existing source-tree classification and
leaked-tarball guard from rpkg and the standalone template. Ported that guard,
including the `Packaged:`/`Built:` exemption and the explicitly set bootstrap
exemption, before mode selection. The guard only diagnoses the leaked file;
it does not remove the archive or rewrite the source manifest.

The regression runs autoconf and the complete generated configure script.
It covers normal source mode, leaked archives under both `.git` forms,
packaged/installed metadata beneath a repository, bootstrap (including an
empty but explicitly set value), and tarballs outside a repository. Cases
use paths with spaces, real vendor archives, and a dependency-free locked
crate, so no network or Rust compilation is needed. Sources and archives
are checked for content changes; rejected cases must not create build files
or unpack vendor sources. Before the fix, the two leak cases failed twelve
assertions while the legitimate install cases passed.

The first package check caught an undeclared processx dependency in the new
test harness. The harness now uses base R's `system2()` with output directed
to log files, so expected configure failures return exit statuses without
adding a dependency or emitting R warnings.

Final validation passed: all 58 configure assertions; the standard R suite
(849 assertions, zero failures/warnings); `just fmt`, `just check`, full
`just test`, `just clippy -- -D warnings`, and all three CI Clippy feature
configurations. `just templates-approve` and `just templates-check` agree.
R CMD check reports zero errors and zero warnings. Its sole AGENTS.md NOTE
is tracked by #1409 and already addressed in PR #1487.


The first CI minirextendr check passed its tests but failed while retrieving a
Bioconductor annotation index during the dependency scan. R interpreted three
packages built inside `test-templates.R` (`testpkg`, `spexport`, `sprename`) as
undeclared external dependencies, then queried repository indexes to filter
them. CI skips the expensive runtime fixtures, but its static dependency scan
still reads their code.

Load the generated package by its computed name and resolve generated exports
with `getExportedValue`, preserving the public-export assertions. A regression
runs R's actual dependency scanner over the test tree: it reproduced the three
false candidates before the change and retains a control that detects an
undeclared `processx::run` call. No repositories or warning checks are disabled.

The AGENTS.md and help-link notes are fixed by integrating PR #1487, including
the subsequently identified scaffold ignore-template omissions and their
`justfile` mappings. The merge conflict in generated `patches/templates.patch`
was resolved by regeneration with `just templates-approve`, followed by
`just templates-check`.

The scanner regression fails with `others = testpkg` and
`imports = spexport, sprename` before the lookup change, and passes all nine
source-package assertions afterward. R's `.check_packages_used_in_tests` guards
repository discovery with `any(lengths(res[1L:3L]))`; the now-empty candidate
lists skip repository selection and `available.packages()` entirely.
The combined branch passes 869 full-suite assertions (11 existing skips), and
R CMD check with recursive test-dependency scanning and declared-package Rd
checking reports zero errors, zero warnings, and zero notes.
