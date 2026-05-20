# Bootstrap-vendor test: `set -o pipefail` + `grep -q` SIGPIPE false negative

## What was attempted

PR #653 was meant to close #551 by rewriting `tests/bootstrap-produces-vendor.sh`
to drive `Rscript bootstrap.R + R CMD build rpkg` directly instead of going
through `pkgbuild::build('rpkg')`. The previous diagnosis blamed pkgbuild
"plumbing" for "losing inst/vendor.tar.xz between bootstrap and the sealed
tarball on CI."

The new direct-invocation test passed locally on macOS but kept failing on
Ubuntu CI with the same `FAIL: miniextendr_0.1.0.tar.gz does not contain
inst/vendor.tar.xz` message.

## What went wrong

Adding diagnostics that printed `tar -tzf "$TARBALL"` between the build and
the assertion proved the file WAS in the sealed tarball — line listing
clearly showed `miniextendr/inst/vendor.tar.xz`. Yet the next line, the
assertion `if ! tar -tzf "$TARBALL" 2>/dev/null | grep -q 'inst/vendor\.tar\.xz$';`
fired the FAIL branch.

## Root cause

The script ran under `set -euo pipefail`. The assertion idiom

```bash
if ! tar -tzf "$TARBALL" 2>/dev/null | grep -q PATTERN; then FAIL
```

has a SIGPIPE race:

1. `tar -tzf` writes the file listing to the pipe (528 entries, ~25 KB).
2. `grep -q` short-circuits on the FIRST match and exits 0, closing its stdin.
3. `tar` continues writing → gets SIGPIPE → exits non-zero (141).
4. With `set -o pipefail`, the pipeline's rc is the worst non-zero → 141.
5. `!` inverts non-zero to zero → `if` body runs → FAIL is printed even
   though the file IS present.

The bug manifests reliably on Ubuntu GH runners (Linux pipe buffer = 64 KB,
~25 KB of tar output flushes quickly enough that grep matches before tar
finishes) and rarely on macOS (pipe buffer = 16 KB, different scheduling).

## Fix

Materialise the tar listing into a shell variable first, then grep the
variable. Drains tar fully before the grep runs — no SIGPIPE possible.

```bash
TAR_LISTING=$(tar -tzf "$TARBALL" 2>/dev/null)
if ! grep -qE 'inst/vendor\.tar\.xz$' <<<"$TAR_LISTING"; then
    echo "FAIL: ..." >&2
    exit 1
fi
```

This is a one-line idiom change and zero-cost (the listing is ~25 KB, fits
in shell memory trivially).

## Wider lesson

`set -o pipefail` makes ALL `! tool | grep -q PATTERN` idioms suspect, not
just this one. Any time you negate a pipeline that ends in `grep -q` /
`head -n1` / any early-exiting consumer, the upstream tool can get SIGPIPE
and trigger a false-positive in the negation. Two safe patterns:

1. Materialise the producer's output first (this fix).
2. Use `grep -c PATTERN` and check the count is non-zero, OR use
   `grep PATTERN > /dev/null` (no `-q` → grep reads stdin to EOF).

Pattern #1 is preferred when the listing is bounded; #2 when the input is
arbitrarily large.

## Files

- `tests/bootstrap-produces-vendor.sh` — assertion rewritten + lesson
  inlined as a code comment so future readers don't reintroduce the bug.
