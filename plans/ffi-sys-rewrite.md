# ffi → sys structural rewrite

Locked-in plan derived from the audit. No backwards compatibility shims.

## Goal

Rename `pub mod ffi` → `pub mod sys`, split the 4047-line `ffi.rs` + 716-line
`ffi/altrep.rs` into four narrower modules, expand the prelude so the
ergonomic surface (`SEXP`, `SEXPTYPE`, `R_xlen_t`, `SexpExt`, `RNativeType`,
`RLogical`, `Rboolean`, `Rcomplex`) is reachable through
`use miniextendr_api::prelude::*;`, and add a `SEXP::alloc()` helper so tests
can stop reaching for raw `Rf_allocVector` + `Rf_protect`/`Rf_unprotect`.

## File-by-file move list

Current `miniextendr-api/src/ffi.rs` (4047 lines), `miniextendr-api/src/ffi/altrep.rs` (716 lines).

Destination layout (no `mod.rs`; use `sys.rs` + `sys/` directory):

| Destination file | Source line range (ffi.rs) | Content |
|---|---|---|
| `sexp_types.rs` | 9–13, 16–104, 1672–1885, 1887–1916 | `R_xlen_t`, `Rbyte`, `Rcomplex`, `SEXPTYPE` enum + impl, `RNativeType` trait + impls, `RLogical`, `Rboolean`, `R_CFinalizer_t`, `cetype_t` + `CE_UTF8` re-export |
| `sexp.rs` | 106–555 | `SEXPREC`, `SEXP` + `Send`/`Sync`, all 72 inherent methods, `Default`/`From` impls. Adds `SEXP::alloc(ty, n)`. |
| `sexp_ext.rs` | 557–1670 | `SexpExt` trait (~86 methods) + `impl SexpExt for SEXP`, `PairListExt` (already `pub(crate)`) + impl |
| `sys.rs` | 1918–4047 | Pure FFI: extern blocks, `Rf_error`/`Rf_warning`/`Rprintf`/`REprintf` hand-rolled, `Rconnection_impl`, `R_CONNECTIONS_VERSION`, `IDENT_*` consts, `DllInfo`, `R_C*MethodDef`, RNG enums, `Rf_lang1..6` / `Rf_list1..4` inline helpers, varargs trampolines, all `unsafe extern "C-unwind"` blocks. |
| `sys/altrep.rs` | (current `ffi/altrep.rs` entire) | Raw ALTREP C API method type aliases. `pub` where macros need them (most cases — codegen emits paths through these), `pub(crate)` otherwise. |

Module boundaries: `sys.rs` brings in `pub mod altrep;` for `sys::altrep`.
`sexp_types.rs`, `sexp.rs`, and `sexp_ext.rs` are siblings, not under
`sys/`, because they aren't "raw FFI" — they're the safe Rust vocabulary on
top of FFI.

Internal `use crate::ffi::…` paths inside `miniextendr-api/src/` need updating:
- `use crate::ffi::SEXP` → `use crate::SEXP` (now at crate root)
- `use crate::ffi::SexpExt` → `use crate::SexpExt`
- `use crate::ffi::SEXPTYPE` → `use crate::SEXPTYPE`
- `use crate::ffi::{Rf_*, R_*, …}` → `use crate::sys::{…}`
- `use crate::ffi::altrep::*` → `use crate::sys::altrep::*`
- `use crate::ffi::PairListExt` → `use crate::sexp_ext::PairListExt`

## Crate-root re-exports added to `lib.rs`

Promote the ergonomic types to crate root so prelude can re-export from there:

```rust
pub mod sys;
pub mod sexp;
pub mod sexp_ext;
pub mod sexp_types;

pub use sexp::SEXP;
pub use sexp_ext::SexpExt;
pub use sexp_types::{
    R_xlen_t, Rbyte, Rcomplex, SEXPTYPE,
    RLogical, Rboolean, RNativeType, cetype_t,
    R_CFinalizer_t,
};
```

`sys` stays a module; nothing from inside it is promoted to crate root
beyond the inherent re-exports above (which were already in `crate::ffi`
land too).

`SEXPREC` stays at crate root for compatibility with prior `crate::ffi::SEXPREC`
references (re-exported from `crate::sexp`).

## Prelude expansion (`prelude.rs`)

Replace:

```rust
// region: FFI (SEXP is needed in almost every crate)
pub use crate::ffi::SEXP;
// endregion
```

