# cargo-revendor changes skipped packaging CI

PR #1525 changed Rust sources in cargo-revendor. The Rust and sync jobs ran,
but the R change filter did not include that standalone crate. Consequently,
Bootstrap Vendor Test, CRAN-like check, and the R package integration jobs were
skipped; the aggregate CI Success status concealed that missing coverage.

Include cargo-revendor/** and its setup-vendor composite action in the R filter.
These inputs now select the existing per-PR packaging and bootstrap jobs without
requiring an incidental R-file or manifest change. Validate the actual changed
paths from #1525 against the filter and lint the workflow before publishing.

Actionlint also caught unquoted environment-file paths and a GitHub expression
inside a shell comment referencing a non-dependent macOS job. Quote the paths
and remove the inactive expressions: GitHub evaluates expressions before the
shell sees comments. The non-gating jobs remain documented in `needs` comments.
