//! Named map vectors have stable positions, value copies and current R writes.
mod r_test_utils;

use miniextendr_api::altrep_data::AltrepSerialize;
use miniextendr_api::altrep_traits::{AltVec, Altrep};
use miniextendr_api::expression::RCall;
use miniextendr_api::{AltrepMap, IntoR, OwnedProtect, SEXP, SexpExt, TryFromSexp};
use std::collections::HashMap;

#[test]
fn indexed_map_keeps_names_order_duplication_and_written_values() {
    r_test_utils::with_r_thread(|| unsafe {
        type Map = AltrepMap<i32>;
        let map = Map::from(HashMap::from([
            ("z".into(), 30),
            ("a".into(), 10),
            ("m".into(), 20),
        ]));
        let x = OwnedProtect::new(map.into_sexp());
        assert!(x.get().is_altrep());
        assert_eq!(x.get().integer_elt(0), 10);
        assert_eq!(x.get().integer_elt(1), 20);
        let names = miniextendr_api::sys::Rf_getAttrib_unchecked(
            x.get(),
            miniextendr_api::sys::R_NamesSymbol,
        );
        assert_eq!(
            Vec::<String>::try_from_sexp(names).unwrap(),
            ["a", "m", "z"]
        );
        let saved = OwnedProtect::new(<Map as Altrep>::serialized_state(x.get()));
        let restored = <Map as AltrepSerialize>::unserialize(saved.get()).unwrap();
        assert_eq!(restored.keys(), &["a", "m", "z"]);
        assert_eq!(restored.get("m"), Some(&20));
        let copy = OwnedProtect::new(miniextendr_api::sys::Rf_duplicate(x.get()));
        assert!(copy.get().is_altrep());
        let ptr = <Map as AltVec>::dataptr(x.get(), true).cast::<i32>();
        *ptr.add(1) = 99;
        assert_eq!(copy.get().integer_elt(1), 20);
        let serialized = OwnedProtect::new(
            RCall::new("serialize")
                .arg(x.get())
                .arg(SEXP::nil())
                .eval_base()
                .unwrap(),
        );
        let written = OwnedProtect::new(
            RCall::new("unserialize")
                .arg(serialized.get())
                .eval_base()
                .unwrap(),
        );
        assert_eq!(
            Vec::<i32>::try_from_sexp(written.get()).unwrap(),
            [10, 99, 30]
        );
        let names = miniextendr_api::sys::Rf_getAttrib_unchecked(
            written.get(),
            miniextendr_api::sys::R_NamesSymbol,
        );
        assert_eq!(
            Vec::<String>::try_from_sexp(names).unwrap(),
            ["a", "m", "z"]
        );
    });
}

#[test]
fn indexed_map_preserves_native_logical_na() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = OwnedProtect::new(
            AltrepMap::from(HashMap::from([
                ("missing".into(), miniextendr_api::RLogical::NA),
                ("present".into(), miniextendr_api::RLogical::TRUE),
            ]))
            .into_sexp(),
        );
        assert_eq!(
            Vec::<miniextendr_api::RLogical>::try_from_sexp(x.get()).unwrap(),
            [
                miniextendr_api::RLogical::NA,
                miniextendr_api::RLogical::TRUE
            ]
        );
    });
}
