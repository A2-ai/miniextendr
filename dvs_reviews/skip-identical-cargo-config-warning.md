# Configure: Remove Cargo Config in Dev Mode

## Summary

When `NOT_CRAN=true` (development mode), the `.cargo/config.toml` is removed so that normal cargo network resolution is used instead of vendored sources.

## Rationale

The previous behavior was:

- Always generate `src/rust/.cargo/config.toml` with vendored source replacement
- When `NOT_CRAN=true` in a monorepo, sync this config to the root `.cargo/config.toml`
- After cargo vendor/lockfile operations, restore the config from backup

This caused problems:

1. The vendored sources don't include dev-dependencies (like `tempfile`)
2. Syncing to root forced the entire workspace to use incomplete vendored sources
3. `cargo metadata` and other tooling (rust-analyzer) failed with missing package errors

The new behavior is:

- `NOT_CRAN=true` (dev mode): Delete the cargo config, don't restore from backup
- `NOT_CRAN=false` (CRAN mode): Keep the cargo config with vendored sources

This is cleaner because:

- Dev builds use normal cargo resolution with full dependency access
- CRAN builds still work offline with vendored sources
- rust-analyzer and other tooling work correctly

## Implementation

Three changes in `configure.ac`:

### 1. Remove config after generation (dev-cargo-config)

```sh
AC_CONFIG_COMMANDS([dev-cargo-config],
[
  RPKG_CFG="src/rust/.cargo/config.toml"

  if test "$NOT_CRAN" = "true"; then
    if test -f "$RPKG_CFG"; then
      rm "$RPKG_CFG"
      echo "configure: removed cargo config (dev mode - using network sources)"
    fi
  fi
],
[NOT_CRAN="$NOT_CRAN"])
```

### 2. Don't restore after cargo vendor

```sh
# After cargo vendor completes:
if test "$NOT_CRAN" != "true" && test -f "$_cargo_cfg_bak"; then
  mv "$_cargo_cfg_bak" "$_cargo_cfg"
else
  rm -f "$_cargo_cfg_bak"
fi
```

### 3. Don't restore after lockfile generation

Same pattern as above - only restore in CRAN mode, otherwise delete the backup.

## Testing

1. `NOT_CRAN=true ./configure` - config should NOT exist after configure completes
2. `./configure` (no NOT_CRAN) - config should exist with vendored sources
3. rust-analyzer should work without "missing package" errors in dev mode
