# #1513 — repeated tarball installs lose Cargo dependency reuse

Investigation: inspect configure's vendor and target paths, tarball cleanup,
frozen manifest shape, and actual Cargo behavior across two extracted packages.

The issue's target-cleanup assumption did not match current main: Makevars
unconditionally deleted `CARGO_TARGET_DIR` in tarball mode. Preserve explicit
caller caches, including ones nested under conventional cleanup directories.
The configured `[build] target-dir` must also match the requested directory.

Frozen manifests originally contained direct `path = "../../vendor/<crate>"`
dependencies plus relative `[patch.crates-io]` entries. Replacing only those
patches with absolute paths produced unused-patch warnings: the direct path
dependencies still won. Cargo `paths` overrides were also unsuitable: they
loaded otherwise-ignored development dependencies from a fallback-packaged API
crate. Merely copying local crates with preserved mtimes installed correctly,
but rebuilt four crates. Cargo fingerprint diagnostics confirmed new package
IDs for those extraction-specific dependency paths.

Fix the source selection at vendor time: `--freeze` now preserves dependency
options/aliases and versions while removing the direct path, resolving through
the existing vendor patch table. Configure can override that table with absolute
cache paths without changing the shipped manifest or lockfile. Ordinary tarball
installs keep relative vendor patches. External git dependencies retain their
existing source-replacement behavior. No local metadata copies or `paths`
overrides are needed.

A second dependency-cache invalidator was the extraction-specific C-object path
in global RUSTFLAGS. Shared-vendor mode uses `cargo rustc` to pass those object
arguments only to the package crate. The default build command stays unchanged.

The base-R helper keys entries by archive contents and publishes only completed
extractions. Changed archives get separate entries; cleanup never deletes
caller-owned entries or unrelated files. The helper is included in both R
scaffolds and the Rust CLI's embedded catalog and common scaffold plan.

Focused validation: `just revendor-test` passes. The real two-install fixture
(including a local core crate with a build script and the framework) rebuilds
only the package on its second install, returns the expected value in a fresh R
process, and retains the wrapper skip. All 34 cache/cleanup assertions pass with
no warnings. On macOS arm64, R 4.6.1, Cargo dev profile: first install 15.63s,
second install 3.32s. These are local fixture measurements, not release-profile
or cross-platform performance guarantees.

The expanded regression passes all 42 assertions, including byte-identical
Cargo.toml, Cargo.lock, and Rust sources after two explicitly extracted package
installs; ordinary tarball installation and runtime behavior without either
cache setting; and caller-cache preservation. The repeat measurement was
16.40s then 3.55s, again with one crate rebuilt. Template, instruction-file, and
site checks pass. An early test assertion used an unsupported `info` argument
on `expect_length`; `expect_identical(length(...), ..., info = ...)` supplies
the intended full Cargo log on a rebuild-count mismatch.

Final broader validation passes: built-tarball `CI=true NOT_CRAN=true just
minirextendr-check` reports 0 errors, 0 warnings, and 0 notes; the existing
monorepo #1429 regression passes 40 assertions, including both layouts and
manifest restoration after repeated builds. The separate source-monorepo smoke
test passes too. `just test` passes every leg, and all sequential Clippy gates
pass with `-D warnings`: the repository recipe, three root CI configurations,
full-feature rpkg, and standalone cargo-revendor. `just configure`, formatting,
template synchronization, instruction-file checks, and site checks pass.
