# Plan: pass cargo's native link settings into R's final link

**Base:** `origin/main` (this branch was cut from it).
**Status:** planned, not implemented. Pick up cold from here.

## Problem

R's build system performs the final link of the package `.so`, not cargo. The
Rust side is compiled to a `staticlib` (`libminiextendr.a`) and R links it in via
`PKG_LIBS`. Today `PKG_LIBS = $(CARGO_AR)` lists **only the archive** — none of
the native libraries that archive depends on.

That works for pure-Rust crates (the std baseline `-lSystem -lc -lm` etc. is
already added by R's `cc` link driver). It **breaks** the moment a scaffolded
package pulls a `-sys` crate that links a system library (OpenSSL, ODBC, …): the
`.a` references `-lssl`/`-lcrypto` symbols R never puts on the link line, and the
install fails to link.

Goal: capture cargo's resolved native link settings during the build we already
run, and feed them into R's `PKG_LIBS`.

Reference material: `~/Documents/GitHub/linking_to_openssl/LINKING.md` (full
walkthrough + per-platform results) and `sys-deps/corpus.md` (only 3/64 extendr
packages need a provided system lib; the Windows line is hand-maintained and
drift-prone across the ecosystem — deriving it is the whole value).

## Empirically verified (macOS arm64, against `linking_to_openssl`)

1. `--print native-static-libs` gives the `-l` list but **NOT** `-L` search
   paths:
   ```
   note: native-static-libs: -lssl -lcrypto -liconv -lSystem -lc -lm
   ```
2. The `-L` search path lives **only** in cargo build-script `output` files:
   ```
   $CARGO_LIBDIR/build/*/output:
     cargo:rustc-link-lib=dylib=ssl
     cargo:rustc-link-lib=dylib=crypto
     cargo:rustc-link-search=native=/opt/homebrew/opt/openssl@3/lib
   ```
   So for a system lib in a non-default prefix (homebrew OpenSSL) we need BOTH
   sources — `-l` from the print, `-L` scraped from build outputs. This is why
   "read those files" is plural.
3. `cargo rustc --lib … -- --print native-static-libs=FILE` writes a **clean**,
   `@file`-ready file (no `note:` prefix, no trailing newline):
   `-lssl -lcrypto -liconv -lSystem -lc -lm`
4. `cc … @file` is honored by the compiler driver (clang on macOS; gcc on
   linux/win-gnu). So we use a linker **`@file`** — this means **no GNU-make
   `$(shell)`**, so we do NOT regress into `SystemRequirements: GNU make` (the
   Makevars is deliberately POSIX; see its header comment).

Why `@file` is ordering-safe: `@file` is inert text at Makefile-parse time. It
only needs to exist on disk when `cc` runs, i.e. in the `$(SHLIB)` link recipe,
which already has `$(SHLIB): $(OBJECTS) $(CARGO_AR)` as a prereq. The
`$(CARGO_AR)` recipe produces the file, so it exists by link time. No
`$(shell)`-timing subtlety.

## Changes — edit `rpkg/` as master, then port to templates

`rpkg/src/Makevars.in` is byte-identical to
`minirextendr/inst/templates/{rpkg,monorepo/rpkg}/Makevars.in` (verified), and
`Makevars.win` / `configure.win` mirror it too. So: edit rpkg, then
`just templates-approve` + `just templates-check` carries all three template
copies.

### 1. `rpkg/src/Makevars.in` — the `$(CARGO_AR)` recipe

- Switch `cargo build --lib` → `cargo rustc --lib … -- --print
  native-static-libs=$(NATIVE_STATIC_LIBS_TXT)`. `cargo rustc` passes trailing
  args to the **top crate only**, so deps stay quiet and only the staticlib
  prints. Keep the existing `RUSTFLAGS="… $$LINK_ARGS"` passing of `$(OBJECTS)`
  as-is.
- Only add the `--print` on native — gate on `[ "$(IS_WASM_INSTALL)" != "true" ]`
  (emcc does its own link; don't ask it for a native print).
- After cargo, assemble one combined `@file` at `$(LINK_ARGS_TXT)`:
  - the native-static-libs content (the `-l` list), then
  - `-L`/`-F` lines scraped from build-script outputs:
    ```sh
    grep -rh '^cargo:rustc-link-search=' "$(CARGO_LIBDIR)"/build/*/output 2>/dev/null \
      | sed -E 's|^cargo:rustc-link-search=framework=|-F|; \
                s|^cargo:rustc-link-search=(native=\|all=\|dependency=\|crate=)?|-L|' \
      | sort -u
    ```
- **Always ensure `$(LINK_ARGS_TXT)` exists** (create empty if cargo skipped a
  fresh relink — see ceiling below) so `@file` never dangles.
- `PKG_LIBS = $(CARGO_AR) @LINK_ARGS_REF@` (configure substitutes the ref).

### 2. `rpkg/configure.ac`

- Compute link-file paths under `$(CARGO_TARGET_DIR)` (gitignored, transient,
  **outside `src/`** → no VCS dirtying, no tarball bloat). e.g.
  `NATIVE_STATIC_LIBS_TXT=$CARGO_TARGET_DIR/miniextendr-native-static-libs.txt`,
  `LINK_ARGS_TXT=$CARGO_TARGET_DIR/miniextendr-link-args.txt`. `AC_SUBST` both.
- `AC_SUBST([LINK_ARGS_REF])` = `@<abs LINK_ARGS_TXT>` on native, empty string
  when `is_wasm_install=true`. This is the wasm gate for `PKG_LIBS`.

### 3. `rpkg/src/Makevars.win`

- Replace the hardcoded `-lws2_32 -lntdll -luserenv -lbcrypt -ladvapi32
  -lsecur32` with the same derived `@file`:
  `PKG_LIBS = $(CARGO_AR) @LINK_ARGS_REF@` (drop the override's manual list; the
  `$(CARGO_AR)` full path already avoids the `.dll.a` pickup the old comment
  worried about).
- **Windows risk to watch (gate merge on Windows CI):** win-gnu
  `native-static-libs` references `libgcc_eh`, which the Rtools UCRT toolchain
  doing R's final link does not provide. Corpus documents the fix (create an
  empty `libgcc_eh.a` mock — see `linking_to_openssl/sys-deps/corpus.md` "CRT
  fault line"). If CI fails on this, add the mock or fall back to the hardcoded
  list for Windows only.

### 4. Propagate to templates

```
just templates-approve   # regenerate templates delta from rpkg
just templates-check     # verify no unexpected drift
```

## Verify — on a FRESHLY SCAFFOLDED package, NOT rpkg

rpkg is a specialised monorepo/source-mode consumer; the real target is an
end-user scaffolded package, which also exercises tarball/offline install.

1. Scaffold a new package via minirextendr (see `minirextendr-roundtrip` /
   templates E2E harness: `MINIEXTENDR_LOCAL_PATH`, `MINIEXTENDR_RUN_E2E=1`).
2. Add `openssl-sys` to the scaffolded crate's `Cargo.toml` and a one-line symbol
   reference so it isn't dead-stripped (mirror
   `linking_to_openssl/src/lib.rs`):
   ```rust
   pub fn openssl_version() -> std::os::raw::c_ulong {
       unsafe { openssl_sys::OpenSSL_version_num() }
   }
   ```
3. Install it and confirm `-lssl -lcrypto -L/opt/homebrew/opt/openssl@3/lib`
   reach R's final link and it resolves. Check both **source** and
   **tarball/offline** mode — build scripts still emit `rustc-link-search` when
   vendored, so the `-L` scrape holds offline.
4. Confirm a plain scaffold (pure-Rust, no system dep) still installs clean —
   baseline libs are redundant but harmless.
5. `just minirextendr-roundtrip` (templates E2E) green. Windows CI green before
   merge.

## Ceilings (mark with `ponytail:` comments in the code)

- **Incremental skip:** cargo re-emits the print only when it actually relinks.
  Between relinks the link file is stale-but-correct (deps unchanged). `cargo
  clean` forces refresh. Don't delete the file between builds.
- **wasm:** excluded via the configure `LINK_ARGS_REF` gate.

## Sandbox note

Any compiling step (`just rcmdinstall`, `R CMD INSTALL`, `cargo build`) needs
`dangerouslyDisableSandbox: true` via the Bash tool.
