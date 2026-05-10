# `rpkg/configure.ac` wasm32 detection + `rpkg/build.rs` snapshot validation

Two coupled rpkg-side compile-time concerns that close out the wasm32
install path. Both are step 8 of `webr-support.md` (split here for
clarity).

Status: **not started**. The plumbing for wasm32 cargo check works
(steps 2–7 merged), but neither piece below is wired up. Without them:
- `R CMD INSTALL` of rpkg with `CC=emcc` doesn't pass
  `--target wasm32-unknown-emscripten` to cargo (step 8).
- A stale or missing `wasm_registry.rs` produces a confusing link error
  rather than an actionable build-time message (build.rs).

## Part A — `rpkg/configure.ac`: detect `CC=emcc`, set wasm flags

`webr-vars.mk` overrides `CC=emcc` and `LDFLAGS=-s SIDE_MODULE=1` for
every R package install under WASM (see `.webr/packages/webr-vars.mk`).
rpkg's `configure.ac` currently doesn't know about that — it writes
`Makevars` + `.cargo/config.toml` assuming a native build.

### What configure.ac needs to do under `CC=emcc`

1. **Set `CARGO_BUILD_TARGET=wasm32-unknown-emscripten`** so the
   staticlib step in `Makevars.in` builds for the right target. The
   variable is already plumbed through `Makevars.in`'s `TARGET_OPT`
   conditional (`if [ -n "$(CARGO_BUILD_TARGET)" ]`), so this is a
   single substitution.

2. **Set `IS_WASM_INSTALL=true`** — new substitution. Used to gate the
   cdylib + wrapper-gen pass off (no host R can `dyn.load` a wasm side
   module). The `WRAPPERS_R` recipe in `Makevars.in` becomes a no-op
   on wasm32 — both `R/<pkg>-wrappers.R` and `src/rust/wasm_registry.rs`
   are expected to be pre-generated on host and shipped into the
   tarball / bundle.

3. **Pass `RUSTFLAGS`** for emscripten side-module link:
   `-C relocation-model=pic -C link-args=-s SIDE_MODULE=1` (exact set
   TBD — verify against rwasm's flags. Plan `webr-support.md` flagged
   this as needing experimental confirmation.)

4. **Use nightly + `-Z build-std=std,panic_abort`** because rustup's
   precompiled std for `wasm32-unknown-emscripten` is built against
   *some* Emscripten that may not match webR's pinned version, and
   doesn't have `panic = "abort"` baked into std. Set
   `RUST_TOOLCHAIN=+nightly` and add
   `-Z build-std=std,panic_abort` to the cargo invocation.

   Plumbing: `RUST_TOOLCHAIN` is already a configure substitution. Add
   `CARGO_BUILD_STD_FLAG = -Z build-std=std,panic_abort` (empty on
   native). Thread through the `cargo build` and `cargo rustc` lines
   in `Makevars.in`.

5. **Refuse to build if `wasm_registry.rs` is absent.** configure
   fails with a clear message ("Run `R CMD INSTALL` on host first to
   generate `src/rust/wasm_registry.rs`, then ship the source tree
   to the wasm builder"). Detection: `[ -f
   "$srcdir/src/rust/wasm_registry.rs" ]`.

### Detection logic

```m4
# In configure.ac, near the existing CC detection.
AC_MSG_CHECKING([whether CC is emscripten (webR/wasm32 install)])
case "${CC}" in
  *emcc*|*em++*)
    is_wasm_install=true
    CARGO_BUILD_TARGET=wasm32-unknown-emscripten
    RUST_TOOLCHAIN="+nightly"
    CARGO_BUILD_STD_FLAG="-Z build-std=std,panic_abort"
    AC_MSG_RESULT([yes])
    ;;
  *)
    is_wasm_install=false
    CARGO_BUILD_TARGET=
    CARGO_BUILD_STD_FLAG=
    AC_MSG_RESULT([no])
    ;;
esac

# Refuse if wasm_registry.rs is missing
if test "$is_wasm_install" = true; then
  if test ! -f "$srcdir/src/rust/wasm_registry.rs"; then
    AC_MSG_ERROR([wasm32 install requires src/rust/wasm_registry.rs.
Run `R CMD INSTALL` (or `just rcmdinstall`) on the host to regenerate
it from the cdylib slice contents, then ship the source tree to the
wasm builder.])
  fi
fi

AC_SUBST([IS_WASM_INSTALL], [$is_wasm_install])
AC_SUBST([CARGO_BUILD_TARGET])
AC_SUBST([CARGO_BUILD_STD_FLAG])
```

Don't forget `autoconf` to regenerate `rpkg/configure` and commit both.

### Makevars.in changes

```make
# New substitution (configure-supplied)
IS_WASM_INSTALL       = @IS_WASM_INSTALL@
CARGO_BUILD_STD_FLAG  = @CARGO_BUILD_STD_FLAG@

# In CARGO_AR / CARGO_CDYLIB recipes, add the build-std flag:
$(CARGO) $(RUST_TOOLCHAIN) build $(CARGO_OFFLINE_FLAG) $(CARGO_FEATURES_FLAG) $(CARGO_BUILD_STD_FLAG) $$TARGET_OPT \
  …

# Gate the wrapper-gen recipe to native:
$(WRAPPERS_R): $(CARGO_CDYLIB)
ifeq ($(IS_WASM_INSTALL),true)
	@echo "wasm32 install: skipping wrapper-gen (uses pre-generated R/$(pkg)-wrappers.R + src/rust/wasm_registry.rs)"
	@touch $(WRAPPERS_R)  # force timestamp so make doesn't retry
