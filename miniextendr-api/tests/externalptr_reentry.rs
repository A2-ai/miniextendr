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
    /// A `&Self` argument is borrowed like any `&T`: the receiver's own
    /// object conflicts with the receiver's exclusive borrow.
    pub fn absorb(&mut self, other: &Self) -> i32 {
        self.value += other.value;
        self.value
    }
    /// A view receiver holds its borrow and reaches the sidecar values.
    pub fn view_shared(self: ExternalPtrRef<'_, Self>, operation: i32) -> bool {
        let len = Self::values(&self).len();
        let conflict = nested(operation);
        assert_eq!(Self::values(&self.rb()).len(), len);
        conflict
    }
    pub fn view_exclusive(mut self: ExternalPtrMut<'_, Self>, operation: i32) -> bool {
        let conflict = nested(operation);
        Self::set_values(&mut self.rb_mut(), vec![1, 2]);
        self.value += 1;
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
        assert_eq!(owner.borrow().value, 13);
        OWNER.with(|slot| slot.set(None));
    });
}

#[test]
fn view_receivers_and_self_arguments_borrow_for_the_call() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        let other = ExternalPtr::new(host());
        OWNER.with(|slot| slot.set(Some(owner.as_sexp())));
        let scope = ProtectScope::new();
        for operation in 0..6_i32 {
            let op = scope.protect(operation.into_sexp()).get();
            let shared = scope.protect(call3(
                "Reentrant__view_shared",
                SEXP::nil(),
                owner.as_sexp(),
                op,
            ));
            assert_eq!(
                bool::try_from_sexp(shared.get()).unwrap(),
                operation % 2 == 1,
                "operation {operation}"
            );
            let exclusive = scope.protect(call3(
                "Reentrant__view_exclusive",
                SEXP::nil(),
                owner.as_sexp(),
                op,
            ));
            assert!(
                bool::try_from_sexp(exclusive.get()).unwrap(),
                "operation {operation}"
            );
        }
        assert_eq!(Reentrant::values(&owner), [1, 2]);
        OWNER.with(|slot| slot.set(None));

        let same = scope.protect(call3(
            "Reentrant__absorb",
            SEXP::nil(),
            owner.as_sexp(),
            owner.as_sexp(),
        ));
        assert!(is_borrow_conflict(
            RCondition::from_tagged_sexp(same.get()).unwrap()
        ));
        let before = owner.borrow().value;
        let distinct = scope.protect(call3(
            "Reentrant__absorb",
            SEXP::nil(),
            owner.as_sexp(),
            other.as_sexp(),
        ));
        assert_eq!(i32::try_from_sexp(distinct.get()).unwrap(), before + 7);
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
        let read = first.borrow();
        let read_again = alias.borrow();
        let write_other = second.borrow_mut();
        let conflict = catch_condition(|| drop(writer.borrow_mut()));
        assert!(is_borrow_conflict(conflict.unwrap_err()));
        miniextendr_api::sys::R_gc();
        drop((read, read_again, write_other));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _write = writer.borrow_mut();
            panic!("release during unwind");
        }));
        assert!(failed.is_err());
        writer.borrow_mut().value = 8;
        assert_eq!(first.borrow().value, 8);
    });
}

#[test]
fn into_inner_refuses_a_borrowed_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        let alias = ExternalPtr::<Reentrant>::wrap_sexp(owner.as_sexp()).unwrap();
        let read = owner.borrow();
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

// region: borrowed arguments (#2092, #2106)

/// One object as the `&mut` argument and the other: the second borrow conflicts.
#[miniextendr(noexport)]
pub fn reentrant_pair(target: &mut Reentrant, source: &Reentrant) -> i32 {
    target.value += source.value;
    target.value
}

/// Hold the argument exclusively across `nested(operation)`.
#[miniextendr(noexport)]
pub fn reentrant_exclusive_arg(obj: &mut Reentrant, operation: i32) -> bool {
    let conflict = nested(operation);
    obj.value += 1;
    conflict
}

