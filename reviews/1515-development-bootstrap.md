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