With:

```rust
// region: FFI core types
pub use crate::{
    R_xlen_t, RLogical, RNativeType, Rboolean, Rcomplex, SEXP, SEXPTYPE, SexpExt,
};
// endregion
```

## `SEXP::alloc(SEXPTYPE, R_xlen_t) -> SEXP`

New helper in `sexp.rs`:

```rust
impl SEXP {
    /// Allocate a fresh R vector of the given type and length.
    ///
    /// Direct wrapper over `Rf_allocVector`. For most cases, use the typed
    /// constructors (`SEXP::integer_vector`, etc.) or `OwnedProtect::alloc`
    /// — this is the low-level escape hatch when the SEXPTYPE is dynamic.
    #[inline]
    pub fn alloc(ty: SEXPTYPE, n: R_xlen_t) -> SEXP {
        unsafe { Rf_allocVector(ty, n) }
    }
}
```

## Macro codegen rename pass

`miniextendr-macros/src/*.rs` references to `::miniextendr_api::ffi::Foo` →
new paths:

| Old path | New path |
|---|---|
| `::miniextendr_api::ffi::SEXP` | `::miniextendr_api::SEXP` |
| `::miniextendr_api::ffi::SEXPTYPE` | `::miniextendr_api::SEXPTYPE` |
| `::miniextendr_api::ffi::R_xlen_t` | `::miniextendr_api::R_xlen_t` |
| `::miniextendr_api::ffi::Rboolean` | `::miniextendr_api::Rboolean` |
| `::miniextendr_api::ffi::Rcomplex` | `::miniextendr_api::Rcomplex` |
| `::miniextendr_api::ffi::Rbyte` | `::miniextendr_api::sys::Rbyte` (or crate root, decide) |
| `::miniextendr_api::ffi::RLogical` | `::miniextendr_api::RLogical` |
| `::miniextendr_api::ffi::RNativeType` | `::miniextendr_api::RNativeType` |
| `::miniextendr_api::ffi::SexpExt` | `::miniextendr_api::SexpExt` |
| `::miniextendr_api::ffi::altrep::…` | `::miniextendr_api::sys::altrep::…` |
| `::miniextendr_api::ffi::<other extern fn / type>` | `::miniextendr_api::sys::<…>` |

Files to update (counts from grep):

- `miniextendr-macros/src/return_type_analysis.rs` (4)
- `miniextendr-macros/src/dataframe_derive/enum_expansion.rs` (8)
- `miniextendr-macros/src/miniextendr_trait.rs` (6)
- `miniextendr-macros/src/miniextendr_impl.rs` (8)
- `miniextendr-macros/src/lib.rs` (21 — incl. diagnostic strings)
- `miniextendr-macros/src/externalptr_derive.rs` (19)
- `miniextendr-macros/src/altrep.rs` (21)
- `miniextendr-macros/src/list_derive.rs` (16)
- `miniextendr-macros/src/c_wrapper_builder.rs` (25)
- `miniextendr-macros/src/dataframe_derive.rs` (6)
- `miniextendr-macros/src/factor_derive.rs` (10)
- `miniextendr-macros/src/vctrs_derive.rs` (12)
- `miniextendr-macros/src/altrep_derive.rs` (12)
- `miniextendr-macros/src/match_arg_derive.rs` (4)
- `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs` (2)

Total ≈ 174 sites in macros.

Diagnostic strings in `miniextendr-macros/src/lib.rs` referring to
`miniextendr_api::ffi::SEXP` as the recommended return type → update to
`miniextendr_api::SEXP`.

UI test stderr snapshots in `miniextendr-macros/tests/ui/*.stderr` may
reference the old paths — regenerate with
`TRYBUILD=overwrite cargo test -p miniextendr-macros`, review diff.

## Lint updates

- `miniextendr-lint/src/rules/ffi_unchecked.rs`: diagnostic wording
  `"`ffi::{}()` …"` → `"`sys::{}()` …"`. Module file name kept (`ffi_unchecked.rs`)
  because the file path isn't user-visible, but consider keeping `MXL301`
  text aligned with the new `sys::` paths.
- `miniextendr-lint/src/crate_index.rs:769`: source-text scanner searches
  for the literal `"ffi::"` to find `_unchecked` calls — switch to `"sys::"`.
