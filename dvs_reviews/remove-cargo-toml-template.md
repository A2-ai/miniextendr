# Remove Cargo.toml.in Template

## Summary

Remove the unnecessary `Cargo.toml.in` template file since it has no configure-time substitutions.

## Rationale

The `Cargo.toml.in` template was identical to the output `Cargo.toml` - it contained no `@VARIABLE@` substitution placeholders. This caused ergonomic issues:

1. Editing `Cargo.toml` directly would be overwritten by configure
2. rust-analyzer and other tools expect `Cargo.toml` to be the source of truth
3. Adding dependencies required editing the `.in` file instead of the standard location

## Implementation

Three changes in configure.ac and related files:

### 1. Update crate name extraction

```diff
-CARGO_STATICLIB_NAME="$($SED -n 's/^name = "\(.*\)"/\1/p' "$RUST_SRC_DIR/Cargo.toml.in" | head -1 | tr '-' '_')"
+CARGO_STATICLIB_NAME="$($SED -n 's/^name = "\(.*\)"/\1/p' "$RUST_SRC_DIR/Cargo.toml" | head -1 | tr '-' '_')"
```

### 2. Remove AC_CONFIG_FILES for Cargo.toml

```diff
-dnl Generate Cargo.toml and document.rs from templates
-AC_CONFIG_FILES([src/rust/Cargo.toml:src/rust/Cargo.toml.in])
-AC_CONFIG_FILES([src/rust/document.rs:src/rust/document.rs.in])
+dnl Generate document.rs from template
+AC_CONFIG_FILES([src/rust/document.rs:src/rust/document.rs.in])
```

### 3. Update .gitignore

Remove `src/rust/Cargo.toml` from `.gitignore` so it can be tracked.

### 4. Delete template and track Cargo.toml

```sh
rm src/rust/Cargo.toml.in
git add src/rust/Cargo.toml
```

## Result

- `Cargo.toml` is now a regular tracked file
- Can be edited directly without configure overwriting it
- rust-analyzer and cargo tooling work as expected
