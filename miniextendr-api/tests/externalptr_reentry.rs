//! Re-enter the generated C boundaries while their receiver reference is live.
mod r_test_utils;

use miniextendr_api::condition::catch_condition;
use miniextendr_api::externalptr::{ExternalPtrMut, ExternalPtrRef, RSidecar, Sidecar};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::{ExternalPtr, IntoR, RCondition, SEXP, TryFromSexp, miniextendr};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

thread_local! {
    static OWNER: Cell<Option<SEXP>> = const { Cell::new(None) };
}

#[derive(ExternalPtr)]
pub struct Reentrant {
    // The selector that makes the derive emit the field accessors.
    #[r_data]
    _r: RSidecar,
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
        _r: RSidecar,
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

fn is_borrow_conflict(condition: RCondition) -> bool {
    let (_, parts) = condition.into_parts();
    parts
        .class
        .iter()
        .any(|class| class == "miniextendr_borrow_conflict")
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
                assert!(is_borrow_conflict(condition));
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
        let conflict = catch_condition(|| view.read()).unwrap_err();
        self.value += 1;
        is_borrow_conflict(conflict)
    }
}

#[test]
fn trait_views_meet_the_producers_borrow() {
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
fn handles_hold_the_borrow_until_dropped_or_unwound() {
    r_test_utils::with_r_thread(|| unsafe {
        let first = ExternalPtr::new(host());
        let mut second = ExternalPtr::new(host());
        let alias = ExternalPtr::<Reentrant>::wrap_sexp(first.as_sexp()).unwrap();
        let mut writer = ExternalPtr::<Reentrant>::wrap_sexp(first.as_sexp()).unwrap();
        let read = ExternalPtrRef::from(&first);
        let read_again = ExternalPtrRef::from(&alias);
        let write_other = ExternalPtrMut::from(&mut second);
        let conflict = catch_condition(|| drop(ExternalPtrMut::from(&mut writer)));
        assert!(is_borrow_conflict(conflict.unwrap_err()));
        miniextendr_api::sys::R_gc();
        drop((read, read_again, write_other));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _write = ExternalPtrMut::from(&mut writer);
            panic!("release during unwind");
        }));
        assert!(failed.is_err());
        ExternalPtrMut::from(&mut writer).value = 8;
        assert_eq!(first.value, 8);
    });
}

#[test]
fn into_inner_refuses_a_borrowed_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        let alias = ExternalPtr::<Reentrant>::wrap_sexp(owner.as_sexp()).unwrap();
        let read = ExternalPtrRef::from(&owner);
        let conflict = catch_condition(|| ExternalPtr::into_inner(alias).value);
        assert!(is_borrow_conflict(conflict.unwrap_err()));
        drop(read);
        assert_eq!(ExternalPtr::into_inner(owner).value, 7);
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

static ROOTED_DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(ExternalPtr)]
pub struct Rooted {
    value: i32,
}

impl Drop for Rooted {
    fn drop(&mut self) {
        ROOTED_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[miniextendr(env, noexport)]
impl Rooted {
    /// Run `callback`, which drops the receiver's last reference and collects.
    pub fn survive(&self, callback: SEXP) -> bool {
        unsafe {
            use miniextendr_api::sys::{R_GlobalEnv, Rf_eval, Rf_lang1};
            let call = OwnedProtect::new(Rf_lang1(callback));
            Rf_eval(call.get(), R_GlobalEnv);
        }
        ROOTED_DROPS.load(Ordering::SeqCst) == 0 && self.value == 3
    }
}

/// A class-wrapped receiver whose `.ptr` goes away mid-call stays alive
/// until the call, and its borrow, ends.
#[test]
fn a_resolved_receiver_outlives_its_binding_during_the_call() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::expression::r_eval_str_global;
        use miniextendr_api::sys::{R_GlobalEnv, Rf_defineVar, Rf_eval, Rf_install, Rf_lang2};
        let scope = ProtectScope::new();
        let receiver = scope
            .protect(r_eval_str_global("new.env(parent = emptyenv())").unwrap())
            .get();
        let ptr = ExternalPtr::new(Rooted { value: 3 });
        Rf_defineVar(Rf_install(c".ptr".as_ptr()), ptr.as_sexp(), receiver);
        drop(ptr);
        let factory = scope.protect(
            r_eval_str_global(r#"function(e) function() { rm(".ptr", envir = e); gc() }"#).unwrap(),
        );
        let factory_call = scope.protect(Rf_lang2(factory.get(), receiver));
        let callback = scope
            .protect(Rf_eval(factory_call.get(), R_GlobalEnv))
            .get();
        let result = scope.protect(call3("Rooted__survive", SEXP::nil(), receiver, callback));
        assert!(bool::try_from_sexp(result.get()).unwrap());
        miniextendr_api::sys::R_gc();
        assert_eq!(ROOTED_DROPS.load(Ordering::SeqCst), 1);
    });
}
