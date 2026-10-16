//! Reject live foreign addresses before treating them as Rust-owned boxes.
mod r_test_utils;

use miniextendr_api::externalptr::{Sidecar, TypeMismatchError};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, SEXP, SEXPTYPE, SexpExt, TryFromSexp};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

// Test-only access to R metadata: these are deliberately private in the facade.
unsafe extern "C-unwind" {
    fn R_MakeExternalPtr(address: *mut c_void, tag: SEXP, prot: SEXP) -> SEXP;
    fn R_SetExternalPtrTag(owner: SEXP, tag: SEXP);
    fn R_ExternalPtrProtected(owner: SEXP) -> SEXP;
}

#[derive(ExternalPtr)]
struct ForeignSidecarHost {
    #[r_data]
    values: Sidecar<Vec<i32>>,
}

fn assert_rejected(sexp: SEXP) {
    unsafe {
        assert!(ExternalPtr::<i32>::wrap_sexp(sexp).is_none());
        assert!(ExternalPtr::<i32>::wrap_sexp_unchecked(sexp).is_none());
        assert!(matches!(
            ExternalPtr::<i32>::wrap_sexp_with_error(sexp),
            Err(TypeMismatchError::InvalidTypeId)
        ));
        assert!(ExternalPtr::<()>::wrap_sexp(sexp).is_none());
        assert!(ExternalPtr::<()>::wrap_sexp_unchecked(sexp).is_none());
        assert!(ExternalPtr::<()>::wrap_sexp_with_error(sexp).is_err());
        assert!(ExternalPtr::<i32>::try_from_sexp(sexp).is_err());
        assert!(ExternalPtr::<i32>::try_from_sexp_unchecked(sexp).is_err());
        assert!(Option::<ExternalPtr<i32>>::try_from_sexp(sexp).is_err());

        // The erased receiver handle can carry arbitrary EXTPTRSXPs so its
        // diagnostic and consuming paths must also check before dereferencing.
        let mut erased = ExternalPtr::from_sexp(sexp);
        assert!(!erased.is::<i32>());
        assert!(!erased.is_consumed());
        assert!(erased.downcast_cell::<i32>().is_none());
        assert!(erased.downcast_ref::<i32>().is_none());
        assert!(erased.downcast_mut::<i32>().is_none());
        assert!(erased.take_for_consuming::<i32>().is_none());
        assert!(catch_unwind(AssertUnwindSafe(|| erased.restore_after_consuming(99))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| ExternalPtr::into_inner(erased))).is_err());
    }
}

#[test]
fn foreign_metadata_never_grants_access_to_an_unrelated_address() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let symbol = miniextendr_api::sys::Rf_install(c"foreign".as_ptr());
        let integer = scope.protect(SEXP::scalar_integer(17)).get();
        let short = scope
            .protect(miniextendr_api::sys::Rf_allocVector(SEXPTYPE::VECSXP, 1))
            .get();
        short.set_vector_elt(0, symbol);
        let missing_id = scope
            .protect(miniextendr_api::sys::Rf_allocVector(SEXPTYPE::VECSXP, 2))
            .get();
        let metadata = scope
            .protect(miniextendr_api::sys::Rf_allocVector(SEXPTYPE::VECSXP, 2))
            .get();
        metadata.set_vector_elt(0, symbol);

        // A real, live allocation with no Any vtable. No finalizer is installed:
        // R does not own this stack value, and rejection must leave it intact.
        let mut foreign_value = 42_i32;
        let address = std::ptr::from_mut(&mut foreign_value).cast();
        for (tag, prot) in [
            (symbol, SEXP::nil()),
            (symbol, integer),
            (symbol, short),
            (symbol, missing_id),
            (SEXP::nil(), metadata),
            (integer, metadata),
        ] {
            let foreign = scope.protect(R_MakeExternalPtr(address, tag, prot)).get();
            assert_rejected(foreign);
            assert_eq!(miniextendr_api::sys::R_ExternalPtrAddr(foreign), address);
            assert_eq!(foreign_value, 42);
        }
    });
}

#[test]
fn trait_abi_sidecars_do_not_make_the_address_an_any_box() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let erased = scope
            .protect(miniextendr_api::mx_abi::mx_wrap(
                __mx_wrap_ForeignSidecarHost(ForeignSidecarHost {
                    values: Sidecar::new(vec![1, 2, 3]),
                }),
            ))
            .get();
        let prot = R_ExternalPtrProtected(erased);
        assert_eq!(prot.type_of(), SEXPTYPE::VECSXP);
        assert_eq!(prot.vector_elt(0).type_of(), SEXPTYPE::SYMSXP);
        assert_rejected(erased);
    });
}

#[test]
fn concrete_type_still_comes_from_any_and_consumed_slots_still_work() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ExternalPtr::new(7_i32);
        let sexp = owner.as_sexp();
        let diagnostic_only = miniextendr_api::sys::Rf_install(c"different display name".as_ptr());
        R_SetExternalPtrTag(sexp, diagnostic_only);
        R_ExternalPtrProtected(sexp).set_vector_elt(0, diagnostic_only);
        assert!(ExternalPtr::<i32>::wrap_sexp(sexp).is_some());
        assert!(matches!(
            ExternalPtr::<f64>::wrap_sexp_with_error(sexp),
            Err(TypeMismatchError::Mismatch { .. })
        ));
        let mut erased = ExternalPtr::from_sexp(sexp);
        assert_eq!(erased.take_for_consuming::<i32>(), Some(7));
        assert!(erased.is_consumed());
        erased.restore_after_consuming(11_i32);
        assert!(!erased.is_consumed());
        drop(erased);
        assert_eq!(ExternalPtr::into_inner(owner), 11);
    });
}
