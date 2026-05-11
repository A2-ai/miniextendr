# tests/cross-package

Trait-ABI integration tests. Two R packages — `producer.pkg` exports trait impls, `consumer.pkg` imports them via `R_GetCCallable` — verify the vtable shim machinery and `mx_abi` registration across DLL boundaries. See root `CLAUDE.md` for shared rules.

## Dev loop
```bash
just cross-install   # build + install both packages
just cross-test      # run testthat across the pair
just cross-check     # R CMD check both
```

## Layout
- `producer/` + `producer.pkg/` — split source vs scaffolded tarball.
- `consumer/` + `consumer.pkg/` — same.
- `shared-traits/` — the trait declarations both sides depend on.
- `bench-interop.R` — cross-package interop benchmark.

## Why packages have dotted names
`PACKAGE_NAME` (Autoconf) preserves dots (`producer.pkg`); `PACKAGE_TARNAME` lowercases + normalises. When deriving C/Rust identifiers, use `PACKAGE_NAME` but convert **both hyphens AND dots** to underscores. `sed 's/[.-]/_/g'` doesn't work inside m4 — bracket expressions get eaten. Use `sed 's/-/_/g; s/\./_/g'`.

## Why this matters
- `R_GetCCallable("pkg", "fn")` **throws an R error** (longjmp) on miss — does NOT return NULL. NAMESPACE `importFrom(vctrs, ...)` (or any function from the producer package) forces the producer DLL to load before the consumer DLL resolves its callables.
- Trait-ABI vtable shims use `Rf_error` (not `error_in_r`) — see `miniextendr-macros/src/miniextendr_trait.rs:808` and the `with_r_unwind_protect` leak note in root `CLAUDE.md`.

## Gotcha
- Each cross-package has its own `Cargo.toml` and its own `[patch.crates-io]` — `just check/clippy/test` iterate them via the workspace recipes. Raw `cargo --workspace` from the repo root won't include them.
