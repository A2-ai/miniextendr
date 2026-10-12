//! Inherent static methods honor worker selection without no_main_thread (#2031).
#![cfg(feature = "worker-thread")]

mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::worker::is_r_main_thread;
use miniextendr_api::{ExternalPtr, SEXP, TryFromSexp, miniextendr};

#[derive(ExternalPtr)]
pub struct StaticDispatch {
    constructed_on_main: bool,
}

#[miniextendr(env, noexport)]
impl StaticDispatch {
    pub fn new() -> Self {
        Self {
            constructed_on_main: is_r_main_thread(),
        }
    }

    pub fn default_static() -> bool {
        is_r_main_thread()
    }

    #[miniextendr(env(worker))]
    pub fn explicit_worker() -> bool {
        is_r_main_thread()
    }

    #[miniextendr(env(no_worker))]
    pub fn local_static() -> bool {
        is_r_main_thread()
    }

    #[miniextendr(env(worker, main_thread))]
    pub fn forced_main() -> bool {
        is_r_main_thread()
    }

    #[miniextendr(env(worker), unsafe(main_thread))]
    pub fn unsafe_main() -> bool {
        is_r_main_thread()
    }

    #[miniextendr(env(worker))]
    pub fn instance(&self) -> bool {
        assert!(is_r_main_thread());
        self.constructed_on_main
    }
}

// Opting out also keeps a non-Send constructor result on R's thread.
#[derive(ExternalPtr)]
pub struct MainOnly(std::rc::Rc<i32>);

#[miniextendr(env, noexport)]
impl MainOnly {
    #[miniextendr(env(no_worker))]
    pub fn new() -> Self {
        assert!(is_r_main_thread());
        Self(std::rc::Rc::new(23))
    }

    pub fn value(&self) -> i32 {
        assert!(is_r_main_thread());
        *self.0
    }
}

type StaticWrapper = unsafe extern "C-unwind" fn(SEXP) -> SEXP;
type InstanceWrapper = unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP;
type ErasedWrapper = unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void;

fn registered(name: &str, arity: i32) -> ErasedWrapper {
    let suffix = format!("_{name}");
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .expect("method wrapper must be registered");
    assert_eq!(entry.numArgs, arity);
    entry.fun.unwrap()
}

unsafe fn call_static(name: &str) -> SEXP {
    let call = unsafe { std::mem::transmute::<ErasedWrapper, StaticWrapper>(registered(name, 1)) };
    unsafe { call(SEXP::nil()) }
}

unsafe fn call_instance(name: &str, receiver: SEXP) -> SEXP {
    let call =
        unsafe { std::mem::transmute::<ErasedWrapper, InstanceWrapper>(registered(name, 2)) };
    unsafe { call(SEXP::nil(), receiver) }
}

#[test]
fn registered_methods_run_on_the_selected_thread() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let inline_worker = cfg!(target_family = "wasm");
        let default_main = !cfg!(feature = "worker-default") || inline_worker;
        for (name, expected) in [
            ("StaticDispatch__default_static", default_main),
            ("StaticDispatch__explicit_worker", inline_worker),
            ("StaticDispatch__local_static", true),
            ("StaticDispatch__forced_main", true),
            ("StaticDispatch__unsafe_main", true),
        ] {
            let result = scope.protect(call_static(name));
            assert_eq!(
                bool::try_from_sexp(result.get()).unwrap(),
                expected,
                "{name}"
            );
        }

        let receiver = scope.protect(call_static("StaticDispatch__new"));
        let result = scope.protect(call_instance("StaticDispatch__instance", receiver.get()));
        assert_eq!(bool::try_from_sexp(result.get()).unwrap(), default_main);

        let receiver = scope.protect(call_static("MainOnly__new"));
        let result = scope.protect(call_instance("MainOnly__value", receiver.get()));
        assert_eq!(i32::try_from_sexp(result.get()).unwrap(), 23);
    });
}
