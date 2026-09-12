# Wrapper rule timestamps (#1530)

A real standalone source package reproduced the report after a body-only Rust
edit: Cargo rebuilt and Make relinked the shared library, but the unchanged
wrappers were still 8.9 seconds older. Each of three subsequent no-change installs
ran the wrapper generator again. The new regression failed exactly its timestamp
assertion and those three repeated-generation assertions against the original rule.

The writer intentionally compares semantic wrapper content and returns without
rewriting when only source locations differ. That policy remains intact. Make now
touches the wrappers after successful generation, as its wasm/tarball/roxygen
branches already do, and describes the invocation as checking wrappers. Failure
must still stop before the touch. The model and cross-package copies use a
separate Make recipe line, which also runs only after Rscript succeeds.

The existing cached-feature regression now covers the body edit and three settled
installs in all six layouts, alongside its real installed-library probes. It also
checks that actual wrapper changes retain the documentation NOTE, while a body
edit preserves wrapper contents and emits no change NOTE. No performance benefit
is claimed; the defect is the rule remaining out of date and its repeated log line.

The corrected rules passed all 251 assertions across six layouts, with no failures,
warnings, or skips. Formatting, workspace checking, all six Clippy gates, template
synchronization, and agent-file checks also passed.
