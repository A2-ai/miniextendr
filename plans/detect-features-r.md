# Plan: Create `tools/detect-features.R` for configure-time feature detection

## Goal

`rpkg/configure.ac` references `tools/detect-features.R` for auto-detecting which cargo
features to enable, but the file doesn't exist. Currently falls back to a hardcoded list
of all features. Create the script so scaffolded packages can auto-detect available features.

## Context

In `rpkg/configure.ac` (lines 66–78):
```sh
if test -f "${srcdir}/tools/detect-features.R"; then
  MINIEXTENDR_FEATURES=$("${R_HOME}/bin/Rscript" "${srcdir}/tools/detect-features.R" 2>/dev/null || echo "")
fi
# If auto-detection returned nothing, enable all features
if test -z "$MINIEXTENDR_FEATURES"; then
  MINIEXTENDR_FEATURES="worker-thread,rayon,rand,..."  # hardcoded fallback
fi
```

The `minirextendr` scaffolding package has `use_configure_feature_detection()` in
`minirextendr/R/feature-detect-configure.R` with `add_feature_rule()` for declaring
detection rules. But no actual `detect-features.R` script exists in `rpkg/tools/`.

## Design

`rpkg/tools/detect-features.R` should:

1. Read `src/rust/Cargo.toml` to discover available features
2. Apply rules to determine which features are available:
   - Check if optional R packages are installed (e.g., `vctrs` for the `vctrs` feature)
   - Check system capabilities (e.g., `connections` feature needs R >= 4.x)
   - Default: enable feature if no rule says otherwise
3. Output a comma-separated feature string to stdout

## Files

- `rpkg/tools/detect-features.R` (new)
- Template: `minirextendr/inst/templates/rpkg/tools/detect-features.R` (new)
- Template: `minirextendr/inst/templates/monorepo/rpkg/tools/detect-features.R` (new)
