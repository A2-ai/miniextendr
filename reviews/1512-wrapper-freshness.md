# #1512 — stale wrappers accepted by native tarball installs

Attempt: scaffold a small package, install its initial `probe()` function, then
add `summary.freshprobe` in Rust and its S3 registration without regenerating
the wrappers. Build with `devtools::build()` and install the resulting tarball.

Failure: the baseline install exits successfully and prints twice that
`summary.freshprobe` is declared in NAMESPACE but not found. The log confirms
that the native #1022 guard reused the nonempty shipped wrappers.

Root cause: neither copying nor tarball extraction preserves useful source
ordering by mtime, and nonemptiness does not establish generation provenance.
The wrapper writer also deliberately preserves an unchanged file's mtime.

Fix: a base-R build helper records package-relative content fingerprints after
native generation. The tarball guard reuses only a matching record; otherwise
it regenerates and stops before namespace loading if the R code changed. The
comparison uses a temporary copy, so failure cannot make a retry appear valid.
Cargo manifest/lock changes during freeze conservatively require verification.
The model and cross-package Makevars always generate wrappers, so they have no
#1022 skip to change; rpkg and both end-user templates contain the guarded path.

The first isolated helper test caught an unqualified `setNames` lookup: it is
from stats, not base. The helper now constructs the named vector with base
`structure`, so it works without relying on attached default packages.

The first full scaffolder run found a stale developer tool: the executable on
PATH was built in June and failed the existing #1429 monorepo regression with
`no workspace root found`, a diagnostic absent from current cargo-revendor.
Building the current tool and prepending its directory to PATH was insufficient:
`cargo --list -v` still selected the June executable. Cargo prioritizes
`$CARGO_HOME/bin` over PATH ([Cargo's external-tool lookup](https://doc.rust-lang.org/cargo/reference/external-tools.html)).
Use `just revendor-install` to update the executable Cargo actually invokes. CI now uses
the shared setup-vendor action in install-only mode in the minirextendr job,
so the new real-tarball regression runs rather than skipping for a missing tool.

Do not overlap vendoring regressions with other Cargo checks in this checkout.
The initial validation did so: `cargo-revendor::package` temporarily appended a
`[patch.crates-io]` entry for the test's `core` crate to the framework workspace
manifest. Clippy saw that transient patch and failed `--locked`. The guards
restored the manifest, but the checks must run sequentially; verify the Git diff
before rerunning the affected Cargo commands. This was a validation scheduling
error, not a reason to weaken `--locked` or accept an unused-patch warning.

The full local suite also caught a real interaction with #1288 recovery. After
the new tarball guard rejected a renamed export, the source-mode roxygen
optimization still reused any nonempty wrappers file. Its documentation pass
therefore saw the old export, and the retry correctly failed again. Require
matching fingerprints for roxygen reuse as well, so the source pass regenerates
wrappers before NAMESPACE reconciliation. The existing end-to-end rename test
is the regression oracle; do not bypass the new tarball check with the force flag.

The corrected rename-recovery regression passes through the expected initial
stale-tarball rejection, source wrapper regeneration, NAMESPACE reconciliation,
and successful reinstall. The force-override propagation case also passes
without the warnings from the aborted earlier run. The final focused wrapper
suite passes all 37 assertions, including the actual tarball install and the
sidecar-shipping assertion. Set `NOT_CRAN=true` for direct `test_file()` calls;
without it, the compilation cases intentionally skip.
