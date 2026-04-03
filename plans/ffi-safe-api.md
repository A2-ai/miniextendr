# FFI Safe API Migration — Remaining Items

Item 1 (attribute access) is done. These items remain.

## 2. String construction and access (~126 call sites)

**Privatize**: `Rf_mkCharLenCE`, `Rf_mkCharCE`, `Rf_mkChar`, `Rf_mkCharLen`, `STRING_ELT`, `SET_STRING_ELT`, `R_CHAR`, `R_NaString`, `R_BlankString`, `Rf_translateCharUTF8`

**Add to `SexpExt`**:
- `string_elt(i: isize) -> SEXP` — `STRING_ELT` (CHARSXP element)
- `string_elt_str(i: isize) -> Option<&str>` — `STRING_ELT` + `R_CHAR` + NA check
- `set_string_elt(i: isize, charsxp: SEXP)` — `SET_STRING_ELT`
- `is_na_string() -> bool` — comparison to `R_NaString`

**Add to `impl SEXP`**:
- `SEXP::charsxp(s: &str) -> SEXP` — `Rf_mkCharLenCE` with CE_UTF8
- `SEXP::na_string() -> SEXP` — `R_NaString`
- `SEXP::blank_string() -> SEXP` — `R_BlankString`

**Key files**: `strvec.rs`, `from_r/strings.rs`, `into_r.rs`, `serde/columnar.rs`, `list.rs`, `optionals/time_impl.rs`, `optionals/arrow_impl.rs`

**Note**: `from_r.rs` already has `charsxp_to_str()` which wraps `R_CHAR`. The new `string_elt_str` composes `STRING_ELT` + `charsxp_to_str` + NA check.

## 3. Scalar construction (~60 call sites)

**Privatize**: `Rf_ScalarInteger`, `Rf_ScalarReal`, `Rf_ScalarLogical`, `Rf_ScalarRaw`, `Rf_ScalarComplex`, `Rf_ScalarString`

**Add to `impl SEXP`**:
- `SEXP::scalar_integer(i32) -> SEXP`
- `SEXP::scalar_real(f64) -> SEXP`
- `SEXP::scalar_logical(bool) -> SEXP` — converts bool to i32 (1/0)
- `SEXP::scalar_raw(u8) -> SEXP`
- `SEXP::scalar_complex(Rcomplex) -> SEXP`
- `SEXP::scalar_string(charsxp: SEXP) -> SEXP`
- `SEXP::scalar_string_from_str(s: &str) -> SEXP` — charsxp + ScalarString

**Key files**: `into_r.rs` (via `impl_scalar_into_r!` macro), `altrep_impl.rs`, `error_value.rs`, `gc_protect.rs`

**Note**: `gc_protect.rs` already has `ProtectScope::scalar_*()` methods wrapping these. The SEXP methods are for code outside protect scopes.

## 4. `R_NilValue` access (~76 call sites)

**Replace**: `R_NilValue` → `SEXP::null()`

`SEXP::null()` already exists. Migration is mechanical.

**Blocker**: Macro-generated code references `::miniextendr_api::ffi::R_NilValue` in `c_wrapper_builder.rs` and `altrep_derive.rs`. Update those to `::miniextendr_api::ffi::SEXP::null()` first, then make `R_NilValue` `pub(crate)`.

**Key files**: `into_r.rs` (19), `preserve.rs` (14), `gc_protect.rs` (13), `list.rs` (8), `expression.rs`, `miniextendr-macros/src/c_wrapper_builder.rs`, `miniextendr-macros/src/altrep_derive.rs`

## 5. List element access (~57 call sites)

**Privatize**: `VECTOR_ELT`, `SET_VECTOR_ELT`

**Add to `SexpExt`**:
- `vector_elt(i: isize) -> SEXP` — `VECTOR_ELT`
- `set_vector_elt(i: isize, val: SEXP)` — `SET_VECTOR_ELT`

`List` already has safe element access. Main work is migrating callers that bypass `List`.

**Key files**: `list.rs` (11), `into_r.rs` (8), `serde/ser.rs` (5), `serde/columnar.rs` (6), `from_r/collections.rs` (4)

## 6. Protection and allocation (~112 call sites)

**Privatize**: `Rf_allocVector`, `Rf_allocMatrix`, `Rf_protect`, `Rf_unprotect`, `R_PreserveObject`, `R_ReleaseObject`

