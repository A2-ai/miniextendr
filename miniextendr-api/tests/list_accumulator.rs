//! `ListAccumulator` under `gctorture(TRUE)`: a pushed value is rooted before
//! the list grows, and a named accumulator always carries `names` (#1907).

mod r_test_utils;

use miniextendr_api::ListAccumulator;
use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{SEXP, SEXPTYPE, SexpExt};

/// Runs `f` with `gctorture(TRUE)`, turned off again however `f` ends.
fn with_gctorture<T>(f: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            unsafe { r_eval_str_global("gctorture(FALSE)").unwrap() };
        }
    }
    let _reset = Reset;
    unsafe { r_eval_str_global("gctorture(TRUE)").unwrap() };
    f()
}

/// A fresh, unrooted SEXP pushed when the accumulator is full survives the
/// growth that follows: every push here grows (capacity 1, then doubling),
/// and under gctorture every allocation collects.
#[test]
fn a_fresh_sexp_survives_the_growth() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let list = with_gctorture(|| {
            let mut acc = ListAccumulator::new(&scope, 1);
            acc.push(1.0_f64);
            acc.push_sexp(miniextendr_api::sys::Rf_allocVector(
                SEXPTYPE::REALSXP,
                1000,
            ));
            for i in 0..9 {
                let fresh = miniextendr_api::sys::Rf_allocVector(SEXPTYPE::INTSXP, i + 1);
                if i % 2 == 0 {
                    acc.push(fresh);
                } else {
                    acc.push_named("v", fresh);
                }
            }
            acc.into_root().get()
        });
        assert_eq!(list.xlength(), 11);
        let second = list.vector_elt(1);
        assert_eq!(second.type_of(), SEXPTYPE::REALSXP);
        assert_eq!(second.xlength(), 1000);
        for i in 0..9 {
            let elt = list.vector_elt(i + 2);
            assert_eq!(elt.type_of(), SEXPTYPE::INTSXP, "element {}", i + 2);
            assert_eq!(elt.xlength(), i + 1, "element {}", i + 2);
        }
    });
}

/// `named()` gives `names = character(0)` with no pushes, as
/// `List::from_raw_pairs(vec![])` does; `new()` gives a bare `list()`.
#[test]
fn an_empty_named_accumulator_has_empty_names() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let named = ListAccumulator::named(&scope, 4).into_root().get();
        assert_eq!(named.xlength(), 0);
        let names = named.get_names();
        assert_eq!(names.type_of(), SEXPTYPE::STRSXP);
        assert_eq!(names.xlength(), 0);

        let plain = ListAccumulator::new(&scope, 4).into_root().get();
        assert_eq!(plain.xlength(), 0);
        assert_eq!(plain.get_names(), SEXP::nil());

        // A named accumulator names an unnamed push `""`.
        let mut acc = ListAccumulator::named(&scope, 1);
        acc.push(1_i32);
        acc.push_named("b", 2_i32);
        let list = acc.into_root().get();
        let names = list.get_names();
        assert_eq!(names.string_elt_str(0), Some(""));
        assert_eq!(names.string_elt_str(1), Some("b"));
    });
}