else
	@echo "Generating R wrappers + wasm_registry.rs..."
	# …existing recipe…
endif
```

Ports to `minirextendr/inst/templates/{,monorepo/}rpkg/Makevars.in` per
the project's templates-derived-from-rpkg convention (CLAUDE.md). Run
`just templates-approve` to lock the delta in `patches/templates.patch`.

### Non-goals

- Detecting webR via mechanisms other than `CC=emcc`. The `--with-wasm`
  configure flag mentioned in older planning is dropped — `CC=emcc` is
  the unambiguous signal that webR's `webr-vars.mk` is in scope.
- Side-module link flags beyond what's strictly needed. Defer
  optimisation (`-O3`, LTO) until a working build exists.
- `tests/cross-package/*` configure changes — those packages don't ship
  a `webr-vars.mk` install path. See `webr-cross-package-stubs.md`.

## Part B — `rpkg/build.rs`: validate `wasm_registry.rs`

`miniextendr_init!` macro emits `#[cfg(target_arch = "wasm32")]
#[path = "wasm_registry.rs"] mod __miniextendr_wasm_registry;` and
references three statics from it. Without `wasm_registry.rs`:

- File missing → `mod` declaration fails to resolve. rustc gives a
  filesystem path error pointing at the macro expansion. Confusing.
- File present but **stale** (generated against an older miniextendr-api
  with a different slice layout) → compiles, links, then crashes
  at `R_init` with `register: <fnptr>` pointing to whatever bytes
  happened to be at that address.

Both are catastrophic and avoidable. A tiny `build.rs` in rpkg gets us
a clear early diagnostic.

### What build.rs should do (wasm32 only)

```rust
// rpkg/src/rust/build.rs
fn main() {
    // Re-run if the snapshot or this build script changes
    println!("cargo:rerun-if-changed=wasm_registry.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "wasm32" {
        return;
    }

    let path = std::path::Path::new("wasm_registry.rs");
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => panic!(
            "wasm32 build requires src/rust/wasm_registry.rs.\n\
             Run `just rcmdinstall` (or equivalent) on the host to regenerate it."
        ),
    };

    // Parse `// generator-version: <N>` from the header
    let header_version: u32 = content
        .lines()
        .find_map(|l| l.strip_prefix("// generator-version: "))
        .and_then(|v| v.trim().parse().ok())
        .expect("wasm_registry.rs missing or malformed `generator-version` header");

    // Must match miniextendr-api's GENERATOR_VERSION constant
    const EXPECTED_VERSION: u32 = 1; // bump in lockstep with
                                      // miniextendr-api/src/wasm_registry_writer.rs
    if header_version != EXPECTED_VERSION {
        panic!(
            "wasm_registry.rs was generated by an older miniextendr (generator-version {header_version}, \
             this build expects {EXPECTED_VERSION}). Run `just rcmdinstall` on host to regenerate."
        );
    }

    // Optional: validate content-hash matches (defends against
    // hand-edited or partial snapshots). The header carries the
    // FNV-1a-64 hash of the body; recompute and compare.
    // Skip for v1 — the timing window where a stale snapshot is
    // shipped is tight enough that generator-version catches most cases.
}
```

### Coupling to miniextendr-api

`EXPECTED_VERSION` must mirror `miniextendr-api/src/wasm_registry_writer.rs`'s
`GENERATOR_VERSION` constant. Two ways to keep them in sync:

1. **Hand-mirror with a comment** (current sketch above). Cheap;
   relies on convention. Bump both together when the format changes.
2. **Re-export from `miniextendr-api` and read in build.rs**. Means
   build.rs has `miniextendr-api` as a build-dependency, which is
   awkward (chicken-and-egg if api itself fails to build).

Lean toward (1). The format changes rarely — every macro/struct
reshape that affects emission. A lint or test in the api crate that
greps for `GENERATOR_VERSION` mentions and bumps both together would
help, but is overengineering until we hit a v2.

### Ports

build.rs is rpkg-specific (it reads rpkg's `wasm_registry.rs`). If
cross-package crates eventually grow their own snapshots
(`webr-cross-package-stubs.md`), they each get their own build.rs
with the same pattern.

## Order of operations

**Land Part A first, then Part B.** Part A makes wasm32 install
actually pass cargo + linker. Part B is the diagnostic improvement on
top — useful but not blocking.

Each is a small focused PR. Combined diff is ~150 lines (configure.ac
+ Makevars.in updates + templates port + build.rs + minor CLAUDE.md
note).

## Verification

For Part A:
- Inside `just docker-webr-shell`: `bash ./rpkg/configure` writes
  `Makevars` with `IS_WASM_INSTALL=true` and `CARGO_BUILD_STD_FLAG`
  populated. `make -C rpkg/src` builds a wasm side-module.
- On native: `IS_WASM_INSTALL=false`, build is unchanged.

For Part B:
- Delete `rpkg/src/rust/wasm_registry.rs`, run `cargo check --target
  wasm32-unknown-emscripten`. Expect: build.rs panic with the
  "Run `just rcmdinstall`" message.
- Tamper with the file's `// generator-version: 1` line (set to 2),
  re-run cargo check. Expect: build.rs panic with version-mismatch
  message.
- Native cargo check: build.rs is a no-op (early-returns on non-wasm32).
