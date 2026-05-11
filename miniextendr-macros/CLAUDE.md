# miniextendr-macros

Proc-macro crate — `#[miniextendr]`, `#[miniextendr_init]`, derives (`ExternalPtr`, `Altrep*`, `DataFrameRow`, `MatchArg*`, `Factor`, `Vctrs`, `List`, `TypedExternal`). See root `CLAUDE.md` for project rules.

## Scope
- All compile-time codegen except whole-crate emission (which lives in `miniextendr-engine`).
- Shared parser layer (formerly `miniextendr-macros-core`) is now in-crate and re-consumed by `miniextendr-lint` directly.

## Layout
- `lib.rs` — proc-macro entrypoints + the `MiniextendrFnAttrs` / `ImplAttrs` / `ParsedImpl` destructuring. **Add a field here → update all 6 class generators.**
- `naming.rs` — canonical name derivations (R wrapper names, `__miniextendr_*` mangling, class symbols). Single source of truth.
- `miniextendr_fn.rs` — `#[miniextendr]` on bare fns. `error_in_r` defaults to true (`unwrap_or(true)` ~L1611).
- `miniextendr_impl.rs` (+ `miniextendr_impl/`) — inherent impl methods. `error_in_r` default at ~L1677. Class-system dispatch (R6/S3/S4/S7/Env/Vctrs).
- `miniextendr_impl_trait.rs` (+ `miniextendr_impl_trait/`) — trait-impl method codegen, ABI-compatible.
- `miniextendr_trait.rs` — `#[miniextendr_trait]` declaration codegen. Trait-ABI vtable shims use `Rf_error` (not `error_in_r`) — see `miniextendr_trait.rs:808`.
- `c_wrapper_builder.rs` (+ dir) — `CWrapperContext` for impl method C wrappers. **Prepends `__miniextendr_call: SEXP` as first param.**
- `externalptr_derive.rs` — hand-rolls C wrappers for sidecar `*_get_field` / `*_set_field` accessors: `(x: SEXP)` / `(x, value: SEXP)`, `numArgs = 1/2`, **no call slot**. Adding `.call = match.call()` to the R side breaks at runtime (PR #344 reverted this).
- `r_wrapper_builder.rs` (+ dir) — R-side `.Call(C_…, .call = match.call(), …)` emission. `DotCallBuilder` at ~L390 is the canonical site; use `.null_call_attribution()` for lambda contexts (R6 finalizer/deep_clone, S7 getter/setter/validator).
- `r_class_formatter.rs` — shared `MethodContext` for all 6 class generators.
- `return_type_analysis.rs` — return-type → codegen for standalone fns (strict-aware).
- `method_return_builder.rs` (+ dir) — same for impl methods.
- `rust_conversion_builder.rs` (+ dir) — TryFromSexp glue for argument conversion.
- `dataframe_derive.rs` (+ dir) — `#[derive(DataFrameRow)]`; supports nested enums/structs, HashMap/BTreeMap (parallel `_keys`/`_values` via `unzip`), `as_factor`/`as_list` attrs.
- `match_arg_derive.rs` + `match_arg_keys.rs` — match.arg codegen; placeholder→choices written at link time via `MX_MATCH_ARG_CHOICES`.
- `r_preconditions.rs` — R-side preconditions (`match.arg`, `stopifnot`, …) emitted into wrapper bodies.
- `roxygen.rs` (+ dir) — doc-comment forwarding; class-system doc builders consume this.
- `typed_external_macro.rs` — `TYPE_NAME_CSTR` / `TYPE_ID_CSTR` for R-visible type tags (display + errors).
- `tests.rs` — UI tests in `tests/ui/*.stderr` snapshots; update when error messages change.

## Gotchas specific to this crate
- **Two C-wrapper codegen paths with different signatures.** `c_wrapper_builder.rs` prepends `__miniextendr_call: SEXP` for all `#[miniextendr]` fns/methods; `externalptr_derive.rs` does NOT for sidecar accessors. Don't unify R-side emission without unifying C-side first (#348).
- **`#[miniextendr]` on 1-field structs is removed.** Use ALTREP derives instead.
- **Lifetime params rejected on `#[miniextendr]`** (MXL112). `extern "C-unwind" #[no_mangle]` can't carry generic params. `Vec<T>` / `String` only; or `'static` literals inside the body. Borrowed fields on `#[derive(DataFrameRow)]` structs DO work (`Vec<Option<&str>>` / `Vec<Option<&[T]>>` companion columns since PR #465).
- **`impl Trait` argument position fails** (E0283 — `TryFromSexp + Trait` across `let` bindings). Return position works.
- **When changing helpers from `TokenStream` → `Result<TokenStream>`, update every caller with `?`.** Otherwise you'll see confusing `ToTokens` bound errors on `Result`.
- **UI test `.stderr` files** must be regenerated when error wording changes (`TRYBUILD=overwrite cargo test -p miniextendr-macros`).
- **MXL111** — `s4_*` method name on `#[miniextendr(s4)]` impl gets `s4_s4_*`. Drop the prefix.

## When changing codegen
- Touched proc-macro output? Run `just configure && just rcmdinstall && just devtools-document` and commit regenerated `rpkg/R/miniextendr-wrappers.R` + `NAMESPACE` + `man/*.Rd` in the same PR. Pre-commit hook (`.githooks/pre-commit`) enforces.
- Added a class-system constructor path? Make sure error-check pattern `(.val <- .Call(...); error_in_r_check_lines())` is wired through — silent object corruption otherwise.
- Added a S3 `@export`? Use `#' @export generic_name` (explicit target) on `if (!exists(...)) generic <- ...` — roxygen2 can't introspect conditional declarations and drifts the export onto the next function.
