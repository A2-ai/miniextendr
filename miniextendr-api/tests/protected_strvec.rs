//! Owned character-vector roots survive source scopes and arbitrary drop order.
mod r_test_utils;

use miniextendr_api::sys::{
    R_EmptyEnv, R_MakeWeakRef, R_PreserveObject, R_ReleaseObject, R_WeakRefKey, R_gc, Rf_install,
    Rf_setAttrib,
};
use miniextendr_api::{
    IntoR, OwnedProtect, ProtectScope, ProtectedStrVec, Rboolean, SEXP, TryFromSexp,
};

/// Whether `x` stays rooted. R takes only reference objects as weak reference
/// keys, so the key is an environment reachable only through an attribute of
/// `x`: a full GC clears it once nothing roots `x`. Reading the strings after
/// a GC can't tell, since freed memory keeps its bytes until it is reused.
/// The watch is preserved (released by `Drop`), so it never shares the
/// protection stack with the owners under test.
struct Watch(SEXP);

impl Watch {
    unsafe fn new(x: SEXP) -> Self {
        unsafe {
            let guard = ProtectScope::new();
            let sentinel = guard.new_env(R_EmptyEnv, false, 0).get();
            Rf_setAttrib(x, Rf_install(c"mx_watch".as_ptr()), sentinel);
            let weak = guard.protect_raw(R_MakeWeakRef(
                sentinel,
                SEXP::nil(),
                SEXP::nil(),
                Rboolean::FALSE,
            ));
            R_PreserveObject(weak);
            Watch(weak)
        }
    }

    fn rooted(&self) -> bool {
        !unsafe { R_WeakRefKey(self.0) }.is_nil()
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.0) };
    }
}

/// Convert a fresh character vector the way a wrapper argument is converted,
/// then drop the source's own protection: only the owner can root it now.
unsafe fn owner(values: Vec<Option<&str>>) -> (ProtectedStrVec, Watch) {
    unsafe {
        let source = OwnedProtect::new(values.into_sexp());
        let watch = Watch::new(source.get());
        (ProtectedStrVec::try_from_sexp(source.get()).unwrap(), watch)
    }
}

#[test]
fn converted_string_owners_outlive_source_roots_and_drop_in_any_order() {
    r_test_utils::with_r_thread(|| unsafe {
        let (first, first_watch) = owner(vec![Some("first"), Some("héllo")]);
        let (second, second_watch) = owner(vec![Some("second"), None]);
        R_gc();
        assert!(first_watch.rooted(), "first owner lost its root");
        assert!(second_watch.rooted(), "second owner lost its root");
        // Dropping the earlier owner releases its own root and leaves the
        // later owner's in place.
        drop(first);
        R_gc();
        assert!(!first_watch.rooted(), "dropped owner kept its root");
        assert!(
            second_watch.rooted(),
            "dropping the first owner unrooted the second"
        );
        assert_eq!(second.iter().collect::<Vec<_>>(), [Some("second"), None]);
        assert_eq!(second.as_strvec().get_str(0), Some("second"));
        let result = OwnedProtect::new(second.into_sexp());
        R_gc();
        assert!(second_watch.rooted());
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(result.get()).unwrap(),
            [Some("second".into()), None]
        );
    });
}

#[test]
fn borrowed_views_use_scope_roots_and_keep_rcow_round_trip_identity() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = miniextendr_api::ProtectScope::new();
        let chars = scope.protect(vec![Some("rooted"), None].into_sexp());
        let strings = miniextendr_api::StrVec::try_from_root(chars).unwrap();
        let nums = scope.protect(vec![1_i32, 2, 3].into_sexp());
        let values = miniextendr_api::RCow::<i32>::try_from_root(nums).unwrap();
        miniextendr_api::sys::R_gc();
        assert_eq!(strings.iter().collect::<Vec<_>>(), [Some("rooted"), None]);
        assert_eq!(&*values, &[1, 2, 3]);
        assert_eq!(values.into_sexp(), nums.get());
        assert!(miniextendr_api::StrVec::try_from_root(nums).is_err());
        assert!(miniextendr_api::RCow::<i32>::try_from_root(chars).is_err());
    });
}
