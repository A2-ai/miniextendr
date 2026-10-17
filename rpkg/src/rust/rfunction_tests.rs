//! Built-in borrowed callbacks and validators with plain messages (#1947).

use miniextendr_api::prelude::*;
use miniextendr_api::{RFunction, SEXP};
use std::sync::atomic::{AtomicI32, Ordering};

static DROPS: AtomicI32 = AtomicI32::new(0);
struct Dropped;
impl Drop for Dropped {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

/// Call an R closure, builtin or special, retaining the caller's handlers.
/// @noRd
#[miniextendr(noexport)]
pub fn rfunction_apply(f: RFunction<'_>, x: SEXP) -> SEXP {
    let _dropped = Dropped;
    unsafe {
        f.call()
            .arg(x)
            .eval_with_handlers(miniextendr_api::sys::R_GlobalEnv)
    }
}

/// Exercise the Rust refusal without an R precondition.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn rfunction_apply_unchecked(f: RFunction<'_>, x: SEXP) -> SEXP {
    rfunction_apply(f, x)
}

/// NULL skips the callback and returns the input.
/// @noRd
#[miniextendr(noexport)]
pub fn rfunction_optional(f: Option<RFunction<'_>>, x: SEXP) -> SEXP {
    f.map_or(x, |f| rfunction_apply(f, x))
}

/// Exercise the optional Rust refusal without an R precondition.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn rfunction_optional_unchecked(f: Option<RFunction<'_>>, x: SEXP) -> SEXP {
    rfunction_optional(f, x)
}

/// Observe cleanup across R exits.
/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn rfunction_drops() -> i32 {
    DROPS.load(Ordering::Relaxed)
}

/// A mutable receiver remains usable after its callback exits through R.
#[derive(ExternalPtr)]
pub struct CallbackHost;

/// @noRd
#[miniextendr(env, noexport)]
impl CallbackHost {
    pub fn new() -> Self {
        Self
    }
    pub fn apply(&mut self, f: RFunction<'_>, x: SEXP) -> SEXP {
        rfunction_apply(f, x)
    }
}

/// Callbacks cross the trait ABI through the same rooted conversion.
#[miniextendr]
pub trait CallbackTrait {
    fn invoke(&self, f: RFunction<'_>, x: SEXP) -> SEXP;
}

#[miniextendr(env, noexport)]
impl CallbackTrait for CallbackHost {
    fn invoke(&self, f: RFunction<'_>, x: SEXP) -> SEXP {
        rfunction_apply(f, x)
    }
}

/// Call the declaration-generated vtable shim.
/// @noRd
#[miniextendr(noexport)]
pub fn rfunction_through_view(f: RFunction<'_>, x: SEXP) -> SEXP {
    unsafe {
        let scope = miniextendr_api::ProtectScope::new();
        let handle = scope.protect(miniextendr_api::mx_abi::mx_wrap(__mx_wrap_CallbackHost(
            CallbackHost,
        )));
        CallbackTraitView::from_sexp(handle.get()).invoke(f, x)
    }
}

fn positive_str(x: SEXP) -> Result<(), &'static str> {
    if i32::try_from_sexp(x).is_ok_and(|n| n > 0) {
        Ok(())
    } else {
        Err("must be positive")
    }
}
fn positive_string(x: SEXP) -> Result<(), String> {
    positive_str(x).map_err(str::to_owned)
}

#[derive(TryFromSexp)]
#[try_from_sexp(validate = positive_str)]
pub struct PositiveStr(i32);

#[derive(TryFromSexp)]
#[try_from_sexp(validate = positive_string)]
pub struct PositiveString(i32);

/// A validator returning a borrowed message.
/// @noRd
#[miniextendr(noexport)]
pub fn validator_str(x: PositiveStr) -> i32 {
    x.0
}

/// A validator returning an owned message.
/// @noRd
#[miniextendr(noexport)]
pub fn validator_string(x: PositiveString) -> i32 {
    x.0
}

/// A callback parameter in the generic trait's concrete shim.
#[miniextendr]
pub trait GenericCallback<T> {
    fn invoke_generic(&self, f: RFunction<'_>, x: T) -> SEXP;
}

#[miniextendr(env, noexport)]
impl GenericCallback<i32> for CallbackHost {
    fn invoke_generic(&self, f: RFunction<'_>, x: i32) -> SEXP {
        rfunction_apply(f, x.into_sexp())
    }
}

/// Call the impl-generated generic vtable shim.
/// @noRd
#[miniextendr(noexport)]
pub fn rfunction_through_generic_shim(f: RFunction<'_>, x: i32) -> SEXP {
    unsafe {
        let scope = miniextendr_api::ProtectScope::new();
        let handle = scope.protect(miniextendr_api::mx_abi::mx_wrap(__mx_wrap_CallbackHost(
            CallbackHost,
        )));
        // Generic traits expose the callback layout, without a generated View.
        let table = miniextendr_api::mx_abi::mx_query(handle.get(), TAG_GenericCallback);
        assert!(!table.is_null());
        let table = &*table.cast::<__VTABLE_miniextendr_GenericCallback_FOR_CallbackHost_Layout>();
        let raw = miniextendr_api::mx_abi::mx_get(handle.get());
        let data = raw.cast::<u8>().add((*(*raw).base).data_offset).cast();
        let x = scope.protect(x.into_sexp());
        let args = [f.as_sexp(), x.get()];
        (table.invoke_generic)(data, 2, args.as_ptr())
    }
}