/// Hold the argument shared across `nested(operation)`.
#[miniextendr(noexport)]
pub fn reentrant_shared_arg(obj: &Reentrant, operation: i32) -> bool {
    let old = obj.value;
    let conflict = nested(operation);
    assert_eq!(obj.value, old);
    conflict
}

/// `NULL` is `None`, which holds nothing.
#[miniextendr(noexport)]
pub fn reentrant_optional_arg(obj: Option<&mut Reentrant>, operation: i32) -> bool {
    let conflict = nested(operation);
    if let Some(obj) = obj {
        obj.value += 1;
    }
    conflict
}

/// An exclusive view holds its borrow and reaches the sidecar values.
#[miniextendr(noexport)]
pub fn reentrant_view_arg(mut obj: ExternalPtrMut<'_, Reentrant>, operation: i32) -> bool {
    let conflict = nested(operation);
    Reentrant::set_values(&mut obj, vec![4]);
    obj.value += 1;
    conflict
}

/// A handle holds nothing: callbacks reach the object freely, and the body
/// borrows only to read.
#[miniextendr(noexport)]
pub fn reentrant_handle_arg(obj: ExternalPtr<Reentrant>, operation: i32) -> bool {
    let conflict = nested(operation);
    assert!(obj.borrow().value > 0);
    conflict
}

/// Moves the value out: a handle takes no borrow.
#[miniextendr(noexport)]
pub fn reentrant_take(obj: ExternalPtr<Reentrant>) -> i32 {
    ExternalPtr::into_inner(obj).value
}

/// A type-erased argument has no value of its own to borrow.
#[miniextendr(noexport)]
pub fn reentrant_erased_arg(mut obj: ExternalPtr<()>) -> bool {
    obj.downcast_mut::<Reentrant>().is_some()
}

#[test]
fn arguments_hold_their_objects_for_the_call() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(host());
        let other = ExternalPtr::new(host());
        OWNER.with(|slot| slot.set(Some(owner.as_sexp())));
        let scope = ProtectScope::new();

        let same = scope.protect(call3(
            "reentrant_pair",
            SEXP::nil(),
            owner.as_sexp(),
            owner.as_sexp(),
        ));
        assert!(is_borrow_conflict(
            RCondition::from_tagged_sexp(same.get()).unwrap()
        ));
        assert_eq!(owner.borrow().value, 7);
        let distinct = scope.protect(call3(
            "reentrant_pair",
            SEXP::nil(),
            owner.as_sexp(),
            other.as_sexp(),
        ));
        assert_eq!(i32::try_from_sexp(distinct.get()).unwrap(), 14);

        for operation in 0..6_i32 {
            let op = scope.protect(operation.into_sexp()).get();
            let call = |name: &str, obj: SEXP| {
                let result = scope.protect(call3(name, SEXP::nil(), obj, op));
                bool::try_from_sexp(result.get()).unwrap()
            };
            let owner = owner.as_sexp();
            assert!(
                call("reentrant_exclusive_arg", owner),
                "operation {operation}"
            );
            assert_eq!(
                call("reentrant_shared_arg", owner),
                operation % 2 == 1,
                "operation {operation}"
            );
            assert!(
                call("reentrant_optional_arg", owner),
                "operation {operation}"
            );
            assert!(
                !call("reentrant_optional_arg", SEXP::nil()),
                "operation {operation}"
            );
            assert!(call("reentrant_view_arg", owner), "operation {operation}");
            assert!(
                !call("reentrant_handle_arg", owner),
                "operation {operation}"
            );
        }
        OWNER.with(|slot| slot.set(None));

        let erased = scope.protect(call2("reentrant_erased_arg", SEXP::nil(), owner.as_sexp()));
        assert!(bool::try_from_sexp(erased.get()).unwrap());
        let taken = scope.protect(call2("reentrant_take", SEXP::nil(), other.as_sexp()));
        assert_eq!(i32::try_from_sexp(taken.get()).unwrap(), 7);
    });
}

