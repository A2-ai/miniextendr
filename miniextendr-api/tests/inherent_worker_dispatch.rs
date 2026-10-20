//! Inherent static methods honor worker selection without no_main_thread (#2031).
#![cfg(feature = "worker-thread")]
// Exercises the deprecated `worker` option (#2117).
#![allow(deprecated)]

mod r_test_utils;

use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::worker::is_r_main_thread;
use miniextendr_api::{ExternalPtr, ExternalPtrRef, SEXP, TryFromSexp, miniextendr};

#[derive(ExternalPtr)]
pub struct StaticDispatch {
    constructed_on_main: bool,
}

#[miniextendr(env, noexport)]
impl StaticDispatch {
    #[allow(clippy::new_without_default)]
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        assert!(is_r_main_thread());
        Self(std::rc::Rc::new(23))
    }

    pub fn value(&self) -> i32 {
        assert!(is_r_main_thread());
        *self.0
    }
}

// A constructor the worker runs builds its `Sidecar` there: the conversion
// and the root go through R's main thread (#2216).
#[derive(ExternalPtr)]
pub struct WorkerSidecar {
    built_on_main: bool,
    #[r_data]
    keys: Sidecar<Vec<i32>>,
}

impl WorkerSidecar {
    fn build() -> Self {
        Self {
            built_on_main: is_r_main_thread(),
            keys: Sidecar::new(vec![2, 3, 5]),
        }
    }
}

#[miniextendr(env, noexport)]
impl WorkerSidecar {
    /// Dispatched as a package's sidecar constructor is: on the worker under
    /// `worker-default`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::build()
    }

    #[miniextendr(env(worker))]
    pub fn on_worker() -> Self {
        Self::build()
    }

    pub fn built_on_main(&self) -> bool {
        self.built_on_main
    }

    pub fn key_sum(self: ExternalPtrRef<'_, Self>) -> i32 {
        Self::keys(&self).iter().sum()
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

#[test]
fn a_worker_constructor_builds_its_sidecars_through_the_main_thread() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let inline_worker = cfg!(target_family = "wasm");
        let default_main = !cfg!(feature = "worker-default") || inline_worker;
        for (name, on_main) in [
            ("WorkerSidecar__new", default_main),
            ("WorkerSidecar__on_worker", inline_worker),
        ] {
            let receiver = scope.protect(call_static(name));
            miniextendr_api::sys::R_gc();
            let built_on_main = scope.protect(call_instance(
                "WorkerSidecar__built_on_main",
                receiver.get(),
            ));
            assert_eq!(
                bool::try_from_sexp(built_on_main.get()).unwrap(),
                on_main,
                "{name}"
            );
            let sum = scope.protect(call_instance("WorkerSidecar__key_sum", receiver.get()));
            assert_eq!(i32::try_from_sexp(sum.get()).unwrap(), 10, "{name}");
        }
    });
}
