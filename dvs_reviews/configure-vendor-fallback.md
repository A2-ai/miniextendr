# Configure: Vendor Fallback to Network

## Summary

When `NOT_CRAN` is unset (CRAN-like build) but there's no cached vendor directory or tarball, configure should fall through to fetch vendors from the network instead of erroring out.

## Rationale

The previous behavior was:
- `NOT_CRAN=false` + no vendor directory + no tarball → **error**

This caused GitHub installs via `remotes::install_github()` to fail because:
1. GitHub installs don't include the vendor tarball (removed from git)
2. They run as CRAN-like builds by default (`NOT_CRAN` unset)
3. There's nothing cached to use

The new behavior is:
- `NOT_CRAN=false` + no vendor directory + no tarball → **fetch from network**

This makes sense because if there's nothing cached, we have to fetch anyway. The `NOT_CRAN` flag should really mean "prefer offline/cached sources if available", not "require offline sources".

## Implementation

In `configure.ac`, the `cargo-vendor` command section:

```sh
if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
  if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
    # unpack tarball...
  else
    # OLD: error out
    # NEW: fall through to network vendoring
    echo "configure: no cached vendors found, will fetch from network"
    _use_prevendored=0
  fi
fi
```

## Testing

1. Remove vendor directory and tarball from a fresh clone
2. Run `remotes::install_github("...")` without setting `NOT_CRAN`
3. Should successfully fetch and vendor dependencies from network
