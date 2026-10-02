# tests/cross-package

Trait-ABI integration tests. Two R packages — `producer.pkg` exports trait impls, `consumer.pkg` imports them via `R_GetCCallable` — verify the vtable shim machinery and `mx_abi` registration across DLL boundaries. See root `CLAUDE.md` for shared rules.

## Dev loop
```bash
just cross-install   # build + install both packages
just cross-test      # run testthat across the pair
```

`just cross-check` (R CMD check both) is left out of this loop until #1716 is
fixed: `devtools::check()` installs from a tarball, and from there the packages'
relative path dependencies on the workspace crates don't resolve.

The R recipes run in the repo root (`[working-directory("../..")]`), where the
root `.Rprofile` activates rv, so devtools / roxygen2 / testthat come from rv's
library; they take the package path as an argument. They install into, load from
and test against `tests/cross-package/.r-lib/` (gitignored, one per checkout), so
worktrees never share `producer.pkg` / `consumer.pkg` installs. rv replaces
`.libPaths()` and drops `R_LIBS`, so each session prepends `.r-lib` itself
(`use_r_lib` in the justfile). `just r-lib` (in this directory) prints the path.
Run R by hand the same way, from the repo root:
`Rscript -e "$(just -f tests/cross-package/justfile --evaluate use_r_lib); library(consumer.pkg)"`.
Not from this directory: an R started here gets the personal library.

## Layout
- `producer/` + `producer.pkg/` — split source vs scaffolded tarball.
- `consumer/` + `consumer.pkg/` — same.
- `shared-traits/` — the trait declarations both sides depend on.
- `bench-interop.R` — cross-package interop benchmark.

## Why packages have dotted names
`PACKAGE_NAME` (Autoconf) preserves dots (`producer.pkg`); `PACKAGE_TARNAME` lowercases + normalises. When deriving C/Rust identifiers, use `PACKAGE_NAME` but convert **both hyphens AND dots** to underscores. `sed 's/[.-]/_/g'` doesn't work inside m4 — bracket expressions get eaten. Use `sed 's/-/_/g; s/\./_/g'`.

## Why this matters
- `R_GetCCallable("pkg", "fn")` **throws an R error** (longjmp) on miss — does NOT return NULL. NAMESPACE `importFrom(vctrs, ...)` (or any function from the producer package) forces the producer DLL to load before the consumer DLL resolves its callables.
- Trait-ABI vtable shims wrap in `with_r_unwind_protect_shim` (returns a tagged error SEXP that the View method re-panics into the consumer's outer `with_r_unwind_protect` guard) — see `miniextendr-macros/src/miniextendr_trait.rs:808` and the `with_r_unwind_protect` leak note in root `CLAUDE.md`.

## Gotcha
- Each cross-package has its own `Cargo.toml` and its own `[patch.crates-io]` — `just check/clippy/test` iterate them via the workspace recipes. Raw `cargo --workspace` from the repo root won't include them.
