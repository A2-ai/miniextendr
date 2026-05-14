# miniextendr-api

Runtime crate — FFI, ExternalPtr, ALTREP, worker thread, error/condition transport. See root `CLAUDE.md` for project-wide rules.

## Scope
- All R API contact lives here. Macros generate code that calls into this crate.
- No proc-macros — those are in `miniextendr-macros`. No codegen logic — that's in `miniextendr-engine`.

## Architecture pointers
- `registry.rs` — linkme `#[distributed_slice]` declarations (`MX_CALL_DEFS`, `MX_MATCH_ARG_CHOICES`, `MX_R_WRAPPERS`, `MX_CLASS_NAMES`, …) + cdylib entry.
- `init.rs::package_init()` — consolidates `R_init_<pkg>` steps; `miniextendr_init!` proc-macro thin-wraps it.
- `mx_abi.rs` — Rust reimpl of `mx_wrap`/`mx_get`/`mx_query`/`mx_abi_register` (replaced the old `mx_abi.c` / `entrypoint.c`).
- `worker.rs` — worker thread + `Sendable<T>`. Without `worker-thread` feature, `run_on_worker(f) → Ok(f())` inline.
- `unwind_protect.rs` — `R_UnwindProtect` wrapper; `with_r_unwind_protect_error_in_r` is the user-facing path.
- `error_value.rs` — tagged-SEXP transport. `make_rust_error_value` (3-elem) + `make_rust_condition_value` (4-elem, custom-class slot). PROTECT discipline matters here (see Gotchas).
- `condition.rs` — `RCondition` enum + `error!`/`warning!`/`message!`/`condition!` macros + `AsRError<E: Error>`.
- `from_r.rs` — `TryFromSexp` + `r_slice` / `r_slice_mut` (handle R's 0x1 empty-vector data pointer).
- `into_r.rs` — `IntoR` impls; `Box<[T]>` blanket + `bool`/`String` overrides.
- `coerce.rs` / `as_coerce.rs` / `strict.rs` — conversion paths; strict-mode checked variants.
- `externalptr.rs` — `Box<Box<dyn Any>>` storage, `Any::downcast` for safety, non-generic `release_any` finalizer.
- `altrep.rs` / `altrep_impl.rs` / `altrep_bridge.rs` / `altrep_traits.rs` — ALTREP guard modes (`unsafe`/`rust_unwind`/`r_unwind`), trampolines stay on main thread.
- `panic_telemetry.rs` — RwLock-based panic hook.

## Gotchas specific to this crate
- **`error_in_r` is the DEFAULT** for `#[miniextendr]` fns/methods. `Rf_error` only fires from trait-ABI vtable shims, ALTREP `RUnwind` guards, and explicit opt-out (`no_error_in_r`/`unwrap_in_r`). Older comments suggesting otherwise are stale.
- **`with_r_unwind_protect_error_in_r` leaks ~8 bytes on R longjmp** (RErrorMarker + Box header). Regular panics don't leak. MXL300 warns about direct `Rf_error()` for this reason.
- **PROTECT discipline against R-devel GC** — `SEXP::scalar_string` and `scalar_logical(true)` allocate fresh STRSXP/LGLSXP; protect across `SET_VECTOR_ELT`/`SETATTRIB`. R-devel's GC is more aggressive — `make_rust_condition_value` crossed the threshold in PR #344 commit `af6b4875` (recursive gc + segfault at small offsets). R-release passing ≠ safe.
- **`#[macro_export]` collides with same-named modules at crate root** — `pub mod error`/`pub mod condition` shadow `error!`/`condition!` macros under `use miniextendr_api::{error, condition}`. Invoke via fully-qualified path; `message!` / `warning!` have no module conflict.
- **`R_GetCCallable` throws on miss** — does NOT return NULL. Force DLL load via `NAMESPACE importFrom(...)`.
- **`R_new_custom_connection` creates connections CLOSED** (`isopen = FALSE`); explicitly call the `open` callback after building.
- **Pointer provenance for ExternalPtr** — cache `*mut T` via `&mut T` / `downcast_mut` / `Box::into_raw`. Never write through a `*mut` derived from `&T` / `downcast_ref` — UB under Stacked Borrows.

## Features
- `worker-thread`, `nonapi`, `strict`/`coerce`, `r6`/`s7` (mutually exclusive defaults), plus optional integrations (`rayon`, `serde`, `ndarray`, …). Full list in `Cargo.toml`. CI's `clippy_all` enables a curated subset; `--all-features` fails (default-r6 vs default-s7).

## When changing this crate
- Add a `#[derive(…)]` or conversion impl? Update in order: `from_r.rs` → `into_r.rs` → `coerce.rs` → serde docs → rpkg fixture + testthat.
- Add a path that stores SEXPs across allocations? Add a no-arg `gc_stress_*()` fixture in `rpkg/src/rust/gc_stress_fixtures.rs` (see #430).
- `_unchecked` FFI variants (`#[r_ffi_checked]`) are valid inside ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` — MXL301 enforces.
