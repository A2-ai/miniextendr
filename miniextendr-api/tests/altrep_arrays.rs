//! Owned arrays retain their type and length while using normal ALTREP storage.
mod r_test_utils;

use miniextendr_api::altrep::RegisterAltrep;
use miniextendr_api::altrep_data::AltrepExtract;
use miniextendr_api::altrep_traits::{AltInteger, AltLogical, AltString, AltVec, Altrep};
use miniextendr_api::{IntoR, OwnedProtect, RLogical, Rcomplex, SEXP, SexpExt, TryFromSexp};

#[test]
fn class_cache_distinguishes_lengths_and_retains_the_array_owner() {
    r_test_utils::with_r_thread(|| {
        let two = unsafe { OwnedProtect::new(miniextendr_api::Altrep([1_i32, 2]).into_sexp()) };
        let three =
            unsafe { OwnedProtect::new(miniextendr_api::Altrep([3_i32, 4, 5]).into_sexp()) };
        let empty =
            unsafe { OwnedProtect::new(miniextendr_api::Altrep::<[i32; 0]>([]).into_sexp()) };
        assert!(two.get().is_altrep());
        assert_eq!(<[i32; 2] as Altrep>::length(two.get()), 2);
        assert_eq!(<[i32; 3] as Altrep>::length(three.get()), 3);
        assert_eq!(<[i32; 0] as Altrep>::length(empty.get()), 0);
        assert_ne!(
            <[i32; 2] as RegisterAltrep>::get_or_init_class().ptr,
            <[i32; 3] as RegisterAltrep>::get_or_init_class().ptr
        );
        assert_eq!(
            <[i32; 2] as RegisterAltrep>::get_or_init_class().ptr,
            <[i32; 2] as RegisterAltrep>::get_or_init_class().ptr
        );
        assert_eq!(
            unsafe { <[i32; 3] as AltrepExtract>::altrep_extract_ref(three.get()) },
            &[3, 4, 5]
        );

        let pointer = <[i32; 2] as AltVec>::dataptr(two.get(), true).cast::<i32>();
        unsafe { *pointer.add(1) = 9 };
        assert_eq!(<[i32; 2] as AltInteger>::elt(two.get(), 1), 9);
        assert_eq!(
            unsafe { <[i32; 2] as AltrepExtract>::altrep_extract_ref(two.get()) },
            &[1, 9]
        );
        let mut region = [0];
        assert_eq!(
            <[i32; 3] as AltInteger>::get_region(three.get(), 1, 99, &mut region),
            1
        );
        assert_eq!(region, [4]);
        assert_eq!(
            <[i32; 3] as AltInteger>::get_region(three.get(), isize::MAX, isize::MAX, &mut region),
            0
        );
    });
}

#[test]
fn every_native_array_family_exposes_its_owned_buffer() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($ty:ty, $values:expr) => {{
                let x = unsafe { OwnedProtect::new(miniextendr_api::Altrep($values).into_sexp()) };
                assert!(x.get().is_altrep());
                let owner = unsafe { <[$ty; 2] as AltrepExtract>::altrep_extract_ref(x.get()) };
                assert_eq!(
                    <[$ty; 2] as AltVec>::dataptr_or_null(x.get()),
                    owner.as_ptr().cast()
                );
            }};
        }
        check!(i32, [1_i32, 2]);
        check!(f64, [1.0_f64, 2.0]);
        check!(u8, [1_u8, 2]);
        check!(Rcomplex, [Rcomplex { r: 1.0, i: 2.0 }; 2]);
        check!(RLogical, [RLogical::TRUE, RLogical::NA]);

        let x = unsafe {
            OwnedProtect::new(
                miniextendr_api::Altrep([RLogical::from_i32(7), RLogical::NA]).into_sexp(),
            )
        };
        assert_eq!(<[RLogical; 2] as AltLogical>::elt(x.get(), 0), 7);
        assert_eq!(<[RLogical; 2] as AltLogical>::elt(x.get(), 1), i32::MIN);
    });
}

#[test]
fn non_native_arrays_use_authoritative_materialized_storage() {
    r_test_utils::with_r_thread(|| {
        let logical =
            unsafe { OwnedProtect::new(miniextendr_api::Altrep([true, false]).into_sexp()) };
        assert!(<[bool; 2] as AltVec>::dataptr_or_null(logical.get()).is_null());
        let pointer = <[bool; 2] as AltVec>::dataptr(logical.get(), true).cast::<i32>();
        unsafe { *pointer = i32::MIN };
        assert_eq!(<[bool; 2] as AltLogical>::elt(logical.get(), 0), i32::MIN);
        assert_eq!(<[bool; 2] as AltLogical>::no_na(logical.get()), 0);
        assert_eq!(
            unsafe { <[bool; 2] as AltrepExtract>::altrep_extract_ref(logical.get()) },
            &[true, false]
        );

        let strings = unsafe {
            OwnedProtect::new(
                miniextendr_api::Altrep(["one".to_owned(), "two".to_owned()]).into_sexp(),
            )
        };
        <[String; 2] as AltString>::set_elt(strings.get(), 0, SEXP::na_string());
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(strings.get()).unwrap(),
            vec![None, Some("two".to_owned())]
        );
    });
}

#[test]
fn arrays_duplicate_and_serialize_as_current_ordinary_vectors() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::expression::RCall;

        let x = unsafe { OwnedProtect::new(miniextendr_api::Altrep([1_i32, 2]).into_sexp()) };
        let pointer = <[i32; 2] as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *pointer = 9 };
        let duplicate = unsafe { OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get())) };
        assert!(!duplicate.get().is_altrep());
        assert_eq!(Vec::<i32>::try_from_sexp(duplicate.get()).unwrap(), [9, 2]);

        let bytes = unsafe {
            OwnedProtect::new(
                RCall::new("serialize")
                    .arg(x.get())
                    .arg(SEXP::nil())
                    .eval_base()
                    .unwrap(),
            )
        };
        let restored = unsafe {
            OwnedProtect::new(
                RCall::new("unserialize")
                    .arg(bytes.get())
                    .eval_base()
                    .unwrap(),
            )
        };
        assert!(!restored.get().is_altrep());
        assert_eq!(Vec::<i32>::try_from_sexp(restored.get()).unwrap(), [9, 2]);
    });
}
