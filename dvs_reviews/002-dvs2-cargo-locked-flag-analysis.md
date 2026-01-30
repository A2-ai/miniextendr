# DVS2 `--locked` Flag Analysis

## The Problem

The `--locked` flag tells Cargo to fail if `Cargo.lock` doesn't match `Cargo.toml`. This ensures reproducible builds but fails when checksums are stripped (as happens with vendor tarballs).

## DVS2's Approach

```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_LOCKED_FLAG="--locked"   # DEV: use locked for reproducibility
else
  CARGO_LOCKED_FLAG=""           # CRAN: no locked, checksums cleared
fi
```

## Miniextendr's Current Approach (After Recent Fix)

```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_LOCKED_FLAG=""           # DEV: no locked, allow lockfile updates
else
  CARGO_LOCKED_FLAG="--locked"   # CRAN: use locked for reproducibility
fi
```

## Analysis

### Why DVS2's approach is better for DEV mode:

1. **Reproducibility**: `--locked` ensures everyone gets the same dependency versions
2. **CI consistency**: Builds fail fast if lockfile is out of sync
3. **No surprises**: Cargo won't silently update dependencies

### Why DVS2's approach is better for CRAN mode:

The vendor tarball creation process clears checksums:

```bash
for _crate_dir in "$_staging_tmp/vendor"/*/; do
  echo '{"files":{}}' > "${_crate_dir}.cargo-checksum.json"
done
```

With cleared checksums, `--locked` will fail because Cargo can't verify the lockfile against actual crate contents.

DVS2 also explicitly strips `--locked` from Makevars when unpacking from tarball:

```bash
if test -f "src/Makevars"; then
  $SED -i.bak 's/--locked//g' src/Makevars && rm -f src/Makevars.bak
  echo "configure: removed --locked from Makevars for tarball compatibility"
fi
```

## Recommendation

Miniextendr should **invert** its `--locked` logic to match DVS2:

```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_LOCKED_FLAG="--locked"   # DEV: reproducible builds
else
  CARGO_LOCKED_FLAG=""           # CRAN: checksums cleared, can't verify
fi
```

## Impact Assessment

- **Risk**: Low - this is a configuration change, not code change
- **Testing**: Verify both `NOT_CRAN=true` and `NOT_CRAN=false` build paths
- **Rollback**: Easy to revert if issues arise