#[test]
fn borrow_checks_the_value_like_generated_code() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut owner = ExternalPtr::new(host());
        let reader = ExternalPtr::<Reentrant>::wrap_sexp(owner.as_sexp()).unwrap();
        let mut writer = ExternalPtr::<Reentrant>::wrap_sexp(owner.as_sexp()).unwrap();
        {
            let read = owner.borrow();
            let read_again = reader.borrow();
            let reborrowed = read.rb();
            assert_eq!(read.value + read_again.value + reborrowed.value, 21);
            assert_eq!(ExternalPtrRef::as_sexp(&reborrowed), owner.as_sexp());
            let conflict = catch_condition(|| drop(writer.borrow_mut()));
            assert!(is_borrow_conflict(conflict.unwrap_err()));
        }
        {
            let mut write = owner.borrow_mut();
            write.rb_mut().value = 9;
            let conflict = catch_condition(|| drop(reader.borrow()));
            assert!(is_borrow_conflict(conflict.unwrap_err()));
        }
        assert_eq!(writer.borrow_mut().value, 9);
        let erased = ExternalPtr::<()>::from_sexp(owner.as_sexp());
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(erased.borrow());
        }));
        assert!(refused.is_err());
    });
}

static ARGUMENT_DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(ExternalPtr)]
pub struct RootedArgument {
    value: i32,
}

impl Drop for RootedArgument {
    fn drop(&mut self) {
        ARGUMENT_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// Evaluate `callback`, which drops the argument's last reference and
/// collects; whether the argument's value survived.
fn run_dropping_callback(callback: SEXP) -> bool {
    let drops = ARGUMENT_DROPS.load(Ordering::SeqCst);
    unsafe {
        use miniextendr_api::expression::RCall;
        RCall::from_sexp(callback).eval_with_handlers(miniextendr_api::sys::R_GlobalEnv);
    }
    ARGUMENT_DROPS.load(Ordering::SeqCst) == drops
}

/// A handle argument whose class handle loses its `.ptr` during the call.
#[miniextendr(noexport)]
pub fn rooted_argument_survives(obj: ExternalPtr<RootedArgument>, callback: SEXP) -> bool {
    run_dropping_callback(callback) && obj.borrow().value == 3
}

/// A borrowed argument whose class handle loses its `.ptr` during the call.
#[miniextendr(noexport)]
pub fn rooted_ref_argument_survives(obj: &RootedArgument, callback: SEXP) -> bool {
    run_dropping_callback(callback) && obj.value == 3
}

/// A class-wrapped argument whose `.ptr` goes away mid-call stays alive until
/// the call, and its borrow, ends: the converted handle roots it.
#[test]
fn a_class_wrapped_argument_outlives_its_binding_during_the_call() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::expression::r_eval_str_global;
        use miniextendr_api::sys::{R_GlobalEnv, Rf_defineVar, Rf_eval, Rf_install, Rf_lang2};
        let scope = ProtectScope::new();
        let factory = scope.protect(
            r_eval_str_global(r#"function(e) function() { rm(".ptr", envir = e); gc() }"#).unwrap(),
        );
        for (drops, wrapper) in [
            (1, "rooted_argument_survives"),
            (2, "rooted_ref_argument_survives"),
        ] {
            let handle = scope
                .protect(r_eval_str_global("new.env(parent = emptyenv())").unwrap())
                .get();
            let ptr = ExternalPtr::new(RootedArgument { value: 3 });
            Rf_defineVar(Rf_install(c".ptr".as_ptr()), ptr.as_sexp(), handle);
            drop(ptr);
            let factory_call = scope.protect(Rf_lang2(factory.get(), handle));
            let callback = scope
                .protect(Rf_eval(factory_call.get(), R_GlobalEnv))
                .get();
            let result = scope.protect(call3(wrapper, SEXP::nil(), handle, callback));
            assert!(bool::try_from_sexp(result.get()).unwrap(), "{wrapper}");
            miniextendr_api::sys::R_gc();
            assert_eq!(ARGUMENT_DROPS.load(Ordering::SeqCst), drops, "{wrapper}");
        }
    });
}

// endregion
