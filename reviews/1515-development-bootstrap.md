# #1515: development bootstrap without distribution vendoring

The issue asks for a dev bootstrap that carries only path siblings and leaves
the source manifest unfrozen. Installed pkgbuild source showed bootstrap runs
before pkgbuild's optional copy. R 4.6.1's tools:::.build_packages (also checked
against background/r-svn/src/library/tools/R/build.R) copies into Rbuild first,
then runs cleanup before applying .Rbuildignore. The portable manifest can be
activated in that staged copy; the checkout's original bytes never need to be
replaced by the dev artifact's manifest. A saved origin path prevents cleanup
from activating it in the checkout, and a digest rejects stale staging.

cargo-revendor --dev uses Cargo metadata to find path siblings and cargo package
to normalize workspace inheritance. It does not run cargo vendor or xz. Git and
registry dependencies keep normal source resolution. The root package's
normalized manifest is saved separately; its source tree is not duplicated in
the final artifact. Source manifest guards in the packaging helper restore the
transient packaging edits. Aliases and transitive path dependencies are covered
by a real relocated Rust fixture with a normal Git dependency.

The first fixture failed with multiple workspace roots: declaring every copied
crate as a workspace caused Cargo to treat nested path dependencies as competing
roots. The portable root now excludes the vendor entries from membership and
only the root declares its workspace. The relocated build passes.

Automatic approval review rejected the first implementation command because it
would permanently remove an existing output directory without an ownership or
recovery check. No edits from that command were applied. The implemented version
instead retains replaced output and staging files under ignored
.dev-vendor-backup-* directories; it never permanently deletes previous output.
Backups are excluded from package artifacts and remain available for recovery.

Both R templates retain their own bootstrap logic, both scaffolders ship the
base-R helper, and miniextendr_build selects dev mode unless the caller supplied
an explicit mode or a release archive was already present. Distribution mode
remains the default for ordinary bootstrap calls.

The first R acceptance run stopped before bootstrap: the installed devtools
requires a logical upgrade argument, not the older string "never". The fixture
and usage example now use upgrade=FALSE.

The R acceptance fixture passes 34 assertions (0 failures/warnings/skips):
two direct devtools::install runs preserve Cargo.toml and produce no vendor
archive; the namespace returns 7; removing the dev opt-in builds and installs
the default offline distribution artifact, excluding dev sidecars and backups.
The separate helper test checks source/staged-copy separation and stale hashes.

Validation also caught a misspelled Clap conflict argument (`exclude`, which this CLI does not define); removing it restored parser validation. The root-crate monorepo fixture exposed nested `rust-target` output entering Cargo packages. Adding ignore patterns alone did not fix it: a focused `cargo package --list` probe showed Cargo ignores those rules while the source manifest is untracked; staging Cargo.toml immediately excluded the sentinel. The existing-project fixture now tracks its initial Rust sources, like the standalone Rust fixtures. Root scaffold ignores also cover the build tree and development staging, existing-project scaffolding appends them without replacing caller rules, and the private test library lives outside the source crate. Assertions inspect child-process warnings and bundled paths.

A Git dependency without a registry version deliberately exercises the development copy fallback. The old recursive fallback would include ignored build output. Development copies now use `cargo package --list` for Cargo’s include/exclude and Git-ignore selection, then resolve workspace inheritance in the staged manifest. The relocated fixture preserves build scripts, build/test path dependencies, and an `include_str!` asset while excluding the ignored artifact. All 122 default cargo-revendor tests pass.

Cargo’s standard file-selection rules remain authoritative, including explicit `package.include`/`package.exclude`; see https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields. For a source tree whose manifest is not tracked by Git, specify package includes/excludes if it contains generated output. No Git index is modified by development bootstrap.

Final repeated-monorepo acceptance: 52 assertions, 0 failures/warnings/skips, across root-crate and virtual-workspace layouts. Both installs in each layout preserve the source manifest, retain runtime calls, produce no vendor archive, and exclude build output and recovery backups.
