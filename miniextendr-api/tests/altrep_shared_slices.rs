//! Shared slice ownership, pointer lifetime, duplication and current-value serialization.
mod r_test_utils;

use miniextendr_api::altrep_data::AltrepExtract;
use miniextendr_api::altrep_traits::{AltInteger, AltLogical, AltString, AltVec};
use miniextendr_api::expression::RCall;
use miniextendr_api::{Altrep, IntoR, OwnedProtect, SEXP, SexpExt, TryFromSexp};

unsafe fn roundtrip(x: SEXP) -> OwnedProtect {
    unsafe {
        let bytes = OwnedProtect::new(
            RCall::new("serialize")
                .arg(x)
                .arg(SEXP::nil())
                .eval_base()
                .unwrap(),
        );
        OwnedProtect::new(
            RCall::new("unserialize")
                .arg(bytes.get())
                .eval_base()
                .unwrap(),
        )
    }
}

macro_rules! shared_owner_tests {
    ($module:ident, $owner:path) => {
        mod $module {
            use super::*;
            use $owner as Owner;

            #[test]
            fn shared_reads_duplicate_cheaply_then_pointer_access_detaches() {
                r_test_utils::with_r_thread(|| unsafe {
                    let external: Owner<[i32]> = Owner::from([10, 20, 30]);
                    let x = OwnedProtect::new(Altrep(external.clone()).into_sexp());
                    let y = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
                    assert!(y.get().is_altrep());
                    assert_eq!(Owner::strong_count(&external), 3);
                    assert!(<Owner<[i32]> as AltVec>::dataptr_or_null(x.get()).is_null());
                    assert_eq!(<Owner<[i32]> as AltInteger>::elt(x.get(), 1), 20);
                    let read = <Owner<[i32]> as AltVec>::dataptr(x.get(), false).cast::<i32>();
                    assert_ne!(read.cast_const(), external.as_ptr());
                    let write = <Owner<[i32]> as AltVec>::dataptr(x.get(), true).cast::<i32>();
                    assert_eq!(read, write);
                    *write.add(1) = 77;
                    assert_eq!(<Owner<[i32]> as AltInteger>::elt(x.get(), 1), 77);
                    assert_eq!(<Owner<[i32]> as AltInteger>::elt(y.get(), 1), 20);
                    assert_eq!(external[1], 20);
                    assert_eq!(
                        <Owner<[i32]> as AltrepExtract>::altrep_extract_ref(x.get())[1],
                        77
                    );
                });
            }

            #[test]
            fn duplicate_after_exposure_never_shares_the_writable_allocation() {
                r_test_utils::with_r_thread(|| unsafe {
                    let x = OwnedProtect::new(Altrep(Owner::<[i32]>::from([1, 2])).into_sexp());
                    let pointer = <Owner<[i32]> as AltVec>::dataptr(x.get(), true).cast::<i32>();
                    *pointer = 9;
                    let y = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
                    let other = <Owner<[i32]> as AltVec>::dataptr(y.get(), true).cast::<i32>();
                    assert_ne!(pointer, other);
                    *pointer = 17;
                    *other.add(1) = 23;
                    assert_eq!(Vec::<i32>::try_from_sexp(x.get()).unwrap(), [17, 2]);
                    assert_eq!(Vec::<i32>::try_from_sexp(y.get()).unwrap(), [9, 23]);
                    assert_eq!(
                        <Owner<[i32]> as AltVec>::dataptr(x.get(), false),
                        pointer.cast()
                    );
                    let restored = roundtrip(x.get());
                    assert!(restored.get().is_altrep());
                    assert_eq!(Vec::<i32>::try_from_sexp(restored.get()).unwrap(), [17, 2]);
                });
            }

            #[test]
            fn weak_handles_are_detached_before_any_pointer_is_published() {
                r_test_utils::with_r_thread(|| unsafe {
                    let owner: Owner<[i32]> = Owner::from([3, 4]);
                    let weak = Owner::downgrade(&owner);
                    let x = OwnedProtect::new(Altrep(owner).into_sexp());
                    assert!(<Owner<[i32]> as AltVec>::dataptr_or_null(x.get()).is_null());
                    let pointer = <Owner<[i32]> as AltVec>::dataptr(x.get(), false);
                    assert!(!pointer.is_null());
                    assert!(weak.upgrade().is_none());
                    assert_eq!(<Owner<[i32]> as AltVec>::dataptr(x.get(), true), pointer);
                });
            }

            #[test]
            fn non_native_duplicates_and_serialization_use_the_written_cache() {
                r_test_utils::with_r_thread(|| unsafe {
                    let logical =
                        OwnedProtect::new(Altrep(Owner::<[bool]>::from([true, false])).into_sexp());
                    let pointer =
                        <Owner<[bool]> as AltVec>::dataptr(logical.get(), true).cast::<i32>();
                    *pointer = i32::MIN;
                    let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(logical.get()));
                    assert!(copy.get().is_altrep());
                    assert_eq!(<Owner<[bool]> as AltLogical>::elt(copy.get(), 0), i32::MIN);
                    *pointer = 0;
                    assert_eq!(<Owner<[bool]> as AltLogical>::elt(copy.get(), 0), i32::MIN);
                    let restored = roundtrip(copy.get());
                    assert_eq!(
                        Vec::<Option<bool>>::try_from_sexp(restored.get()).unwrap(),
                        [None, Some(false)]
                    );

                    let strings = OwnedProtect::new(
                        Altrep(Owner::<[String]>::from(["a".into(), "b".into()])).into_sexp(),
                    );
                    <Owner<[String]> as AltString>::set_elt(strings.get(), 0, SEXP::na_string());
                    let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(strings.get()));
                    let replacement = OwnedProtect::new("changed".into_sexp());
                    <Owner<[String]> as AltString>::set_elt(
                        strings.get(),
                        1,
                        replacement.get().string_elt(0),
                    );
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(copy.get()).unwrap(),
                        [None, Some("b".into())]
                    );
                    let restored = roundtrip(copy.get());
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(restored.get()).unwrap(),
                        [None, Some("b".into())]
                    );
                });
            }

            #[test]
            fn read_only_strings_keep_the_class_through_duplicate_and_serialize() {
                r_test_utils::with_r_thread(|| unsafe {
                    use miniextendr_api::altrep_traits::Altrep as AltrepClass;

                    let x = OwnedProtect::new(
                        Altrep(Owner::<[String]>::from([
                            "a".into(),
                            "b".into(),
                            "c".into(),
                        ]))
                        .into_sexp(),
                    );
                    // One slot read: a partial cache, with NULL in the others.
                    assert_eq!(
                        <Owner<[String]> as AltString>::elt(x.get(), 1).r_char_str(),
                        Some("b")
                    );
                    let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
                    assert!(copy.get().is_altrep());
                    assert!(
                        !<Owner<[String]> as AltrepClass>::serialized_state(copy.get())
                            .is_null_ptr()
                    );
                    let restored = roundtrip(copy.get());
                    assert!(restored.get().is_altrep());
                    assert_eq!(
                        Vec::<String>::try_from_sexp(restored.get()).unwrap(),
                        ["a", "b", "c"]
                    );

                    // A copy of a written cache is written too.
                    <Owner<[String]> as AltString>::set_elt(x.get(), 2, SEXP::na_string());
                    let written = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
                    assert!(
                        <Owner<[String]> as AltrepClass>::serialized_state(written.get())
                            .is_null_ptr()
                    );
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(written.get()).unwrap(),
                        [Some("a".into()), Some("b".into()), None]
                    );
                });
            }

            #[test]
            fn all_native_families_and_empty_slices_keep_their_owned_allocation() {
                r_test_utils::with_r_thread(|| unsafe {
                    macro_rules! check {
                        ($elem:ty, $values:expr) => {{
                            let owner: Owner<[$elem]> = Owner::from($values);
                            let original = owner.as_ptr();
                            let x = OwnedProtect::new(Altrep(owner).into_sexp());
                            assert!(x.get().is_altrep());
                            assert_eq!(
                                <Owner<[$elem]> as AltVec>::dataptr_or_null(x.get()),
                                original.cast()
                            );
                            assert_eq!(
                                <Owner<[$elem]> as AltVec>::dataptr(x.get(), true).cast_const(),
                                original.cast()
                            );
                        }};
                    }
                    check!(i32, []);
                    check!(f64, [1.0, 2.0]);
                    check!(u8, [1, 2]);
                    check!(
                        miniextendr_api::Rcomplex,
                        [miniextendr_api::Rcomplex { r: 1.0, i: 2.0 }]
                    );
                    check!(
                        miniextendr_api::RLogical,
                        [
                            miniextendr_api::RLogical::NA,
                            miniextendr_api::RLogical::from_i32(7)
                        ]
                    );
                });
            }
        }
    };
}
shared_owner_tests!(rc, std::rc::Rc);
shared_owner_tests!(arc, std::sync::Arc);
