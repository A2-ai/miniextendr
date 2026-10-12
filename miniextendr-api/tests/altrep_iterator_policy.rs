//! A concrete iterator class retains its type, cache and serialization policy.
mod r_test_utils;

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen, AltrepSerialize, IterIntData};
use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, SEXP, SexpExt};

#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, serialize, duplicate = "clone", class = "ConcreteRangeCache")]
struct RangeCache(IterIntData<std::ops::Range<i32>>);

impl AltrepLen for RangeCache {
    fn altrep_len(&self) -> usize {
        self.0.len()
    }
}
impl AltIntegerData for RangeCache {
    fn elt(&self, i: usize) -> i32 {
        self.0.elt(i)
    }
}
impl AltrepSerialize for RangeCache {
    fn serialized_state(&self) -> SEXP {
        self.0.serialized_state()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        IterIntData::unserialize(state).map(Self)
    }
}

#[test]
fn concrete_iterator_roundtrips_before_and_after_r_writes() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(RangeCache(IterIntData::from_exact_iter(10..16i32)).into_sexp());
        assert_eq!(<RangeCache as AltInteger>::elt(x.get(), 2), 12);
        let cloned = OwnedProtect::new(<RangeCache as Altrep>::duplicate(x.get(), false));
        assert!(cloned.get().is_altrep());
        assert_eq!(<RangeCache as AltInteger>::elt(cloned.get(), 4), 14);
        let state = OwnedProtect::new(<RangeCache as Altrep>::serialized_state(x.get()));
        let restored = OwnedProtect::new(
            <RangeCache as AltrepSerialize>::unserialize(state.get())
                .unwrap()
                .into_sexp(),
        );
        assert_eq!(<RangeCache as AltInteger>::elt(restored.get(), 5), 15);
        let ptr = <RangeCache as AltVec>::dataptr(x.get(), true).cast::<i32>();
        *ptr.add(2) = 99;
        assert!(<RangeCache as Altrep>::serialized_state(x.get()).is_null_ptr());
        // Written data2 asks R to save an ordinary vector with current values.
        let written = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("serialize")
                .arg(x.get())
                .arg(SEXP::nil())
                .eval_base()
                .unwrap(),
        );
        let roundtrip = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("unserialize")
                .arg(written.get())
                .eval_base()
                .unwrap(),
        );
        assert_eq!(roundtrip.get().integer_elt(2), 99);
        assert_eq!(<RangeCache as AltInteger>::elt(cloned.get(), 2), 12);
    });
}
