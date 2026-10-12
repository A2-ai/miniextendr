//! Guard selection must not silently disable the default data pointer.
mod r_test_utils;

use miniextendr_api::altrep_traits::{Altrep, AltrepGuard};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{AltIntegerData, AltrepLen, IntoR};

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, rust_unwind, class = "GuardedPureIntegers")]
struct PureIntegers;

impl AltrepLen for PureIntegers {
    fn len(&self) -> usize {
        3
    }
}
impl AltIntegerData for PureIntegers {
    fn elt(&self, i: usize) -> i32 {
        [10, 20, 30][i]
    }
}

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, r#unsafe, class = "GuardedTrivialIntegers")]
struct TrivialIntegers;

impl AltrepLen for TrivialIntegers {
    fn len(&self) -> usize {
        3
    }
}
impl AltIntegerData for TrivialIntegers {
    fn elt(&self, _: usize) -> i32 {
        42
    }
}

#[test]
fn native_read_guards_keep_materializing_dataptr() {
    r_test_utils::with_r_thread(|| unsafe {
        assert_eq!(PureIntegers::GUARD, AltrepGuard::RustUnwind);
        assert_eq!(TrivialIntegers::GUARD, AltrepGuard::Unsafe);
        let pure = OwnedProtect::new(PureIntegers.into_sexp());
        let trivial = OwnedProtect::new(TrivialIntegers.into_sexp());
        let p = miniextendr_api::altrep_bridge::t_dataptr::<PureIntegers>(
            pure.get(),
            miniextendr_api::Rboolean::FALSE,
        );
        let q = miniextendr_api::altrep_bridge::t_dataptr::<TrivialIntegers>(
            trivial.get(),
            miniextendr_api::Rboolean::FALSE,
        );
        assert_eq!(
            std::slice::from_raw_parts(p.cast::<i32>(), 3),
            &[10, 20, 30]
        );
        assert_eq!(
            std::slice::from_raw_parts(q.cast::<i32>(), 3),
            &[42, 42, 42]
        );
    });
}
