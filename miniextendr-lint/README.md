# miniextendr-lint

Internal build-time linter for the `miniextendr` workspace.

This crate provides proc-macro helpers used by the workspace tooling and is not
intended for external use or publication as a standalone utility.

## Usage

Typically invoked indirectly via workspace checks/tests:

```sh
cargo test -p miniextendr-lint
```

## Notes

If you are looking for the user-facing API, see:
- `miniextendr-api`
- `miniextendr-macros`
