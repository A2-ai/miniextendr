//! Re-enter the generated C boundaries while their receiver reference is live.
mod r_test_utils;

use miniextendr_api::externalptr::{ExternalBorrow, Sidecar};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::{ExternalPtr, IntoR, RCondition, SEXP, TryFromSexp, miniextendr};
use std::cell::Cell;

thread_local! {
    static OWNER: Cell<Option<SEXP>> = const { Cell::new(None) };
}

#[derive(ExternalPtr)]
pub struct Reentrant {
    #[r_data]
    pub value: i32,
    #[r_data]
    pub values: Sidecar<Vec<i32>>,
}

#[miniextendr(env, noexport)]
impl Reentrant {
    pub fn read(&self) -> i32 {
        self.value
    }
    pub fn write(&mut self, value: i32) {
        self.value = value;
    }
    pub fn fail(&mut self) {
        panic!("intentional receiver unwind");
    }
    pub fn shared(&self, operation: i32) -> bool {
        let old = self.value;
        let conflict = nested(operation);
        assert_eq!(self.value, old); // the shared reference remains live across re-entry
        conflict
    }
    pub fn exclusive(&mut self, operation: i32) -> bool {
        let conflict = nested(operation);
        self.value += 1; // the exclusive reference remains live across re-entry
        conflict
    }
}

fn host() -> Reentrant {
    Reentrant {
        value: 7,
        values: Sidecar::new(vec![2, 3, 5]),
    }
}

unsafe fn registration(
    suffix: &str,
    arity: i32,
) -> unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .expect("generated wrapper must be registered");
    assert_eq!(entry.numArgs, arity);
    entry.fun.unwrap()
}

unsafe fn call2(suffix: &str, a: SEXP, b: SEXP) -> SEXP {
    let call = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(registration(suffix, 2))
    };
    unsafe { call(a, b) }
}

unsafe fn call3(suffix: &str, a: SEXP, b: SEXP, c: SEXP) -> SEXP {
    let call = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP,
        >(registration(suffix, 3))
    };
    unsafe { call(a, b, c) }
}

fn nested(operation: i32) -> bool {
    let owner = OWNER.with(Cell::get).unwrap();
    unsafe {
        let scope = ProtectScope::new();
        let value = scope.protect(99_i32.into_sexp()).get();
        let flag = scope.protect(false.into_sexp()).get();
        let result = scope
            .protect(match operation {
                0 => call2("Reentrant__read", SEXP::nil(), owner),
                1 => call3("Reentrant__write", SEXP::nil(), owner, value),
                2 => call2("__mx_rdata_get_Reentrant_value", owner, flag),
                3 => call3("__mx_rdata_set_Reentrant_value", owner, value, flag),
                4 => call2("__mx_rdata_get_Reentrant_values", owner, flag),
                5 => call3("__mx_rdata_set_Reentrant_values", owner, value, flag),
                _ => unreachable!(),
            })
            .get();
        match RCondition::from_tagged_sexp(result) {
            Some(condition) => {
                let (_, parts) = condition.into_parts();
                assert!(
                    parts
                        .class
                        .iter()
                        .any(|class| class == "miniextendr_borrow_conflict"),
                    "{:?}",
                    parts.class
                );
                true
            }
            None => false,
        }
    }
}

#[test]
fn generated_methods_and_fields_reject_conflicting_reentry() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        OWNER.with(|slot| slot.set(Some(owner.as_sexp())));
        let scope = ProtectScope::new();
        for operation in 0..6_i32 {
            let op = scope.protect(operation.into_sexp()).get();
            let shared =
                scope.protect(call3("Reentrant__shared", SEXP::nil(), owner.as_sexp(), op));
            assert_eq!(
                bool::try_from_sexp(shared.get()).unwrap(),
                operation % 2 == 1
            );
            let exclusive = scope.protect(call3(
                "Reentrant__exclusive",
                SEXP::nil(),
                owner.as_sexp(),
                op,
            ));
            assert!(bool::try_from_sexp(exclusive.get()).unwrap());
        }
        let failed = scope.protect(call2("Reentrant__fail", SEXP::nil(), owner.as_sexp()));
        assert!(RCondition::from_tagged_sexp(failed.get()).is_some());
        let value = scope.protect(13_i32.into_sexp()).get();
        let result = scope.protect(call3(
            "Reentrant__write",
            SEXP::nil(),
            owner.as_sexp(),
            value,
        ));
        assert!(RCondition::from_tagged_sexp(result.get()).is_none());
        assert_eq!(owner.value, 13);
        OWNER.with(|slot| slot.set(None));
    });
}