- `miniextendr-lint/src/rules.rs:40`: comment reference to `ffi::*_unchecked()` →
  `sys::*_unchecked()`.

## Consuming files

- ≈125 unique files under `miniextendr-api/tests/`, `rpkg/src/rust/`,
  `tests/cross-package/`, `miniextendr-bench/` that `use miniextendr_api::ffi::…`.
  Most can simply rename `ffi` → `sys`. Where the imported symbols are now
  in prelude (`SEXP`, `SEXPTYPE`, `SexpExt`, `RNativeType`, `R_xlen_t`),
  optionally drop the explicit import — but a blanket sed is safest.
- ≈116 internal `crate::ffi::` references inside `miniextendr-api/src/`.

## Risks

1. **`Rf_error` callers in `rpkg/src/rust/worker_tests.rs` with `// mxl::allow(MXL300)`** —
   path change `miniextendr_api::ffi::Rf_error` → `miniextendr_api::sys::Rf_error`.
   `Rf_error` is kept `pub` per locked decision (genuine consumers in
   trait-ABI vtable shims and ALTREP `RUnwind` guard).
2. **ALTREP type alias visibility** — codegen path
   `::miniextendr_api::ffi::altrep::R_altrep_*` is consumed only by
   `miniextendr-macros`. Keep `pub` so the emitted code outside this crate
   can name the types.
3. **UI test stderr snapshots** — `TRYBUILD=overwrite` regenerates them;
   review for unexpected error wording shifts.
4. **`Rconnection_impl` is feature-gated** under `connections`. Must stay
   under the same `#[cfg(feature = "connections")]` gate in `sys.rs`.
5. **`#[r_ffi_checked]`** macro tags extern blocks. The proc-macro produces
   `*_unchecked` siblings. Moving extern blocks together must keep their
   `#[r_ffi_checked]` attribute and the `cfg(feature = "nonapi")` gates
   intact.
6. **The `lib.rs` module declaration order matters** if any module uses
   another's items at parse time. Declare `sys`, `sexp_types`, `sexp`,
   `sexp_ext` before modules that reference them.
7. **`crate::ffi::DllInfo`** is referenced in `lib.rs:355` —
   `pub fn altrep_dll_info() -> *mut ffi::DllInfo`. Update to
   `*mut sys::DllInfo`.

## Order of operations

1. Plan file commit (this file).
2. Create `sexp_types.rs`, `sexp.rs`, `sexp_ext.rs`, `sys.rs`, `sys/altrep.rs`
   alongside the existing `ffi.rs` (don't delete yet). Internal `use` paths
   in the new files use `crate::sys`, `crate::sexp`, etc.
3. Wire crate root: declare new modules in `lib.rs`, add re-exports,
   keep `pub mod ffi;` temporarily so the rest of the crate still compiles.
4. Expand prelude.
5. Add `SEXP::alloc(...)`.
6. Update macro codegen output paths.
7. Update lint source-text scanner + diagnostic wording.
8. Update macro diagnostic strings.
9. Update all consuming files (tests/, rpkg/, internal crate uses).
10. Delete `ffi.rs` and `ffi/altrep.rs`; remove `pub mod ffi;` from `lib.rs`.
11. `cargo check` clean, `cargo clippy` clean, `cargo test` clean.
12. Regenerate UI snapshots; review.
13. `just configure && just rcmdinstall && just force-document`.
14. `just devtools-test`.

## Go/no-go checklist for PR

- [ ] `cargo grep "miniextendr_api::ffi::"` returns zero hits across the repo.
- [ ] `grep "::ffi::"` inside `miniextendr-api/src/*.rs` returns zero hits
      (excluding `std::ffi::`).
- [ ] `cargo check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
      `cargo test` pass.
- [ ] CI's curated clippy feature set passes locally.
- [ ] `just configure && just rcmdinstall && just force-document` runs clean.
- [ ] `just devtools-test` passes.
- [ ] `*-wrappers.R` and `NAMESPACE` are in sync after `force-document`
      (pre-commit hook enforces this).
- [ ] UI snapshots regenerated and reviewed.
- [ ] Prelude exports `SEXP, SEXPTYPE, R_xlen_t, SexpExt, RNativeType, RLogical, Rboolean, Rcomplex`.
- [ ] `SEXP::alloc(SEXPTYPE, R_xlen_t)` exists.
- [ ] No new `#[allow(...)]` blankets added.
