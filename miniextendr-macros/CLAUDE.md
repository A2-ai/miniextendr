# miniextendr-macros

Proc-macro crate — `#[miniextendr]`, `#[miniextendr_init]`, derives (`ExternalPtr`, `Altrep*`, `DataFrameRow`, `MatchArg*`, `Factor`, `Vctrs`, `List`, `TypedExternal`). See root `CLAUDE.md` for project rules.

## Scope
- Compile-time item codegen lives here. Registration aggregation and the
  host-side wrapper/wasm file writers live in `miniextendr-api::registry`;
  `miniextendr-engine` is only for standalone R embedding.
- Shared parser layer (formerly `miniextendr-macros-core`) is now in-crate and re-consumed by `miniextendr-lint` directly.

## Layout
- `lib.rs` — proc-macro entrypoints + the `MiniextendrFnAttrs` / `ImplAttrs` / `ParsedImpl` destructuring. **Add a field here → update all 6 class generators.**
- `naming.rs` — canonical name derivations (R wrapper names, `__miniextendr_*` mangling, class symbols). Single source of truth. All macro-emitted `#[no_mangle]` C symbols are crate-prefixed here (`C_<crate>_<fn>`, `C_<crate>_<Type>__<method>`, `__VTABLE_<CRATE>_…`, `__vtshim_<crate>_…`, `__mx_altrep_reg_<crate>_…`) for webR cross-package uniqueness (#1273) — never reconstruct these shapes inline; call the helpers.
- `type_inspect.rs` — shared type helpers incl. the visibility-marker peel (`peel_visibility_marker`, `peel_return_visibility`, `resolve_visibility` = marker > `invisible`/`visible` attr > shape default, `visibility_marker_error` for nested / argument-position markers, #1213). Bare fns peel in `lib.rs`; `ParsedMethod` / `TraitMethod` peel at parse time (`visibility_marker` / `invisible` fields) so every return-shape predicate sees the inner type; `MethodReturnBuilder::with_invisible` and `r_wrappers::finish_instance_body` apply the decision.
- `crate_config.rs` — the only manifest read at expansion time: `[package.metadata.miniextendr]` in the user crate's `Cargo.toml` (via `CARGO_MANIFEST_DIR`, cached per manifest dir). Today one key, `noexport_postfix`, the crate-wide default suffix for `noexport` / `internal` free functions (#1454). Line-based scanner on purpose (no TOML dependency in a proc-macro crate); add keys there, not ad hoc.
- `miniextendr_fn.rs` — `#[miniextendr]` on bare fns. Tagged-condition transport is the only mode; `unwrap_in_r` is orthogonal.
- `miniextendr_impl.rs` (+ `miniextendr_impl/`) — inherent impl methods. Same transport. Class-system dispatch (R6/S3/S4/S7/Env/Vctrs).
- `miniextendr_impl_trait.rs` (+ `miniextendr_impl_trait/`) — trait-impl method codegen, ABI-compatible.
- `miniextendr_trait.rs` — `#[miniextendr_trait]` declaration codegen. Trait-ABI vtable shims wrap in `with_r_unwind_protect_shim` (returns a tagged error SEXP that the View method re-panics into the consumer's outer `with_r_unwind_protect` guard) — see `miniextendr_trait.rs:808`.
- `c_wrapper_builder.rs` (+ dir) — `CWrapperContext` for impl method C wrappers. **Prepends `__miniextendr_call: SEXP` as first param.**
- `externalptr_derive.rs` — hand-rolls C wrappers for sidecar `*_get_field` / `*_set_field` accessors: `(x: SEXP)` / `(x, value: SEXP)`, `numArgs = 1/2`, **no call slot**. Adding `.call = match.call()` to the R side breaks at runtime (PR #344 reverted this).
- `r_wrapper_builder.rs` (+ dir) — R-side `.Call(C_…, .call = match.call(), …)` emission. `DotCallBuilder` at ~L390 is the canonical site; use `.null_call_attribution()` for lambda contexts (R6 finalizer/deep_clone, S7 getter/setter/validator).
- `r_class_formatter.rs` — shared `MethodContext` for all 6 class generators.
- `return_type_analysis.rs` — return-type → codegen for standalone fns (strict-aware).
- `method_return_builder.rs` (+ dir) — same for impl methods.
- `rust_conversion_builder.rs` (+ dir) — TryFromSexp glue for argument conversion.
- `dataframe_derive.rs` (+ dir) — `#[derive(DataFrameRow)]`; supports nested enums/structs, HashMap/BTreeMap (parallel `_keys`/`_values` via `unzip`), `as_factor`/`as_list` attrs.
- `condition_derive.rs` — `#[derive(RConditionError)]`: class vector (`#[condition(class)]` on type/variant, snake_case defaults, variant = `<family>_<variant>`), message (`#[condition(message = "fmt")]` else `Display`), `data` from fields (`rename`/`skip`/`debug`; tuple fields need `rename`/`skip`; `message`/`call`/`kind` rejected). Generated code drives `defer_*` and `Result` Err arms (#1448).
- `match_arg_derive.rs` + `match_arg_keys.rs` — match.arg codegen; placeholder→choices written at link time via `MX_MATCH_ARG_CHOICES`.
- `r_preconditions.rs` — R-side preconditions (`match.arg`, `stopifnot`, …) emitted into wrapper bodies.
- `roxygen.rs` (+ dir) — doc-comment forwarding; class-system doc builders consume this.
- `typed_external_macro.rs` — `TYPE_NAME_CSTR` / `TYPE_ID_CSTR` for R-visible type tags (display + errors).
- `tests.rs` — UI tests in `tests/ui/*.stderr` snapshots; update when error messages change.

## Gotchas specific to this crate
- **Two C-wrapper codegen paths with different signatures.** `c_wrapper_builder.rs` prepends `__miniextendr_call: SEXP` for all `#[miniextendr]` fns/methods; `externalptr_derive.rs` does NOT for sidecar accessors. Don't unify R-side emission without unifying C-side first (#348).
- **`#[miniextendr]` on 1-field structs is removed.** Use ALTREP derives instead.
- **Lifetime params ARE allowed on `#[miniextendr]`.** Lifetimes are erased at codegen — `#[no_mangle] extern "C-unwind" fn f<'a>(...)` produces a single monomorphic symbol and is FFI-safe. Only type/const generic params are rejected (they require monomorphization → multiple symbols → incompatible with `#[no_mangle]`). Borrowed fields on `#[derive(DataFrameRow)]` structs also work (`Vec<Option<&str>>` / `Vec<Option<&[T]>>` companion columns since PR #465).
- **`impl Trait` argument position fails** (E0283 — `TryFromSexp + Trait` across `let` bindings). Return position works.
- **When changing helpers from `TokenStream` → `Result<TokenStream>`, update every caller with `?`.** Otherwise you'll see confusing `ToTokens` bound errors on `Result`.
- **UI test `.stderr` files** must be regenerated when error wording changes (`TRYBUILD=overwrite cargo test -p miniextendr-macros`). **Never overwrite under a `rust-src`-equipped toolchain** — it bakes stdlib spans into the snapshot that CI's minimal stable never renders (#1239). Run the suite via `just test-ui` (auto-isolates under a minimal-profile toolchain) and see root CLAUDE.md "UI test snapshots".
- **MXL111** — `s4_*` method name on `#[miniextendr(s4)]` impl gets `s4_s4_*`. Drop the prefix.
- **S7 fast-path shortcuts** — every non-fallback S7 instance method (inherent *and* trait impl) emits a `<ClassName>_<method>(self, ...)` function that bypasses `S7::S7_dispatch()`. Opt out per-method with `#[miniextendr(s7(no_shortcut))]`. Shortcut names share a namespace with static-method functions; same-impl-block collisions are a `compile_error!` (`check_s7_shortcut_collisions` in `miniextendr_impl.rs`). Sidecar-accessor collisions (`<ClassName>_get_<field>`) are NOT detectable at macro time — see #991. The advisory roxygen prose is shared via `s7_class::shortcut_advisory_lines`.

## When changing codegen
- Touched proc-macro output? Run `just configure && just rcmdinstall && just force-document`. Commit regenerated `NAMESPACE` + `man/*.Rd` in the same PR. `rpkg/R/miniextendr-wrappers.R` and `rpkg/src/rust/wasm_registry.rs` are gitignored, regenerated during installs, and shipped from disk in the tarball; CI's `wrappers-sync-check` detects tracked doc drift.
- Added a class-system constructor path? Make sure error-check pattern `(.val <- .Call(...); condition_check_lines())` is wired through — silent object corruption otherwise.
- Added a S3 `@export`? Use `#' @export generic_name` (explicit target) on `if (!exists(...)) generic <- ...` — roxygen2 can't introspect conditional declarations and drifts the export onto the next function.
