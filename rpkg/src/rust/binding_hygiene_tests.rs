//! Parameters named like the bindings that generated wrappers use internally
//! (#1677), driven by `tests/testthat/test-binding-hygiene.R`.
//!
//! Each case here compiled before the fix and bound the wrong value, so the
//! methods hand back what their parameters received. The names that only
//! failed to compile are covered by the `*_binding_hygiene.rs` UI fixtures in
//! `miniextendr-macros/tests/ui/pass/`.

use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{SEXP, TryFromSexp, miniextendr};

/// Env class whose methods take parameters named like wrapper internals.
#[derive(miniextendr_api::ExternalPtr)]
pub struct HygieneHost {
    value: i32,
}

#[miniextendr(env, noexport)]
impl HygieneHost {
    /// Create a host holding `value`.
    /// @param value The initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The host's value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Return the second argument. `arg_1` is also the name of the C
    /// wrapper's raw slot for `value`.
    /// @param arg_1 Ignored.
    /// @param value Returned unchanged.
    pub fn second(&self, arg_1: SEXP, value: SEXP) -> SEXP {
        let _ = arg_1;
        value
    }

    /// Store an integer and return the receiver. `self_sexp` is also the name
    /// of the C wrapper's receiver slot, which it hands back.
    /// @param self_sexp The integer to store.
    pub fn store(&mut self, self_sexp: SEXP) -> &mut Self {
        self.value = i32::try_from_sexp(self_sexp).expect("an integer");
        self
    }
}

/// Trait whose parameters are named like the vtable shim's and the trait
/// C wrapper's own bindings.
#[miniextendr]
pub trait ShimNames {
    /// The receiver's value followed by each argument, in order.
    fn shim_args(&self, argc: i32, data: i32, argv: i32, self_ref: i32) -> Vec<i32>;

    /// The receiver's value, then `self_ref`'s.
    fn pair(&self, self_ref: &Self) -> Vec<i32>;
}

#[miniextendr(env, noexport)]
impl ShimNames for HygieneHost {
    /// @param argc,data,argv,self_ref Returned in order.
    fn shim_args(&self, argc: i32, data: i32, argv: i32, self_ref: i32) -> Vec<i32> {
        vec![self.value, argc, data, argv, self_ref]
    }

    /// @param self_ref Another host.
    fn pair(&self, self_ref: &Self) -> Vec<i32> {
        vec![self.value, self_ref.value]
    }
}

/// Call `shim_args` through the trait's View, the vtable-shim path another
/// package takes. The shim reads every argument after `argc` by index, so an
/// `argc` parameter that replaced the shim's own count refuses them.
#[miniextendr(noexport)]
pub fn shim_names_through_view() -> Vec<i32> {
    // SAFETY: R's main thread (the wrapper); the wrapped handle is protected for
    // the whole View call.
    unsafe {
        let handle = OwnedProtect::new(miniextendr_api::trait_abi::ccall::mx_wrap(
            __mx_wrap_HygieneHost(HygieneHost { value: 7 }),
        ));
        ShimNamesView::from_sexp(handle.get()).shim_args(1, 10, 100, 1000)
    }
}
