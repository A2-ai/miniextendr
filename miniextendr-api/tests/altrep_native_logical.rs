//! Native logical containers share one buffer across R access paths.
mod r_test_utils;

use std::borrow::Cow;

use miniextendr_api::altrep_data::{AltrepExtract, AltrepSerialize};
use miniextendr_api::altrep_traits::{AltLogical, AltVec, Altrep};
use miniextendr_api::{IntoR, OwnedProtect, RLogical, SexpExt, TryFromSexp};

#[test]
fn native_logical_containers_preserve_storage_and_serialize_writes() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($ty:ty, $source:expr) => {{
                let source: $ty = $source;
                let original: &[RLogical] = source.as_ref();
                let original_ptr = original.as_ptr().cast();
                let x = unsafe { OwnedProtect::new(miniextendr_api::Altrep(source).into_sexp()) };
                assert!(x.get().is_altrep());
                assert!(x.get().is_logical());
                assert_eq!(<$ty as AltVec>::dataptr_or_null(x.get()), original_ptr);
                assert_eq!(<$ty as AltLogical>::elt(x.get(), 1), i32::MIN);
                assert_eq!(<$ty as AltLogical>::elt(x.get(), 3), 7);
                assert_eq!(<$ty as AltLogical>::no_na(x.get()), 0);

                let mut region = [-1; 3];
                assert_eq!(
                    <$ty as AltLogical>::get_region(x.get(), 1, 99, &mut region),
                    3
                );
                assert_eq!(region, [i32::MIN, 1, 7]);
                assert_eq!(
                    <$ty as AltLogical>::get_region(x.get(), isize::MAX, 99, &mut region),
                    0
                );

                let pointer = <$ty as AltVec>::dataptr(x.get(), true).cast::<i32>();
                unsafe { *pointer.add(1) = 12 };
                assert_eq!(<$ty as AltLogical>::elt(x.get(), 1), 12);
                assert_eq!(<$ty as AltLogical>::no_na(x.get()), 1);
                assert_eq!(
                    <$ty as AltLogical>::get_region(x.get(), 1, 1, &mut region),
                    1
                );
                assert_eq!(region[0], 12);

                let expected = [0, 12, 1, 7].map(RLogical::from_i32);
                assert_eq!(Vec::<RLogical>::try_from_sexp(x.get()).unwrap(), expected);
                let state =
                    unsafe { OwnedProtect::new(<$ty as Altrep>::serialized_state(x.get())) };
                assert!(state.get().is_logical());
                let restored = <$ty as AltrepSerialize>::unserialize(state.get()).unwrap();
                drop(state);
                unsafe { miniextendr_api::sys::R_gc() };
                let values: &[RLogical] = restored.as_ref();
                assert_eq!(values, expected);
                assert_ne!(values.as_ptr().cast::<i32>(), pointer.cast_const());
            }};
        }
        let values = [0, i32::MIN, 1, 7].map(RLogical::from_i32);
        check!(Vec<RLogical>, values.to_vec());
        check!(Box<[RLogical]>, values.to_vec().into_boxed_slice());
        check!(Cow<'static, [RLogical]>, Cow::Owned(values.to_vec()));
    });
}

#[test]
fn borrowed_native_logicals_detach_only_for_writes_and_restore_owned() {
    r_test_utils::with_r_thread(|| {
        type Data = Cow<'static, [RLogical]>;
        static SOURCE: [RLogical; 3] = [RLogical::FALSE, RLogical::NA, RLogical::TRUE];
        let x = unsafe {
            OwnedProtect::new(miniextendr_api::Altrep(Data::Borrowed(&SOURCE)).into_sexp())
        };
        assert_eq!(
            <Data as AltVec>::dataptr(x.get(), false)
                .cast::<RLogical>()
                .cast_const(),
            SOURCE.as_ptr()
        );
        assert!(matches!(
            unsafe { <Data as AltrepExtract>::altrep_extract_ref(x.get()) },
            Cow::Borrowed(_)
        ));
        // Even serialization of a borrowed source must restore owned storage.
        let state = unsafe { OwnedProtect::new(<Data as Altrep>::serialized_state(x.get())) };
        let restored = <Data as AltrepSerialize>::unserialize(state.get()).unwrap();
        assert!(matches!(restored, Cow::Owned(_)));
        drop(state);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(restored.as_ref(), &SOURCE);

        let pointer = <Data as AltVec>::dataptr(x.get(), true).cast::<i32>();
        assert_ne!(pointer.cast_const(), SOURCE.as_ptr().cast::<i32>());
        unsafe { *pointer = i32::MIN };
        assert_eq!(<Data as AltLogical>::elt(x.get(), 0), i32::MIN);
        assert_eq!(SOURCE[0], RLogical::FALSE);
        assert!(matches!(
            unsafe { <Data as AltrepExtract>::altrep_extract_ref(x.get()) },
            Cow::Owned(_)
        ));
    });
}

#[test]
fn native_logical_callbacks_follow_vec_reallocation() {
    r_test_utils::with_r_thread(|| {
        type Data = Vec<RLogical>;
        let x =
            unsafe { OwnedProtect::new(miniextendr_api::Altrep(vec![RLogical::TRUE]).into_sexp()) };
        let old = <Data as AltVec>::dataptr_or_null(x.get());
        // The old allocation remains live while this distinct buffer is allocated.
        let replacement = vec![RLogical::FALSE, RLogical::NA, RLogical::from_i32(9)];
        unsafe { *<Data as AltrepExtract>::altrep_extract_mut(x.get()) = replacement };
        assert_ne!(<Data as AltVec>::dataptr_or_null(x.get()), old);
        assert_eq!(<Data as Altrep>::length(x.get()), 3);
        assert_eq!(<Data as AltLogical>::elt(x.get(), 2), 9);
        let pointer = <Data as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *pointer = 1 };
        assert_eq!(<Data as AltLogical>::elt(x.get(), 0), 1);
    });
}

#[test]
fn empty_native_logical_containers_expose_valid_empty_buffers() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($ty:ty, $source:expr) => {{
                let x = unsafe { OwnedProtect::new(miniextendr_api::Altrep($source).into_sexp()) };
                assert_eq!(<$ty as Altrep>::length(x.get()), 0);
                assert!(!<$ty as AltVec>::dataptr(x.get(), true).is_null());
                assert_eq!(<$ty as AltLogical>::get_region(x.get(), 0, 1, &mut [0]), 0);
                assert_eq!(<$ty as AltLogical>::no_na(x.get()), 1);
            }};
        }
        check!(Vec<RLogical>, Vec::<RLogical>::new());
        check!(Box<[RLogical]>, Vec::<RLogical>::new().into_boxed_slice());
        check!(
            Cow<'static, [RLogical]>,
            Cow::<'static, [RLogical]>::Borrowed(&[])
        );
    });
}
