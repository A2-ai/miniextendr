# miniextendr-api

Runtime crate — FFI, ExternalPtr, ALTREP, worker thread, error/condition transport. See root `CLAUDE.md` for project-wide rules.

## Scope
- All R API contact lives here. Macros generate code that calls into this crate.
- No proc-macros — those are in `miniextendr-macros`. No codegen logic — that's in `miniextendr-engine`.

## Architecture pointers
- `registry.rs` — linkme `#[distributed_slice]` declarations (`MX_CALL_DEFS`, `MX_MATCH_ARG_CHOICES`, `MX_R_WRAPPERS`, `MX_CLASS_NAMES`, …) + cdylib entry.
- `init.rs::package_init()` — consolidates `R_init_<pkg>` steps; `miniextendr_init!` proc-macro thin-wraps it.
- `mx_abi.rs` — Rust reimpl of `mx_wrap`/`mx_get`/`mx_query`/`mx_abi_register` (replaced the old `mx_abi.c` / `entrypoint.c`).
- `worker.rs` — worker thread + `Sendable<T>`. Without `worker-thread` feature, `run_on_worker(f) → Ok(f())` inline. On wasm the feature stays *enabled* but every spawn path is gated `not(target_family = "wasm")` so it also runs inline (see wasm gotcha below).
- `unwind_protect.rs` — `R_UnwindProtect` wrapper; `with_r_unwind_protect` is the user-facing path (returns a tagged-condition SEXP; the R wrapper raises). `with_r_unwind_protect_or_raise` is the legacy panics-as-R-error variant kept for test/bench use.
- `error_value.rs` — tagged-SEXP transport. `make_rust_condition_value_with_data` (5-element list: message/kind/class/call/data) is the only producer; `make_rust_condition_value` is the no-data thin wrapper used by all proc-macro codegen. PROTECT discipline matters here (see Gotchas).
- `condition.rs` — `RCondition` enum + `error!`/`warning!`/`message!`/`condition!` macros (optional `class =` and structured `data =` payloads — Send-safe `ConditionDataValue`, materialised on the main thread) + `AsRError<E: Error>`.
- `from_r.rs` — `TryFromSexp` + `r_slice` / `r_slice_mut` (handle R's 0x1 empty-vector data pointer).
- `into_r.rs` — `IntoR` impls; `Box<[T]>` blanket + `bool`/`String` overrides.
- `coerce.rs` / `r_coerce.rs` / `strict.rs` — conversion paths; strict-mode checked variants. `r_coerce.rs` holds the `RCoerce*` S3-coercion trait family (`#[miniextendr(as = "…")]`).
- `externalptr.rs` — `Box<Box<dyn Any>>` storage, `Any::downcast` for safety, non-generic `release_any` finalizer.
- `altrep.rs` / `altrep_impl.rs` / `altrep_bridge.rs` / `altrep_traits.rs` — ALTREP guard modes (`unsafe`/`rust_unwind`/`r_unwind`), trampolines stay on main thread.
- `panic_telemetry.rs` — RwLock-based panic hook.

## Gotchas specific to this crate
- **Tagged-condition transport is the only path** for `#[miniextendr]` fns/methods. All Rust-origin failures (panics, `Result::Err`, `Option::None`) and user conditions (`error!`/`warning!`/`message!`/`condition!`) flow through `make_rust_condition_value` → R wrapper → `stop(structure(..., class = c("rust_*", ...)))`. `Rf_error` only fires from trait-ABI vtable shims and ALTREP `RUnwind` guards. `unwrap_in_r` is semantically distinct (Result-as-value vs Result-as-error-boundary) and orthogonal to transport.
- **R-longjmp leaks: two profiles.** Plain `with_r_unwind_protect` / `with_r_unwind_protect_or_raise` (body run inline): the leaked payload is an `RErrorMarker` *ZST*, so `Box::new` doesn't heap-allocate → **~0 bytes**. Worker/dispatch path (`run_on_worker` → `dispatch_to_worker`): `R_ContinueUnwind` skips destructors for `worker_rx` / `response_tx` / the parked `Done(Err)` message / a `job_tx` refcount → **fixed-per-unwind, ~hundreds of bytes, linearly bounded, non-compounding** (steady-state ~1.8 KB/unwind RSS; bounded < 8 KB/unwind by `test-worker-longjmp.R`). Regular panics don't leak. MXL300 warns about direct `Rf_error()` for this reason. See `unwind_protect.rs` "Leaks" + `worker.rs::dispatch_to_worker` "Leak note".
- **PROTECT discipline against R-devel GC** — `SEXP::scalar_string` and `scalar_logical(true)` allocate fresh STRSXP/LGLSXP; protect across `SET_VECTOR_ELT`/`SETATTRIB`. R-devel's GC is more aggressive — `make_rust_condition_value` crossed the threshold in PR #344 commit `af6b4875` (recursive gc + segfault at small offsets). R-release passing ≠ safe.
- **`#[macro_export]` collides with same-named modules at crate root** — `pub mod error`/`pub mod condition` shadow `error!`/`condition!` macros under `use miniextendr_api::{error, condition}`. Invoke via fully-qualified path; `message!` / `warning!` have no module conflict.
- **`R_GetCCallable` throws on miss** — does NOT return NULL. Force DLL load via `NAMESPACE importFrom(...)`.
- **`R_new_custom_connection` creates connections CLOSED with `text = TRUE`** — `RCustomConnection::build` papers over both (infers `text` from mode, invokes the `open` trampoline before returning). If you bypass the builder, you must handle both yourself or `writeBin`/`writeLines`/`seek` will fail and binary modes will reject `readBin`/`writeBin`.
- **Pointer provenance for ExternalPtr** — cache `*mut T` via `&mut T` / `downcast_mut` / `Box::into_raw`. Never write through a `*mut` derived from `&T` / `downcast_ref` — UB under Stacked Borrows.
- **Don't drop `worker-thread` on wasm** — it's tempting (R-on-wasm is single-threaded, emscripten has no usable pthreads). But `rpkg/src/rust/wasm_registry.rs` is generated by a *native* build with rpkg's default features (which include `worker-thread`); disabling it on wasm would compile out ~31 worker-gated routines and leave the snapshot with dangling entries. So the feature stays *enabled* on wasm and every spawn path in `worker.rs` is gated `all(feature = "worker-thread", not(target_family = "wasm"))` → runs inline, identical to a non-feature build (#758). This is a knock-on cost of generating the snapshot from a differently-configured build; if the snapshot ever becomes wasm-configured, revisit.

## Features
- `worker-thread`, `nonapi`, `strict`/`coerce`, `r6`/`s7` (mutually exclusive defaults), plus optional integrations (`rayon`, `serde`, `ndarray`, …). Full list in `Cargo.toml`. CI's `clippy_all` enables a curated subset; `--all-features` fails (r6-default vs s7-default).

## When changing this crate
- Add a `#[derive(…)]` or conversion impl? Update in order: `from_r.rs` → `into_r.rs` → `coerce.rs` → serde docs → rpkg fixture + testthat.
- Add a path that stores SEXPs across allocations? Add a no-arg `gc_stress_*()` fixture in `rpkg/src/rust/gc_stress_fixtures.rs` (see #430).
- `_unchecked` FFI variants (`#[r_ffi_checked]`) are valid inside ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` — MXL301 enforces.