#[miniextendr]
pub trait ReentrantProbe {
    fn read(&self) -> i32;
    fn nested_view(&mut self) -> bool;
}

#[miniextendr(env, noexport)]
impl ReentrantProbe for Reentrant {
    fn read(&self) -> i32 {
        self.value
    }
    fn nested_view(&mut self) -> bool {
        let owner = OWNER.with(Cell::get).unwrap();
        let view = unsafe { ReentrantProbeView::from_sexp(owner) };
        let conflict = miniextendr_api::condition::catch_condition(|| view.read()).unwrap_err();
        let (_, parts) = conflict.into_parts();
        self.value += 1;
        parts
            .class
            .iter()
            .any(|class| class == "miniextendr_borrow_conflict")
    }
}

#[test]
fn distinct_trait_views_share_the_owners_borrow_counter() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(
            __mx_wrap_Reentrant(host()),
        ));
        OWNER.with(|slot| slot.set(Some(owner.get())));
        let mut first = ReentrantProbeView::from_sexp(owner.get());
        let second = ReentrantProbeView::from_sexp(owner.get());
        assert!(first.nested_view());
        assert_eq!(second.read(), 8);
        OWNER.with(|slot| slot.set(None));
    });
}

#[test]
fn guards_release_after_unwind_and_keep_independent_objects_independent() {
    r_test_utils::with_r_thread(|| unsafe {
        let first = ExternalPtr::new(host());
        let second = ExternalPtr::new(host());
        let read = ExternalBorrow::acquire(first.as_sexp(), false);
        let read_again = ExternalBorrow::acquire(first.as_sexp(), false);
        let write_other = ExternalBorrow::acquire(second.as_sexp(), true);
        miniextendr_api::sys::R_gc();
        drop((read, read_again, write_other));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _write = ExternalBorrow::acquire(first.as_sexp(), true);
            panic!("release during unwind");
        }));
        assert!(failed.is_err());
        let _write = ExternalBorrow::acquire(first.as_sexp(), true);
    });
}

#[test]
fn guards_retain_the_counter_after_moving_out_the_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        let write = ExternalBorrow::acquire(owner.as_sexp(), true);
        let value = ExternalPtr::into_inner(owner);
        miniextendr_api::sys::R_gc();
        drop(write);
        assert_eq!(value.value, 7);
    });
}

#[test]
fn generated_receivers_resolve_active_bindings_once() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::expression::r_eval_str_global;
        use miniextendr_api::sys::{R_GlobalEnv, Rf_eval, Rf_lang2};
        let owner = ExternalPtr::new(host());
        let scope = ProtectScope::new();
        let factory = scope.protect(
            r_eval_str_global(
                r#"function(p) {
            force(p)
            e <- new.env(parent = emptyenv())
            n <- 0L
            makeActiveBinding(".ptr", function() {
                n <<- n + 1L
                if (n == 1L) p else stop("receiver resolved twice")
            }, e)
            e
        }"#,
            )
            .unwrap(),
        );
        let factory_call = scope.protect(Rf_lang2(factory.get(), owner.as_sexp()));
        let flag = scope.protect(false.into_sexp());
        for field in [false, true] {
            let receiver = scope.protect(Rf_eval(factory_call.get(), R_GlobalEnv));
            let result = scope.protect(if field {
                call2("__mx_rdata_get_Reentrant_value", receiver.get(), flag.get())
            } else {
                call2("Reentrant__read", SEXP::nil(), receiver.get())
            });
            assert_eq!(i32::try_from_sexp(result.get()).unwrap(), 7);
        }
    });
}
