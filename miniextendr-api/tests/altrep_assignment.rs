//! R replacement-function boundaries can refresh Rust-owned ALTREP storage.
mod r_test_utils;

use miniextendr_api::altrep_data::AltrepExtract;
use miniextendr_api::altrep_traits::{AltVec, Altrep};
use miniextendr_api::expression::{RCall, r_eval_str};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, SEXP, SexpExt, TryFromSexp};
use std::borrow::Cow;

// The duplicate policy is essential: R can duplicate before entering a
// replacement function. Its default would materialize an ordinary R vector.
// The live view's hand-written Duplicate returns the same object; the derive
// offers only the value-semantics `duplicate = "clone"` policy.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, no_lowlevel, class = "AssignmentLiveVec")]
struct LiveVec(Vec<i32>);

#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, dataptr, duplicate = "clone", class = "AssignmentCowVec")]
struct CowVec(Cow<'static, [i32]>);

macro_rules! direct_integer_view {
    ($ty:ty) => {
        impl miniextendr_api::altrep_data::AltrepLen for $ty {
            fn len(&self) -> usize {
                self.0.len()
            }
        }
        impl miniextendr_api::altrep_data::AltIntegerData for $ty {
            fn elt(&self, i: usize) -> i32 {
                self.0[i]
            }
        }
    };
}
direct_integer_view!(LiveVec);
direct_integer_view!(CowVec);
miniextendr_api::__impl_altvec_dataptr!(LiveVec, i32);
miniextendr_api::__impl_altinteger_methods!(LiveVec);
miniextendr_api::impl_inferbase_integer!(LiveVec);

impl miniextendr_api::altrep_data::AltrepDataptr<i32> for LiveVec {
    fn dataptr(&mut self, _: bool) -> Option<*mut i32> {
        Some(self.0.as_mut_ptr())
    }
    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.0.as_ptr())
    }
}
impl miniextendr_api::altrep_data::AltrepDataptr<i32> for CowVec {
    fn dataptr(&mut self, _: bool) -> Option<*mut i32> {
        Some(self.0.to_mut().as_mut_ptr())
    }
    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.0.as_ptr())
    }
}
impl Altrep for LiveVec {
    fn length(x: SEXP) -> miniextendr_api::R_xlen_t {
        unsafe { Self::altrep_extract_ref(x).0.len().try_into().unwrap() }
    }
    const HAS_DUPLICATE: bool = true;
    fn duplicate(x: SEXP, _: bool) -> SEXP {
        // Explicit reference semantics: aliases observe the same owner's growth.
        x
    }
}

// Test-only .Call callbacks. Typed accessors validate the object as generated
// entry points do; a regression that materializes it must fail, not cast an
// ordinary vector's header as ALTREP data1.
unsafe extern "C-unwind" fn grow_live_vec(x: SEXP, size: SEXP) -> SEXP {
    let size = usize::try_from_sexp(size).unwrap();
    let Ok(mut view) = LiveVecMut::try_from_sexp(x) else {
        return SEXP::nil();
    };
    let data = &mut (*view).0;
    data.resize(size, 99);
    x
}

unsafe extern "C-unwind" fn grow_cow_value(x: SEXP, size: SEXP) -> SEXP {
    let size = usize::try_from_sexp(size).unwrap();
    let Ok(mut view) = CowVecMut::try_from_sexp(x) else {
        return SEXP::nil();
    };
    (*view).0.to_mut().resize(size, 99);
    x
}

unsafe fn assignment_env(
    x: SEXP,
    callback: unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
) -> OwnedProtect {
    unsafe {
        let env = OwnedProtect::new(RCall::new("new.env").eval_base().unwrap());
        miniextendr_api::sys::Rf_defineVar(
            miniextendr_api::sys::Rf_install(c"x".as_ptr()),
            x,
            env.get(),
        );
        let address = OwnedProtect::new(miniextendr_api::sys::R_MakeExternalPtrFn(
            Some(std::mem::transmute::<
                unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
                unsafe extern "C-unwind" fn() -> *mut std::ffi::c_void,
            >(callback)),
            miniextendr_api::sys::Rf_install(c"native symbol".as_ptr()),
            SEXP::nil(),
        ));
        miniextendr_api::sys::Rf_defineVar(
            miniextendr_api::sys::Rf_install(c"grow_address".as_ptr()),
            address.get(),
            env.get(),
        );
        r_eval_str(
            "`grow<-` <- function(x, value) .Call(grow_address, x, value)",
            env.get(),
        )
        .unwrap();
        env
    }
}

#[test]
fn replacement_function_refreshes_a_live_vec_after_capacity_growth() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(LiveVec(vec![10i32, 20]).into_sexp());
        let old_capacity = <LiveVec as AltrepExtract>::altrep_extract_ref(x.get())
            .0
            .capacity();
        let old_ptr = <LiveVec as AltVec>::dataptr(x.get(), false);
        let env = assignment_env(x.get(), grow_live_vec);
        r_eval_str(
            "gctorture(TRUE);
             tryCatch({
               alias <- x; grow(x) <- 4096L;
               stopifnot(length(x) == 4096L, x[[4096L]] == 99L,
                         length(alias) == 4096L, sum(x) == 30 + 4094 * 99)
             }, finally = gctorture(FALSE))",
            env.get(),
        )
        .unwrap();
        let current_ptr = <LiveVec as AltVec>::dataptr(x.get(), false);
        assert_eq!(<LiveVec as Altrep>::length(x.get()), 4096);
        let data = <LiveVec as AltrepExtract>::altrep_extract_ref(x.get());
        assert!(data.0.capacity() > old_capacity);
        assert_eq!(data.0.as_ptr().cast(), current_ptr.cast_const());
        // The allocator may grow in place. The storage regression separately
        // forces a changed allocation; both paths must reacquire the pointer.
        // old_ptr is never dereferenced after the replacement boundary.
        let _ = old_ptr;
    });
}

