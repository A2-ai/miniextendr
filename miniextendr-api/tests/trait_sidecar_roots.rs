//! Erased trait objects must transfer pending sidecars into R-traced slots.
mod r_test_utils;

use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{ExternalPtr, IntoR, SEXP, SEXPTYPE, SexpExt, TryFromSexp};

// Inspect the actual GC edge, without manufacturing an ordinary ExternalPtr
// over erased storage. This R API is crate-private in the runtime facade.
unsafe extern "C-unwind" {
    fn R_ExternalPtrProtected(owner: SEXP) -> SEXP;
}

#[derive(ExternalPtr)]
struct SidecarHost {
    #[r_data]
    source: Sidecar<SEXP>,
    #[r_data]
    numbers: Sidecar<Vec<i32>>,
    #[r_data]
    label: Sidecar<String>,
}

#[derive(ExternalPtr)]
struct PlainHost;

#[test]
fn wrapping_erased_storage_roots_every_pending_slot() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = {
            let source = OwnedProtect::new(vec![13_i32, 21, 34].into_sexp());
            let owner =
                miniextendr_api::trait_abi::ccall::mx_wrap(__mx_wrap_SidecarHost(SidecarHost {
                    source: Sidecar::new(source.get()),
                    numbers: Sidecar::new(vec![55, 89]),
                    label: Sidecar::new("retained".to_string()),
                }));
            let slots = R_ExternalPtrProtected(owner);
            // Fail before collecting if the GC edge was never installed.
            assert_eq!(slots.type_of(), SEXPTYPE::VECSXP);
            assert_eq!(slots.len(), 5);
            assert_eq!(slots.vector_elt(2), source.get());
            owner
        };
        // No allocation in the gap between the constructor's temporary roots
        // and this owner root. From now on only the owner roots the sidecars.
        let owner = OwnedProtect::new(owner);
        miniextendr_api::sys::R_gc();
        let slots = R_ExternalPtrProtected(owner.get());
        assert_eq!(slots.vector_elt(0).type_of(), SEXPTYPE::SYMSXP);
        assert_eq!(slots.vector_elt(1), SEXP::nil());
        assert_eq!(
            Vec::<i32>::try_from_sexp(slots.vector_elt(2)).unwrap(),
            [13, 21, 34]
        );
        assert_eq!(
            Vec::<i32>::try_from_sexp(slots.vector_elt(3)).unwrap(),
            [55, 89]
        );
        assert_eq!(
            String::try_from_sexp(slots.vector_elt(4)).unwrap(),
            "retained"
        );
    });
}

#[test]
fn erased_types_without_sidecars_allocate_no_slot_list() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(miniextendr_api::trait_abi::ccall::mx_wrap(
            __mx_wrap_PlainHost(PlainHost),
        ));
        assert_eq!(R_ExternalPtrProtected(owner.get()), SEXP::nil());
    });
}
