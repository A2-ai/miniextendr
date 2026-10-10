//! Storage consistency across direct and materialized ALTREP buffers.
mod r_test_utils;

use miniextendr_api::altrep_traits::{
    AltComplex, AltInteger, AltLogical, AltRaw, AltReal, AltVec, Altrep, UNKNOWN_SORTEDNESS,
};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, IntoRAltrep, Rcomplex};

#[test]
fn materialized_logical_writes_reach_elt() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(vec![true, false].into_sexp_altrep()) };
        let ptr = <Vec<bool> as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *ptr = 0 };
        assert_eq!(<Vec<bool> as AltLogical>::elt(x.get(), 0), 0);
    });
}

#[test]
fn materialized_logical_writes_reach_region() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(vec![true, false].into_sexp_altrep()) };
        let ptr = <Vec<bool> as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *ptr = 0 };
        let mut out = [-1; 2];
        assert_eq!(
            <Vec<bool> as AltLogical>::get_region(x.get(), 0, 2, &mut out),
            2
        );
        assert_eq!(out, [0, 0]);
    });
}

#[test]
fn materialized_serialization_uses_r_fallback() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(vec![true, false].into_sexp_altrep()) };
        let ptr = <Vec<bool> as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *ptr = 0 };
        // The custom Rust serialization state is stale after R-buffer writes.
        // A C NULL requests ordinary R serialization of the current elements.
        assert!(<Vec<bool> as Altrep>::serialized_state(x.get()).is_null_ptr());
    });
}

#[derive(miniextendr_api::AltrepRaw)]
#[altrep(manual, class = "StorageAuditRaw")]
pub struct MaterializingRaw(Vec<u8>);

impl miniextendr_api::altrep_data::AltrepLen for MaterializingRaw {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl miniextendr_api::altrep_data::AltRawData for MaterializingRaw {
    fn elt(&self, i: usize) -> u8 {
        self.0[i]
    }
}

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(manual, class = "StorageAuditComplex")]
pub struct MaterializingComplex(Vec<Rcomplex>);

impl miniextendr_api::altrep_data::AltrepLen for MaterializingComplex {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl miniextendr_api::altrep_data::AltComplexData for MaterializingComplex {
    fn elt(&self, i: usize) -> Rcomplex {
        self.0[i]
    }
}

#[test]
fn every_atomic_family_reads_its_materialized_buffer() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($ty:ty, $family:ident, $elem:ty, $source:expr, $replacement:expr) => {{
                let x = unsafe { OwnedProtect::new($source) };
                assert!(<$ty as AltVec>::dataptr_or_null(x.get()).is_null());
                let ptr = <$ty as AltVec>::dataptr(x.get(), true).cast::<$elem>();
                unsafe { *ptr = $replacement };
                assert_eq!(<$ty as $family>::elt(x.get(), 0), $replacement);
                let mut out = [$replacement; 2];
                assert_eq!(<$ty as $family>::get_region(x.get(), 0, 20, &mut out), 2);
                assert_eq!(out[0], $replacement);
                assert_eq!(<$ty as $family>::get_region(x.get(), 20, 2, &mut out), 0);
                assert_eq!(<$ty as AltVec>::dataptr_or_null(x.get()), ptr.cast());
                assert_eq!(<$ty as AltVec>::dataptr(x.get(), true), ptr.cast());
            }};
        }
        check!(
            std::ops::Range<i32>,
            AltInteger,
            i32,
            (1i32..4).into_sexp_altrep(),
            91
        );
        check!(
            std::ops::Range<f64>,
            AltReal,
            f64,
            (1.0..4.0).into_sexp_altrep(),
            9.5
        );
        check!(
            Vec<bool>,
            AltLogical,
            i32,
            vec![true, false].into_sexp_altrep(),
            i32::MIN
        );
        check!(
            MaterializingRaw,
            AltRaw,
            u8,
            MaterializingRaw(vec![1, 2]).into_sexp(),
            255
        );
        check!(
            MaterializingComplex,
            AltComplex,
            Rcomplex,
            MaterializingComplex(vec![Rcomplex { r: 1.0, i: 2.0 }; 2]).into_sexp(),
            Rcomplex { r: 9.0, i: -3.0 }
        );
    });
}

#[test]
fn materialized_metadata_requests_r_fallback() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new((1i32..4).into_sexp_altrep()) };
        assert_ne!(
            <std::ops::Range<i32> as AltInteger>::is_sorted(x.get()),
            UNKNOWN_SORTEDNESS
        );
        let ptr = <std::ops::Range<i32> as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *ptr = i32::MIN };
        assert_eq!(
            <std::ops::Range<i32> as AltInteger>::is_sorted(x.get()),
            UNKNOWN_SORTEDNESS
        );
        assert_eq!(<std::ops::Range<i32> as AltInteger>::no_na(x.get()), 0);
        assert!(<std::ops::Range<i32> as AltInteger>::sum(x.get(), false).is_null_ptr());
        assert!(<std::ops::Range<i32> as AltInteger>::min(x.get(), false).is_null_ptr());
        assert!(<std::ops::Range<i32> as AltInteger>::max(x.get(), false).is_null_ptr());
    });
}

#[test]
fn vec_callbacks_follow_replaced_allocation() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(vec![10i32, 20].into_sexp_altrep()) };
        let old_ptr = <Vec<i32> as AltVec>::dataptr_or_null(x.get());
        // Allocate the replacement while the original allocation is still live,
        // forcing a different address. No old pointer is dereferenced afterwards.
        let replacement = vec![30i32, 40, 50, 60];
        unsafe {
            *<Vec<i32> as miniextendr_api::altrep_data::AltrepExtract>::altrep_extract_mut(
                x.get(),
            ) = replacement;
        }
        let current_ptr = <Vec<i32> as AltVec>::dataptr_or_null(x.get());
        assert_ne!(old_ptr, current_ptr);
        assert_eq!(<Vec<i32> as Altrep>::length(x.get()), 4);
        assert_eq!(<Vec<i32> as AltInteger>::elt(x.get(), 3), 60);
        // A caller already holding old_ptr must reacquire it. Callback dispatch
        // follows the current Vec, but cannot repair an escaped C pointer.
    });
}

#[test]
fn materialized_length_tracks_the_r_buffer() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe { OwnedProtect::new(vec![true, false].into_sexp_altrep()) };
        <Vec<bool> as AltVec>::dataptr(x.get(), false);
        unsafe {
            <Vec<bool> as miniextendr_api::altrep_data::AltrepExtract>::altrep_extract_mut(x.get())
                .push(true);
        }
        assert_eq!(<Vec<bool> as Altrep>::length(x.get()), 2);
    });
}
