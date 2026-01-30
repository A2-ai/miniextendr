# DVS2 Configuration Review: Summary and Action Items

## Executive Summary

DVS2 represents a real-world downstream consumer of miniextendr. Its configuration reveals several patterns that could improve miniextendr's build system and documentation. Key learnings:

1. **`--locked` flag inversion**: DVS2 uses `--locked` in DEV mode (for reproducibility) and omits it in CRAN mode (because checksums are cleared). Miniextendr currently does the opposite.

2. **Automatic feature-to-CPPFLAGS**: DVS2 auto-generates C preprocessor flags from cargo features instead of manual case statements.

3. **Staticlib name extraction**: DVS2 extracts the crate name via sed at configure time, avoiding the need for `cargo pkgid`.

4. **Git dependency vendoring**: DVS2 shows the pattern for vendoring git dependencies (not just crates-io).

5. **Downstream package template**: DVS2's Cargo.toml.in is effectively a template for how external packages should depend on miniextendr.

## Action Items

### High Priority

| Item | Description | File(s) |
|------|-------------|---------|
| **Invert `--locked` logic** | Use `--locked` in DEV mode, omit in CRAN mode | `rpkg/configure.ac` |

**Rationale:** DEV mode benefits from reproducible builds matching the committed lockfile. CRAN mode can't use `--locked` because checksums are cleared in vendor tarballs.

### Medium Priority

| Item | Description | File(s) |
|------|-------------|---------|
| **Auto-generate feature CPPFLAGS** | Loop over features instead of manual case statements | `rpkg/configure.ac` |
| **Sed-based staticlib extraction** | Extract crate name from Cargo.toml.in at configure time | `rpkg/configure.ac` |
| **Wildcard wrapper filename** | Use `@PACKAGE_TARNAME@-wrappers.R` pattern | `rpkg/src/Makevars.in` |

### Low Priority

| Item | Description | File(s) |
|------|-------------|---------|
| **Add ra_target to gitignore** | Include rust-analyzer's target directory | `rpkg/.gitignore` |
| **R CMD check context detection** | Detect when running on extracted tarball | `rpkg/configure.ac` |

### Documentation

| Item | Description |
|------|-------------|
| **Downstream package guide** | Document the pattern for external packages depending on miniextendr |
| **Cargo.toml.in template** | Provide a template with standalone workspace, git deps, and patches |
| **Git dep vendoring guide** | Explain `[source."git+..."]` replacement in cargo config |

## Files Reviewed

| DVS2 File | Review Document |
|-----------|-----------------|
| `dvs-rpkg/configure.ac` | 001-build-system-overview, 002-locked-flag-analysis |
| `dvs-rpkg/src/Makevars.in` | 005-makevars-in-patterns |
| `dvs-rpkg/src/rust/Cargo.toml.in` | 006-cargo-toml-in-patterns |
| `dvs-rpkg/src/rust/document.rs.in` | 004-document-rs-in-pattern |
| `dvs-rpkg/.gitignore` | 003-gitignore-patterns |
| `dvs-rpkg/src/entrypoint.c.in` | (same as miniextendr) |
| `dvs-rpkg/src/mx_abi.c.in` | (same as miniextendr) |
| `dvs-rpkg/src/rust/cargo-config.toml.in` | 001-build-system-overview |

## Code Changes Required

### 1. `--locked` Flag Inversion (HIGH PRIORITY)

Current (miniextendr):
```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_LOCKED_FLAG=""
else
  CARGO_LOCKED_FLAG="--locked"
fi
```

Proposed (match DVS2):
```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_LOCKED_FLAG="--locked"
else
  CARGO_LOCKED_FLAG=""
fi
```

### 2. Auto-generate Feature CPPFLAGS (MEDIUM PRIORITY)

Current (miniextendr):
```m4
case ",$MINIEXTENDR_FEATURES," in
  *,nonapi,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_NONAPI" ;;
esac
case ",$MINIEXTENDR_FEATURES," in
  *,rayon,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_RAYON" ;;
esac
```

Proposed (DVS2 pattern):
```m4
if test -n "$MINIEXTENDR_FEATURES"; then
  CARGO_FEATURES_FLAG="--features=$MINIEXTENDR_FEATURES"
  CARGO_FEATURE_CPPFLAGS=""
  IFS=','
  for _feature in $MINIEXTENDR_FEATURES; do
    _feature_upper=$(echo "$_feature" | tr 'a-z-' 'A-Z_')
    CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_$_feature_upper"
  done
  unset IFS
fi
```

## Conclusion

DVS2 validates miniextendr's design while revealing refinements. The most impactful change is the `--locked` flag inversion, which affects both reproducibility (DEV) and CRAN compatibility. The other changes are quality-of-life improvements that reduce maintenance burden.
