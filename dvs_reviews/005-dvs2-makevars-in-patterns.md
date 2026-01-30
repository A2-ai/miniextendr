# DVS2 Makevars.in Patterns Review

## Overview

DVS2's `Makevars.in` demonstrates several clean patterns for R package builds with Rust backends.

## Key Patterns

### 1. Clean Variable Section

```makefile
## --- filled by configure ----------------------------------------------
ABS_TOP_SRCDIR = @ABS_TOP_SRCDIR@
ABS_RPKG_SRCDIR = @ABS_RPKG_SRCDIR@
CARGO = @CARGO@
...
## ----------------------------------------------------------------------
```

Clear demarcation between configured variables and build rules.

### 2. Static Library Reference

```makefile
CARGO_AR = $(CARGO_LIBDIR)/lib$(CARGO_STATICLIB_NAME).a
PKG_LIBS = -L$(CARGO_LIBDIR) -l$(CARGO_STATICLIB_NAME)
```

Uses the static library name variable for flexibility.

### 3. R Wrappers Paths

```makefile
R_WRAPPERS_GENERATED = $(ABS_RPKG_SRCDIR)/@PACKAGE_TARNAME@-wrappers.R
R_WRAPPERS_CURRENT   = $(ABS_RPKG_SRCDIR)/../R/@PACKAGE_TARNAME@-wrappers.R
```

Two-stage R wrapper handling:
1. Generate to `src/` directory
2. Copy to `R/` directory

This keeps generated files in `src/` during build but installs them to `R/`.

### 4. FORCE_CARGO Phony Target

```makefile
.PHONY: all clean FORCE_CARGO

# Phony target to force Cargo invocation
FORCE_CARGO:

$(CARGO_AR): FORCE_CARGO $(CARGO_TOML) $(CARGO_LOCK) ...
```

Forces Cargo to run every time, letting Cargo's own incremental compilation decide what to rebuild.

### 5. NOT_CRAN Mode Handling

```makefile
all: $(SHLIB)
	@# Dev mode: touch Cargo.toml so pkgbuild always thinks sources changed
	@if [ "$(NOT_CRAN_FLAG)" = "true" ]; then \
	  touch "$(CARGO_TOML)"; \
	fi
	@# CRAN mode: cleanup vendor and target directories to save space
	@if [ "$(NOT_CRAN_FLAG)" != "true" ]; then \
	  ...cleanup...
	fi
```

**Dev mode**: Touches `Cargo.toml` to force rebuilds (useful when editing Rust code)
**CRAN mode**: Cleans up large directories after build to save installed package size

### 6. Vendor Tarball Unpacking in Makefile

```makefile
$(CARGO_AR): FORCE_CARGO ...
	@# Unpack vendor.tar.xz if vendor/ is missing (CRAN tarball case)
	@if [ ! -d "$(VENDOR_OUT)" ] && [ -f "$(VENDOR_TARBALL)" ]; then \
	  echo "Unpacking vendor.tar.xz from inst/..."; \
	  tar -xJf "$(VENDOR_TARBALL)" -C "$(ABS_RPKG_SRCDIR)"; \
	fi
```

Fallback unpacking in Makefile catches cases where configure didn't unpack.

### 7. Feature Flags to C Preprocessor

```makefile
PKG_CPPFLAGS = @CARGO_FEATURE_CPPFLAGS@
```

Passes cargo features to C compiler for conditional compilation in C code.

## Comparison with Miniextendr

Both projects have similar Makevars.in structure. Key differences:

1. **Wrapper path pattern**: DVS2 uses `@PACKAGE_TARNAME@-wrappers.R`, miniextendr uses hardcoded filename
2. **Feature flags**: DVS2 auto-generates CPPFLAGS from features, miniextendr is manual

## Recommendations

1. **Adopt wildcard wrapper pattern**: Use `@PACKAGE_TARNAME@-wrappers.R` for flexibility
2. **Auto-generate CPPFLAGS**: Implement DVS2's feature-to-CPPFLAGS loop
3. **Verify vendor fallback**: The Makefile-level tarball unpacking is a good safety net
