# issue-243: Retarget standalone-fn codegen at CWrapperContext

Issue: #243 (deferred #19 from the macros-refactor sweep)

## Background

`lib.rs::build_fn_c_wrapper` hand-rolls three variants of the `extern "C-unwind"` wrapper
(main-thread × `{rng, no-rng}` × error_in_r collapsed into a single-branch shape), mirroring
what `c_wrapper_builder::CWrapperContext` already does for impl methods.

## Blockers (from macros-refactor.md deferred note)

1. **Param name mangling**: `CWrapperContext::build_c_params` uses `arg_0`, `arg_1`, ...
   The fn path preserves user's original identifiers (visible in rustdoc). Switching would
   be a user-observable rename.
2. **Visibility**: `CWrapperContext` always emits wrappers with default visibility; the fn
   path honors `#vis` from the user's source.
3. **Generics**: fn path emits `fn #c_ident #generics(...)` for generic functions; builder
   does not support generics.
4. **abi early return**: When the fn is `extern "C-unwind"` with `#[no_mangle]`, no wrapper
   is needed but the `call_method_def` must still be emitted.
5. **Return handling bridge**: fn path uses `analyze_return_type` which produces
   `(return_expression, post_call_statements)` directly; builder uses `detect_return_handling`
   which maps to `ReturnHandling` enum. These cover the same cases with two exceptions:
   - `Result<T, ()>` (NullOnErr special case) — no rpkg test coverage; note in PR body
   - `unwrap_in_r` — map to `ReturnHandling::IntoR` at routing site

## Work items (flat priority)

1. Add `preserve_param_names: bool` to `CWrapperContext` + `CWrapperContextBuilder`.
   When true, `build_c_params` uses original param names from `inputs` instead of `arg_N`.

2. Add `vis: syn::Visibility` to `CWrapperContext` + builder.
   Emit `#vis extern "C-unwind" fn` in wrappers. Default: `syn::Visibility::Inherited`.

3. Add `generics: syn::Generics` to `CWrapperContext` + builder.
   Emit `fn #c_ident #generics(...)` in wrappers. Default: empty generics.

4. Handle abi-is-some case in builder `generate()`: when a new `skip_wrapper: bool` field
   is set, skip the `generate_main_thread_wrapper`/`generate_worker_thread_wrapper` call
   but still emit `generate_call_method_def`.

5. Route the fn call site in `lib.rs` through `CWrapperContext::builder()`:
   - `thread_strategy`: from `use_main_thread` (MainThread/WorkerThread)
   - `return_handling`: from `detect_return_handling(output)`, with `unwrap_in_r=true`
     overriding to `ReturnHandling::IntoR`
   - `call_expr`: `rust_ident(rust_arg1, rust_arg2, ...)`
   - `pre_call`: `pre_call_statements` (interrupt check is already handled by `check_interrupt`)
   - All other fields mapped from existing fn-path locals

6. Remove the inline `call_method_def` emission from the fn path `quote!` block. The builder's
   `generate()` already emits it.

7. Delete `FnCWrapperInputs` struct and `build_fn_c_wrapper` fn from `lib.rs`.

8. Run tests + verify `rpkg/R/miniextendr-wrappers.R` diff is empty after
   `just rcmdinstall && just devtools-document`.

9. Update `plans/macros-refactor.md`: move [#19] from Deferred into Done.

## Verification gate

- `just check` passes (all crates)
- `just test` passes (all crates; snapshot `.new` files reviewed individually)
- `rpkg/R/miniextendr-wrappers.R` diff is empty after `just rcmdinstall && just devtools-document`
- `just clippy` passes with `-D warnings`
- UI trybuild tests pass unchanged

## Deferred / known gaps

- `Result<T, ()>` (NullOnErr) path: `detect_return_handling` returns `ResultIntoR` for
  `Result<T, ()>` where T is not unit, while the old path converted `Err(())` to `NullOnErr`
  and returned NULL. No rpkg tests cover this case. → **#359**
- `ReturnPref::List/ExternalPtr/Native`: wraps return in `AsList`/`AsExternalPtr`/`AsRNative`
  before `into_sexp`. Builder's `ReturnHandling::IntoR` calls `into_sexp` directly. Difference
  only matters if the type doesn't implement `IntoR` directly. No rpkg usage. → **#358**
