# Issue #64 — S3 generic `@title` triggers roxygen2 multiline warning

## Problem
The S3 generic doc generation emits two consecutive content lines without a tag boundary:
```
#' @title S3 generic for `len`
#' S3 generic for `len`
```
roxygen2 reads the second line as a continuation of `@title` and fires the
"multiline title" warning during `just devtools-document`.

## Sites
- `miniextendr-macros/src/miniextendr_impl/s3_class.rs:94-95`
- `miniextendr-macros/src/miniextendr_impl/vctrs_class.rs:222-223`
(s7_class.rs lines from the original issue are no longer present — verified
via grep on 2026-05-06; only s3 and vctrs remain.)

## Fix
For both sites, prefix the second line with `@description`:
```rust
lines.push(format!("#' @title S3 generic for `{}`", generic_name));
lines.push(format!("#' @description S3 generic for `{}`", generic_name));
```

## Acceptance
- `just devtools-document 2>&1 > /tmp/devtools-doc.log` no longer emits the multiline-title warning for S3/vctrs blocks.
- `R/miniextendr-wrappers.R` shows the new `@description` line in S3/vctrs generic blocks.
- `NAMESPACE` and `man/*.Rd` regenerated and committed in sync.