Extend `ProtectScope` to be the allocator:
- `scope.alloc_vector(SEXPTYPE, n) -> Root`
- `scope.alloc_integer(n) -> Root` (etc. for each type)
- `scope.alloc_matrix(SEXPTYPE, nrow, ncol) -> Root`
- `scope.duplicate(SEXP) -> Root`
- `scope.shallow_duplicate(SEXP) -> Root`

Eliminates the allocate-then-protect gap. Methods are still `unsafe` (must be on R main thread) but structurally prevent forgetting to protect.

**This is the biggest structural change.** Most existing allocation code follows `let x = Rf_allocVector(...); Rf_protect(x);` — fusing these into `scope.alloc_*()` requires threading a `ProtectScope` through call chains.

**Key files**: `into_r.rs` (27), `list.rs` (11), `serde/columnar.rs` (13), `serde/ser.rs` (8), `error_value.rs` (7)

## 7. Data pointer access (~11 call sites outside RNativeType)

**Privatize**: `INTEGER`, `REAL`, `LOGICAL`, `RAW`, `COMPLEX`, `DATAPTR_RO`, `DATAPTR_OR_NULL`

`SexpExt::as_slice<T>()` and `as_slice_unchecked<T>()` already exist. Add:
- `as_mut_slice<T: RNativeType>() -> &mut [T]` — mutable data access with type check

`RNativeType::dataptr_mut` impls continue using `pub(crate)` FFI internally.

**Key files**: `from_r/na_vectors.rs`, `serde/de.rs`, `altrep_impl.rs`

## 8. Pairlist / expression building (~25 call sites)

**Privatize**: `CAR`, `CDR`, `TAG`, `CADR`, `CDDR`, `CADDR`, `SETCAR`, `SETCDR`, `SET_TAG`, `Rf_cons`, `Rf_lcons`, `Rf_install`, `Rf_lang1`–`Rf_lang6`, `Rf_list1`–`Rf_list4`

`RCall`/`RSymbol` in `expression.rs` already wrap most of this. Optional: add `PairList` wrapper for remaining direct cons-cell access.

**Note**: `Rf_install` must stay `pub` if macro codegen references it. Check `miniextendr-macros/src/` before privatizing.

**Key files**: `expression.rs`, `preserve.rs`, `error_value.rs`

## 9. Environment operations (~15 call sites)

**Privatize**: `Rf_findVar`, `Rf_findVarInFrame`, `Rf_defineVar`, `Rf_setVar`, `Rf_eval`, `R_tryEval`, `R_tryEvalSilent`, `R_GlobalEnv`, `R_BaseEnv`, `R_EmptyEnv`

`REnv` already wraps the globals. Extend with:
- `REnv::find_var(sym) -> Option<SEXP>`
- `REnv::define_var(sym, val)`
- `REnv::eval(expr) -> Result<SEXP, String>`

**Key files**: `expression.rs`, `thread.rs`, `worker.rs`

## 10. External pointer internals (~15 call sites)

**Privatize**: `R_MakeExternalPtr`, `R_ExternalPtrAddr`, `R_ClearExternalPtr`, `R_RegisterCFinalizer`, `R_RegisterCFinalizerEx`

`ExternalPtr<T>` already wraps everything. Only `externalptr.rs` itself calls these directly.

**Key files**: `externalptr.rs`

## `_unchecked` variants

Each `#[r_ffi_checked]` function generates a `*_unchecked` variant. These bypass thread-checking for perf-critical paths (ALTREP callbacks, `with_r_thread` closures). They should remain `pub` — they serve a real need. When we add safe methods, the unchecked variants become the escape hatch for hot paths.

Current unchecked usage outside ffi.rs: `into_r/collections.rs` (`Rf_setAttrib_unchecked`), `rayon_bridge.rs` (`Rf_setAttrib_unchecked`).

## Verification (same for each item)

1. `cargo check` — workspace
2. `cd rpkg/src/rust && MINIEXTENDR_LINT=0 cargo check` — rpkg
3. `cargo check --manifest-path=miniextendr-bench/Cargo.toml` — bench
4. `cargo check --manifest-path=tests/cross-package/consumer.pkg/src/rust/Cargo.toml` — cross-package
5. `cargo clippy` — no warnings
6. `cargo test` — Rust unit tests pass
