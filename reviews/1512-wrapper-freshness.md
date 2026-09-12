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

Validation results will be recorded after the focused and package checks.
