# miniextendr-lint

Internal build-time linter for the `miniextendr` workspace.

This crate provides proc-macro helpers used by the workspace tooling and is not
intended for external use as a standalone library.

## Usage

Typically invoked indirectly via workspace checks/tests:

```sh
cargo test -p miniextendr-lint
```

## Publishing to CRAN

This crate is an internal Rust build tool and should never be part of an R
package distributed to CRAN.

## Maintainer

- Keep lint rules aligned with current workspace conventions.
- If the lint depends on macro output, update it alongside macro changes.
- Ensure any errors are actionable for contributors (clear diagnostics).

## Notes

If you are looking for the user-facing API, see:
- `miniextendr-api`
- `miniextendr-macros`