#[test]
fn replacement_function_can_detach_cow_and_preserve_the_old_alias() {
    static ORIGINAL: &[i32] = &[10, 20];
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(CowVec(Cow::Borrowed(ORIGINAL)).into_sexp());
        let old_ptr = <CowVec as AltVec>::dataptr(x.get(), false);
        let env = assignment_env(x.get(), grow_cow_value);
        r_eval_str(
            "gctorture(TRUE);
             tryCatch({
               alias <- x; grow(x) <- 4096L;
               stopifnot(length(x) == 4096L, x[[4096L]] == 99L,
                         length(alias) == 2L, alias[[1L]] == 10L)
             }, finally = gctorture(FALSE))",
            env.get(),
        )
        .unwrap();
        let replacement = OwnedProtect::new(r_eval_str("x", env.get()).unwrap());
        let data = <CowVec as AltrepExtract>::altrep_extract_ref(replacement.get());
        assert!(replacement.get().is_altrep());
        assert!(matches!(data.0, Cow::Owned(_)));
        assert_eq!(data.0.len(), 4096);
        assert_ne!(
            old_ptr,
            <CowVec as AltVec>::dataptr(replacement.get(), false)
        );
        assert_eq!(<CowVec as Altrep>::length(x.get()), 2);
        assert_eq!(<CowVec as AltVec>::dataptr(x.get(), false), old_ptr);
        assert_eq!(ORIGINAL, &[10, 20]);
    });
}

#[test]
fn ordinary_element_replacement_preserves_cow_altrep_and_the_old_pointer() {
    static ORIGINAL: &[i32] = &[10, 20];
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(CowVec(Cow::Borrowed(ORIGINAL)).into_sexp());
        let old_ptr = <CowVec as AltVec>::dataptr(x.get(), false);
        let env = assignment_env(x.get(), grow_cow_value);
        r_eval_str(
            "gctorture(TRUE);
             tryCatch({
               alias <- x; x[1L] <- 77L;
               stopifnot(x[[1L]] == 77L, alias[[1L]] == 10L)
             }, finally = gctorture(FALSE))",
            env.get(),
        )
        .unwrap();
        let replacement = OwnedProtect::new(r_eval_str("x", env.get()).unwrap());
        assert!(replacement.get().is_altrep());
        assert_ne!(replacement.get(), x.get());
        let data = <CowVec as AltrepExtract>::altrep_extract_ref(replacement.get());
        assert!(matches!(data.0, Cow::Owned(_)));
        assert_eq!(&*data.0, &[77, 20]);
        // Unlike the live-view policy, CoW preserves already exposed storage
        // on the old object: it can still be read after the replacement call.
        assert_eq!(*old_ptr.cast::<i32>(), 10);
        assert_eq!(<CowVec as AltVec>::dataptr(x.get(), false), old_ptr);
    });
}

#[derive(Clone, miniextendr_api::AltrepString)]
#[altrep(manual, duplicate = "clone", class = "AssignmentStringCache")]
struct CachedStrings(Vec<String>);
impl miniextendr_api::altrep_data::AltrepLen for CachedStrings {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl miniextendr_api::altrep_data::AltStringData for CachedStrings {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(&self.0[i])
    }
}

#[test]
fn clone_policy_keeps_partial_and_written_string_caches_and_attributes() {
    use miniextendr_api::altrep_traits::AltString;
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(CachedStrings(vec!["a".into(), "b".into()]).into_sexp());
        let names = OwnedProtect::new(vec!["first", "second"].into_sexp());
        miniextendr_api::sys::Rf_setAttrib(
            x.get(),
            miniextendr_api::sys::R_NamesSymbol,
            names.get(),
        );
        // Computing one element leaves a C NULL slot in data2. Duplication
        // must copy it without sending that slot through SET_STRING_ELT.
        <CachedStrings as AltString>::elt(x.get(), 0);
        let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
        assert!(copy.get().is_altrep());
        assert_eq!(
            Vec::<String>::try_from_sexp(copy.get()).unwrap(),
            ["a", "b"]
        );
        assert_eq!(
            Vec::<String>::try_from_sexp(miniextendr_api::sys::Rf_getAttrib(
                copy.get(),
                miniextendr_api::sys::R_NamesSymbol
            ))
            .unwrap(),
            ["first", "second"]
        );
        let changed = OwnedProtect::new(SEXP::charsxp("changed"));
        <CachedStrings as AltString>::set_elt(x.get(), 0, changed.get());
        let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
        assert_eq!(
            Vec::<String>::try_from_sexp(copy.get()).unwrap(),
            ["changed", "b"]
        );
    });
}
