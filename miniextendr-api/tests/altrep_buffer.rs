//! Native buffer growth, value duplication and serialization through R.
mod r_test_utils;
use miniextendr_api::altrep_data::AltrepExtract;
use miniextendr_api::altrep_traits::AltVec;
use miniextendr_api::{Altrep, AltrepBuffer, IntoR, OwnedProtect, SexpExt, TryFromSexp};

#[test]
fn native_buffer_preserves_old_pointers_and_current_serialized_values() {
    r_test_utils::with_r_thread(|| unsafe {
        type Buffer = AltrepBuffer<i32>;
        let x = OwnedProtect::new(Altrep(Buffer::new(vec![1, 2])).into_sexp());
        let old = <Buffer as AltVec>::dataptr(x.get(), true).cast::<i32>();
        *old = 4;
        <Buffer as AltrepExtract>::altrep_extract_mut(x.get()).edit(|v| {
            v.resize(4096, 9);
            v[0] = 7;
        });
        assert_eq!(*old, 4);
        assert_eq!(x.get().len(), 4096);
        assert_eq!(x.get().integer_elt(0), 7);
        let current = <Buffer as AltVec>::dataptr(x.get(), true).cast::<i32>();
        assert_ne!(current, old);
        let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
        assert!(copy.get().is_altrep());
        *<Buffer as AltVec>::dataptr(copy.get(), true).cast::<i32>() = 11;
        assert_eq!(*current, 7);
        let serialized = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("serialize")
                .arg(x.get())
                .arg(miniextendr_api::SEXP::nil())
                .eval_base()
                .unwrap(),
        );
        let restored = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("unserialize")
                .arg(serialized.get())
                .eval_base()
                .unwrap(),
        );
        let values = Vec::<i32>::try_from_sexp(restored.get()).unwrap();
        assert_eq!(values.len(), 4096);
        assert_eq!(values[0], 7);
        assert_eq!(values[4095], 9);
    });
}
