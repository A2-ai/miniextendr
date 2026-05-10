//! Fixtures for GC stress tests.
//!
//! Provides `SharedData` (R6 class) and `into_sexp_altrep` for the GC stress
//! and ALTREP serialization test suites.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::ffi::{SEXP, SEXPTYPE, SexpExt};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::{IntoRAltrep, miniextendr};

use crate::dataframe_enum_payload_matrix::{BTreeMapEvent, HashMapEvent};

/// Simple R6 class for GC stress tests.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SharedData {
    x: f64,
    y: f64,
    label: String,
}

/// @param x Numeric x-coordinate.
/// @param y Numeric y-coordinate.
/// @param label Character label.
#[miniextendr(r6)]
impl SharedData {
    pub fn new(x: f64, y: f64, label: String) -> Self {
        SharedData { x, y, label }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

/// Exercise `Vec<Option<collection>>` conversions under GC pressure.
///
/// Allocates `Vec<Option<Vec<i32>>>`, `Vec<Option<HashSet<String>>>`, and
/// `Vec<Option<BTreeSet<i32>>>` and converts each to SEXP, verifying that the
/// `OwnedProtect` in `vec_option_of_into_r_to_list` keeps the outer list live
/// across inner `into_sexp()` calls.
#[miniextendr]
pub fn gc_stress_vec_option_collection() {
    // Vec<Option<Vec<i32>>>: mix of Some and None
    let vec_opt: Vec<Option<Vec<i32>>> = vec![
        Some(vec![1, 2, 3]),
        None,
        Some(vec![4, 5]),
        None,
        Some(vec![]),
    ];
    let _ = vec_opt.into_sexp();

    // Vec<Option<HashSet<String>>>: some with multiple strings, some None
    let hs_opt: Vec<Option<HashSet<String>>> = vec![
        Some(["a", "b", "c"].iter().map(|s| s.to_string()).collect()),
        None,
        Some(["d"].iter().map(|s| s.to_string()).collect()),
    ];
    let _ = hs_opt.into_sexp();

    // Vec<Option<BTreeSet<i32>>>: sorted elements, some None
    let bt_opt: Vec<Option<BTreeSet<i32>>> = vec![
        Some([3, 1, 2].iter().copied().collect()),
        None,
        Some([5, 4].iter().copied().collect()),
    ];
    let _ = bt_opt.into_sexp();
}

/// Exercise `Vec<Option<&str>>` and `Vec<Option<&[T]>>` conversions under GC pressure.
///
/// Allocates STRSXP + list-column SEXPs with interleaved None/Some values to verify
/// PROTECT discipline across string and slice allocations.
#[miniextendr]
pub fn gc_stress_vec_option_borrowed() {
    // Vec<Option<&str>>: STRSXP with NA_character_
    let str_opt: Vec<Option<&str>> = vec![Some("hello"), None, Some("world"), None];
    let _ = str_opt.into_sexp();

    // Vec<Option<&[f64]>>: list-column, NULL for None
    let a: &[f64] = &[1.0, 2.0, 3.0];
    let b: &[f64] = &[4.0];
    let slice_opt: Vec<Option<&[f64]>> = vec![Some(a), None, Some(b), None];
    let _ = slice_opt.into_sexp();

    // Vec<Option<&[String]>>: list-column (character vector per row)
    let sa: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let sb: Vec<String> = vec!["z".to_string()];
    let str_slice_opt: Vec<Option<&[String]>> =
        vec![Some(sa.as_slice()), None, Some(sb.as_slice())];
    let _ = str_slice_opt.into_sexp();
}

/// Exercise map-field `Vec<Vec<K>>` / `Vec<Vec<V>>` column codegen under GC pressure.
///
/// Synthesizes realistic `HashMapEvent` and `BTreeMapEvent` rows and drives both
/// `to_dataframe` (align) and `to_dataframe_split` (per-variant partition) paths.
/// Verifies that the `ProtectScope::protect_raw` calls in the generated map-column
/// code keep each inner `Vec<K>` / `Vec<V>` SEXP live across the subsequent
/// `into_sexp()` call for the parallel column.
#[miniextendr]
pub fn gc_stress_dataframe_map() {
    // HashMap align path — multiple variants, multiple rows, includes empty map.
    let hm_rows = vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([
                ("x".to_string(), 1i32),
                ("y".to_string(), 2i32),
                ("z".to_string(), 3i32),
            ]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::new(), // empty map → Some(vec![]) in both columns
        },
        HashMapEvent::Tally {
            label: "d".into(),
            tally: HashMap::from([("p".to_string(), 10i32)]),
        },
        HashMapEvent::Empty { label: "e".into() },
    ];
    let _ = HashMapEvent::to_dataframe(hm_rows.clone());
    let _ = HashMapEvent::to_dataframe_split(hm_rows);

    // BTreeMap align path — same shape, sorted key order exercised.
    let bt_rows = vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([
                ("z".to_string(), 3i32),
                ("a".to_string(), 1i32),
                ("m".to_string(), 2i32),
            ]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "d".into(),
            tally: BTreeMap::from([("q".to_string(), 99i32)]),
        },
        BTreeMapEvent::Empty { label: "e".into() },
    ];
    let _ = BTreeMapEvent::to_dataframe(bt_rows.clone());
    let _ = BTreeMapEvent::to_dataframe_split(bt_rows);
}

/// Convert an R vector to an ALTREP-backed vector by materializing then re-wrapping.
/// Dispatches on `type_of()`: INTSXP, REALSXP, STRSXP.
/// @param x An integer, numeric, or character vector to convert.
#[miniextendr]
pub fn into_sexp_altrep(x: SEXP) -> SEXP {
    let sxp_type = x.type_of();
    match sxp_type {
        SEXPTYPE::INTSXP => {
            let v: Vec<i32> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::REALSXP => {
            let v: Vec<f64> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::STRSXP => {
            // Use Vec<Option<String>> to preserve NA_character_ values
            let v: Vec<Option<String>> =
                miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        _ => panic!("into_sexp_altrep: unsupported SEXP type {:?}", sxp_type),
    }
}
